#![cfg(feature = "console")]

use std::fmt::Write;

use anyhow::{anyhow, Context, Result};
use axum_mcp::prelude::{ClientContext, SecurityContext};
use llm::{
    builder::LLMBuilder,
    chat::{ChatMessage, Tool as LlmTool},
    error::LLMError,
    FunctionCall, LLMProvider, ToolCall,
};
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing;

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

pub struct ChatSession {
    db: DatabaseConnection,
    session_id: Option<String>,
    project_id: i32,
    user_id: i32,
    provider: ChatProvider,
    model_name: String,
    system_prompt: String,
    messages: Vec<ChatMessage>,
    llm: Box<dyn LLMProvider>,
    bridge: McpBridge,
    security: SecurityContext,
    llm_tools: Vec<LlmTool>,
    tool_use_enabled: bool,
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
        let llm_tools = bridge
            .llm_tools(&security)
            .await
            .map_err(|err| anyhow!("failed to load MCP tools: {}", err))?;
        let system_prompt = compose_system_prompt(
            config,
            project_id,
            &llm_tools
                .iter()
                .map(|t| t.function.name.clone())
                .collect::<Vec<_>>(),
        );
        let (llm, model_name) =
            build_llm_client(provider, config, &credentials, &system_prompt).await?;

        Ok(Self {
            db,
            session_id: None,
            project_id,
            user_id: user.id,
            provider,
            model_name,
            system_prompt,
            messages: Vec::new(),
            llm,
            bridge,
            security,
            llm_tools,
            tool_use_enabled: true,
        })
    }

    /// Resume an existing chat session from the database
    pub async fn resume(
        db: DatabaseConnection,
        session: chat_sessions::Model,
        user: users::Model,
        config: &ChatConfig,
    ) -> Result<Self> {
        use crate::services::chat_history_service::ChatHistoryService;

        let history_service = ChatHistoryService::new(db.clone());

        // Load message history
        let messages_history = history_service
            .get_history(&session.session_id, 1000, 0)
            .await?;

        // Convert provider string to enum
        let provider = match session.provider.as_str() {
            "openai" => ChatProvider::OpenAi,
            "claude" | "anthropic" => ChatProvider::Claude,
            "ollama" => ChatProvider::Ollama,
            "gemini" | "google" => ChatProvider::Gemini,
            _ => return Err(anyhow!("Unknown provider: {}", session.provider)),
        };

        let credentials = ChatCredentialStore::new(db.clone());
        let bridge = McpBridge::new(db.clone());
        let security = build_user_security_context(
            ClientContext::default(),
            user.id,
            &user.user_type,
            Some(session.project_id),
        );
        let llm_tools = bridge
            .llm_tools(&security)
            .await
            .map_err(|err| anyhow!("failed to load MCP tools: {}", err))?;

        let system_prompt = session.system_prompt.clone().unwrap_or_else(|| {
            compose_system_prompt(
                config,
                session.project_id,
                &llm_tools
                    .iter()
                    .map(|t| t.function.name.clone())
                    .collect::<Vec<_>>(),
            )
        });

        let (llm, model_name) =
            build_llm_client(provider, config, &credentials, &system_prompt).await?;

        // Convert database messages to LLM messages
        let mut messages = Vec::new();
        for msg in messages_history {
            let chat_msg = match msg.role.as_str() {
                "user" => ChatMessage::user().content(msg.content).build(),
                "assistant" => ChatMessage::assistant().content(msg.content).build(),
                _ => continue, // Skip tool messages for now
            };

            messages.push(chat_msg);
        }

        Ok(Self {
            db,
            session_id: Some(session.session_id.clone()),
            project_id: session.project_id,
            user_id: session.user_id,
            provider,
            model_name,
            system_prompt,
            messages,
            llm,
            bridge,
            security,
            llm_tools,
            tool_use_enabled: true,
        })
    }

    /// Persist the session to the database if not already persisted
    pub async fn ensure_persisted(&mut self) -> Result<String> {
        if let Some(ref session_id) = self.session_id {
            return Ok(session_id.clone());
        }

        use crate::services::chat_history_service::ChatHistoryService;
        let history_service = ChatHistoryService::new(self.db.clone());

        let session = history_service
            .create_session(
                self.project_id,
                self.user_id,
                self.provider.to_string(),
                self.model_name.clone(),
                None,
                Some(self.system_prompt.clone()),
            )
            .await?;

        self.session_id = Some(session.session_id.clone());
        Ok(session.session_id)
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub async fn interactive_loop(&mut self) -> Result<()> {
        println!(
            "Starting chat for project {} with {} ({})",
            self.project_id,
            self.provider.display_name(),
            self.model_name
        );
        if !self.llm_tools.is_empty() {
            let tool_list = self
                .llm_tools
                .iter()
                .map(|tool| tool.function.name.clone())
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
        // Ensure session is persisted before adding messages
        let session_id = self.ensure_persisted().await?;

        self.messages
            .push(ChatMessage::user().content(input).build());

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
        for _ in 0..MAX_TOOL_ITERATIONS {
            let mut messages_log = String::new();
            for message in &self.messages {
                let role = &message.role;
                let content = &message.content;
                messages_log.push_str(&format!("Role: {:?}, Content: {}\n", role, content));
            }
            tracing::info!("Sending messages to LLM: \n{}", messages_log);

            let response = self.request_llm_response(observer).await?;

            let maybe_tool_calls = response.tool_calls();
            let response_text = response.text();

            if let Some(tool_calls) = maybe_tool_calls {
                self.handle_tool_calls(tool_calls, response_text, observer)
                    .await?;
                continue;
            }

            let final_text = response_text.unwrap_or_else(|| "(no response)".to_string());
            observer(ChatEvent::AssistantMessage {
                text: final_text.clone(),
            });
            self.messages
                .push(ChatMessage::assistant().content(final_text.clone()).build());

            // Persist assistant message
            if let Some(ref session_id) = self.session_id {
                use crate::services::chat_history_service::ChatHistoryService;
                let history_service = ChatHistoryService::new(self.db.clone());
                history_service
                    .store_message(
                        session_id,
                        "assistant".to_string(),
                        final_text,
                        None,
                        None,
                        None,
                    )
                    .await?;
            }

            break;
        }

        Ok(())
    }

    async fn handle_tool_calls<F>(
        &mut self,
        tool_calls: Vec<ToolCall>,
        response_text: Option<String>,
        observer: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        if let Some(text) = response_text.as_ref().filter(|t| !t.trim().is_empty()) {
            observer(ChatEvent::AssistantMessage { text: text.clone() });
        }

        let content = response_text.clone().unwrap_or_default();
        self.messages.push(
            ChatMessage::assistant()
                .tool_use(tool_calls.clone())
                .content(content.clone())
                .build(),
        );

        // Persist assistant message with tool calls
        if let Some(ref session_id) = self.session_id {
            use crate::services::chat_history_service::ChatHistoryService;
            let history_service = ChatHistoryService::new(self.db.clone());

            for call in &tool_calls {
                history_service
                    .store_message(
                        session_id,
                        "assistant".to_string(),
                        content.clone(),
                        Some(call.function.name.clone()),
                        Some(call.id.clone()),
                        Some(call.function.arguments.clone()),
                    )
                    .await?;
            }
        }

        let mut tool_results_for_llm = Vec::new();
        let mut tool_history_entries = Vec::new();

        for call in &tool_calls {
            let args = parse_tool_arguments(&call.function.arguments)?;
            let exec_result = self
                .bridge
                .execute_tool(&call.function.name, &self.security, Some(args.clone()))
                .await
                .map_err(|err| anyhow!("tool '{}' failed: {}", call.function.name, err))?;

            let payload = McpBridge::serialize_tool_result(&exec_result);
            let payload_string =
                serde_json::to_string(&payload).context("serializing tool result payload")?;

            tool_results_for_llm.push(ToolCall {
                id: call.id.clone(),
                call_type: call.call_type.clone(),
                function: FunctionCall {
                    name: call.function.name.clone(),
                    arguments: payload_string.clone(),
                },
            });

            let summary = McpBridge::summarize_tool_result(&exec_result);
            let summary = if summary.trim().is_empty() {
                "(no output)".to_string()
            } else {
                summary
            };
            observer(ChatEvent::ToolInvocation {
                name: call.function.name.clone(),
                summary: summary.clone(),
            });

            tool_history_entries.push((
                call.function.name.clone(),
                call.id.clone(),
                summary,
                payload_string,
            ));
        }

        self.messages.push(
            ChatMessage::user()
                .tool_result(tool_results_for_llm.clone())
                .content("")
                .build(),
        );

        // Persist tool results
        if let Some(ref session_id) = self.session_id {
            use crate::services::chat_history_service::ChatHistoryService;
            let history_service = ChatHistoryService::new(self.db.clone());

            for (tool_name, call_id, summary, metadata_json) in &tool_history_entries {
                history_service
                    .store_message(
                        session_id,
                        "tool".to_string(),
                        summary.clone(),
                        Some(tool_name.clone()),
                        Some(call_id.clone()),
                        Some(metadata_json.clone()),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn request_llm_response<F>(
        &mut self,
        observer: &mut F,
    ) -> Result<Box<dyn llm::chat::ChatResponse>, anyhow::Error>
    where
        F: FnMut(ChatEvent),
    {
        let use_tools = self.tool_use_enabled && !self.llm_tools.is_empty();

        // Log request details at DEBUG level before making the call
        self.log_llm_request_debug(use_tools);

        if use_tools {
            match self
                .llm
                .chat_with_tools(&self.messages, Some(&self.llm_tools))
                .await
            {
                Ok(response) => Ok(response),
                Err(err) if self.should_disable_tools(&err) => {
                    self.tool_use_enabled = false;
                    let notice = "Ollama server rejected function/tool calls. Continuing without tool access; responses now rely on model knowledge only.";
                    observer(ChatEvent::AssistantMessage {
                        text: notice.to_string(),
                    });
                    self.messages
                        .push(ChatMessage::assistant().content(notice).build());
                    tracing::warn!("Disabling tool usage for session: {}", err);
                    self.llm
                        .chat(&self.messages)
                        .await
                        .map_err(|fallback_err| {
                            self.log_llm_error_debug(&fallback_err);
                            anyhow!("llm call failed: {}", fallback_err)
                        })
                }
                Err(err) => {
                    self.log_llm_error_debug(&err);
                    Err(anyhow!("llm call failed: {}", err))
                }
            }
        } else {
            self.llm
                .chat(&self.messages)
                .await
                .map_err(|err| {
                    self.log_llm_error_debug(&err);
                    anyhow!("llm call failed: {}", err)
                })
        }
    }

    fn log_llm_request_debug(&self, with_tools: bool) {
        if !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }

        // Build a detailed representation of the request
        let mut request_debug = String::new();
        request_debug.push_str(&format!("\n=== LLM Request Details ===\n"));
        request_debug.push_str(&format!("Provider: {}\n", self.provider.to_string()));
        request_debug.push_str(&format!("Model: {}\n", self.model_name));
        request_debug.push_str(&format!("Session ID: {:?}\n", self.session_id));
        request_debug.push_str(&format!("With Tools: {}\n", with_tools));
        request_debug.push_str(&format!("Message Count: {}\n", self.messages.len()));

        if with_tools {
            request_debug.push_str(&format!("Tool Count: {}\n", self.llm_tools.len()));
            request_debug.push_str("Tools: ");
            for tool in &self.llm_tools {
                request_debug.push_str(&format!("{}, ", tool.function.name));
            }
            request_debug.push('\n');
        }

        request_debug.push_str("\n--- Messages ---\n");
        for (idx, msg) in self.messages.iter().enumerate() {
            request_debug.push_str(&format!(
                "\nMessage {}: role={:?}, type={:?}\n",
                idx,
                msg.role,
                msg.message_type
            ));
            request_debug.push_str(&format!("Content ({}): {}\n",
                msg.content.len(),
                msg.content
            ));
        }

        request_debug.push_str("\n--- System Prompt ---\n");
        request_debug.push_str(&self.system_prompt);
        request_debug.push_str("\n=========================\n");

        tracing::debug!("{}", request_debug);
    }

    fn log_llm_error_debug(&self, err: &LLMError) {
        // Only log at DEBUG level if it's an HTTP error in 4xx or 5xx range
        if let LLMError::HttpError(msg) = err {
            // Check if the error message contains status code indicators
            let is_client_error = msg.contains("400") || msg.contains("401") || msg.contains("403")
                || msg.contains("404") || msg.contains("422") || msg.contains("429");
            let is_server_error = msg.contains("500") || msg.contains("502") || msg.contains("503")
                || msg.contains("504");

            if is_client_error || is_server_error {
                // Sanitise API keys from error messages
                let sanitised_msg = Self::sanitise_api_keys(msg);

                tracing::debug!(
                    provider = %self.provider.to_string(),
                    model = %self.model_name,
                    session_id = ?self.session_id,
                    "\n=== LLM HTTP Error Response ===\n{}\n===============================",
                    sanitised_msg
                );
            }
        }
    }

    /// Sanitise API keys from error messages to prevent leaking secrets in logs
    fn sanitise_api_keys(msg: &str) -> String {
        use regex::Regex;

        // Pattern to match API keys in URLs (query parameters)
        // Matches patterns like: ?key=ACTUAL_KEY or &key=ACTUAL_KEY
        let re = Regex::new(r"([?&]key=)[A-Za-z0-9_-]+").unwrap();
        let sanitised = re.replace_all(msg, "${1}[REDACTED]");

        // Also sanitise bearer tokens if present
        let re_bearer = Regex::new(r"(Bearer\s+)[A-Za-z0-9_.-]+").unwrap();
        re_bearer.replace_all(&sanitised, "${1}[REDACTED]").to_string()
    }

    fn should_disable_tools(&self, err: &LLMError) -> bool {
        if self.provider != ChatProvider::Ollama || self.tool_use_enabled == false {
            return false;
        }

        matches!(err, LLMError::HttpError(message) if message.contains("/api/chat") && message.contains("400"))
    }
}

async fn build_llm_client(
    provider: ChatProvider,
    config: &ChatConfig,
    credentials: &ChatCredentialStore,
    system_prompt: &str,
) -> Result<(Box<dyn LLMProvider>, String)> {
    let provider_config = config.provider(provider).clone();
    let mut builder = LLMBuilder::new()
        .backend(provider.backend())
        .model(provider_config.model.clone())
        .timeout_seconds(config.request_timeout.as_secs().max(5))
        .temperature(
            std::env::var("LAYERCAKE_CHAT_TEMPERATURE")
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.2),
        )
        .system(system_prompt.to_string());

    if let Some(base_url) = credentials
        .base_url_override(provider)
        .await?
        .or(provider_config.base_url)
    {
        builder = builder.base_url(base_url);
    }

    if let Some(api_key) = credentials.api_key(provider).await? {
        builder = builder.api_key(api_key);
    } else if provider.requires_api_key() {
        return Err(anyhow!(
            "{} requires an API key (set the appropriate environment variable)",
            provider.display_name()
        ));
    }

    let client = builder
        .build()
        .map_err(|err| anyhow!("failed to initialize {}: {}", provider.display_name(), err))?;

    Ok((client, provider_config.model))
}

fn compose_system_prompt(config: &ChatConfig, project_id: i32, tool_names: &[String]) -> String {
    let tool_list = if tool_names.is_empty() {
        "none".to_string()
    } else {
        tool_names.join(", ")
    };

    let mut prompt = format!(
        "You are the Layercake assistant for project {project_id}. \
Assist with graph analysis and data operations using the available tools. \
Call tools when you need fresh data before answering.\nAvailable tools: {}.\n\n",
        tool_list
    );

    // Load data model documentation
    if let Ok(data_model) = load_system_prompt_file("chat-data-model.md") {
        let _ = write!(prompt, "{}\n\n", data_model);
    }

    // Load output formatting guidelines
    if let Ok(formatting) = load_system_prompt_file("chat-output-formatting.md") {
        let _ = write!(prompt, "{}\n\n", formatting);
    }

    if let Some(extra) = config.system_prompt.as_ref() {
        let _ = write!(prompt, "{}", extra);
    }

    prompt
}

fn load_system_prompt_file(filename: &str) -> Result<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(filename);

    std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to load system prompt file: {}", filename))
}

fn parse_tool_arguments(raw: &str) -> Result<Value> {
    if raw.trim().is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_str(raw).or_else(|_| Ok(json!({ "arguments": raw })))
    }
}
