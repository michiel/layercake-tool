#![cfg(feature = "graphql")]

use std::{collections::HashMap, sync::Arc};

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
}

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
        let (event_tx, _) = broadcast::channel::<ChatEvent>(64);

        let mut chat_session = ChatSession::new(db.clone(), project_id, provider, &config).await?;
        let model_name = chat_session.model_name().to_string();

        {
            let mut sessions = self.inner.sessions.lock().await;
            sessions.insert(
                session_id.clone(),
                ChatSessionRuntime {
                    input_tx: input_tx.clone(),
                    event_tx: event_tx.clone(),
                },
            );
        }

        let manager = self.inner.clone();
        let session_key = session_id.clone();

        tokio::spawn(async move {
            while let Some(message) = input_rx.recv().await {
                let mut sink = |event: ChatEvent| {
                    let _ = event_tx.send(event);
                };

                if let Err(err) = chat_session
                    .send_message_with_observer(&message, &mut sink)
                    .await
                {
                    let _ = event_tx.send(ChatEvent::AssistantMessage {
                        text: format!("Chat error: {}", err),
                    });
                }
            }

            let mut sessions = manager.sessions.lock().await;
            sessions.remove(&session_key);
        });

        Ok(StartedChatSession {
            session_id,
            model_name,
        })
    }

    pub async fn enqueue_message(&self, session_id: &str, message: String) -> Result<()> {
        let tx = {
            let sessions = self.inner.sessions.lock().await;
            sessions
                .get(session_id)
                .map(|session| session.input_tx.clone())
        };

        match tx {
            Some(tx) => tx
                .send(message)
                .await
                .map_err(|_| anyhow!("chat session closed")),
            None => Err(anyhow!("chat session not found")),
        }
    }

    pub async fn subscribe(&self, session_id: &str) -> Result<broadcast::Receiver<ChatEvent>> {
        let rx = {
            let sessions = self.inner.sessions.lock().await;
            sessions
                .get(session_id)
                .map(|session| session.event_tx.subscribe())
        };

        rx.ok_or_else(|| anyhow!("chat session not found"))
    }
}
