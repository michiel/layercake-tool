use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Edge entity for unified graph_data
///
/// Represents individual edges within a GraphData instance. Edges connect
/// nodes via their external_id references. The `source` and `target` fields
/// reference node external_ids within the same graph_data instance.
///
/// Related entities:
/// - `graph_data`: Parent graph this edge belongs to
/// - `graph_data_nodes`: Nodes this edge connects (via composite FK)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_data_edges")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub graph_data_id: i32,
    pub external_id: String, // User-provided edge ID
    pub source: String,      // References node external_id
    pub target: String,      // References node external_id
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
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
