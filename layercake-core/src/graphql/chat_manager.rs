#![cfg(feature = "graphql")]

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex as StdMutex},
};

use anyhow::{anyhow, Result};
use tokio::sync::{broadcast, mpsc, Mutex};
use uuid::Uuid;

use crate::console::chat::{ChatConfig, ChatEvent, ChatProvider, ChatSession};
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
        provider: ChatProvider,
        config: Arc<ChatConfig>,
    ) -> Result<StartedChatSession> {
        let session_id = Uuid::new_v4().to_string();
        let (input_tx, mut input_rx) = mpsc::channel::<String>(16);
        let (event_tx, keeper_rx) = broadcast::channel::<ChatEvent>(64);
        let history = Arc::new(StdMutex::new(VecDeque::new()));

        let mut chat_session = ChatSession::new(db.clone(), project_id, provider, &config).await?;
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
                    let error_event = ChatEvent::AssistantMessage {
                        text: format!("Chat error: {}", err),
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
            sessions.get(session_id).map(|session| {
                let history = {
                    let guard = session.history.lock().unwrap();
                    guard.iter().cloned().collect::<Vec<_>>()
                };
                (history, session.event_tx.subscribe())
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
    let mut guard = history.lock().unwrap();
    guard.push_back(event.clone());
    if guard.len() > MAX_EVENT_HISTORY {
        guard.pop_front();
    }
}
