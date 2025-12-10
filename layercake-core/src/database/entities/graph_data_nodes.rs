use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Node entity for unified graph_data
///
/// Represents individual nodes within a GraphData instance. Nodes can belong
/// to datasets, computed graphs, or manual graphs. The `external_id` is the
/// user-facing node identifier, while `id` is a surrogate key for database
/// efficiency.
///
/// Related entities:
/// - `graph_data`: Parent graph this node belongs to
/// - `graph_data_edges`: Edges that reference this node as source/target
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_data_nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub graph_data_id: i32,
    pub external_id: String, // User-provided node ID
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub is_partition: bool,
    pub belongs_to: Option<String>, // References another node's external_id
    #[sea_orm(column_type = "Text")]
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>, // Traceability to original dataset
    #[sea_orm(column_type = "JsonBinary")]
    pub attributes: Option<serde_json::Value>,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::graph_data::Entity",
        from = "Column::GraphDataId",
        to = "super::graph_data::Column::Id"
    )]
    GraphData,
}

impl Related<super::graph_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
