use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::database::entities::{users, user_sessions, project_collaborators, user_presence};
use crate::graphql::context::GraphQLContext;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_color: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<users::Model> for User {
    fn from(model: users::Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            username: model.username,
            display_name: model.display_name,
            avatar_color: model.avatar_color,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            last_login_at: model.last_login_at,
        }
    }
}

#[ComplexObject]
impl User {
    async fn sessions(&self, ctx: &Context<'_>) -> Result<Vec<UserSession>> {
        let context = ctx.data::<GraphQLContext>()?;
        let sessions = user_sessions::Entity::find()
            .filter(user_sessions::Column::UserId.eq(self.id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(sessions.into_iter().map(UserSession::from).collect())
    }

    async fn collaborations(&self, ctx: &Context<'_>) -> Result<Vec<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(self.id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(collaborations.into_iter().map(ProjectCollaborator::from).collect())
    }

    async fn presence(&self, ctx: &Context<'_>, project_id: i32) -> Result<Option<UserPresence>> {
        let context = ctx.data::<GraphQLContext>()?;
        let presence = user_presence::Entity::find()
            .filter(user_presence::Column::UserId.eq(self.id))
            .filter(user_presence::Column::ProjectId.eq(project_id))
            .filter(user_presence::Column::IsOnline.eq(true))
            .one(&context.db)
            .await?;

        Ok(presence.map(UserPresence::from))
    }
}

#[derive(SimpleObject)]
pub struct UserSession {
    pub id: i32,
    pub session_id: String,
    pub user_id: i32,
    pub user_name: String,
    pub project_id: i32,
    pub layercake_graph_id: Option<i32>,
    pub cursor_position: Option<String>,
    pub selected_node_id: Option<String>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<user_sessions::Model> for UserSession {
    fn from(model: user_sessions::Model) -> Self {
        Self {
            id: model.id,
            session_id: model.session_id,
            user_id: model.user_id,
            user_name: model.user_name,
            project_id: model.project_id,
            layercake_graph_id: model.layercake_graph_id,
            cursor_position: model.cursor_position,
            selected_node_id: model.selected_node_id,
            last_activity: model.last_activity,
            is_active: model.is_active,
            created_at: model.created_at,
            expires_at: model.expires_at,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ProjectCollaborator {
    pub id: i32,
    pub project_id: i32,
    pub user_id: i32,
    pub role: String,
    pub permissions: String,
    pub invited_by: Option<i32>,
    pub invitation_status: String,
    pub invited_at: DateTime<Utc>,
    pub joined_at: Option<DateTime<Utc>>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<project_collaborators::Model> for ProjectCollaborator {
    fn from(model: project_collaborators::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            user_id: model.user_id,
            role: model.role,
            permissions: model.permissions,
            invited_by: model.invited_by,
            invitation_status: model.invitation_status,
            invited_at: model.invited_at,
            joined_at: model.joined_at,
            last_active_at: model.last_active_at,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[ComplexObject]
impl ProjectCollaborator {
    async fn user(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;
        let user = users::Entity::find_by_id(self.user_id)
            .one(&context.db)
            .await?;

        Ok(user.map(User::from))
    }

    async fn invited_by_user(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        if let Some(invited_by_id) = self.invited_by {
            let context = ctx.data::<GraphQLContext>()?;
            let user = users::Entity::find_by_id(invited_by_id)
                .one(&context.db)
                .await?;

            Ok(user.map(User::from))
        } else {
            Ok(None)
        }
    }
}

#[derive(SimpleObject)]
pub struct UserPresence {
    pub id: i32,
    pub user_id: i32,
    pub project_id: i32,
    pub session_id: String,
    pub layercake_graph_id: Option<i32>,
    pub cursor_position: Option<String>,
    pub selected_node_id: Option<String>,
    pub viewport_position: Option<String>,
    pub current_tool: Option<String>,
    pub is_online: bool,
    pub last_seen: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<user_presence::Model> for UserPresence {
    fn from(model: user_presence::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            project_id: model.project_id,
            session_id: model.session_id,
            layercake_graph_id: model.layercake_graph_id,
            cursor_position: model.cursor_position,
            selected_node_id: model.selected_node_id,
            viewport_position: model.viewport_position,
            current_tool: model.current_tool,
            is_online: model.is_online,
            last_seen: model.last_seen,
            last_heartbeat: model.last_heartbeat,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

// Input types for mutations
#[derive(InputObject)]
pub struct RegisterUserInput {
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(InputObject)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(InputObject)]
pub struct UpdateUserInput {
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(InputObject)]
pub struct InviteCollaboratorInput {
    pub project_id: i32,
    pub email: String,
    pub role: String,
}

#[derive(InputObject)]
pub struct UpdateCollaboratorRoleInput {
    pub collaborator_id: i32,
    pub role: String,
}

#[derive(InputObject)]
pub struct UpdateUserPresenceInput {
    pub project_id: i32,
    pub session_id: String,
    pub cursor_position: Option<CursorPositionInput>,
    pub selected_node_id: Option<String>,
    pub viewport_position: Option<ViewportPositionInput>,
    pub current_tool: Option<String>,
}

#[derive(InputObject)]
pub struct CursorPositionInput {
    pub x: f64,
    pub y: f64,
}

#[derive(InputObject)]
pub struct ViewportPositionInput {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

// Response types
#[derive(SimpleObject)]
pub struct LoginResponse {
    pub user: User,
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(SimpleObject)]
pub struct RegisterResponse {
    pub user: User,
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ProjectRole {
    Owner,
    Editor,
    Viewer,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Revoked,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum UserStatus {
    Active,
    Idle,
    Away,
    Offline,
}

// Collaboration-specific types that match the frontend expectations
#[derive(SimpleObject)]
pub struct UserPresenceInfo {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
    pub cursor_position: Option<CursorPosition>,
    pub selected_node_id: Option<String>,
    pub is_active: bool,
    pub last_seen: String, // ISO 8601 timestamp
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

impl From<user_presence::Model> for UserPresenceInfo {
    fn from(presence: user_presence::Model) -> Self {
        // Parse cursor position from JSON string
        let cursor_position = presence.cursor_position
            .as_ref()
            .and_then(|pos| serde_json::from_str::<crate::database::entities::user_presence::CursorPosition>(pos).ok())
            .map(|pos| CursorPosition { x: pos.x, y: pos.y });

        Self {
            user_id: presence.user_id.to_string(),
            user_name: format!("User {}", presence.user_id), // Default name - should be replaced with actual user lookup
            avatar_color: "#3b82f6".to_string(), // Default blue - should be replaced with actual user color
            cursor_position,
            selected_node_id: presence.selected_node_id,
            is_active: presence.is_online && presence.status == "active",
            last_seen: presence.last_seen.to_rfc3339(),
        }
    }
}