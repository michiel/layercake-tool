use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub session_id: i32,
    #[sea_orm(unique)]
    pub message_id: String,
    pub role: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::chat_sessions::Entity",
        from = "Column::SessionId",
        to = "super::chat_sessions::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ChatSession,
}

impl Related<super::chat_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
