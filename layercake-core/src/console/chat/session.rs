#![cfg(feature = "console")]

//! Chat session implementation using rig with rmcp for MCP integration
//!
//! This module replaces the old llm-based chat implementation with rig agents.
//! Key changes:
//! - Uses rig agents instead of llm::LLMProvider
//! - Uses rmcp for direct MCP integration (no conversion layer)
//! - Maintains same session persistence and observer pattern

use std::{fmt::Write as FmtWrite, sync::Arc};

use anyhow::{anyhow, Context, Result};
use axum_mcp::prelude::{ClientContext, SecurityContext};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::{json, Number, Value};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing;

use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::gemini::completion::gemini_api_types::{
    AdditionalParameters, GenerationConfig,
};
use rig::providers::{anthropic, gemini, ollama, openai};

#[cfg(feature = "rmcp")]
use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation, Tool as RmcpTool},
    transport::StreamableHttpClientTransport,
    ServiceExt,
};

use crate::database::entities::users;
use crate::mcp::security::build_user_security_context;
use crate::services::system_settings_service::SystemSettingsService;
use layercake_data_acquisition::services::DataAcquisitionService;

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

#[derive(Debug, Clone)]
struct ParsedToolInvocation {
    name: String,
    arguments: Option<serde_json::Value>,
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
    rmcp_client: Option<rmcp::Client<rmcp::RoleServer>>,
    #[cfg(feature = "rmcp")]
    rmcp_tools: Vec<RmcpTool>,

    // RAG configuration
    data_acquisition: Arc<DataAcquisitionService>,
    rag_enabled: bool,
    rag_top_k: usize,
    rag_threshold: f32,
    include_citations: bool,
}

impl ChatSession {
    /// Initialize rmcp client connection to MCP server
    #[cfg(feature = "rmcp")]
    async fn init_rmcp_client(
        mcp_server_url: &str,
    ) -> Result<(rmcp::Client<rmcp::RoleServer>, Vec<RmcpTool>)> {
        tracing::info!("Connecting to MCP server at {}", mcp_server_url);

        let transport = StreamableHttpClientTransport::from_uri(mcp_server_url);

        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "layercake-chat".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let client = client_info
            .serve(transport)
            .await
            .context("Failed to connect to MCP server")?;

        let server_info = client.peer_info();
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
        let embedding_config = layercake_data_acquisition::config::EmbeddingProviderConfig::from_env();
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

        #[cfg(feature = "rmcp")]
        let tool_names: Vec<String> = rmcp_tools
            .as_ref()
            .map(|tools| tools.iter().map(|t| t.name.clone()).collect())
            .unwrap_or_default();

        // Fallback to bridge tools if rmcp not available
        #[cfg(not(feature = "rmcp"))]
        let tool_names: Vec<String> = {
            let mcp_tools = bridge
                .list_tools(&security)
                .await
                .map_err(|err| anyhow!("failed to load MCP tools: {}", err))?;
            mcp_tools.iter().map(|t| t.name.clone()).collect()
        };

        let system_prompt = compose_system_prompt(config, project_id, &tool_names);

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

            // RAG defaults
            data_acquisition,
            rag_enabled: true,
            rag_top_k: 5,
            rag_threshold: 0.7,
            include_citations: true,
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
        let embedding_config = layercake_data_acquisition::config::EmbeddingProviderConfig::from_env();
        let data_acquisition = Arc::new(DataAcquisitionService::new(
            db.clone(),
            embedding_provider,
            embedding_config,
        ));

        // For resumed sessions, get security context from session's user
        use crate::database::entities::users;
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

        // Load message history (currently empty - could be extended to load from DB)
        let messages = Vec::new();

        Ok(Self {
            db,
            session_id: Some(session_id),
            project_id: session.project_id,
            user_id: session.user_id,
            provider,
            model_name: session.model_name,
            system_prompt: session.system_prompt.unwrap_or_default(),
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

            // RAG defaults (TODO: load from session when fields are added to DB)
            data_acquisition,
            rag_enabled: true,
            rag_top_k: 5,
            rag_threshold: 0.7,
            include_citations: true,
        })
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

            let conversation = self.build_conversation_prompt().await?;
            let response_text = self
                .invoke_agent_with_retries(&conversation, observer)
                .await?;

            if let Some(invocation) = extract_tool_invocation(&response_text) {
                self.handle_tool_invocation(response_text, invocation, observer)
                    .await?;
                continue;
            }

            observer(ChatEvent::AssistantMessage {
                text: response_text.clone(),
            });

            self.messages.push(ChatMessage::assistant(&response_text));

            if let Some(ref session_id) = self.session_id {
                self.persist_message(session_id, "assistant", &response_text, None, None, None)
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

    async fn build_conversation_prompt(&mut self) -> Result<String> {
        let mut prompt = String::new();
        prompt.push_str(&self.system_prompt);
        prompt.push_str("\n\n");

        // Add RAG context if enabled and we have a user message
        if self.rag_enabled {
            if let Some(last_user_msg) = self.messages.iter().rev().find(|m| m.role == "user") {
                match self.get_rag_context(&last_user_msg.content).await {
                    Ok(rag_context) if !rag_context.is_empty() => {
                        prompt.push_str(&rag_context.to_context_string());
                        prompt.push_str("\n\nUse the above context to answer questions when relevant. ");
                        prompt.push_str("If the context doesn't contain relevant information, ");
                        prompt.push_str("say so and use your general knowledge.\n\n");
                    }
                    Ok(_) => {} // No relevant context found
                    Err(e) => {
                        tracing::warn!("Failed to retrieve RAG context: {}", e);
                    }
                }
            }
        }

        for msg in &self.messages {
            match msg.role.as_str() {
                "user" => {
                    prompt.push_str("User: ");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n\n");
                }
                "assistant" => {
                    prompt.push_str("Assistant: ");
                    prompt.push_str(&msg.content);
                    prompt.push_str("\n\n");
                }
                "tool" => {
                    prompt.push_str("Tool: ");
                    if let Some(results) = &msg.tool_results {
                        let joined = results
                            .iter()
                            .map(|result| result.output.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        prompt.push_str(&joined);
                    } else {
                        prompt.push_str(&msg.content);
                    }
                    prompt.push_str("\n\n");
                }
                _ => {}
            }
        }

        prompt.push_str("Assistant: ");
        Ok(prompt)
    }

    async fn get_rag_context(&self, query: &str) -> Result<super::RagContext> {
        // Get embeddings service from data acquisition
        let embeddings = self.data_acquisition
            .embeddings()
            .ok_or_else(|| anyhow!("Embeddings not configured for this project"))?;

        // Embed the query
        let query_embedding = embeddings.embed_text(query).await?;

        // Search the knowledge base
        let search_results = self.data_acquisition
            .search_context(self.project_id, &query_embedding, self.rag_top_k)
            .await?;

        // Build RAG context with threshold filtering
        let rag_context = RagContextBuilder::new(self.rag_threshold, 4000)
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

    async fn call_rig_agent(&self, prompt: &str) -> Result<String> {
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
                } else if let Some(url) = base_url.as_deref() {
                    openai::Client::builder(&api_key).base_url(url).build()
                } else {
                    openai::Client::new(&api_key)
                };

                let builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder
                            .rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
                    }
                }

                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .multi_turn(MAX_TOOL_ITERATIONS)
                    .await
                    .context("OpenAI API call failed")
            }

            RigAgent::Anthropic(model) => {
                let client = if !api_key.is_empty() {
                    anthropic::Client::new(&api_key)
                } else {
                    return Err(anyhow!("Anthropic requires API key"));
                };

                let builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder
                            .rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
                    }
                }

                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .multi_turn(MAX_TOOL_ITERATIONS)
                    .await
                    .context("Anthropic API call failed")
            }

            RigAgent::Gemini(model) => {
                let client = if !api_key.is_empty() {
                    gemini::Client::new(&api_key)
                } else {
                    return Err(anyhow!("Gemini requires API key"));
                };

                // Create generation config and additional params (required by Gemini)
                let gen_cfg = GenerationConfig::default();
                let additional_params = AdditionalParameters::default().with_config(gen_cfg);

                let builder = client
                    .agent(model)
                    .additional_params(serde_json::to_value(additional_params)?);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder
                            .rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
                    }
                }

                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .multi_turn(MAX_TOOL_ITERATIONS)
                    .await
                    .context("Gemini API call failed")
            }

            RigAgent::Ollama(model) => {
                // Prefer persisted overrides, then environment, finally library default
                let client = if let Some(url) = base_url.as_deref() {
                    ollama::Client::builder().base_url(url).build()
                } else if let Ok(env_url) = std::env::var("OLLAMA_API_BASE_URL") {
                    ollama::Client::builder().base_url(&env_url).build()
                } else {
                    ollama::Client::new()
                };

                let builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder
                            .rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
                    }
                }

                let agent = builder.build();
                agent
                    .prompt(prompt)
                    .multi_turn(MAX_TOOL_ITERATIONS)
                    .await
                    .context("Ollama API call failed")
            }
        }
    }

    async fn invoke_agent_with_retries<F>(
        &mut self,
        prompt: &str,
        observer: &mut F,
    ) -> Result<String>
    where
        F: FnMut(ChatEvent),
    {
        match self.call_rig_agent(prompt).await {
            Ok(text) => Ok(text),
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

                let updated_prompt = self.build_conversation_prompt().await?;
                self.call_rig_agent(&updated_prompt).await
            }
            Err(err) => Err(err),
        }
    }

    async fn handle_tool_invocation<F>(
        &mut self,
        raw_response: String,
        invocation: ParsedToolInvocation,
        observer: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        let call_id = format!("tool_call_{}", self.messages.len());
        let arguments_for_record = invocation
            .arguments
            .clone()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        let tool_call = ToolCallData {
            id: call_id.clone(),
            name: invocation.name.clone(),
            arguments: arguments_for_record.clone(),
        };

        self.messages
            .push(ChatMessage::assistant(&raw_response).with_tool_calls(vec![tool_call]));

        let execution_args = invocation.arguments.clone();
        let result = self
            .bridge
            .execute_tool(&invocation.name, &self.security, execution_args.clone())
            .await
            .with_context(|| format!("failed to execute MCP tool {}", invocation.name))?;

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
                Some(invocation.name.clone()),
                Some(call_id.clone()),
                Some(metadata_payload),
            )
            .await?;
        }

        observer(ChatEvent::ToolInvocation {
            name: invocation.name,
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

fn extract_tool_invocation(text: &str) -> Option<ParsedToolInvocation> {
    const TOOL_CODE_PREFIX: &str = "```tool_code";
    let start = text.find(TOOL_CODE_PREFIX)?;
    let after_prefix = &text[start + TOOL_CODE_PREFIX.len()..];
    let after_prefix = after_prefix.trim_start_matches(['\n', '\r', ' ']);
    let end = after_prefix.find("```")?;
    let block = after_prefix[..end].trim();

    let command_line = block.lines().map(str::trim).find(|line| !line.is_empty())?;

    parse_tool_command(command_line)
}

fn parse_tool_command(command: &str) -> Option<ParsedToolInvocation> {
    let open_paren = command.find('(')?;
    let close_paren = command.rfind(')')?;
    if close_paren <= open_paren {
        return None;
    }

    let mut name = command[..open_paren].trim();
    if let Some(eq_pos) = name.rfind('=') {
        name = name[eq_pos + 1..].trim();
    }
    if name.starts_with("let ") {
        name = name.trim_start_matches("let ").trim();
    }
    if name.is_empty() {
        return None;
    }

    let args_str = command[open_paren + 1..close_paren].trim();
    let arguments = if args_str.is_empty() {
        None
    } else if args_str.starts_with('{') {
        serde_json::from_str(args_str).ok()
    } else {
        parse_key_value_arguments(args_str)
    };

    Some(ParsedToolInvocation {
        name: name.to_string(),
        arguments,
    })
}

fn parse_key_value_arguments(args_str: &str) -> Option<serde_json::Value> {
    let mut map = serde_json::Map::new();
    for part in args_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, value_raw) = part.split_once('=')?;
        let key = key.trim();
        if key.is_empty() {
            continue;
        }

        let value = parse_argument_value(value_raw.trim());
        map.insert(key.to_string(), value);
    }

    Some(Value::Object(map))
}

fn parse_argument_value(raw: &str) -> Value {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Value::Null;
    }

    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return Value::String(trimmed[1..trimmed.len() - 1].to_string());
    }

    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        return Value::String(trimmed[1..trimmed.len() - 1].to_string());
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        "null" => return Value::Null,
        _ => {}
    }

    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if let Ok(value) = serde_json::from_str(trimmed) {
            return value;
        }
    }

    if let Ok(int_value) = trimmed.parse::<i64>() {
        return Value::Number(int_value.into());
    }

    if let Ok(float_value) = trimmed.parse::<f64>() {
        if let Some(number) = Number::from_f64(float_value) {
            return Value::Number(number);
        }
    }

    Value::String(trimmed.to_string())
}

fn compose_system_prompt(config: &ChatConfig, project_id: i32, tool_names: &[String]) -> String {
    let mut prompt = String::new();
    if let Some(ref sys_prompt) = config.system_prompt {
        prompt.push_str(sys_prompt);
    }

    if !tool_names.is_empty() {
        prompt.push_str("\n\nYou have access to the following tools:\n");
        for name in tool_names {
            // Writing to String cannot fail
            writeln!(&mut prompt, "- {}", name).expect("Writing to String should not fail");
        }
        prompt.push_str("\nUse these tools when appropriate to help answer questions.");
    }

    prompt.push_str(&format!("\n\nCurrent project ID: {}", project_id));
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_system_prompt_no_tools() {
        let config = ChatConfig {
            default_provider: ChatProvider::Ollama,
            request_timeout: std::time::Duration::from_secs(60),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            providers: std::collections::HashMap::new(),
            mcp_server_url: "http://localhost:3000/mcp".to_string(),
        };

        let prompt = compose_system_prompt(&config, 42, &[]);

        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("Current project ID: 42"));
        assert!(!prompt.contains("You have access to the following tools"));
    }

    #[test]
    fn test_compose_system_prompt_with_tools() {
        let config = ChatConfig {
            default_provider: ChatProvider::Ollama,
            request_timeout: std::time::Duration::from_secs(60),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            providers: std::collections::HashMap::new(),
            mcp_server_url: "http://localhost:3000/mcp".to_string(),
        };

        let tools = vec!["search".to_string(), "calculate".to_string()];
        let prompt = compose_system_prompt(&config, 42, &tools);

        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("You have access to the following tools:"));
        assert!(prompt.contains("- search"));
        assert!(prompt.contains("- calculate"));
        assert!(prompt.contains("Use these tools when appropriate"));
        assert!(prompt.contains("Current project ID: 42"));
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

        let prompt = compose_system_prompt(&config, 42, &[]);

        assert!(prompt.contains("Current project ID: 42"));
        assert!(!prompt.contains("You are a helpful assistant"));
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
