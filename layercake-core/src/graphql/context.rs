#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::collections::HashMap;
use tokio::sync::RwLock;
use sea_orm::DatabaseConnection;
use crate::services::{ImportService, ExportService, GraphService};

#[derive(Clone)]
pub struct GraphQLContext {
    pub db: DatabaseConnection,
    pub import_service: Arc<ImportService>,
    pub export_service: Arc<ExportService>,
    pub graph_service: Arc<GraphService>,
    pub session_manager: Arc<SessionManager>,
}

/// Simple session manager to track browser sessions and assign user IDs
#[derive(Debug)]
pub struct SessionManager {
    sessions: RwLock<HashMap<String, SessionInfo>>,
    next_user_id: AtomicI32,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: i32,
    pub user_name: String,
    pub avatar_color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            next_user_id: AtomicI32::new(100), // Start from 100 to differentiate from real users
        }
    }

    /// Get or create a session for the given session ID
    pub async fn get_or_create_session(&self, session_id: &str) -> SessionInfo {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(session_id) {
            return session.clone();
        }

        drop(sessions);

        // Create new session
        let mut sessions = self.sessions.write().await;

        // Double-check after acquiring write lock
        if let Some(session) = sessions.get(session_id) {
            return session.clone();
        }

        // Atomically fetch and increment the user ID
        let user_id = self.next_user_id.fetch_add(1, Ordering::SeqCst);

        // Generate avatar colors
        let colors = [
            "#ef4444", "#f97316", "#eab308", "#22c55e", "#06b6d4",
            "#3b82f6", "#8b5cf6", "#ec4899", "#14b8a6", "#f59e0b",
        ];
        let color_index = (user_id as usize) % colors.len();
        let avatar_color = colors[color_index].to_string();

        let session_info = SessionInfo {
            user_id,
            user_name: format!("User {}", user_id),
            avatar_color,
            created_at: chrono::Utc::now(),
        };

        sessions.insert(session_id.to_string(), session_info.clone());
        session_info
    }

    /// Get session info if it exists
    pub async fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }
}

impl GraphQLContext {
    pub fn new(
        db: DatabaseConnection,
        import_service: Arc<ImportService>,
        export_service: Arc<ExportService>,
        graph_service: Arc<GraphService>,
    ) -> Self {
        Self {
            db,
            import_service,
            export_service,
            graph_service,
            session_manager: Arc::new(SessionManager::new()),
        }
    }

    /// Extract session ID from GraphQL context headers (browser-generated session ID)
    pub fn get_session_id(&self, ctx: &async_graphql::Context<'_>) -> String {
        // In a real implementation, this would come from HTTP headers
        // For now, use a simple approach with context extensions
        ctx.data_opt::<String>()
            .cloned()
            .unwrap_or_else(|| {
                // Generate a session ID based on connection info
                format!("browser_session_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
            })
    }
}