use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub avatar_color: String,
    pub is_active: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
    pub last_login_at: Option<ChronoDateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_sessions::Entity")]
    UserSessions,
    #[sea_orm(has_many = "super::project_collaborators::Entity")]
    ProjectCollaborators,
    // REMOVED: UserPresence relation - user presence now handled via WebSocket only
}

impl Related<super::user_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSessions.def()
    }
}

impl Related<super::project_collaborators::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectCollaborators.def()
    }
}

// REMOVED: Related implementation for user_presence - user presence now handled via WebSocket only

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            email: ActiveValue::NotSet,
            username: ActiveValue::NotSet,
            display_name: ActiveValue::NotSet,
            password_hash: ActiveValue::NotSet,
            avatar_color: Self::generate_random_color(),
            is_active: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            last_login_at: ActiveValue::NotSet,
        }
    }

    pub fn set_updated_at(mut self) -> Self {
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_last_login(mut self) -> Self {
        self.last_login_at = Set(Some(chrono::Utc::now()));
        self
    }

    fn generate_random_color() -> ActiveValue<String> {
        // Generate a random color for user avatars
        let colors = [
            "#228be6", "#51cf66", "#ff8cc8", "#ffd43b",
            "#74c0fc", "#ff6b6b", "#845ef7", "#20c997",
            "#fd7e14", "#495057"
        ];
        let index = (chrono::Utc::now().timestamp() % colors.len() as i64) as usize;
        Set(colors[index].to_string())
    }
}