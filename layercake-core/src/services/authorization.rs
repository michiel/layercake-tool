#![allow(dead_code)]

#[cfg(feature = "graphql")]
use async_graphql::{Context, Error};
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::database::entities::{project_collaborators, user_sessions, users};
use crate::errors::{CoreError, CoreResult};
#[cfg(feature = "graphql")]
use crate::graphql::context::GraphQLContext;

/// Authorization service for checking user permissions
#[allow(dead_code)] // Authorization service reserved for future use
#[derive(Clone, Debug)]
pub struct AuthorizationService {
    db: DatabaseConnection,
}

impl AuthorizationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get authenticated user from GraphQL context
    #[cfg(feature = "graphql")]
    pub async fn get_authenticated_user(&self, ctx: &Context<'_>) -> Result<users::Model, Error> {
        // Look for session ID in GraphQL context extensions
        let session_id = ctx
            .data_opt::<String>()
            .ok_or_else(|| Error::new("No session ID provided"))?;

        self.get_user_from_session(session_id)
            .await
            .map_err(|e| Error::new(e.to_string()))
    }

    /// Get user from session ID
    pub async fn get_user_from_session(&self, session_id: &str) -> CoreResult<users::Model> {
        // Find active session
        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::SessionId.eq(session_id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let session =
            session.ok_or_else(|| CoreError::unauthorized("Invalid or expired session"))?;

        // Check if session is expired
        if session.expires_at <= Utc::now() {
            return Err(CoreError::unauthorized("Session expired"));
        }

        // Get user from session
        let user = users::Entity::find_by_id(session.user_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::unauthorized("Invalid or expired session"))?;

        // Check if user is active
        if !user.is_active {
            return Err(CoreError::forbidden("Account is deactivated"));
        }

        Ok(user)
    }

    /// Check if user has access to a project
    pub async fn check_project_access(
        &self,
        user_id: i32,
        project_id: i32,
        required_role: Option<ProjectRole>,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        if local_auth_bypass_enabled() {
            return Ok(ProjectCollaboratorInfo {
                collaboration_id: 0,
                role: ProjectRole::Owner,
                permissions: "[]".to_string(),
                joined_at: Some(Utc::now()),
            });
        }

        let collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let collaboration = collaboration
            .ok_or_else(|| CoreError::forbidden("Access denied: No access to this project"))?;

        // Check invitation status
        if collaboration.invitation_status != "accepted" {
            return Err(CoreError::forbidden(
                "Access denied: Invitation not accepted",
            ));
        }

        let user_role = ProjectRole::from_str(&collaboration.role)
            .map_err(|_| CoreError::internal("Invalid user role"))?;

        // Check if user has required role
        if let Some(required) = required_role {
            if !user_role.has_permission(&required) {
                return Err(CoreError::forbidden(format!(
                    "Access denied: Requires {} role",
                    required.as_str()
                )));
            }
        }

        Ok(ProjectCollaboratorInfo {
            collaboration_id: collaboration.id,
            role: user_role,
            permissions: collaboration.permissions,
            joined_at: collaboration.joined_at,
        })
    }

    /// Check if user can modify a project (owner or editor)
    pub async fn check_project_write_access(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.check_project_access(user_id, project_id, Some(ProjectRole::Editor))
            .await
    }

    /// Check if user can delete/manage a project (owner only)
    pub async fn check_project_admin_access(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.check_project_access(user_id, project_id, Some(ProjectRole::Owner))
            .await
    }

    /// Check if user can view a project (any role)
    pub async fn check_project_read_access(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.check_project_access(user_id, project_id, None).await
    }

    /// Get user's role in a project
    pub async fn get_user_project_role(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> CoreResult<Option<ProjectRole>> {
        let collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        if let Some(collaboration) = collaboration {
            if collaboration.invitation_status == "accepted" {
                let role = ProjectRole::from_str(&collaboration.role)
                    .map_err(|_| CoreError::internal("Invalid role"))?;
                return Ok(Some(role));
            }
        }

        Ok(None)
    }

    /// Check if user can manage collaborators (owner only)
    pub async fn check_collaboration_management_access(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> CoreResult<()> {
        self.check_project_admin_access(user_id, project_id).await?;
        Ok(())
    }
}

fn local_auth_bypass_enabled() -> bool {
    match std::env::var("LAYERCAKE_LOCAL_AUTH_BYPASS") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => cfg!(debug_assertions),
    }
}

/// Project role with permission hierarchy
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectRole {
    Owner,
    Editor,
    Viewer,
}

impl ProjectRole {
    pub fn from_str(s: &str) -> CoreResult<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(ProjectRole::Owner),
            "editor" => Ok(ProjectRole::Editor),
            "viewer" => Ok(ProjectRole::Viewer),
            _ => Err(CoreError::validation(format!(
                "Invalid project role: {}",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectRole::Owner => "owner",
            ProjectRole::Editor => "editor",
            ProjectRole::Viewer => "viewer",
        }
    }

    /// Check if this role has the permissions of another role
    pub fn has_permission(&self, required: &ProjectRole) -> bool {
        use ProjectRole::*;
        match (self, required) {
            (Owner, _) => true,       // Owner can do everything
            (Editor, Editor) => true, // Editor can edit
            (Editor, Viewer) => true, // Editor can view
            (Viewer, Viewer) => true, // Viewer can view
            _ => false,
        }
    }

    /// Check if role can read
    pub fn can_read(&self) -> bool {
        true // All roles can read
    }

    /// Check if role can write/edit
    pub fn can_write(&self) -> bool {
        matches!(self, ProjectRole::Owner | ProjectRole::Editor)
    }

    /// Check if role can delete/admin
    pub fn can_admin(&self) -> bool {
        matches!(self, ProjectRole::Owner)
    }
}

/// Information about a user's collaboration on a project
#[derive(Debug, Clone)]
pub struct ProjectCollaboratorInfo {
    pub collaboration_id: i32,
    pub role: ProjectRole,
    pub permissions: String,
    pub joined_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Middleware for extracting authentication from GraphQL context
#[cfg(feature = "graphql")]
pub async fn extract_auth_from_context(ctx: &Context<'_>) -> Result<AuthenticatedUser, Error> {
    let context = ctx
        .data::<GraphQLContext>()
        .map_err(|_| Error::new("GraphQL context not found"))?;

    let auth_service = AuthorizationService::new(context.db.clone());

    // Try to get session ID from headers or context
    let session_id = ctx
        .data_opt::<String>()
        .ok_or_else(|| Error::new("Authentication required: No session provided"))?;

    let user = auth_service
        .get_user_from_session(session_id)
        .await
        .map_err(|e| Error::new(e.to_string()))?;

    Ok(AuthenticatedUser {
        user,
        session_id: session_id.clone(),
        auth_service,
    })
}

/// Authenticated user with authorization helpers
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: users::Model,
    pub session_id: String,
    pub auth_service: AuthorizationService,
}

impl AuthenticatedUser {
    /// Check project access
    pub async fn check_project_access(
        &self,
        project_id: i32,
        required_role: Option<ProjectRole>,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.auth_service
            .check_project_access(self.user.id, project_id, required_role)
            .await
    }

    /// Check if user can read project
    pub async fn can_read_project(&self, project_id: i32) -> bool {
        self.auth_service
            .check_project_read_access(self.user.id, project_id)
            .await
            .is_ok()
    }

    /// Check if user can write to project
    pub async fn can_write_project(&self, project_id: i32) -> bool {
        self.auth_service
            .check_project_write_access(self.user.id, project_id)
            .await
            .is_ok()
    }

    /// Check if user can admin project
    pub async fn can_admin_project(&self, project_id: i32) -> bool {
        self.auth_service
            .check_project_admin_access(self.user.id, project_id)
            .await
            .is_ok()
    }

    /// Require project read access (throws error if denied)
    pub async fn require_project_read(
        &self,
        project_id: i32,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.auth_service
            .check_project_read_access(self.user.id, project_id)
            .await
    }

    /// Require project write access (throws error if denied)
    pub async fn require_project_write(
        &self,
        project_id: i32,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.auth_service
            .check_project_write_access(self.user.id, project_id)
            .await
    }

    /// Require project admin access (throws error if denied)
    pub async fn require_project_admin(
        &self,
        project_id: i32,
    ) -> CoreResult<ProjectCollaboratorInfo> {
        self.auth_service
            .check_project_admin_access(self.user.id, project_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_role_permissions() {
        let owner = ProjectRole::Owner;
        let editor = ProjectRole::Editor;
        let viewer = ProjectRole::Viewer;

        // Owner can do everything
        assert!(owner.has_permission(&ProjectRole::Owner));
        assert!(owner.has_permission(&ProjectRole::Editor));
        assert!(owner.has_permission(&ProjectRole::Viewer));
        assert!(owner.can_read());
        assert!(owner.can_write());
        assert!(owner.can_admin());

        // Editor can edit and view
        assert!(!editor.has_permission(&ProjectRole::Owner));
        assert!(editor.has_permission(&ProjectRole::Editor));
        assert!(editor.has_permission(&ProjectRole::Viewer));
        assert!(editor.can_read());
        assert!(editor.can_write());
        assert!(!editor.can_admin());

        // Viewer can only view
        assert!(!viewer.has_permission(&ProjectRole::Owner));
        assert!(!viewer.has_permission(&ProjectRole::Editor));
        assert!(viewer.has_permission(&ProjectRole::Viewer));
        assert!(viewer.can_read());
        assert!(!viewer.can_write());
        assert!(!viewer.can_admin());
    }

    #[test]
    fn test_role_from_string() {
        assert_eq!(
            ProjectRole::from_str("owner").expect("Should parse owner"),
            ProjectRole::Owner
        );
        assert_eq!(
            ProjectRole::from_str("EDITOR").expect("Should parse EDITOR"),
            ProjectRole::Editor
        );
        assert_eq!(
            ProjectRole::from_str("viewer").expect("Should parse viewer"),
            ProjectRole::Viewer
        );
        assert!(ProjectRole::from_str("admin").is_err());
    }

    #[test]
    fn test_role_to_string() {
        assert_eq!(ProjectRole::Owner.as_str(), "owner");
        assert_eq!(ProjectRole::Editor.as_str(), "editor");
        assert_eq!(ProjectRole::Viewer.as_str(), "viewer");
    }
}
