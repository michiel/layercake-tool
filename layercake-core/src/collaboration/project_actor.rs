use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use super::types::{ProjectCommand, ProjectHealthReport};
use crate::server::websocket::types::{
    CursorPosition, DocumentActivityData, DocumentPresence, DocumentType, DocumentUser,
    ServerMessage, UserPresenceData,
};

/// Actor for managing a single project's collaboration state
pub struct ProjectActor {
    #[allow(dead_code)]
    project_id: i32,
    command_tx: mpsc::Sender<ProjectCommand>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl ProjectActor {
    pub fn spawn(project_id: i32) -> Self {
        let (tx, rx) = mpsc::channel(1000);

        let task_handle = tokio::spawn(async move {
            let state = ProjectState {
                project_id,
                users: HashMap::new(),
                connections: HashMap::new(),
                documents: HashMap::new(),
            };

            state.run(rx).await;
        });

        debug!("ProjectActor spawned for project {}", project_id);

        Self {
            project_id,
            command_tx: tx,
            task_handle,
        }
    }

    pub async fn join(
        &self,
        user_id: String,
        user_name: String,
        avatar_color: Option<String>,
        sender: mpsc::Sender<ServerMessage>,
    ) -> Result<(), String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.command_tx
            .send(ProjectCommand::Join {
                user_id,
                user_name,
                avatar_color,
                sender,
                response: tx,
            })
            .await
            .map_err(|_| "Project actor unavailable".to_string())?;

        rx.await
            .map_err(|_| "Response channel closed".to_string())?
    }

    pub async fn leave(&self, user_id: &str) -> Result<(), String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.command_tx
            .send(ProjectCommand::Leave {
                user_id: user_id.to_string(),
                response: tx,
            })
            .await
            .map_err(|_| "Project actor unavailable".to_string())?;

        rx.await
            .map_err(|_| "Response channel closed".to_string())?
    }

    pub async fn update_cursor(
        &self,
        user_id: String,
        document_id: String,
        position: CursorPosition,
        selected_node_id: Option<String>,
    ) {
        let _ = self
            .command_tx
            .send(ProjectCommand::UpdateCursor {
                user_id,
                document_id,
                position,
                selected_node_id,
            })
            .await;
    }

    pub async fn switch_document(
        &self,
        user_id: String,
        document_id: String,
        document_type: DocumentType,
    ) {
        let _ = self
            .command_tx
            .send(ProjectCommand::SwitchDocument {
                user_id,
                document_id,
                document_type,
            })
            .await;
    }

    pub async fn is_empty(&self) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self
            .command_tx
            .send(ProjectCommand::IsEmpty { response: tx })
            .await
            .is_err()
        {
            return true; // Actor dead = empty
        }
        rx.await.unwrap_or(true)
    }

    pub async fn health_report(&self) -> ProjectHealthReport {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self
            .command_tx
            .send(ProjectCommand::HealthReport { response: tx })
            .await
            .is_err()
        {
            return ProjectHealthReport::not_found();
        }
        rx.await
            .unwrap_or_else(|_| ProjectHealthReport::not_found())
    }

    pub async fn shutdown(self) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self
            .command_tx
            .send(ProjectCommand::Shutdown { response: tx })
            .await;
        let _ = rx.await;
        let _ = self.task_handle.await;
    }
}

/// Internal state for a project actor
struct ProjectState {
    project_id: i32,
    users: HashMap<String, UserData>,
    connections: HashMap<String, mpsc::Sender<ServerMessage>>,
    documents: HashMap<String, DocumentData>,
}

#[derive(Clone)]
struct UserData {
    user_id: String,
    user_name: String,
    avatar_color: String,
    #[allow(dead_code)]
    joined_at: Instant,
    last_active: Instant,
}

struct DocumentData {
    document_type: DocumentType,
    active_users: HashMap<String, DocumentUserData>,
}

#[derive(Clone)]
struct DocumentUserData {
    position: Option<CursorPosition>,
    selected_node_id: Option<String>,
    last_active: Instant,
}

impl ProjectState {
    async fn run(mut self, mut command_rx: mpsc::Receiver<ProjectCommand>) {
        debug!(
            "ProjectState event loop started for project {}",
            self.project_id
        );

        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                ProjectCommand::Join {
                    user_id,
                    user_name,
                    avatar_color,
                    sender,
                    response,
                } => {
                    let user_data = UserData {
                        user_id: user_id.clone(),
                        user_name: user_name.clone(),
                        avatar_color: avatar_color
                            .clone()
                            .unwrap_or_else(|| "#4A90E2".to_string()),
                        joined_at: Instant::now(),
                        last_active: Instant::now(),
                    };

                    self.users.insert(user_id.clone(), user_data.clone());
                    self.connections.insert(user_id.clone(), sender.clone());

                    debug!("User {} joined project {}", user_id, self.project_id);

                    // Send current presence to new user
                    let presence_list = self.collect_presence_data();
                    let welcome_msg = ServerMessage::BulkPresence {
                        data: presence_list,
                    };
                    let _ = sender.send(welcome_msg).await;

                    // Broadcast new user to others
                    self.broadcast_user_presence(&user_id).await;

                    let _ = response.send(Ok(()));
                }

                ProjectCommand::Leave { user_id, response } => {
                    debug!("User {} leaving project {}", user_id, self.project_id);

                    self.users.remove(&user_id);
                    self.connections.remove(&user_id);

                    // Remove from all documents
                    for doc_data in self.documents.values_mut() {
                        doc_data.active_users.remove(&user_id);
                    }

                    // Broadcast user left
                    self.broadcast_user_left(&user_id).await;

                    let _ = response.send(Ok(()));
                }

                ProjectCommand::UpdateCursor {
                    user_id,
                    document_id,
                    position,
                    selected_node_id,
                } => {
                    if let Some(user) = self.users.get_mut(&user_id) {
                        user.last_active = Instant::now();
                    }

                    // Update document state
                    let doc = self
                        .documents
                        .entry(document_id.clone())
                        .or_insert_with(|| DocumentData {
                            document_type: DocumentType::Canvas, // Default, will be updated on SwitchDocument
                            active_users: HashMap::new(),
                        });

                    doc.active_users.insert(
                        user_id.clone(),
                        DocumentUserData {
                            position: Some(position.clone()),
                            selected_node_id: selected_node_id.clone(),
                            last_active: Instant::now(),
                        },
                    );

                    // Broadcast cursor update to users in same document
                    self.broadcast_cursor_update(
                        &user_id,
                        &document_id,
                        position,
                        selected_node_id,
                    )
                    .await;
                }

                ProjectCommand::SwitchDocument {
                    user_id,
                    document_id,
                    document_type,
                } => {
                    if let Some(user) = self.users.get_mut(&user_id) {
                        user.last_active = Instant::now();
                    }

                    // Create or update document
                    let doc = self
                        .documents
                        .entry(document_id.clone())
                        .or_insert_with(|| DocumentData {
                            document_type: document_type.clone(),
                            active_users: HashMap::new(),
                        });

                    doc.document_type = document_type.clone();
                    doc.active_users.insert(
                        user_id.clone(),
                        DocumentUserData {
                            position: None,
                            selected_node_id: None,
                            last_active: Instant::now(),
                        },
                    );

                    // Broadcast document activity
                    self.broadcast_document_activity(&document_id).await;
                }

                ProjectCommand::IsEmpty { response } => {
                    let _ = response.send(self.connections.is_empty());
                }

                ProjectCommand::HealthReport { response } => {
                    let report = ProjectHealthReport {
                        project_id: self.project_id,
                        active_users: self.users.len(),
                        active_connections: self.connections.len(),
                        active_documents: self.documents.len(),
                    };
                    let _ = response.send(report);
                }

                ProjectCommand::Shutdown { response } => {
                    debug!("ProjectState shutting down for project {}", self.project_id);

                    // Send disconnect to all users
                    for (user_id, connection) in self.connections.drain() {
                        let disconnect_msg = ServerMessage::Error {
                            message: "Server shutting down".to_string(),
                        };
                        if let Err(_) = connection.send(disconnect_msg).await {
                            warn!("Failed to send disconnect to user {}", user_id);
                        }
                    }

                    let _ = response.send(());
                    break;
                }
            }
        }

        debug!(
            "ProjectState event loop ended for project {}",
            self.project_id
        );
    }

    fn collect_presence_data(&self) -> Vec<UserPresenceData> {
        self.users
            .iter()
            .map(|(user_id, user_data)| {
                let mut documents = HashMap::new();

                // Collect document presence for this user
                for (doc_id, doc_data) in &self.documents {
                    if let Some(doc_user_data) = doc_data.active_users.get(user_id) {
                        documents.insert(
                            doc_id.clone(),
                            DocumentPresence {
                                document_type: doc_data.document_type.clone(),
                                position: doc_user_data.position.clone(),
                                selected_node_id: doc_user_data.selected_node_id.clone(),
                                last_active_in_document: format_instant(doc_user_data.last_active),
                            },
                        );
                    }
                }

                UserPresenceData {
                    user_id: user_data.user_id.clone(),
                    user_name: user_data.user_name.clone(),
                    avatar_color: user_data.avatar_color.clone(),
                    is_online: true,
                    last_active: format_instant(user_data.last_active),
                    documents,
                }
            })
            .collect()
    }

    async fn broadcast_user_presence(&self, user_id: &str) {
        let user_data = match self.users.get(user_id) {
            Some(u) => u,
            None => return,
        };

        let mut documents = HashMap::new();
        for (doc_id, doc_data) in &self.documents {
            if let Some(doc_user_data) = doc_data.active_users.get(user_id) {
                documents.insert(
                    doc_id.clone(),
                    DocumentPresence {
                        document_type: doc_data.document_type.clone(),
                        position: doc_user_data.position.clone(),
                        selected_node_id: doc_user_data.selected_node_id.clone(),
                        last_active_in_document: format_instant(doc_user_data.last_active),
                    },
                );
            }
        }

        let presence_data = UserPresenceData {
            user_id: user_data.user_id.clone(),
            user_name: user_data.user_name.clone(),
            avatar_color: user_data.avatar_color.clone(),
            is_online: true,
            last_active: format_instant(user_data.last_active),
            documents,
        };

        let message = ServerMessage::UserPresence {
            data: presence_data,
        };

        // Broadcast to all other users
        for (target_user_id, connection) in &self.connections {
            if target_user_id != user_id {
                if let Err(_) = connection.send(message.clone()).await {
                    warn!(
                        "Dead connection detected for user {} in project {}",
                        target_user_id, self.project_id
                    );
                }
            }
        }
    }

    async fn broadcast_user_left(&self, user_id: &str) {
        let presence_data = UserPresenceData {
            user_id: user_id.to_string(),
            user_name: String::new(),
            avatar_color: String::new(),
            is_online: false,
            last_active: format_instant(Instant::now()),
            documents: HashMap::new(),
        };

        let message = ServerMessage::UserPresence {
            data: presence_data,
        };

        for connection in self.connections.values() {
            let _ = connection.send(message.clone()).await;
        }
    }

    async fn broadcast_cursor_update(
        &self,
        user_id: &str,
        document_id: &str,
        position: CursorPosition,
        selected_node_id: Option<String>,
    ) {
        let user_data = match self.users.get(user_id) {
            Some(u) => u,
            None => return,
        };

        let doc_data = match self.documents.get(document_id) {
            Some(d) => d,
            None => return,
        };

        let doc_user = DocumentUser {
            user_id: user_data.user_id.clone(),
            user_name: user_data.user_name.clone(),
            position: Some(position),
            selected_node_id,
        };

        let message = ServerMessage::DocumentActivity {
            data: DocumentActivityData {
                document_id: document_id.to_string(),
                active_users: vec![doc_user],
            },
        };

        // Broadcast to users in same document
        for target_user_id in doc_data.active_users.keys() {
            if target_user_id != user_id {
                if let Some(connection) = self.connections.get(target_user_id) {
                    let _ = connection.send(message.clone()).await;
                }
            }
        }
    }

    async fn broadcast_document_activity(&self, document_id: &str) {
        let doc_data = match self.documents.get(document_id) {
            Some(d) => d,
            None => return,
        };

        let active_users: Vec<DocumentUser> = doc_data
            .active_users
            .iter()
            .filter_map(|(user_id, doc_user_data)| {
                self.users.get(user_id).map(|user_data| DocumentUser {
                    user_id: user_data.user_id.clone(),
                    user_name: user_data.user_name.clone(),
                    position: doc_user_data.position.clone(),
                    selected_node_id: doc_user_data.selected_node_id.clone(),
                })
            })
            .collect();

        let message = ServerMessage::DocumentActivity {
            data: DocumentActivityData {
                document_id: document_id.to_string(),
                active_users,
            },
        };

        // Broadcast to all users in document
        for target_user_id in doc_data.active_users.keys() {
            if let Some(connection) = self.connections.get(target_user_id) {
                let _ = connection.send(message.clone()).await;
            }
        }
    }
}

fn format_instant(_instant: Instant) -> String {
    // Convert Instant to approximate UTC time for client display
    // Note: This is approximate as Instant doesn't have a direct UTC conversion
    Utc::now().to_rfc3339()
}
