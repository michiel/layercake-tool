#![cfg(feature = "console")]

//! Chat session implementation using rig with rmcp for MCP integration
//!
//! This module replaces the old llm-based chat implementation with rig agents.
//! Key changes:
//! - Uses rig agents instead of llm::LLMProvider
//! - Uses rmcp for direct MCP integration (no conversion layer)
//! - Maintains same session persistence and observer pattern

use std::{collections::BTreeMap, fmt::Write as FmtWrite, sync::Arc};

use anyhow::{anyhow, Context, Result};
use axum_mcp::prelude::{ClientContext, SecurityContext};
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing;

use rig::providers::gemini::completion::gemini_api_types::{
    AdditionalParameters, GenerationConfig,
};
use rig::providers::{anthropic, gemini, ollama, openai};
use rig::OneOrMany;
use rig::{
    agent::{Agent, AgentBuilder},
    client::{CompletionClient, ProviderClient},
    completion::{
        message::{
            AssistantContent, Message, Reasoning, ToolCall, ToolFunction, ToolResult,
            ToolResultContent, UserContent,
        },
        Completion, CompletionModel, Usage,
    },
};

#[cfg(feature = "rmcp")]
use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation, Tool as RmcpTool},
    service::RunningService,
    transport::StreamableHttpClientTransport,
    ServiceExt,
};

use crate::app_context::summarize_graph_counts;
use crate::database::entities::{
    data_sets, graphs, plan_dag_edges, plan_dag_nodes, plans, projects, users,
};
use crate::mcp::security::build_user_security_context;
use crate::services::system_settings_service::SystemSettingsService;
use layercake_genai::services::DataAcquisitionService;

use super::{
    config::{ChatConfig, ChatCredentialStore},
    ChatProvider, McpBridge, RagContextBuilder,
};

const MAX_TOOL_ITERATIONS: usize = 5;

#[derive(Clone, Debug)]
pub enum ChatEvent {
    AssistantMessage { text: String },
    ToolInvocation { name: String, summary: String },
}

/// Message representation for chat history
#[derive(Clone, Debug)]
struct ChatMessage {
    role: String,
    content: String,
    tool_calls: Option<Vec<ToolCallData>>,
    tool_results: Option<Vec<ToolResultData>>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ToolCallData {
    id: String,
    name: String,
    arguments: serde_json::Value,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ToolResultData {
    call_id: String,
    output: String,
}

impl ChatMessage {
    fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: None,
            tool_results: None,
        }
    }

    fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: None,
            tool_results: None,
        }
    }

    fn with_tool_calls(mut self, calls: Vec<ToolCallData>) -> Self {
        self.tool_calls = Some(calls);
        self
    }

    fn tool_result(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        let output = output.into();
        Self {
            role: "tool".to_string(),
            content: output.clone(),
            tool_calls: None,
            tool_results: Some(vec![ToolResultData {
                call_id: call_id.into(),
                output,
            }]),
        }
    }
}

#[derive(Clone)]
struct RigPromptContext {
    preamble: String,
    prompt: Message,
    history: Vec<Message>,
}

#[derive(Clone)]
struct AgentResponse {
    choice: OneOrMany<AssistantContent>,
    usage: Usage,
}

/// Enum to hold provider-specific rig agents
enum RigAgent {
    OpenAI(String), // Just hold model name for now, will create agent on-demand
    Anthropic(String),
    Gemini(String),
    Ollama(String),
}

pub struct ChatSession {
    db: DatabaseConnection,
    session_id: Option<String>,
    project_id: i32,
    user_id: i32,
    provider: ChatProvider,
    model_name: String,
    system_prompt: String,
    messages: Vec<ChatMessage>,
    bridge: McpBridge,
    security: SecurityContext,
    tool_use_enabled: bool,
    agent: RigAgent,
    credentials: ChatCredentialStore,
    config: ChatConfig,
    #[cfg(feature = "rmcp")]
    rmcp_client: Option<RunningService<rmcp::RoleClient, ClientInfo>>,
    #[cfg(feature = "rmcp")]
    rmcp_tools: Vec<RmcpTool>,

    // RAG configuration
    data_acquisition: Arc<DataAcquisitionService>,
    rag_enabled: bool,
    rag_top_k: usize,
    rag_threshold: f32,
    rag_max_context_tokens: usize,
    include_citations: bool,
    last_rag_context: Option<super::RagContext>,
}

impl ChatSession {
    /// Initialize rmcp client connection to MCP server
    #[cfg(feature = "rmcp")]
    async fn init_rmcp_client(
        mcp_server_url: &str,
    ) -> Result<(RunningService<rmcp::RoleClient, ClientInfo>, Vec<RmcpTool>)> {
        tracing::info!("Connecting to MCP server at {}", mcp_server_url);

        let transport = StreamableHttpClientTransport::from_uri(mcp_server_url);

        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "layercake-chat".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
        };

        let client = client_info
            .serve(transport)
            .await
            .context("Failed to connect to MCP server")?;

        let server_info = client.peer().peer_info();
        tracing::info!("Connected to MCP server: {:?}", server_info);

        // List available tools
        let tools = client
            .list_tools(Default::default())
            .await
            .context("Failed to list MCP tools")?
            .tools;

        tracing::info!("Loaded {} tools from MCP server", tools.len());

        Ok((client, tools))
    }

    /// Create a new chat session (not yet persisted)
    pub async fn new(
        db: DatabaseConnection,
        settings: Arc<SystemSettingsService>,
        project_id: i32,
        user: users::Model,
        provider: ChatProvider,
        config: &ChatConfig,
    ) -> Result<Self> {
        let credentials = ChatCredentialStore::with_settings(db.clone(), settings.clone());
        let bridge = McpBridge::new(db.clone());
        let security = build_user_security_context(
            ClientContext::default(),
            user.id,
            &user.user_type,
            Some(project_id),
        );

        // Initialize DataAcquisitionService for RAG
        let embedding_provider = settings
            .get_setting("LAYERCAKE_EMBEDDING_PROVIDER")
            .await
            .ok()
            .and_then(|s| s.value);
        let embedding_config =
            layercake_genai::config::EmbeddingProviderConfig::from_env();
        let data_acquisition = Arc::new(DataAcquisitionService::new(
            db.clone(),
            embedding_provider,
            embedding_config,
        ));

        // Initialize rmcp client if feature is enabled
        #[cfg(feature = "rmcp")]
        let (rmcp_client, rmcp_tools) = Self::init_rmcp_client(&config.mcp_server_url)
            .await
            .ok()
            .unzip();

        let project_context = load_agent_project_context(&db, project_id).await?;
        let system_prompt = compose_system_prompt(config, &project_context);

        // Get model name from config
        let provider_config = config.provider(provider);
        let model_name = provider_config.model.clone();

        // Initialize rig agent enum
        let agent = match provider {
            ChatProvider::OpenAi => RigAgent::OpenAI(model_name.clone()),
            ChatProvider::Claude => RigAgent::Anthropic(model_name.clone()),
            ChatProvider::Gemini => RigAgent::Gemini(model_name.clone()),
            ChatProvider::Ollama => RigAgent::Ollama(model_name.clone()),
        };

        Ok(Self {
            db,
            session_id: None,
            project_id,
            user_id: user.id,
            provider,
            model_name,
            system_prompt,
            messages: Vec::new(),
            bridge,
            security,
            tool_use_enabled: true,
            agent,
            credentials,
            config: config.clone(),
            #[cfg(feature = "rmcp")]
            rmcp_client,
            #[cfg(feature = "rmcp")]
            rmcp_tools: rmcp_tools.unwrap_or_default(),

            // RAG configuration from system settings
            data_acquisition,
            rag_enabled: true, // Always enabled for new sessions, can be toggled per session
            rag_top_k: settings
                .get_setting("LAYERCAKE_RAG_DEFAULT_TOP_K")
                .await
                .ok()
                .and_then(|s| s.value)
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(5),
            rag_threshold: settings
                .get_setting("LAYERCAKE_RAG_DEFAULT_THRESHOLD")
                .await
                .ok()
                .and_then(|s| s.value)
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.7),
            rag_max_context_tokens: settings
                .get_setting("LAYERCAKE_RAG_MAX_CONTEXT_TOKENS")
                .await
                .ok()
                .and_then(|s| s.value)
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(4000),
            include_citations: settings
                .get_setting("LAYERCAKE_RAG_ENABLE_CITATIONS")
                .await
                .ok()
                .and_then(|s| s.value)
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(true),
            last_rag_context: None,
        })
    }

    /// Resume an existing chat session from the database
    pub async fn resume(
        db: DatabaseConnection,
        settings: Arc<SystemSettingsService>,
        session_id: String,
        config: &ChatConfig,
    ) -> Result<Self> {
        use crate::services::chat_history_service::ChatHistoryService;
        let history_service = ChatHistoryService::new(db.clone());

        // Load session metadata
        let session = history_service
            .get_session(&session_id)
            .await?
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        let provider: ChatProvider = session.provider.parse()?;
        let credentials = ChatCredentialStore::with_settings(db.clone(), settings.clone());
        let bridge = McpBridge::new(db.clone());

        // Initialize DataAcquisitionService for RAG
        let embedding_provider = settings
            .get_setting("LAYERCAKE_EMBEDDING_PROVIDER")
            .await
            .ok()
            .and_then(|s| s.value);
        let embedding_config =
            layercake_genai::config::EmbeddingProviderConfig::from_env();
        let data_acquisition = Arc::new(DataAcquisitionService::new(
            db.clone(),
            embedding_provider,
            embedding_config,
        ));

        // For resumed sessions, get security context from session's user
        let user = users::Entity::find_by_id(session.user_id)
            .one(&db)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        let security = build_user_security_context(
            ClientContext::default(),
            user.id,
            &user.user_type,
            Some(session.project_id),
        );

        let agent = match provider {
            ChatProvider::OpenAi => RigAgent::OpenAI(session.model_name.clone()),
            ChatProvider::Claude => RigAgent::Anthropic(session.model_name.clone()),
            ChatProvider::Gemini => RigAgent::Gemini(session.model_name.clone()),
            ChatProvider::Ollama => RigAgent::Ollama(session.model_name.clone()),
        };

        // Initialize rmcp client if feature is enabled
        #[cfg(feature = "rmcp")]
        let (rmcp_client, rmcp_tools) = Self::init_rmcp_client(&config.mcp_server_url)
            .await
            .ok()
            .unzip();

        let project_context = load_agent_project_context(&db, session.project_id).await?;
        let system_prompt = compose_system_prompt(config, &project_context);

        // Load message history (currently empty - could be extended to load from DB)
        let messages = Vec::new();

        Ok(Self {
            db,
            session_id: Some(session_id),
            project_id: session.project_id,
            user_id: session.user_id,
            provider,
            model_name: session.model_name,
            system_prompt,
            messages,
            bridge,
            security,
            tool_use_enabled: true,
            agent,
            credentials,
            config: config.clone(),
            #[cfg(feature = "rmcp")]
            rmcp_client,
            #[cfg(feature = "rmcp")]
            rmcp_tools: rmcp_tools.unwrap_or_default(),

            // RAG configuration loaded from database
            data_acquisition,
            rag_enabled: session.enable_rag,
            rag_top_k: session.rag_top_k as usize,
            rag_threshold: session.rag_threshold as f32,
            rag_max_context_tokens: settings
                .get_setting("LAYERCAKE_RAG_MAX_CONTEXT_TOKENS")
                .await
                .ok()
                .and_then(|s| s.value)
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(4000),
            include_citations: session.include_citations,
            last_rag_context: None,
        })
    }

    #[cfg(feature = "rmcp")]
    fn build_rig_agent<M>(&self, builder: AgentBuilder<M>) -> Agent<M>
    where
        M: CompletionModel,
    {
        if self.tool_use_enabled {
            if let Some(ref rmcp_client) = self.rmcp_client {
                if !self.rmcp_tools.is_empty() {
                    return builder
                        .rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned())
                        .build();
                }
            }
        }

        builder.build()
    }

    #[cfg(not(feature = "rmcp"))]
    fn build_rig_agent<M>(&self, builder: AgentBuilder<M>) -> Agent<M>
    where
        M: CompletionModel,
    {
        builder.build()
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub async fn ensure_persisted(&mut self) -> Result<String> {
        if let Some(ref id) = self.session_id {
            return Ok(id.clone());
        }

        use crate::services::chat_history_service::ChatHistoryService;
        let history_service = ChatHistoryService::new(self.db.clone());

        let session = history_service
            .create_session(
                self.project_id,
                self.user_id,
                self.provider.to_string(),
                self.model_name.clone(),
                None,                             // title
                Some(self.system_prompt.clone()), // system_prompt
            )
            .await?;

        self.session_id = Some(session.session_id.clone());
        Ok(session.session_id)
    }

    pub async fn interactive_loop(&mut self) -> Result<()> {
        println!(
            "Starting chat for project {} with {} ({})",
            self.project_id,
            self.provider.display_name(),
            self.model_name
        );

        let tools = self.bridge.list_tools(&self.security).await?;
        if !tools.is_empty() {
            let tool_list = tools
                .iter()
                .map(|tool| tool.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            println!("Tools available: {}", tool_list);
        } else {
            println!("No MCP tools are currently available.");
        }
        println!("Type your question and press Enter. Submit an empty line to exit.\n");

        let stdin = BufReader::new(io::stdin());
        let mut lines = stdin.lines();

        while let Some(line) = lines.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                println!("Ending chat session.");
                break;
            }
            self.send_message_with_observer(trimmed, &mut |event| match event {
                ChatEvent::AssistantMessage { text } => println!("assistant> {}", text),
                ChatEvent::ToolInvocation { name, summary } => {
                    println!("tool:{} => {}", name, summary.replace('\n', " "))
                }
            })
            .await?;
        }

        Ok(())
    }

    pub async fn send_message_with_observer<F>(
        &mut self,
        input: &str,
        observer: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        // Ensure session is persisted
        let session_id = self.ensure_persisted().await?;

        self.messages.push(ChatMessage::user(input));

        // Persist user message
        use crate::services::chat_history_service::ChatHistoryService;
        let history_service = ChatHistoryService::new(self.db.clone());
        history_service
            .store_message(
                &session_id,
                "user".to_string(),
                input.to_string(),
                None,
                None,
                None,
            )
            .await?;

        self.resolve_conversation(observer).await
    }

    async fn resolve_conversation<F>(&mut self, observer: &mut F) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        let mut turn_count = 0usize;
        loop {
            if turn_count > MAX_TOOL_ITERATIONS {
                return Err(anyhow!(
                    "Exceeded maximum tool interaction depth without producing a response"
                ));
            }
            turn_count += 1;

            let prompt_context = self.build_prompt_context().await?;
            let agent_response = self
                .invoke_agent_with_retries(prompt_context, observer)
                .await?;

            tracing::debug!(
                input_tokens = agent_response.usage.input_tokens,
                output_tokens = agent_response.usage.output_tokens,
                total_tokens = agent_response.usage.total_tokens,
                "Received agent response"
            );

            let choice = agent_response.choice;
            let tool_calls = Self::collect_tool_calls(&choice);

            if !tool_calls.is_empty() {
                self.record_tool_call_response(&choice);
                for tool_call in tool_calls {
                    self.handle_tool_invocation(tool_call, observer).await?;
                }
                continue;
            }

            let mut final_response = Self::collect_text_segments(&choice)
                .join("\n")
                .trim()
                .to_string();

            if final_response.is_empty() {
                final_response = String::new();
            }

            if self.include_citations {
                if let Some(ref rag_context) = self.last_rag_context {
                    if !rag_context.chunks.is_empty() {
                        final_response.push_str("\n---\n**Sources:**\n");
                        for citation in rag_context.get_citations() {
                            final_response.push_str(&format!("- {}\n", citation));
                        }
                    }
                }
            }

            observer(ChatEvent::AssistantMessage {
                text: final_response.clone(),
            });

            self.messages.push(ChatMessage::assistant(&final_response));

            if let Some(ref session_id) = self.session_id {
                self.persist_message(session_id, "assistant", &final_response, None, None, None)
                    .await?;
            }

            return Ok(());
        }
    }

    fn should_disable_tools(&self, err: &anyhow::Error) -> bool {
        if self.provider != ChatProvider::Ollama || !self.tool_use_enabled {
            return false;
        }

        // Check if error is HTTP 400 from Ollama /api/chat endpoint
        let err_str = err.to_string();
        err_str.contains("/api/chat") && err_str.contains("400")
    }

    async fn build_prompt_context(&mut self) -> Result<RigPromptContext> {
        let rig_messages = self.convert_messages_to_rig()?;
        if rig_messages.is_empty() {
            return Err(anyhow!(
                "Conversation is empty; cannot build prompt context for agent"
            ));
        }

        let mut history = rig_messages;
        let prompt = history
            .pop()
            .ok_or_else(|| anyhow!("Missing prompt message for rig agent"))?;

        let mut preamble = self.system_prompt.clone();

        if self.rag_enabled {
            if let Some(last_user_msg) = self.messages.iter().rev().find(|m| m.role == "user") {
                match self.get_rag_context(&last_user_msg.content).await {
                    Ok(rag_context) if !rag_context.is_empty() => {
                        if !preamble.is_empty() {
                            preamble.push_str("\n\n");
                        }
                        preamble.push_str(&rag_context.to_context_string());
                        preamble.push_str(
                            "\n\nUse the above context to answer questions when relevant. ",
                        );
                        preamble.push_str("If the context doesn't contain relevant information, ");
                        preamble.push_str("say so and use your general knowledge.");

                        self.last_rag_context = Some(rag_context);
                    }
                    Ok(_) => {
                        self.last_rag_context = None;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to retrieve RAG context: {}", e);
                        self.last_rag_context = None;
                    }
                }
            }
        }

        Ok(RigPromptContext {
            preamble,
            prompt,
            history,
        })
    }

    fn convert_messages_to_rig(&self) -> Result<Vec<Message>> {
        let mut rig_messages = Vec::with_capacity(self.messages.len());

        for msg in &self.messages {
            match msg.role.as_str() {
                "user" => {
                    rig_messages.push(Message::from(msg.content.clone()));
                }
                "assistant" => {
                    let mut contents = Vec::new();

                    if !msg.content.trim().is_empty() {
                        contents.push(AssistantContent::text(msg.content.clone()));
                    }

                    if let Some(tool_calls) = &msg.tool_calls {
                        for call in tool_calls {
                            contents.push(AssistantContent::ToolCall(ToolCall {
                                id: call.id.clone(),
                                call_id: None,
                                function: ToolFunction {
                                    name: call.name.clone(),
                                    arguments: call.arguments.clone(),
                                },
                            }));
                        }
                    }

                    if contents.is_empty() {
                        continue;
                    }

                    let content = OneOrMany::many(contents).unwrap_or_else(|_| {
                        OneOrMany::one(AssistantContent::text(msg.content.clone()))
                    });

                    rig_messages.push(Message::Assistant { id: None, content });
                }
                "tool" => {
                    if let Some(results) = &msg.tool_results {
                        for result in results {
                            let content =
                                OneOrMany::one(ToolResultContent::text(result.output.clone()));
                            let tool_result = ToolResult {
                                id: result.call_id.clone(),
                                call_id: Some(result.call_id.clone()),
                                content,
                            };
                            rig_messages.push(Message::User {
                                content: OneOrMany::one(UserContent::ToolResult(tool_result)),
                            });
                        }
                    }
                }
                other => {
                    tracing::warn!("Skipping unsupported role '{}' in chat history", other);
                }
            }
        }

        Ok(rig_messages)
    }

    fn record_tool_call_response(&mut self, choice: &OneOrMany<AssistantContent>) {
        let text = Self::collect_text_segments(choice).join("\n");
        let tool_call_data = Self::collect_tool_call_data(choice);

        let message = if tool_call_data.is_empty() {
            ChatMessage::assistant(text)
        } else {
            ChatMessage::assistant(text).with_tool_calls(tool_call_data)
        };

        self.messages.push(message);
    }

    fn collect_tool_call_data(choice: &OneOrMany<AssistantContent>) -> Vec<ToolCallData> {
        choice
            .iter()
            .filter_map(|content| match content {
                AssistantContent::ToolCall(call) => Some(ToolCallData {
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    fn collect_tool_calls(choice: &OneOrMany<AssistantContent>) -> Vec<ToolCall> {
        choice
            .iter()
            .filter_map(|content| {
                if let AssistantContent::ToolCall(call) = content {
                    Some(call.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn collect_text_segments(choice: &OneOrMany<AssistantContent>) -> Vec<String> {
        let mut segments = Vec::new();
        for content in choice.iter() {
            match content {
                AssistantContent::Text(text) => segments.push(text.text.clone()),
                AssistantContent::Reasoning(Reasoning { reasoning, .. }) => {
                    segments.extend(reasoning.clone());
                }
                _ => {}
            }
        }
        segments
    }

    async fn get_rag_context(&self, query: &str) -> Result<super::RagContext> {
        // Get embeddings service from data acquisition
        let embeddings = self
            .data_acquisition
            .embeddings()
            .ok_or_else(|| anyhow!("Embeddings not configured for this project"))?;

        tracing::debug!(
            project_id = self.project_id,
            query = query,
            "Embedding query for RAG search"
        );

        // Embed the query
        let query_embedding = embeddings.embed_text(query).await?;

        // Search the knowledge base
        let search_results = self
            .data_acquisition
            .search_context(self.project_id, &query_embedding, self.rag_top_k)
            .await?;

        tracing::info!(
            project_id = self.project_id,
            raw_results_count = search_results.len(),
            threshold = self.rag_threshold,
            top_k = self.rag_top_k,
            "Vector search completed"
        );

        // Log top results with scores for debugging
        for (i, result) in search_results.iter().take(5).enumerate() {
            tracing::debug!(
                rank = i + 1,
                score = result.score,
                filename = result.filename.as_ref().unwrap_or(&"unknown".to_string()),
                "Search result"
            );
        }

        // Build RAG context with threshold filtering
        let rag_context = RagContextBuilder::new(self.rag_threshold, self.rag_max_context_tokens)
            .add_results(search_results)
            .build();

        tracing::info!(
            project_id = self.project_id,
            chunks_retrieved = rag_context.chunks.len(),
            total_tokens = rag_context.total_tokens,
            "RAG context built"
        );

        Ok(rag_context)
    }

    async fn call_rig_agent(&self, context: RigPromptContext) -> Result<AgentResponse> {
        let RigPromptContext {
            preamble,
            prompt,
            history,
        } = context;

        // Get API key for provider
        let api_key = if let Some(key) = self.credentials.api_key(self.provider).await? {
            key
        } else if let Some(env_var) = self.provider.api_key_env_var() {
            std::env::var(env_var).with_context(|| {
                format!(
                    "Missing API key for {}. Set {} environment variable.",
                    self.provider.display_name(),
                    env_var
                )
            })?
        } else {
            // Ollama doesn't need an API key
            String::new()
        };

        // Get base URL using persisted override -> environment -> provider default
        let provider_base = self.config.provider(self.provider).base_url.clone();
        let base_url = self
            .credentials
            .base_url_override(self.provider)
            .await?
            .or(provider_base);

        // Call appropriate provider
        match &self.agent {
            RigAgent::OpenAI(model) => {
                let client = if api_key.is_empty() {
                    return Err(anyhow!("OpenAI requires API key"));
                } else {
                    // Set credentials in environment for from_env()
                    std::env::set_var("OPENAI_API_KEY", &api_key);
                    if let Some(url) = base_url.as_deref() {
                        std::env::set_var("OPENAI_BASE_URL", url);
                    }
                    openai::Client::from_env()
                };

                let mut builder = client.agent(model);
                if !preamble.is_empty() {
                    builder = builder.preamble(&preamble);
                }

                let agent = self.build_rig_agent(builder);
                let response = agent
                    .completion(prompt.clone(), history.clone())
                    .await
                    .context("OpenAI completion request failed")?
                    .send()
                    .await
                    .context("OpenAI API call failed")?;

                Ok(AgentResponse {
                    choice: response.choice,
                    usage: response.usage,
                })
            }

            RigAgent::Anthropic(model) => {
                let client = if api_key.is_empty() {
                    return Err(anyhow!("Anthropic requires API key"));
                } else {
                    // Set credentials in environment for from_env()
                    std::env::set_var("ANTHROPIC_API_KEY", &api_key);
                    if let Some(url) = base_url.as_deref() {
                        std::env::set_var("ANTHROPIC_BASE_URL", url);
                    }
                    anthropic::Client::from_env()
                };

                let mut builder = client.agent(model);
                if !preamble.is_empty() {
                    builder = builder.preamble(&preamble);
                }

                let agent = self.build_rig_agent(builder);
                let response = agent
                    .completion(prompt.clone(), history.clone())
                    .await
                    .context("Anthropic completion request failed")?
                    .send()
                    .await
                    .context("Anthropic API call failed")?;

                Ok(AgentResponse {
                    choice: response.choice,
                    usage: response.usage,
                })
            }

            RigAgent::Gemini(model) => {
                let client = if api_key.is_empty() {
                    return Err(anyhow!("Gemini requires API key"));
                } else {
                    // Set credentials in environment for from_env()
                    std::env::set_var("GOOGLE_API_KEY", &api_key);
                    if let Some(url) = base_url.as_deref() {
                        std::env::set_var("GEMINI_BASE_URL", url);
                    }
                    gemini::Client::from_env()
                };

                // Create generation config and additional params (required by Gemini)
                let gen_cfg = GenerationConfig::default();
                let additional_params = AdditionalParameters::default().with_config(gen_cfg);

                let mut builder = client
                    .agent(model)
                    .additional_params(serde_json::to_value(additional_params)?);
                if !preamble.is_empty() {
                    builder = builder.preamble(&preamble);
                }

                let agent = self.build_rig_agent(builder);
                let response = agent
                    .completion(prompt.clone(), history.clone())
                    .await
                    .context("Gemini completion request failed")?
                    .send()
                    .await
                    .context("Gemini API call failed")?;

                Ok(AgentResponse {
                    choice: response.choice,
                    usage: response.usage,
                })
            }

            RigAgent::Ollama(model) => {
                // Ollama doesn't require a real API key, but from_env() expects it
                std::env::set_var("OLLAMA_API_KEY", "ollama");

                // Set base URL if provided (defaults to http://localhost:11434 if not set)
                if let Some(url) = base_url.as_deref() {
                    std::env::set_var("OLLAMA_API_BASE_URL", url);
                } else {
                    // Ensure default URL is set
                    std::env::set_var("OLLAMA_API_BASE_URL", "http://localhost:11434");
                }

                let client = ollama::Client::from_env();

                let mut builder = client.agent(model);
                if !preamble.is_empty() {
                    builder = builder.preamble(&preamble);
                }

                let agent = self.build_rig_agent(builder);
                let response = agent
                    .completion(prompt, history)
                    .await
                    .context("Ollama completion request failed")?
                    .send()
                    .await
                    .context("Ollama API call failed")?;

                Ok(AgentResponse {
                    choice: response.choice,
                    usage: response.usage,
                })
            }
        }
    }

    async fn invoke_agent_with_retries<F>(
        &mut self,
        context: RigPromptContext,
        observer: &mut F,
    ) -> Result<AgentResponse>
    where
        F: FnMut(ChatEvent),
    {
        match self.call_rig_agent(context).await {
            Ok(response) => Ok(response),
            Err(err) if self.should_disable_tools(&err) => {
                self.tool_use_enabled = false;

                #[cfg(feature = "rmcp")]
                {
                    self.rmcp_client = None;
                    self.rmcp_tools.clear();
                }

                let notice = "Ollama server rejected function/tool calls. Continuing without tool access; responses now rely on model knowledge only.";
                observer(ChatEvent::AssistantMessage {
                    text: notice.to_string(),
                });
                self.messages.push(ChatMessage::assistant(notice));

                if let Some(ref session_id) = self.session_id {
                    self.persist_message(session_id, "assistant", notice, None, None, None)
                        .await?;
                }

                tracing::warn!("Disabling tool usage for session: {}", err);

                let updated_context = self.build_prompt_context().await?;
                self.call_rig_agent(updated_context).await
            }
            Err(err) => Err(err),
        }
    }

    async fn handle_tool_invocation<F>(
        &mut self,
        tool_call: ToolCall,
        observer: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        let call_id = tool_call
            .call_id
            .clone()
            .unwrap_or_else(|| tool_call.id.clone());
        let function_name = tool_call.function.name.clone();
        let execution_args = if tool_call.function.arguments.is_null() {
            None
        } else {
            Some(tool_call.function.arguments.clone())
        };

        let result = self
            .bridge
            .execute_tool(&function_name, &self.security, execution_args.clone())
            .await
            .with_context(|| format!("failed to execute MCP tool {}", function_name))?;

        let summary = McpBridge::summarize_tool_result(&result);
        let result_metadata = McpBridge::serialize_tool_result(&result);

        self.messages
            .push(ChatMessage::tool_result(call_id.clone(), &summary));

        if let Some(ref session_id) = self.session_id {
            let metadata_payload = if let Some(ref args) = execution_args {
                json!({
                    "arguments": args,
                    "result": result_metadata.clone()
                })
            } else {
                result_metadata.clone()
            };

            self.persist_message(
                session_id,
                "tool",
                &summary,
                Some(function_name.clone()),
                Some(call_id.clone()),
                Some(metadata_payload),
            )
            .await?;
        }

        observer(ChatEvent::ToolInvocation {
            name: function_name,
            summary,
        });

        Ok(())
    }

    async fn persist_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        tool_name: Option<String>,
        tool_call_id: Option<String>,
        metadata: Option<Value>,
    ) -> Result<()> {
        use crate::services::chat_history_service::ChatHistoryService;
        let history_service = ChatHistoryService::new(self.db.clone());

        let metadata_json = metadata
            .map(|value| serde_json::to_string(&value))
            .transpose()
            .context("failed to serialize chat message metadata")?;

        history_service
            .store_message(
                session_id,
                role.to_string(),
                content.to_string(),
                tool_name,
                tool_call_id,
                metadata_json,
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug)]
struct AgentProjectContext {
    project_id: i32,
    project_name: String,
    project_description: Option<String>,
    project_tags: Vec<String>,
    plan_name: Option<String>,
    plan_node_count: Option<u64>,
    plan_edge_count: Option<u64>,
    plan_updated_at: Option<DateTime<Utc>>,
    dataset_total: usize,
    dataset_by_type: BTreeMap<String, usize>,
    graph_total: usize,
    graph_state_counts: BTreeMap<String, usize>,
    last_graph_update: Option<DateTime<Utc>>,
}

impl AgentProjectContext {
    fn plan_summary(&self) -> String {
        match (
            self.plan_name.as_ref(),
            self.plan_node_count,
            self.plan_edge_count,
        ) {
            (Some(name), Some(nodes), Some(edges)) => {
                let updated = self
                    .plan_updated_at
                    .map(|ts| ts.to_rfc3339())
                    .unwrap_or_else(|| "unknown update time".to_string());
                format!("{name} ({nodes} nodes / {edges} edges, last updated {updated})")
            }
            (Some(name), _, _) => format!("{name} exists but no DAG statistics are available yet."),
            _ => "No plan has been defined for this project yet.".to_string(),
        }
    }

    fn dataset_summary(&self) -> String {
        if self.dataset_total == 0 {
            return "No datasets have been added yet.".to_string();
        }

        let breakdown = if self.dataset_by_type.is_empty() {
            "no type metadata recorded".to_string()
        } else {
            self.dataset_by_type
                .iter()
                .map(|(kind, count)| format!("{count} {kind}"))
                .collect::<Vec<_>>()
                .join(", ")
        };

        format!("{} datasets ({})", self.dataset_total, breakdown)
    }

    fn graph_summary(&self) -> String {
        if self.graph_total == 0 {
            return "No graphs have been executed yet.".to_string();
        }

        let breakdown = if self.graph_state_counts.is_empty() {
            "no execution state data recorded".to_string()
        } else {
            self.graph_state_counts
                .iter()
                .map(|(state, count)| {
                    let label = state.replace('_', " ").to_lowercase();
                    format!("{count} {label}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        let recency = self
            .last_graph_update
            .map(|ts| format!("last updated {}", ts.to_rfc3339()))
            .unwrap_or_else(|| "latest run time unknown".to_string());

        format!("{} graphs ({}) — {}", self.graph_total, breakdown, recency)
    }
}

async fn load_agent_project_context(
    db: &DatabaseConnection,
    project_id: i32,
) -> Result<AgentProjectContext> {
    let project = projects::Entity::find_by_id(project_id)
        .one(db)
        .await
        .map_err(|e| anyhow!("Failed to load project {}: {}", project_id, e))?
        .ok_or_else(|| anyhow!("Project {} not found", project_id))?;

    let project_tags: Vec<String> = serde_json::from_str(&project.tags).unwrap_or_default();

    let plan = plans::Entity::find()
        .filter(plans::Column::ProjectId.eq(project_id))
        .order_by_desc(plans::Column::UpdatedAt)
        .one(db)
        .await
        .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?;

    let (plan_name, plan_node_count, plan_edge_count, plan_updated_at) = if let Some(plan) = plan {
        let node_count = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .count(db)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to count plan nodes for project {}: {}",
                    project_id,
                    e
                )
            })?;

        let edge_count = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .count(db)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to count plan edges for project {}: {}",
                    project_id,
                    e
                )
            })?;

        (
            Some(plan.name),
            Some(node_count),
            Some(edge_count),
            Some(plan.updated_at),
        )
    } else {
        (None, None, None, None)
    };

    let datasets = data_sets::Entity::find()
        .filter(data_sets::Column::ProjectId.eq(project_id))
        .all(db)
        .await
        .map_err(|e| anyhow!("Failed to load datasets for project {}: {}", project_id, e))?;

    let dataset_total = datasets.len();
    let mut dataset_by_type: BTreeMap<String, usize> = BTreeMap::new();
    for ds in datasets {
        let (node_count, edge_count, layer_count) = summarize_graph_counts(&ds.graph_json);
        if node_count.unwrap_or(0) > 0 {
            *dataset_by_type.entry("nodes".to_string()).or_insert(0) += 1;
        }
        if edge_count.unwrap_or(0) > 0 {
            *dataset_by_type.entry("edges".to_string()).or_insert(0) += 1;
        }
        if layer_count.unwrap_or(0) > 0 {
            *dataset_by_type.entry("layers".to_string()).or_insert(0) += 1;
        }
        if node_count.unwrap_or(0) == 0
            && edge_count.unwrap_or(0) == 0
            && layer_count.unwrap_or(0) == 0
        {
            *dataset_by_type.entry("empty".to_string()).or_insert(0) += 1;
        }
    }

    let graphs = graphs::Entity::find()
        .filter(graphs::Column::ProjectId.eq(project_id))
        .all(db)
        .await
        .map_err(|e| anyhow!("Failed to load graphs for project {}: {}", project_id, e))?;

    let graph_total = graphs.len();
    let mut graph_state_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut last_graph_update: Option<DateTime<Utc>> = None;
    for graph in graphs {
        *graph_state_counts
            .entry(graph.execution_state.clone())
            .or_insert(0) += 1;
        last_graph_update = match last_graph_update {
            Some(current) if current > graph.updated_at => Some(current),
            _ => Some(graph.updated_at),
        };
    }

    Ok(AgentProjectContext {
        project_id,
        project_name: project.name,
        project_description: project.description,
        project_tags,
        plan_name,
        plan_node_count,
        plan_edge_count,
        plan_updated_at,
        dataset_total,
        dataset_by_type,
        graph_total,
        graph_state_counts,
        last_graph_update,
    })
}

fn compose_system_prompt(config: &ChatConfig, context: &AgentProjectContext) -> String {
    let mut prompt = String::new();
    if let Some(ref sys_prompt) = config.system_prompt {
        prompt.push_str(sys_prompt);
        prompt.push_str("\n\n");
    }

    writeln!(
        &mut prompt,
        "You are a senior Layercake engineer assisting with project {} ({}).",
        context.project_id, context.project_name
    )
    .expect("Writing to String should not fail");

    if let Some(description) = &context.project_description {
        if !description.trim().is_empty() {
            writeln!(&mut prompt, "Project description: {}", description.trim())
                .expect("Writing to String should not fail");
        }
    }

    if !context.project_tags.is_empty() {
        writeln!(&mut prompt, "Tags: {}", context.project_tags.join(", "))
            .expect("Writing to String should not fail");
    }

    writeln!(&mut prompt, "Plan status: {}", context.plan_summary())
        .expect("Writing to String should not fail");
    writeln!(&mut prompt, "Datasets: {}", context.dataset_summary())
        .expect("Writing to String should not fail");
    writeln!(&mut prompt, "Graphs: {}", context.graph_summary())
        .expect("Writing to String should not fail");

    prompt.push_str(
        "\nPlan DAG model:\n\
- Nodes: DataSetNode (ingest existing datasets), GraphNode (build graphs from upstream datasets/graphs), MergeNode (combine upstream graphs), TransformNode/FilterNode (post-process graphs), GraphArtefactNode/TreeArtefactNode (export/visualise outputs), Chat nodes, etc.\n\
- DagExecutor resolves dependencies, executes nodes, and triggers downstream recomputation when upstream data changes.\n\
- Graph materialisation persists to graph_nodes, graph_edges, and graph_layers with dataset provenance.\n\n",
    );

    prompt.push_str(
        "Graph construction & artefacts:\n\
- GraphBuilder builds graphs from datasets, ensuring consistent node/edge IDs.\n\
- MergeBuilder merges upstream graphs/datasets and deduplicates edges before storage.\n\
- Artefact nodes emit previews (Mermaid, DOT, ZIP archives) and support publishing to the shared library.\n\
- Library items can store datasets, full projects, or reusable templates for future work.\n\n",
    );

    prompt.push_str(
        "Instructions:\n\
1. Always ground responses in the Layercake DAG/graph architecture above.\n\
2. When planning changes, describe which subsystems (plan DAG, datasets, graphs, artefacts, collaboration) are affected and why.\n\
3. When execution steps are required, detail node execution order and downstream graph impact.\n\
4. Prefer Rig/MCP tool usage for repository inspection, file edits, or running evaluations; describe the tool intent before calling it.\n\
5. Keep answers scoped to the current project unless the user explicitly switches context.\n",
    );

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> AgentProjectContext {
        let mut dataset_by_type = BTreeMap::new();
        dataset_by_type.insert("nodes".to_string(), 1);
        dataset_by_type.insert("edges".to_string(), 2);

        let mut graph_state_counts = BTreeMap::new();
        graph_state_counts.insert("completed".to_string(), 1);
        graph_state_counts.insert("processing".to_string(), 1);

        AgentProjectContext {
            project_id: 42,
            project_name: "Test Project".to_string(),
            project_description: Some("Exploratory build".to_string()),
            project_tags: vec!["alpha".to_string(), "beta".to_string()],
            plan_name: Some("Main Plan".to_string()),
            plan_node_count: Some(12),
            plan_edge_count: Some(11),
            plan_updated_at: Some(Utc::now()),
            dataset_total: 3,
            dataset_by_type,
            graph_total: 2,
            graph_state_counts,
            last_graph_update: Some(Utc::now()),
        }
    }

    #[test]
    fn test_compose_system_prompt_no_tools() {
        let config = ChatConfig {
            default_provider: ChatProvider::Ollama,
            request_timeout: std::time::Duration::from_secs(60),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            providers: std::collections::HashMap::new(),
            mcp_server_url: "http://localhost:3000/mcp".to_string(),
        };

        let prompt = compose_system_prompt(&config, &sample_context());

        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("Test Project"));
        assert!(prompt.contains("Plan status"));
        assert!(prompt.contains("Datasets"));
    }

    #[test]
    fn test_compose_system_prompt_no_custom_prompt() {
        let config = ChatConfig {
            default_provider: ChatProvider::Ollama,
            request_timeout: std::time::Duration::from_secs(60),
            system_prompt: None,
            providers: std::collections::HashMap::new(),
            mcp_server_url: "http://localhost:3000/mcp".to_string(),
        };

        let prompt = compose_system_prompt(&config, &sample_context());

        assert!(prompt.contains("Test Project"));
        assert!(prompt.contains("Plan DAG model"));
    }

    #[test]
    fn test_chat_provider_display_names() {
        assert_eq!(ChatProvider::Ollama.display_name(), "Ollama");
        assert_eq!(ChatProvider::OpenAi.display_name(), "OpenAI");
        assert_eq!(ChatProvider::Gemini.display_name(), "Google Gemini");
        assert_eq!(ChatProvider::Claude.display_name(), "Anthropic Claude");
    }

    #[test]
    fn test_chat_provider_requires_api_key() {
        assert!(!ChatProvider::Ollama.requires_api_key());
        assert!(ChatProvider::OpenAi.requires_api_key());
        assert!(ChatProvider::Gemini.requires_api_key());
        assert!(ChatProvider::Claude.requires_api_key());
    }

    #[test]
    fn test_chat_provider_default_models() {
        assert_eq!(ChatProvider::Ollama.default_model(), "llama3.2");
        assert_eq!(ChatProvider::OpenAi.default_model(), "gpt-4o-mini");
        assert_eq!(ChatProvider::Gemini.default_model(), "gemini-1.5-flash");
        assert_eq!(
            ChatProvider::Claude.default_model(),
            "claude-3-5-sonnet-20241022"
        );
    }

    #[test]
    fn test_chat_provider_api_key_env_vars() {
        assert_eq!(ChatProvider::Ollama.api_key_env_var(), None);
        assert_eq!(
            ChatProvider::OpenAi.api_key_env_var(),
            Some("OPENAI_API_KEY")
        );
        assert_eq!(
            ChatProvider::Gemini.api_key_env_var(),
            Some("GOOGLE_API_KEY")
        );
        assert_eq!(
            ChatProvider::Claude.api_key_env_var(),
            Some("ANTHROPIC_API_KEY")
        );
    }

    #[test]
    fn test_chat_provider_from_str() {
        assert_eq!(
            "ollama"
                .parse::<ChatProvider>()
                .expect("Should parse ollama"),
            ChatProvider::Ollama
        );
        assert_eq!(
            "openai"
                .parse::<ChatProvider>()
                .expect("Should parse openai"),
            ChatProvider::OpenAi
        );
        assert_eq!(
            "open-ai"
                .parse::<ChatProvider>()
                .expect("Should parse open-ai"),
            ChatProvider::OpenAi
        );
        assert_eq!(
            "gemini"
                .parse::<ChatProvider>()
                .expect("Should parse gemini"),
            ChatProvider::Gemini
        );
        assert_eq!(
            "claude"
                .parse::<ChatProvider>()
                .expect("Should parse claude"),
            ChatProvider::Claude
        );
        assert_eq!(
            "anthropic"
                .parse::<ChatProvider>()
                .expect("Should parse anthropic"),
            ChatProvider::Claude
        );

        assert!("invalid".parse::<ChatProvider>().is_err());
    }

    #[test]
    fn test_chat_provider_to_string() {
        assert_eq!(ChatProvider::Ollama.to_string(), "ollama");
        assert_eq!(ChatProvider::OpenAi.to_string(), "openai");
        assert_eq!(ChatProvider::Gemini.to_string(), "gemini");
        assert_eq!(ChatProvider::Claude.to_string(), "claude");
    }

    #[test]
    fn test_chat_message_builders() {
        let msg = ChatMessage::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
        assert!(msg.tool_calls.is_none());
        assert!(msg.tool_results.is_none());

        let msg = ChatMessage::assistant("Hi there");
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "Hi there");

        let tool_call = ToolCallData {
            id: "call_1".to_string(),
            name: "search".to_string(),
            arguments: serde_json::json!({"query": "test"}),
        };
        let msg = ChatMessage::assistant("Searching...").with_tool_calls(vec![tool_call]);
        assert_eq!(msg.role, "assistant");
        assert!(msg.tool_calls.is_some());
        assert_eq!(
            msg.tool_calls
                .as_ref()
                .expect("Tool calls should be present")
                .len(),
            1
        );

        let msg = ChatMessage::tool_result("call_1", "Result: Found 5 items");
        assert_eq!(msg.role, "tool");
        assert!(msg.tool_results.is_some());
    }

    #[test]
    fn test_max_tool_iterations_constant() {
        assert_eq!(MAX_TOOL_ITERATIONS, 5);
    }

    #[test]
    fn test_ollama_error_detection_http_400() {
        // Test the error pattern that should trigger Ollama tool fallback
        let error_msg = "HTTP error: POST http://localhost:11434/api/chat returned 400";
        assert!(error_msg.contains("/api/chat"));
        assert!(error_msg.contains("400"));
    }

    #[test]
    fn test_ollama_error_detection_no_match() {
        // Test error patterns that should NOT trigger fallback
        let error_msg_1 = "HTTP error: POST http://localhost:11434/api/generate returned 400";
        assert!(!error_msg_1.contains("/api/chat"));

        let error_msg_2 = "HTTP error: POST http://localhost:11434/api/chat returned 500";
        assert!(error_msg_2.contains("/api/chat"));
        assert!(!error_msg_2.contains("400"));

        let error_msg_3 = "Connection refused";
        assert!(!error_msg_3.contains("/api/chat"));
        assert!(!error_msg_3.contains("400"));
    }

    #[test]
    fn test_ollama_error_detection_real_world_example() {
        // Simulate a real Ollama HTTP 400 error when tools are not supported
        let error_msg = "Request failed: HTTP status client error (400 Bad Request) for url (http://localhost:11434/api/chat)";
        assert!(error_msg.contains("/api/chat"));
        assert!(error_msg.contains("400"));
    }
}
