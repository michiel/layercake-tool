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
}

impl ChatSession {
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

        // Get available tools from MCP
        let mcp_tools = bridge
            .list_tools(&security)
            .await
            .map_err(|err| anyhow!("failed to load MCP tools: {}", err))?;

        let tool_names: Vec<String> = mcp_tools.iter().map(|t| t.name.clone()).collect();
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

        // For now, make a simple non-streaming call
        // TODO: Add streaming, tool calling, multi-iteration loop
        let response_text = self.call_rig_agent(&conversation).await?;

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
        let base_url = provider_config.base_url.clone();

        // Call appropriate provider
        match &self.agent {
            RigAgent::OpenAI(model) => {
                let client = if !api_key.is_empty() {
                    openai::Client::new(&api_key)
                } else {
                    return Err(anyhow!("OpenAI requires API key"));
                };

                let agent = client.agent(model).build();
                agent
                    .prompt(prompt)
                    .await
                    .context("OpenAI API call failed")
            }

            RigAgent::Anthropic(model) => {
                let client = if !api_key.is_empty() {
                    anthropic::Client::new(&api_key)
                } else {
                    return Err(anyhow!("Anthropic requires API key"));
                };

                let agent = client.agent(model).build();
                agent
                    .prompt(prompt)
                    .await
                    .context("Anthropic API call failed")
            }

            RigAgent::Gemini(model) => {
                let client = if !api_key.is_empty() {
                    gemini::Client::new(&api_key)
                } else {
                    return Err(anyhow!("Gemini requires API key"));
                };

                let agent = client.agent(model).build();
                agent.prompt(prompt).await.context("Gemini API call failed")
            }

            RigAgent::Ollama(model) => {
                // Ollama client uses environment variable or defaults to localhost
                let client = ollama::Client::from_env();

                let agent = client.agent(model).build();
                agent
                    .prompt(prompt)
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
