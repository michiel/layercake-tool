use std::time::Instant;
use chrono::{DateTime, Utc};
use dashmap::mapref::one::Ref;

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
    pub fn join_project_session(
        &self,
        project_id: i32,
        user_id: String,
        user_name: String,
        avatar_color: String,
        tx: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
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
        self.broadcast_user_presence(&project, &user_id)?;

        Ok(())
    }

    /// Leave a project session
    pub fn leave_project_session(&self, project_id: i32, user_id: &str) -> Result<(), String> {
        if let Some(project) = self.state.projects.get(&project_id) {
            project.users.remove(user_id);
            project.connections.remove(user_id);

            // Remove user from all documents
            for doc in project.documents.iter_mut() {
                doc.active_users.remove(user_id);
            }

            // Broadcast updated presence
            self.broadcast_user_presence(&project, user_id)?;
        }

        Ok(())
    }

    /// Join or switch to a document within a project
    pub fn join_document(
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
        self.broadcast_document_activity(&project, &document_id)?;

        Ok(())
    }

    /// Update cursor position for a user in a document
    pub fn update_cursor_position(
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
        self.broadcast_document_activity(&project, document_id)?;

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
    fn broadcast_user_presence(
        &self,
        project: &Ref<i32, ProjectSession>,
        changed_user_id: &str,
    ) -> Result<(), String> {
        let presence_data = self.collect_user_presence_data(project);

        // Send to all connected users except the one who changed
        for connection in project.connections.iter() {
            if connection.key() != changed_user_id {
                let message = ServerMessage::BulkPresence {
                    data: presence_data.clone(),
                };

                if let Err(_) = connection.value().send(message) {
                    // Connection is dead, will be cleaned up later
                    tracing::warn!("Failed to send message to user {}", connection.key());
                }
            }
        }

        Ok(())
    }

    /// Broadcast document activity to all users in the project
    fn broadcast_document_activity(
        &self,
        project: &Ref<i32, ProjectSession>,
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

            // Send to all connected users in the project
            for connection in project.connections.iter() {
                if let Err(_) = connection.value().send(message.clone()) {
                    tracing::warn!("Failed to send document activity to user {}", connection.key());
                }
            }
        }

        Ok(())
    }

    /// Collect user presence data for a project
    fn collect_user_presence_data(&self, project: &Ref<i32, ProjectSession>) -> Vec<UserPresenceData> {
        project.users
            .iter()
            .map(|entry| {
                let user = entry.value();
                let mut documents = std::collections::HashMap::new();

                // Collect document presence for this user
                for doc_entry in project.documents.iter() {
                    if let Some(user_state) = doc_entry.active_users.get(entry.key()) {
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