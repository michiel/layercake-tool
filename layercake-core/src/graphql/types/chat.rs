#![cfg(feature = "graphql")]

use async_graphql::{Enum, SimpleObject};

use crate::console::chat::{ChatEvent, ChatProvider};

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ChatProviderOption {
    Ollama,
    OpenAi,
    Gemini,
    Claude,
}

impl From<ChatProviderOption> for ChatProvider {
    fn from(value: ChatProviderOption) -> Self {
        match value {
            ChatProviderOption::Ollama => ChatProvider::Ollama,
            ChatProviderOption::OpenAi => ChatProvider::OpenAi,
            ChatProviderOption::Gemini => ChatProvider::Gemini,
            ChatProviderOption::Claude => ChatProvider::Claude,
        }
    }
}

impl From<ChatProvider> for ChatProviderOption {
    fn from(value: ChatProvider) -> Self {
        match value {
            ChatProvider::Ollama => ChatProviderOption::Ollama,
            ChatProvider::OpenAi => ChatProviderOption::OpenAi,
            ChatProvider::Gemini => ChatProviderOption::Gemini,
            ChatProvider::Claude => ChatProviderOption::Claude,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ChatSessionPayload {
    pub session_id: String,
    pub provider: ChatProviderOption,
    pub model: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ChatEventKind {
    AssistantMessage,
    ToolInvocation,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ChatEventPayload {
    pub kind: ChatEventKind,
    pub message: String,
    pub tool_name: Option<String>,
}

impl From<ChatEvent> for ChatEventPayload {
    fn from(event: ChatEvent) -> Self {
        match event {
            ChatEvent::AssistantMessage { text } => Self {
                kind: ChatEventKind::AssistantMessage,
                message: text,
                tool_name: None,
            },
            ChatEvent::ToolInvocation { name, summary } => Self {
                kind: ChatEventKind::ToolInvocation,
                message: summary,
                tool_name: Some(name),
            },
        }
    }
}

#[derive(SimpleObject)]
pub struct ChatSendResult {
    pub accepted: bool,
}
