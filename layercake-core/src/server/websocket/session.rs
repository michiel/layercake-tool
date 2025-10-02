use std::time::Instant;
use chrono::{DateTime, Utc};
use dashmap::mapref::one::Ref;
use tokio::sync::mpsc;

use super::types::{
    CollaborationState, ProjectSession, UserPresence, DocumentSession, DocumentUserState,
    DocumentType, CursorPosition, ServerMessage, UserPresenceData, DocumentPresence,
    DocumentActivityData, DocumentUser,
};

pub struct SessionManager {
    state: CollaborationState,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            state: CollaborationState::new(),
        }
    }

    /// Join a project session
    pub async fn join_project_session(
        &self,
        project_id: i32,
        user_id: String,
        user_name: String,
        avatar_color: String,
        tx: tokio::sync::mpsc::Sender<ServerMessage>,
    ) -> Result<(), String> {
        let project = self.state.get_or_create_project(project_id);

        // Add user presence
        let user_presence = UserPresence {
            user_id: user_id.clone(),
            user_name,
            avatar_color,
            is_online: true,
            last_active: Instant::now(),
        };

        project.users.insert(user_id.clone(), user_presence);
        project.connections.insert(user_id.clone(), tx);

        // Broadcast user presence to all other users in the project
        self.broadcast_user_presence(&project, &user_id).await?;

        Ok(())
    }

    /// Leave a project session
    pub async fn leave_project_session(&self, project_id: i32, user_id: &str) -> Result<(), String> {
        if let Some(project) = self.state.projects.get(&project_id) {
            project.users.remove(user_id);
            project.connections.remove(user_id);

            // Remove user from all documents
            // CRITICAL FIX: Collect document IDs first to avoid holding DashMap iterator
            let document_ids: Vec<String> = project
                .documents
                .iter()
                .map(|entry| entry.key().clone())
                .collect();

            for doc_id in document_ids {
                if let Some(doc) = project.documents.get_mut(&doc_id) {
                    doc.active_users.remove(user_id);
                }
            }

            // Broadcast updated presence
            self.broadcast_user_presence(&project, user_id).await?;
        }

        Ok(())
    }

    /// Join or switch to a document within a project
    pub async fn join_document(
        &self,
        project_id: i32,
        user_id: &str,
        document_id: String,
        document_type: DocumentType,
    ) -> Result<(), String> {
        let project = self.state.get_or_create_project(project_id);

        // Create or get document session
        let doc_entry = project.documents.entry(document_id.clone()).or_insert_with(|| {
            DocumentSession {
                document_type: document_type.clone(),
                active_users: dashmap::DashMap::new(),
            }
        });

        // Add user to document
        let user_state = DocumentUserState {
            position: None,
            selected_node_id: None,
            last_active_in_document: Instant::now(),
        };

        doc_entry.active_users.insert(user_id.to_string(), user_state);

        // Update user's last active time
        if let Some(mut user) = project.users.get_mut(user_id) {
            user.last_active = Instant::now();
        }

        // Broadcast document activity
        self.broadcast_document_activity(&project, &document_id).await?;

        Ok(())
    }

    /// Update cursor position for a user in a document
    pub async fn update_cursor_position(
        &self,
        project_id: i32,
        user_id: &str,
        document_id: &str,
        position: CursorPosition,
        selected_node_id: Option<String>,
    ) -> Result<(), String> {
        let project = self.state.get_or_create_project(project_id);

        // Update document user state
        if let Some(doc) = project.documents.get(document_id) {
            if let Some(mut user_state) = doc.active_users.get_mut(user_id) {
                user_state.position = Some(position);
                user_state.selected_node_id = selected_node_id;
                user_state.last_active_in_document = Instant::now();
            }
        }

        // Update user's last active time
        if let Some(mut user) = project.users.get_mut(user_id) {
            user.last_active = Instant::now();
        }

        // Broadcast updated document activity
        self.broadcast_document_activity(&project, document_id).await?;

        Ok(())
    }

    /// Get all users in a project
    #[allow(dead_code)]
    pub fn get_project_users(&self, project_id: i32) -> Vec<UserPresenceData> {
        if let Some(project) = self.state.projects.get(&project_id) {
            self.collect_user_presence_data(&project)
        } else {
            Vec::new()
        }
    }

    /// Broadcast user presence to all connections in a project
    async fn broadcast_user_presence<'a>(
        &self,
        project: &Ref<'a, i32, ProjectSession>,
        changed_user_id: &str,
    ) -> Result<(), String> {
        let presence_data = self.collect_user_presence_data(project);

        // CRITICAL FIX: Collect all connections into a Vec FIRST to avoid holding
        // DashMap iterator while performing async operations. The old pattern of
        // iterating directly over project.connections.iter() causes complete server
        // deadlock when async tasks modify the map concurrently.
        let connections: Vec<(String, mpsc::Sender<ServerMessage>)> = project
            .connections
            .iter()
            .filter(|entry| entry.key() != changed_user_id)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // Collect dead connections for cleanup
        let mut dead_connections = Vec::new();

        // Send to all connected users except the one who changed
        for (user_id, sender) in connections {
            let message = ServerMessage::BulkPresence {
                data: presence_data.clone(),
            };

            if let Err(_) = sender.send(message).await {
                // Connection is dead, mark for cleanup
                tracing::warn!("Dead connection detected for user {}, scheduling removal", user_id);
                dead_connections.push(user_id);
            }
        }

        // Remove dead connections immediately
        for user_id in dead_connections {
            tracing::info!("Removing dead connection for user {}", user_id);
            project.connections.remove(&user_id);
        }

        Ok(())
    }

    /// Broadcast document activity to all users in the project
    async fn broadcast_document_activity<'a>(
        &self,
        project: &Ref<'a, i32, ProjectSession>,
        document_id: &str,
    ) -> Result<(), String> {
        if let Some(doc) = project.documents.get(document_id) {
            let active_users: Vec<DocumentUser> = doc.active_users
                .iter()
                .filter_map(|entry| {
                    // Get user name from project users
                    project.users.get(entry.key()).map(|user| {
                        DocumentUser {
                            user_id: entry.key().clone(),
                            user_name: user.user_name.clone(),
                            position: entry.value().position.clone(),
                            selected_node_id: entry.value().selected_node_id.clone(),
                        }
                    })
                })
                .collect();

            let activity_data = DocumentActivityData {
                document_id: document_id.to_string(),
                active_users,
            };

            let message = ServerMessage::DocumentActivity {
                data: activity_data,
            };

            // CRITICAL FIX: Collect all connections into a Vec FIRST to avoid holding
            // DashMap iterator while performing async operations. The old pattern of
            // iterating directly over project.connections.iter() causes complete server
            // deadlock when async tasks modify the map concurrently.
            let connections: Vec<(String, mpsc::Sender<ServerMessage>)> = project
                .connections
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect();

            // Collect dead connections for cleanup
            let mut dead_connections = Vec::new();

            // Send to all connected users in the project
            for (user_id, sender) in connections {
                if let Err(_) = sender.send(message.clone()).await {
                    // Connection is dead, mark for cleanup
                    tracing::warn!("Dead connection detected for user {} (document activity), scheduling removal", user_id);
                    dead_connections.push(user_id);
                }
            }

            // Remove dead connections immediately
            for user_id in dead_connections {
                tracing::info!("Removing dead connection for user {}", user_id);
                project.connections.remove(&user_id);
            }
        }

        Ok(())
    }

    /// Collect user presence data for a project
    fn collect_user_presence_data(&self, project: &Ref<i32, ProjectSession>) -> Vec<UserPresenceData> {
        // CRITICAL FIX: This method had nested DashMap iterations causing deadlock!
        // The old pattern: project.users.iter() -> map -> project.documents.iter()
        // was holding iterators across both maps simultaneously.
        //
        // Solution: Collect all users first, then collect all documents, avoiding nested iterations.

        // First pass: Collect all user data (clone values to release iterator quickly)
        let users_snapshot: Vec<(String, UserPresence)> = project
            .users
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        // Second pass: For each user, collect their document presence
        users_snapshot
            .into_iter()
            .map(|(user_id, user)| {
                let mut documents = std::collections::HashMap::new();

                // Collect document presence for this user
                // Now we're not holding any iterators when we do this lookup
                for doc_entry in project.documents.iter() {
                    if let Some(user_state) = doc_entry.active_users.get(&user_id) {
                        documents.insert(
                            doc_entry.key().clone(),
                            DocumentPresence {
                                document_type: doc_entry.document_type.clone(),
                                position: user_state.position.clone(),
                                selected_node_id: user_state.selected_node_id.clone(),
                                last_active_in_document: Self::instant_to_iso_string(user_state.last_active_in_document),
                            },
                        );
                    }
                }

                UserPresenceData {
                    user_id: user.user_id.clone(),
                    user_name: user.user_name.clone(),
                    avatar_color: user.avatar_color.clone(),
                    is_online: user.is_online,
                    last_active: Self::instant_to_iso_string(user.last_active),
                    documents,
                }
            })
            .collect()
    }

    /// Convert Instant to ISO 8601 string
    fn instant_to_iso_string(instant: Instant) -> String {
        let duration_since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        let instant_duration = instant.elapsed();
        let timestamp = duration_since_epoch.saturating_sub(instant_duration);

        let datetime: DateTime<Utc> = DateTime::from_timestamp(
            timestamp.as_secs() as i64,
            timestamp.subsec_nanos(),
        ).unwrap_or_else(|| Utc::now());

        datetime.to_rfc3339()
    }

    /// Clean up inactive sessions (to be called periodically)
    #[allow(dead_code)]
    pub fn cleanup_inactive_sessions(&self, max_inactive_duration: std::time::Duration) {
        let now = Instant::now();

        for project in self.state.projects.iter() {
            // Clean up inactive users
            let inactive_users: Vec<String> = project.users
                .iter()
                .filter_map(|entry| {
                    if now.duration_since(entry.value().last_active) > max_inactive_duration {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            for user_id in inactive_users {
                project.users.remove(&user_id);
                project.connections.remove(&user_id);

                // Remove from all documents
                for doc in project.documents.iter_mut() {
                    doc.active_users.remove(&user_id);
                }
            }

            // Clean up empty documents
            let empty_docs: Vec<String> = project.documents
                .iter()
                .filter_map(|entry| {
                    if entry.active_users.is_empty() {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            for doc_id in empty_docs {
                project.documents.remove(&doc_id);
            }
        }

        // Clean up empty projects
        let empty_projects: Vec<i32> = self.state.projects
            .iter()
            .filter_map(|entry| {
                if entry.users.is_empty() && entry.documents.is_empty() {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        for project_id in empty_projects {
            self.state.projects.remove(&project_id);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}