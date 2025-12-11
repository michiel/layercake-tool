use sea_orm::entity::prelude::*;

/// Legacy graphs entity for backward compatibility during migration
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "graphs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub node_id: String,
    pub name: String,
    pub execution_state: String,
    pub computed_date: Option<ChronoDateTimeUtc>,
    pub source_hash: Option<String>,
    pub node_count: i32,
    pub edge_count: i32,
    #[sea_orm(column_type = "Text")]
    pub error_message: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Option<serde_json::Value>,
    #[sea_orm(column_type = "Text")]
    pub annotations: Option<String>,
    pub last_edit_sequence: i32,
    pub has_pending_edits: bool,
    pub last_replay_at: Option<ChronoDateTimeUtc>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
