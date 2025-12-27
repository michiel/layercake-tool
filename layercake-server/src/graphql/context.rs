#![allow(dead_code)]

use crate::graphql::chat_manager::ChatManager;
use layercake_core::auth::Actor;
use layercake_core::app_context::AppContext;
use crate::chat::ChatConfig;
use layercake_core::database::entities::project_collaborators;
use layercake_core::services::{
    authorization::AuthorizationService, ExportService, GraphService, ImportService, PlanDagService,
    SystemSettingsService,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct GraphQLContext {
    pub app: Arc<AppContext>,
    pub db: sea_orm::DatabaseConnection,
    pub import_service: Arc<ImportService>,
    pub export_service: Arc<ExportService>,
    pub graph_service: Arc<GraphService>,
    pub plan_dag_service: Arc<PlanDagService>,
    pub session_manager: Arc<SessionManager>,
    pub chat_manager: Arc<ChatManager>,
    pub system_settings: Arc<SystemSettingsService>,
}

#[derive(Clone, Debug)]
pub struct RequestSession(pub String);

impl RequestSession {
    pub fn as_str(&self) -> &str {
        &self.0
    }
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

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
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
            "#ef4444", "#f97316", "#eab308", "#22c55e", "#06b6d4", "#3b82f6", "#8b5cf6", "#ec4899",
            "#14b8a6", "#f59e0b",
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
        app: Arc<AppContext>,
        system_settings: Arc<SystemSettingsService>,
        chat_manager: Arc<ChatManager>,
    ) -> Self {
        let db = app.db().clone();
        let import_service = Arc::clone(app.import_service());
        let export_service = Arc::clone(app.export_service());
        let graph_service = Arc::clone(app.graph_service());
        let plan_dag_service = Arc::clone(app.plan_dag_service());

        Self {
            app,
            db,
            import_service,
            export_service,
            graph_service,
            plan_dag_service,
            session_manager: Arc::new(SessionManager::new()),
            system_settings,
            chat_manager,
        }
    }

    /// Extract session ID from GraphQL context headers (browser-generated session ID)
    pub fn get_session_id(&self, ctx: &async_graphql::Context<'_>) -> String {
        // In a real implementation, this would come from HTTP headers
        // For now, use a simple approach with context extensions
        ctx.data_opt::<RequestSession>()
            .map(|session| session.0.clone())
            .or_else(|| ctx.data_opt::<String>().cloned())
            .unwrap_or_else(|| {
            // Generate a session ID based on connection info
            format!(
                "browser_session_{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
            )
        })
    }

    pub fn db(&self) -> &sea_orm::DatabaseConnection {
        &self.db
    }

    pub fn import_service(&self) -> Arc<ImportService> {
        self.import_service.clone()
    }

    pub fn export_service(&self) -> Arc<ExportService> {
        self.export_service.clone()
    }

    pub fn graph_service(&self) -> Arc<GraphService> {
        self.graph_service.clone()
    }

    pub fn plan_dag_service(&self) -> Arc<PlanDagService> {
        self.plan_dag_service.clone()
    }

    pub async fn chat_config(&self) -> Arc<ChatConfig> {
        let values = self.system_settings.settings_map().await;
        Arc::new(ChatConfig::from_map(&values))
    }

    pub async fn actor_for_request(&self, ctx: &async_graphql::Context<'_>) -> Actor {
        let session_id = self.get_session_id(ctx);
        let auth_service = AuthorizationService::new(self.db.clone());

        if let Ok(user) = auth_service.get_user_from_session(&session_id).await {
            if let Some(role) = self.resolve_user_role(user.id).await {
                return Actor::user(user.id).with_role(role);
            }

            return Actor::user(user.id);
        }

        let session = self.session_manager.get_or_create_session(&session_id).await;
        Actor::user(session.user_id).with_role("viewer")
    }

    async fn resolve_user_role(&self, user_id: i32) -> Option<String> {
        let collaborations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .filter(project_collaborators::Column::InvitationStatus.eq("accepted"))
            .all(&self.db)
            .await
            .ok()?;

        let mut best_role = None;
        for collab in collaborations {
            match collab.role.to_lowercase().as_str() {
                "owner" => return Some("owner".to_string()),
                "editor" => best_role = Some("editor".to_string()),
                "viewer" => {
                    if best_role.is_none() {
                        best_role = Some("viewer".to_string());
                    }
                }
                _ => {}
            }
        }

        best_role
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layercake_core::database::entities::{project_collaborators, projects, users};
    use layercake_core::database::migrations::Migrator;
    use layercake_core::services::system_settings_service::SystemSettingsService;
    use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
    use sea_orm_migration::MigratorTrait;

    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test database");
        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
        db
    }

    async fn seed_project(db: &DatabaseConnection, name: &str) -> projects::Model {
        let mut project = projects::ActiveModel::new();
        project.name = Set(name.to_string());
        project.insert(db).await.expect("Failed to insert project")
    }

    async fn seed_user(db: &DatabaseConnection, username: &str) -> users::Model {
        let mut user = users::ActiveModel::new();
        user.email = Set(format!("{}@example.com", username));
        user.username = Set(username.to_string());
        user.display_name = Set(username.to_string());
        user.password_hash = Set("hash".to_string());
        user.insert(db).await.expect("Failed to insert user")
    }

    async fn seed_collaborator(
        db: &DatabaseConnection,
        project: &projects::Model,
        user: &users::Model,
        role: project_collaborators::ProjectRole,
    ) -> project_collaborators::Model {
        let collaborator = project_collaborators::ActiveModel::new(
            project.id,
            user.id,
            role,
            Some(user.id),
        )
        .accept_invitation();

        collaborator
            .insert(db)
            .await
            .expect("Failed to insert collaborator")
    }

    #[tokio::test]
    async fn resolve_user_role_prefers_owner_over_editor() {
        let db = setup_db().await;
        let user = seed_user(&db, "role_test").await;
        let project_a = seed_project(&db, "Project A").await;
        let project_b = seed_project(&db, "Project B").await;

        seed_collaborator(
            &db,
            &project_a,
            &user,
            project_collaborators::ProjectRole::Editor,
        )
        .await;
        seed_collaborator(
            &db,
            &project_b,
            &user,
            project_collaborators::ProjectRole::Owner,
        )
        .await;

        let app = Arc::new(AppContext::new(db.clone()));
        let system_settings = Arc::new(
            SystemSettingsService::new(db.clone())
                .await
                .expect("System settings init"),
        );
        let chat_manager = Arc::new(ChatManager::new());
        let context = GraphQLContext::new(app, system_settings, chat_manager);

        let role = context
            .resolve_user_role(user.id)
            .await
            .expect("Expected collaborator role");

        assert_eq!(role, "owner");
    }
}
