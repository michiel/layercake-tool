use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,

    pub source_type: String,
    pub dag_node_id: Option<String>,

    pub file_format: Option<String>,
    pub origin: Option<String>,
    pub filename: Option<String>,
    #[sea_orm(column_type = "Binary(BlobSize::Long)")]
    pub blob: Option<Vec<u8>>,
    pub file_size: Option<i64>,
    pub processed_at: Option<ChronoDateTimeUtc>,

    pub source_hash: Option<String>,
    pub computed_date: Option<ChronoDateTimeUtc>,

    pub last_edit_sequence: i32,
    pub has_pending_edits: bool,
    pub last_replay_at: Option<ChronoDateTimeUtc>,

    pub node_count: i32,
    pub edge_count: i32,
    #[sea_orm(column_type = "Text")]
    pub error_message: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Option<serde_json::Value>,
    #[sea_orm(column_type = "JsonBinary")]
    pub annotations: Option<serde_json::Value>,
    pub status: String,

    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::graph_data_nodes::Entity")]
    GraphDataNodes,
    #[sea_orm(has_many = "super::graph_data_edges::Entity")]
    GraphDataEdges,
}

impl Related<super::graph_data_nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphDataNodes.def()
    }
}

impl Related<super::graph_data_edges::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphDataEdges.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
