use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_collaborators")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub user_id: i32,
    pub role: String,        // "owner", "editor", "viewer"
    pub permissions: String, // JSON: ["read", "write", "admin", "delete"]
    pub invited_by: Option<i32>,
    pub invitation_status: String, // "pending", "accepted", "declined", "revoked"
    pub invited_at: ChronoDateTimeUtc,
    pub joined_at: Option<ChronoDateTimeUtc>,
    pub last_active_at: Option<ChronoDateTimeUtc>,
    pub is_active: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::InvitedBy",
        to = "super::users::Column::Id"
    )]
    InvitedByUser,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectRole {
    Owner,
    Editor,
    Viewer,
}

impl ToString for ProjectRole {
    fn to_string(&self) -> String {
        match self {
            ProjectRole::Owner => "owner".to_string(),
            ProjectRole::Editor => "editor".to_string(),
            ProjectRole::Viewer => "viewer".to_string(),
        }
    }
}

impl ProjectRole {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "owner" => Ok(ProjectRole::Owner),
            "editor" => Ok(ProjectRole::Editor),
            "viewer" => Ok(ProjectRole::Viewer),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }

    pub fn default_permissions(&self) -> Vec<String> {
        match self {
            ProjectRole::Owner => vec![
                "read".to_string(),
                "write".to_string(),
                "admin".to_string(),
                "delete".to_string(),
                "invite".to_string(),
            ],
            ProjectRole::Editor => vec!["read".to_string(), "write".to_string()],
            ProjectRole::Viewer => vec!["read".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Revoked,
}

impl ToString for InvitationStatus {
    fn to_string(&self) -> String {
        match self {
            InvitationStatus::Pending => "pending".to_string(),
            InvitationStatus::Accepted => "accepted".to_string(),
            InvitationStatus::Declined => "declined".to_string(),
            InvitationStatus::Revoked => "revoked".to_string(),
        }
    }
}

impl InvitationStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "pending" => Ok(InvitationStatus::Pending),
            "accepted" => Ok(InvitationStatus::Accepted),
            "declined" => Ok(InvitationStatus::Declined),
            "revoked" => Ok(InvitationStatus::Revoked),
            _ => Err(format!("Invalid invitation status: {}", s)),
        }
    }
}

impl ActiveModel {
    pub fn new(project_id: i32, user_id: i32, role: ProjectRole, invited_by: Option<i32>) -> Self {
        let now = chrono::Utc::now();
        let permissions_json =
            serde_json::to_string(&role.default_permissions()).unwrap_or_else(|_| "[]".to_string());

        Self {
            id: ActiveValue::NotSet,
            project_id: Set(project_id),
            user_id: Set(user_id),
            role: Set(role.to_string()),
            permissions: Set(permissions_json),
            invited_by: Set(invited_by),
            invitation_status: Set(InvitationStatus::Pending.to_string()),
            invited_at: Set(now),
            joined_at: ActiveValue::NotSet,
            last_active_at: ActiveValue::NotSet,
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }

    pub fn accept_invitation(mut self) -> Self {
        let now = chrono::Utc::now();
        self.invitation_status = Set(InvitationStatus::Accepted.to_string());
        self.joined_at = Set(Some(now));
        self.last_active_at = Set(Some(now));
        self.updated_at = Set(now);
        self
    }

    pub fn decline_invitation(mut self) -> Self {
        self.invitation_status = Set(InvitationStatus::Declined.to_string());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn revoke_invitation(mut self) -> Self {
        self.invitation_status = Set(InvitationStatus::Revoked.to_string());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn update_role(mut self, new_role: ProjectRole) -> Self {
        let permissions_json = serde_json::to_string(&new_role.default_permissions())
            .unwrap_or_else(|_| "[]".to_string());

        self.role = Set(new_role.to_string());
        self.permissions = Set(permissions_json);
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn update_permissions(mut self, permissions: Vec<String>) -> Self {
        let permissions_json =
            serde_json::to_string(&permissions).unwrap_or_else(|_| "[]".to_string());

        self.permissions = Set(permissions_json);
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_last_active(mut self) -> Self {
        self.last_active_at = Set(Some(chrono::Utc::now()));
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn deactivate(mut self) -> Self {
        self.is_active = Set(false);
        self.updated_at = Set(chrono::Utc::now());
        self
    }
}

impl Model {
    pub fn get_role(&self) -> Result<ProjectRole, String> {
        ProjectRole::from_str(&self.role)
    }

    pub fn get_invitation_status(&self) -> Result<InvitationStatus, String> {
        InvitationStatus::from_str(&self.invitation_status)
    }

    pub fn get_permissions(&self) -> Vec<String> {
        serde_json::from_str::<Vec<String>>(&self.permissions).unwrap_or_default()
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.get_permissions().contains(&permission.to_string())
    }

    pub fn can_read(&self) -> bool {
        self.has_permission("read") && self.is_active
    }

    pub fn can_write(&self) -> bool {
        self.has_permission("write") && self.is_active
    }

    pub fn can_admin(&self) -> bool {
        self.has_permission("admin") && self.is_active
    }

    pub fn can_delete(&self) -> bool {
        self.has_permission("delete") && self.is_active
    }

    pub fn can_invite(&self) -> bool {
        self.has_permission("invite") && self.is_active
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.get_role(), Ok(ProjectRole::Owner))
    }

    pub fn is_accepted(&self) -> bool {
        matches!(self.get_invitation_status(), Ok(InvitationStatus::Accepted))
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.get_invitation_status(), Ok(InvitationStatus::Pending))
    }

    pub fn days_since_invitation(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.invited_at).num_days()
    }

    pub fn days_since_last_active(&self) -> Option<i64> {
        self.last_active_at.map(|last_active| {
            let now = chrono::Utc::now();
            (now - last_active).num_days()
        })
    }
}
