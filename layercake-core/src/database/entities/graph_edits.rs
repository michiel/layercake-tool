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

// `graph_id` refers to a `graph_data` row in the current single-schema model.
// The former foreign key to the dropped `graphs` table was removed in migration
// m20260709_000001_rebuild_graph_edits_drop_graphs_fk; no ORM relation is defined.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
