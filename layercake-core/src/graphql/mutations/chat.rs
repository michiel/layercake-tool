use async_graphql::*;

use crate::console::chat::ChatProvider;
use crate::graphql::context::{GraphQLContext, RequestSession};
use crate::graphql::errors::StructuredError;
use crate::graphql::types::chat::{ChatProviderOption, ChatSendResult, ChatSessionPayload};
use crate::services::authorization::AuthorizationService;
use crate::services::chat_history_service::ChatHistoryService;
use super::helpers::ensure_local_user_session;

#[derive(Default)]
pub struct ChatMutation;

#[Object]
impl ChatMutation {
/// Start a conversational chat session bound to a project.
async fn start_chat_session(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "projectId")] project_id: i32,
    #[graphql(name = "provider")] provider: Option<ChatProviderOption>,
    #[graphql(name = "sessionId")] existing_session_id: Option<String>,
    message: Option<String>,
) -> Result<ChatSessionPayload> {
    let context = ctx.data::<GraphQLContext>()?;
    let session = ctx
        .data_opt::<RequestSession>()
        .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

    let auth_service = AuthorizationService::new(context.db.clone());
    let user = match auth_service.get_user_from_session(session.as_str()).await {
        Ok(user) => user,
        Err(_) => ensure_local_user_session(&context.db, session.as_str(), project_id).await?,
    };

    auth_service
        .check_project_read_access(user.id, project_id)
        .await
        .map_err(|err| StructuredError::forbidden(err.to_string()))?;

    let chat_config = context.chat_config().await;
    let mut resolved_provider = provider
        .map(ChatProvider::from)
        .unwrap_or(chat_config.default_provider);

    let started = if let Some(existing_id) = existing_session_id {
        let history_service = ChatHistoryService::new(context.db.clone());
        let existing = history_service
            .get_session(&existing_id)
            .await
            .map_err(|e| StructuredError::service("ChatHistoryService::get_session", e))?
            .ok_or_else(|| StructuredError::not_found("ChatSession", existing_id.clone()))?;

        if existing.project_id != project_id {
            return Err(StructuredError::forbidden(
                "Chat session does not belong to this project",
            ));
        }

        if existing.user_id != user.id {
            return Err(StructuredError::forbidden(
                "Chat session belongs to another user",
            ));
        }

        resolved_provider = match existing.provider.as_str() {
            "openai" => ChatProvider::OpenAi,
            "claude" | "anthropic" => ChatProvider::Claude,
            "ollama" => ChatProvider::Ollama,
            "gemini" | "google" => ChatProvider::Gemini,
            other => {
                return Err(StructuredError::bad_request(format!(
                    "Unknown chat provider: {}",
                    other
                )))
            }
        };

        context
            .chat_manager
            .resume_session(
                context.db.clone(),
                existing.clone(),
                user.clone(),
                chat_config.clone(),
                context.system_settings.clone(),
            )
            .await
            .map_err(|e| StructuredError::service("ChatManager::resume_session", e))?
    } else {
        context
            .chat_manager
            .start_session(
                context.db.clone(),
                project_id,
                user.clone(),
                resolved_provider,
                chat_config.clone(),
                context.system_settings.clone(),
            )
            .await
            .map_err(|e| StructuredError::service("ChatManager::start_session", e))?
    };

    if let Some(message) = message {
        context
            .chat_manager
            .enqueue_message(&started.session_id, message)
            .await
            .map_err(|e| StructuredError::service("ChatManager::enqueue_message", e))?;
    }

    Ok(ChatSessionPayload {
        session_id: started.session_id,
        provider: ChatProviderOption::from(resolved_provider),
        model: started.model_name,
    })
}

/// Queue a new user message for an active chat session.
async fn send_chat_message(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "sessionId")] session_id: String,
    message: String,
) -> Result<ChatSendResult> {
    let context = ctx.data::<GraphQLContext>()?;
    let session = ctx
        .data_opt::<RequestSession>()
        .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

    let history_service = ChatHistoryService::new(context.db.clone());
    let chat_session = history_service
        .get_session(&session_id)
        .await
        .map_err(|e| StructuredError::service("ChatHistoryService::get_session", e))?
        .ok_or_else(|| StructuredError::not_found("ChatSession", session_id.clone()))?;

    let auth_service = AuthorizationService::new(context.db.clone());
    let user = match auth_service.get_user_from_session(session.as_str()).await {
        Ok(user) => user,
        Err(_) => {
            ensure_local_user_session(&context.db, session.as_str(), chat_session.project_id)
                .await?
        }
    };

    if chat_session.user_id != user.id {
        return Err(StructuredError::forbidden(
            "You do not have access to this chat session",
        ));
    }

    auth_service
        .check_project_read_access(user.id, chat_session.project_id)
        .await
        .map_err(|err| StructuredError::forbidden(err.to_string()))?;

    if !context.chat_manager.is_session_active(&session_id).await {
        context
            .chat_manager
            .resume_session(
                context.db.clone(),
                chat_session.clone(),
                user.clone(),
                context.chat_config().await,
                context.system_settings.clone(),
            )
            .await
            .map_err(|e| StructuredError::service("ChatManager::resume_session", e))?;
    }

    context
        .chat_manager
        .enqueue_message(&session_id, message)
        .await
        .map_err(|e| StructuredError::service("ChatManager::enqueue_message", e))?;

    Ok(ChatSendResult { accepted: true })
}

/// Update chat session title
async fn update_chat_session_title(
    &self,
    ctx: &Context<'_>,
    session_id: String,
    title: String,
) -> Result<bool> {
    use crate::services::chat_history_service::ChatHistoryService;
    let context = ctx.data::<GraphQLContext>()?;
    let service = ChatHistoryService::new(context.db.clone());

    service
        .update_session_title(&session_id, title)
        .await
        .map_err(|e| StructuredError::service("ChatHistoryService::update_session_title", e))?;

    Ok(true)
}

/// Archive a chat session
async fn archive_chat_session(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
    use crate::services::chat_history_service::ChatHistoryService;
    let context = ctx.data::<GraphQLContext>()?;
    let service = ChatHistoryService::new(context.db.clone());

    service
        .archive_session(&session_id)
        .await
        .map_err(|e| StructuredError::service("ChatHistoryService::archive_session", e))?;

    Ok(true)
}

/// Unarchive a chat session
async fn unarchive_chat_session(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
    use crate::services::chat_history_service::ChatHistoryService;
    let context = ctx.data::<GraphQLContext>()?;
    let service = ChatHistoryService::new(context.db.clone());

    service
        .unarchive_session(&session_id)
        .await
        .map_err(|e| StructuredError::service("ChatHistoryService::unarchive_session", e))?;

    Ok(true)
}

/// Delete a chat session and all its messages
async fn delete_chat_session(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
    use crate::services::chat_history_service::ChatHistoryService;
    let context = ctx.data::<GraphQLContext>()?;
    let service = ChatHistoryService::new(context.db.clone());

    service
        .delete_session(&session_id)
        .await
        .map_err(|e| StructuredError::service("ChatHistoryService::delete_session", e))?;

    Ok(true)
}
}
