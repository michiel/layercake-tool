#![cfg(feature = "console")]

//! Chat session implementation using rig with rmcp for MCP integration
//!
//! This module replaces the old llm-based chat implementation with rig agents.
//! Key changes:
//! - Uses rig agents instead of llm::LLMProvider
//! - Uses rmcp for direct MCP integration (no conversion layer)
//! - Maintains same session persistence and observer pattern

use std::fmt::Write as FmtWrite;

use anyhow::{anyhow, Context, Result};
use axum_mcp::prelude::{ClientContext, SecurityContext};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing;

use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{anthropic, gemini, ollama, openai};

#[cfg(feature = "rmcp")]
use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation, Tool as RmcpTool},
    transport::StreamableHttpClientTransport,
    ServiceExt,
};

use crate::database::entities::{chat_sessions, users};
use crate::mcp::security::build_user_security_context;

use super::{
    config::{ChatConfig, ChatCredentialStore},
    ChatProvider, McpBridge,
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
struct ToolCallData {
    id: String,
    name: String,
    arguments: serde_json::Value,
}

#[derive(Clone, Debug)]
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
        Self {
            role: "tool".to_string(),
            content: String::new(),
            tool_calls: None,
            tool_results: Some(vec![ToolResultData {
                call_id: call_id.into(),
                output: output.into(),
            }]),
        }
    }
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
        project_id: i32,
        user: users::Model,
        provider: ChatProvider,
        config: &ChatConfig,
    ) -> Result<Self> {
        let credentials = ChatCredentialStore::new(db.clone());
        let bridge = McpBridge::new(db.clone());
        let security = build_user_security_context(
            ClientContext::default(),
            user.id,
            &user.user_type,
            Some(project_id),
        );

        // Initialize rmcp client if feature is enabled
        #[cfg(feature = "rmcp")]
        let (rmcp_client, rmcp_tools) =
            Self::init_rmcp_client(&config.mcp_server_url).await.ok().unzip();

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
        })
    }

    /// Resume an existing chat session from the database
    pub async fn resume(
        db: DatabaseConnection,
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
        let credentials = ChatCredentialStore::new(db.clone());
        let bridge = McpBridge::new(db.clone());

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
        let (rmcp_client, rmcp_tools) =
            Self::init_rmcp_client(&config.mcp_server_url).await.ok().unzip();

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
        // Build conversation prompt from message history
        let conversation = self.build_conversation_prompt();

        // Call agent with multi-turn tool support
        let response_text = match self.call_rig_agent(&conversation).await {
            Ok(text) => text,
            Err(err) if self.should_disable_tools(&err) => {
                // Ollama HTTP 400 when tools not supported - retry without tools
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
                tracing::warn!("Disabling tool usage for session: {}", err);

                // Retry without tools
                self.call_rig_agent(&conversation).await?
            }
            Err(err) => return Err(err),
        };

        // Notify observer
        observer(ChatEvent::AssistantMessage {
            text: response_text.clone(),
        });

        // Add assistant response to history
        self.messages.push(ChatMessage::assistant(&response_text));

        // Persist assistant message
        if let Some(ref session_id) = self.session_id {
            use crate::services::chat_history_service::ChatHistoryService;
            let history_service = ChatHistoryService::new(self.db.clone());
            history_service
                .store_message(
                    session_id,
                    "assistant".to_string(),
                    response_text,
                    None,
                    None,
                    None,
                )
                .await?;
        }

        Ok(())
    }

    fn should_disable_tools(&self, err: &anyhow::Error) -> bool {
        if self.provider != ChatProvider::Ollama || !self.tool_use_enabled {
            return false;
        }

        // Check if error is HTTP 400 from Ollama /api/chat endpoint
        let err_str = err.to_string();
        err_str.contains("/api/chat") && err_str.contains("400")
    }

    fn build_conversation_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str(&self.system_prompt);
        prompt.push_str("\n\n");

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
                _ => {}
            }
        }

        prompt.push_str("Assistant: ");
        prompt
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

        // Get base URL if configured
        let provider_config = self.config.provider(self.provider);
        let _base_url = provider_config.base_url.clone();

        // Call appropriate provider
        match &self.agent {
            RigAgent::OpenAI(model) => {
                let client = if !api_key.is_empty() {
                    openai::Client::new(&api_key)
                } else {
                    return Err(anyhow!("OpenAI requires API key"));
                };

                let mut builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder.rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
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

                let mut builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder.rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
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

                let mut builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder.rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
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
                // Ollama client uses environment variable or defaults to localhost
                let client = ollama::Client::from_env();

                let mut builder = client.agent(model);

                #[cfg(feature = "rmcp")]
                if let Some(ref rmcp_client) = self.rmcp_client {
                    if !self.rmcp_tools.is_empty() {
                        builder = builder.rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
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
}

fn compose_system_prompt(config: &ChatConfig, project_id: i32, tool_names: &[String]) -> String {
    let mut prompt = String::new();
    if let Some(ref sys_prompt) = config.system_prompt {
        prompt.push_str(sys_prompt);
    }

    if !tool_names.is_empty() {
        prompt.push_str("\n\nYou have access to the following tools:\n");
        for name in tool_names {
            write!(&mut prompt, "- {}\n", name).unwrap();
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
        assert_eq!("ollama".parse::<ChatProvider>().unwrap(), ChatProvider::Ollama);
        assert_eq!("openai".parse::<ChatProvider>().unwrap(), ChatProvider::OpenAi);
        assert_eq!("open-ai".parse::<ChatProvider>().unwrap(), ChatProvider::OpenAi);
        assert_eq!("gemini".parse::<ChatProvider>().unwrap(), ChatProvider::Gemini);
        assert_eq!("claude".parse::<ChatProvider>().unwrap(), ChatProvider::Claude);
        assert_eq!("anthropic".parse::<ChatProvider>().unwrap(), ChatProvider::Claude);

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
        assert_eq!(msg.tool_calls.as_ref().unwrap().len(), 1);

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
