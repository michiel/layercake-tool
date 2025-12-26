use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex as StdMutex},
};

use anyhow::{anyhow, Result};
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::chat::{ChatConfig, ChatEvent, ChatProvider, ChatSession};
use layercake_core::database::entities::{chat_sessions, users};
use layercake_core::services::system_settings_service::SystemSettingsService;
use sea_orm::DatabaseConnection;

pub struct StartedChatSession {
    pub session_id: String,
    pub model_name: String,
}

struct ChatSessionRuntime {
    input_tx: mpsc::Sender<String>,
    event_tx: broadcast::Sender<ChatEvent>,
    history: Arc<StdMutex<VecDeque<ChatEvent>>>,
    // Keep an initial receiver alive to prevent message loss when no subscribers
    _keeper: broadcast::Receiver<ChatEvent>,
}

const MAX_EVENT_HISTORY: usize = 64;

#[derive(Default)]
pub struct ChatManager {
    inner: Arc<ChatManagerInner>,
}

#[derive(Default)]
struct ChatManagerInner {
    sessions: Mutex<HashMap<String, ChatSessionRuntime>>,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ChatManagerInner::default()),
        }
    }

    pub async fn start_session(
        &self,
        db: DatabaseConnection,
        project_id: i32,
        user: users::Model,
        provider: ChatProvider,
        config: Arc<ChatConfig>,
        settings: Arc<SystemSettingsService>,
    ) -> Result<StartedChatSession> {
        let (input_tx, mut input_rx) = mpsc::channel::<String>(16);
        let (event_tx, keeper_rx) = broadcast::channel::<ChatEvent>(64);
        let history = Arc::new(StdMutex::new(VecDeque::new()));

        let mut chat_session = ChatSession::new(
            db.clone(),
            settings.clone(),
            project_id,
            user.clone(),
            provider,
            &config,
        )
        .await?;
        let session_id = chat_session.ensure_persisted().await?;
        let model_name = chat_session.model_name().to_string();

        {
            let mut sessions = self.inner.sessions.lock().await;
            sessions.insert(
                session_id.clone(),
                ChatSessionRuntime {
                    input_tx: input_tx.clone(),
                    event_tx: event_tx.clone(),
                    history: history.clone(),
                    _keeper: keeper_rx,
                },
            );
        }

        let manager = self.inner.clone();
        let session_key = session_id.clone();
        let history_for_task = history.clone();
        let event_tx_for_task = event_tx.clone();

        tokio::spawn(async move {
            tracing::info!("Chat session task started for session {}", session_key);
            while let Some(message) = input_rx.recv().await {
                tracing::info!("Received message in session {}: {}", session_key, message);
                let history_for_sink = history_for_task.clone();
                let event_tx_for_sink = event_tx_for_task.clone();
                let mut sink = move |event: ChatEvent| {
                    store_event(&history_for_sink, &event);
                    tracing::debug!("Broadcasting event: {:?}", event);
                    match event_tx_for_sink.send(event) {
                        Ok(count) => tracing::info!("Event sent to {} subscribers", count),
                        Err(e) => tracing::error!("Failed to broadcast event: {:?}", e),
                    }
                };

                if let Err(err) = chat_session
                    .send_message_with_observer(&message, &mut sink)
                    .await
                {
                    tracing::error!("Chat session error: {}", err);
                    let sanitised_error = sanitise_error_message(&format!("Chat error: {}", err));
                    let error_event = ChatEvent::AssistantMessage {
                        text: sanitised_error,
                    };
                    store_event(&history_for_task, &error_event);
                    tracing::info!("Broadcasting error event");
                    match event_tx_for_task.send(error_event) {
                        Ok(count) => tracing::info!("Error event sent to {} subscribers", count),
                        Err(e) => tracing::error!("Failed to send error event: {:?}", e),
                    }
                }
            }

            tracing::info!("Chat session task ending for session {}", session_key);
            let mut sessions = manager.sessions.lock().await;
            sessions.remove(&session_key);
        });

        Ok(StartedChatSession {
            session_id,
            model_name,
        })
    }

    pub async fn resume_session(
        &self,
        db: DatabaseConnection,
        session: chat_sessions::Model,
        _user: users::Model,
        config: Arc<ChatConfig>,
        settings: Arc<SystemSettingsService>,
    ) -> Result<StartedChatSession> {
        if self.is_session_active(&session.session_id).await {
            return Ok(StartedChatSession {
                session_id: session.session_id,
                model_name: session.model_name,
            });
        }

        let (input_tx, mut input_rx) = mpsc::channel::<String>(16);
        let (event_tx, keeper_rx) = broadcast::channel::<ChatEvent>(64);
        let history = Arc::new(StdMutex::new(VecDeque::new()));

        let mut chat_session = ChatSession::resume(
            db.clone(),
            settings.clone(),
            session.session_id.clone(),
            &config,
        )
        .await?;
        let session_id = session.session_id.clone();
        let model_name = chat_session.model_name().to_string();

        {
            let mut sessions = self.inner.sessions.lock().await;
            sessions.insert(
                session_id.clone(),
                ChatSessionRuntime {
                    input_tx: input_tx.clone(),
                    event_tx: event_tx.clone(),
                    history: history.clone(),
                    _keeper: keeper_rx,
                },
            );
        }

        let manager = self.inner.clone();
        let session_key = session_id.clone();
        let history_for_task = history.clone();
        let event_tx_for_task = event_tx.clone();

        tokio::spawn(async move {
            tracing::info!(
                "Resumed chat session task started for session {}",
                session_key
            );
            while let Some(message) = input_rx.recv().await {
                tracing::info!("Received message in session {}: {}", session_key, message);
                let history_for_sink = history_for_task.clone();
                let event_tx_for_sink = event_tx_for_task.clone();
                let mut sink = move |event: ChatEvent| {
                    store_event(&history_for_sink, &event);
                    tracing::debug!("Broadcasting event: {:?}", event);
                    match event_tx_for_sink.send(event) {
                        Ok(count) => tracing::info!("Event sent to {} subscribers", count),
                        Err(e) => tracing::error!("Failed to broadcast event: {:?}", e),
                    }
                };

                if let Err(err) = chat_session
                    .send_message_with_observer(&message, &mut sink)
                    .await
                {
                    tracing::error!("Chat session error: {}", err);
                    let sanitised_error = sanitise_error_message(&format!("Chat error: {}", err));
                    let error_event = ChatEvent::AssistantMessage {
                        text: sanitised_error,
                    };
                    store_event(&history_for_task, &error_event);
                    tracing::info!("Broadcasting error event");
                    match event_tx_for_task.send(error_event) {
                        Ok(count) => tracing::info!("Error event sent to {} subscribers", count),
                        Err(e) => tracing::error!("Failed to send error event: {:?}", e),
                    }
                }
            }

            tracing::info!("Chat session task ending for session {}", session_key);
            let mut sessions = manager.sessions.lock().await;
            sessions.remove(&session_key);
        });

        Ok(StartedChatSession {
            session_id,
            model_name,
        })
    }

    pub async fn is_session_active(&self, session_id: &str) -> bool {
        let sessions = self.inner.sessions.lock().await;
        sessions.contains_key(session_id)
    }

    pub async fn enqueue_message(&self, session_id: &str, message: String) -> Result<()> {
        tracing::info!("Enqueuing message to session {}: {}", session_id, message);
        let tx = {
            let sessions = self.inner.sessions.lock().await;
            tracing::debug!("Active sessions: {}", sessions.len());
            sessions
                .get(session_id)
                .map(|session| session.input_tx.clone())
        };

        match tx {
            Some(tx) => {
                tracing::debug!("Found session, sending to channel");
                tx.send(message)
                    .await
                    .map_err(|_| anyhow!("chat session closed"))
            }
            None => {
                tracing::error!("Session {} not found", session_id);
                Err(anyhow!("chat session not found"))
            }
        }
    }

    pub async fn subscribe(
        &self,
        session_id: &str,
    ) -> Result<(Vec<ChatEvent>, broadcast::Receiver<ChatEvent>)> {
        tracing::info!("Subscribing to session {}", session_id);
        let rx = {
            let sessions = self.inner.sessions.lock().await;
            tracing::debug!("Active sessions for subscription: {}", sessions.len());
            sessions.get(session_id).and_then(|session| {
                let history = {
                    let guard = session
                        .history
                        .lock()
                        .map_err(|e| {
                            tracing::error!("Failed to acquire lock on session history: {}", e)
                        })
                        .ok()?;
                    guard.iter().cloned().collect::<Vec<_>>()
                };
                Some((history, session.event_tx.subscribe()))
            })
        };

        match &rx {
            Some(_) => tracing::info!("Subscription successful for session {}", session_id),
            None => tracing::error!("Session {} not found for subscription", session_id),
        }

        rx.ok_or_else(|| anyhow!("chat session not found"))
    }
}

fn store_event(history: &Arc<StdMutex<VecDeque<ChatEvent>>>, event: &ChatEvent) {
    match history.lock() {
        Ok(mut guard) => {
            guard.push_back(event.clone());
            if guard.len() > MAX_EVENT_HISTORY {
                guard.pop_front();
            }
        }
        Err(e) => {
            tracing::error!("Failed to acquire lock on event history for storage: {}", e);
        }
    }
}

/// Sanitise API keys and secrets from error messages before sending to clients
fn sanitise_error_message(msg: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;

    // Pattern to match API keys in URLs (query parameters)
    // Matches patterns like: ?key=ACTUAL_KEY or &key=ACTUAL_KEY
    static RE_API_KEY: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"([?&]key=)[A-Za-z0-9_-]+")
            .expect("Invalid regex pattern for API key sanitisation")
    });
    let sanitised = RE_API_KEY.replace_all(msg, "${1}[REDACTED]");

    // Also sanitise bearer tokens if present
    static RE_BEARER: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(Bearer\s+)[A-Za-z0-9_.-]+")
            .expect("Invalid regex pattern for bearer token sanitisation")
    });
    RE_BEARER
        .replace_all(&sanitised, "${1}[REDACTED]")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitise_error_message_query_param() {
        let msg = "HTTP error at https://api.example.com/chat?key=sk-1234567890abcdef";
        let result = sanitise_error_message(msg);
        assert_eq!(
            result,
            "HTTP error at https://api.example.com/chat?key=[REDACTED]"
        );
    }

    #[test]
    fn test_sanitise_error_message_ampersand_param() {
        let msg = "Request failed: https://api.example.com/v1?model=gpt-4&key=AIzaSyD123456789";
        let result = sanitise_error_message(msg);
        assert_eq!(
            result,
            "Request failed: https://api.example.com/v1?model=gpt-4&key=[REDACTED]"
        );
    }

    #[test]
    fn test_sanitise_error_message_bearer_token() {
        let msg = "Authorization failed with Bearer sk-proj-1234567890abcdefghijklmnop";
        let result = sanitise_error_message(msg);
        assert_eq!(result, "Authorization failed with Bearer [REDACTED]");
    }

    #[test]
    fn test_sanitise_error_message_multiple_keys() {
        let msg = "Failed: ?key=secret123 and Bearer token-abc-def and &key=another-key";
        let result = sanitise_error_message(msg);
        assert_eq!(
            result,
            "Failed: ?key=[REDACTED] and Bearer [REDACTED] and &key=[REDACTED]"
        );
    }

    #[test]
    fn test_sanitise_error_message_no_secrets() {
        let msg = "Simple error message with no secrets";
        let result = sanitise_error_message(msg);
        assert_eq!(result, "Simple error message with no secrets");
    }

    #[test]
    fn test_sanitise_error_message_anthropic_api_key() {
        let msg = "Anthropic API error: Bearer sk-ant-api03-1234567890-abcdefghijklmnop";
        let result = sanitise_error_message(msg);
        assert_eq!(result, "Anthropic API error: Bearer [REDACTED]");
    }

    #[test]
    fn test_sanitise_error_message_gemini_api_key() {
        let msg = "Gemini request to https://generativelanguage.googleapis.com/v1beta/models?key=AIzaSyABCDEF123456789";
        let result = sanitise_error_message(msg);
        assert_eq!(
            result,
            "Gemini request to https://generativelanguage.googleapis.com/v1beta/models?key=[REDACTED]"
        );
    }
}
