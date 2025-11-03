#![cfg(feature = "graphql")]

use async_graphql::{Enum, SimpleObject};

use crate::console::chat::{ChatEvent, ChatProvider};

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "PascalCase")]
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

// Chat History Types

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "lowercase")]
pub enum ChatMessageRole {
    User,
    Assistant,
    Tool,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ChatSession {
    pub id: i32,
    pub session_id: String,
    pub project_id: i32,
    pub user_id: i32,
    pub title: Option<String>,
    pub provider: String,
    pub model_name: String,
    pub system_prompt: Option<String>,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_activity_at: String,
}

impl From<crate::database::entities::chat_sessions::Model> for ChatSession {
    fn from(model: crate::database::entities::chat_sessions::Model) -> Self {
        Self {
            id: model.id,
            session_id: model.session_id,
            project_id: model.project_id,
            user_id: model.user_id,
            title: model.title,
            provider: model.provider,
            model_name: model.model_name,
            system_prompt: model.system_prompt,
            is_archived: model.is_archived,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
            last_activity_at: model.last_activity_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ChatMessage {
    pub id: i32,
    pub session_id: i32,
    pub message_id: String,
    pub role: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

impl From<crate::database::entities::chat_messages::Model> for ChatMessage {
    fn from(model: crate::database::entities::chat_messages::Model) -> Self {
        Self {
            id: model.id,
            session_id: model.session_id,
            message_id: model.message_id,
            role: model.role,
            content: model.content,
            tool_name: model.tool_name,
            tool_call_id: model.tool_call_id,
            metadata_json: model.metadata_json,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}

// MCP Agent Types

#[derive(SimpleObject, Clone, Debug)]
pub struct McpAgent {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub scoped_project_id: Option<i32>,
    pub created_at: String,
    pub is_active: bool,
}

impl From<crate::database::entities::users::Model> for McpAgent {
    fn from(model: crate::database::entities::users::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            display_name: model.display_name,
            scoped_project_id: model.scoped_project_id,
            created_at: model.created_at.to_rfc3339(),
            is_active: model.is_active,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct McpAgentCredentials {
    pub user_id: i32,
    pub api_key: String,
    pub project_id: i32,
    pub name: String,
}

impl From<crate::services::mcp_agent_service::McpAgentCredentials> for McpAgentCredentials {
    fn from(creds: crate::services::mcp_agent_service::McpAgentCredentials) -> Self {
        Self {
            user_id: creds.user_id,
            api_key: creds.api_key,
            project_id: creds.project_id,
            name: creds.name,
        }
    }
}
