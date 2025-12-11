use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Unified GraphData entity that represents both datasets and computed graphs
///
/// This entity replaces the previous separate `data_sets` and `graphs` entities.
/// It stores graph data from any source: file uploads (datasets), DAG execution
/// (computed graphs), or direct user creation (manual graphs).
///
/// The `source_type` discriminator determines which optional fields are populated:
/// - 'dataset': file_format, origin, filename, blob, file_size, processed_at
/// - 'computed': source_hash, computed_date, edit tracking fields
/// - 'manual': neither set of source-specific fields
///
/// Related entities:
/// - `graph_data_nodes`: Individual nodes in this graph
/// - `graph_data_edges`: Individual edges in this graph
/// - `graph_edits`: Edit history for replay (computed graphs only)
/// - `projects`: Parent project
/// - `plan_dag_nodes`: DAG node reference (computed graphs only)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,

    // Source and lifecycle
    pub source_type: String, // 'dataset', 'computed', 'manual'
    pub dag_node_id: Option<String>,

    // Dataset-specific fields
    pub file_format: Option<String>, // 'csv', 'tsv', 'json', etc.
    pub origin: Option<String>,      // 'file_upload', 'rag_agent', 'manual_edit'
    pub filename: Option<String>,
    #[sea_orm(column_type = "Binary(BlobSize::Long)")]
    pub blob: Option<Vec<u8>>,
    pub file_size: Option<i64>,
    pub processed_at: Option<ChronoDateTimeUtc>,

    // Computed graph-specific fields
    pub source_hash: Option<String>,
    pub computed_date: Option<ChronoDateTimeUtc>,

    // Edit tracking (for computed graphs)
    pub last_edit_sequence: i32,
    pub has_pending_edits: bool,
    pub last_replay_at: Option<ChronoDateTimeUtc>,

    // Common metadata
    pub node_count: i32,
    pub edge_count: i32,
    #[sea_orm(column_type = "Text")]
    pub error_message: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Option<serde_json::Value>,
    #[sea_orm(column_type = "JsonBinary")]
    pub annotations: Option<serde_json::Value>, // Array of markdown strings
    pub status: String, // 'active', 'processing', or 'error'

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
    #[sea_orm(has_many = "super::graph_data_nodes::Entity")]
    GraphDataNodes,
    #[sea_orm(has_many = "super::graph_data_edges::Entity")]
    GraphDataEdges,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
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

/// Source type discriminator for GraphData
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphDataSourceType {
    #[serde(rename = "dataset")]
    Dataset,
    #[serde(rename = "computed")]
    Computed,
    #[serde(rename = "manual")]
    Manual,
}

impl GraphDataSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dataset => "dataset",
            Self::Computed => "computed",
            Self::Manual => "manual",
        }
    }
}

impl From<GraphDataSourceType> for String {
    fn from(source_type: GraphDataSourceType) -> Self {
        source_type.as_str().to_string()
    }
}

/// Status lifecycle values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphDataStatus {
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "error")]
    Error,
}

impl GraphDataStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Processing => "processing",
            Self::Active => "active",
            Self::Error => "error",
        }
    }
}

impl From<GraphDataStatus> for String {
    fn from(status: GraphDataStatus) -> Self {
        status.as_str().to_string()
    }
}
