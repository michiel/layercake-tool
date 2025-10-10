use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Graph edit tracking for change replay
///
/// Records discrete edit operations made to graph nodes, edges, and layers.
/// Used to preserve user edits when upstream data is refreshed.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_edits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub graph_id: i32,
    pub target_type: String, // 'node', 'edge', 'layer'
    pub target_id: String,
    pub operation: String, // 'create', 'update', 'delete'
    pub field_name: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub old_value: Option<serde_json::Value>,
    #[sea_orm(column_type = "JsonBinary")]
    pub new_value: Option<serde_json::Value>,
    pub sequence_number: i32,
    pub applied: bool,
    pub created_at: ChronoDateTimeUtc,
    pub created_by: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::graphs::Entity",
        from = "Column::GraphId",
        to = "super::graphs::Column::Id",
        on_delete = "Cascade"
    )]
    Graphs,
}

impl Related<super::graphs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Graphs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
