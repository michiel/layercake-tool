#![cfg(feature = "console")]

use std::fmt::Write;

use anyhow::{anyhow, Context, Result};
use axum_mcp::prelude::SecurityContext;
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
    project_id: i32,
    provider: ChatProvider,
    model_name: String,
    messages: Vec<ChatMessage>,
    llm: Box<dyn LLMProvider>,
    bridge: McpBridge,
    security: SecurityContext,
    llm_tools: Vec<LlmTool>,
    tool_use_enabled: bool,
}

impl ChatSession {
    pub async fn new(
        db: DatabaseConnection,
        project_id: i32,
        provider: ChatProvider,
        config: &ChatConfig,
    ) -> Result<Self> {
        let credentials = ChatCredentialStore::new(db.clone());
        let bridge = McpBridge::new(db);
        let security = SecurityContext::system();
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
            project_id,
            provider,
            model_name,
            messages: Vec::new(),
            llm,
            bridge,
            security,
            llm_tools,
            tool_use_enabled: true,
        })
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
        self.messages
            .push(ChatMessage::user().content(input).build());
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
                .push(ChatMessage::assistant().content(final_text).build());
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

        self.messages.push(
            ChatMessage::assistant()
                .tool_use(tool_calls.clone())
                .content(response_text.unwrap_or_default())
                .build(),
        );

        let mut tool_results_for_llm = Vec::new();

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
                    arguments: payload_string,
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
                summary,
            });
        }

        self.messages.push(
            ChatMessage::user()
                .tool_result(tool_results_for_llm)
                .content("")
                .build(),
        );
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
        if use_tools {
            match self
                .llm
                .chat_with_tools(&self.messages, Some(&self.llm_tools))
                .await
            {
                Ok(response) => Ok(response),
                Err(err) if self.should_disable_tools(&err) => {
                    self.tool_use_enabled = false;
                    let notice =
                        "Ollama server rejected tool calls; continuing without tool integration.";
                    observer(ChatEvent::AssistantMessage {
                        text: notice.to_string(),
                    });
                    tracing::warn!("Disabling tool usage for session: {}", err);
                    self.llm
                        .chat(&self.messages)
                        .await
                        .map_err(|fallback_err| anyhow!("llm call failed: {}", fallback_err))
                }
                Err(err) => Err(anyhow!("llm call failed: {}", err)),
            }
        } else {
            self.llm
                .chat(&self.messages)
                .await
                .map_err(|err| anyhow!("llm call failed: {}", err))
        }
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
Call tools when you need fresh data before answering.\nAvailable tools: {}.",
        tool_list
    );

    if let Some(extra) = config.system_prompt.as_ref() {
        let _ = write!(prompt, "\n\n{}", extra);
    }

    prompt
}

fn parse_tool_arguments(raw: &str) -> Result<Value> {
    if raw.trim().is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_str(raw).or_else(|_| Ok(json!({ "arguments": raw })))
    }
}
