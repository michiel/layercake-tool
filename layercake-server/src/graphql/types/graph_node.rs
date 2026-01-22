use async_graphql::*;
use layercake_core::database::entities::graph_data_nodes;
use serde::{Deserialize, Serialize};

// Input type for bulk node updates
#[derive(InputObject, Clone, Debug)]
pub struct GraphNodeUpdateInput {
    #[graphql(name = "nodeId")]
    pub node_id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub attrs: Option<serde_json::Value>,
    /// Preferred alias for node attributes (string/int values only)
    pub attributes: Option<serde_json::Value>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    #[graphql(name = "isPartition")]
    pub is_partition: bool,
    #[graphql(name = "belongsTo")]
    pub belongs_to: Option<String>,
    pub comment: Option<String>,
    /// Deprecated: use attributes
    pub attrs: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
    #[graphql(name = "datasetId")]
    pub dataset_id: Option<i32>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<layercake_core::database::entities::graph_nodes::Model> for GraphNode {
    fn from(model: layercake_core::database::entities::graph_nodes::Model) -> Self {
        Self {
            id: model.id,
            graph_id: model.graph_id,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            is_partition: model.is_partition,
            belongs_to: model.belongs_to,
            comment: model.comment,
            attrs: model.attrs.clone(),
            attributes: model.attrs,
            dataset_id: model.dataset_id,
            created_at: model.created_at,
        }
    }
}

impl From<graph_data_nodes::Model> for GraphNode {
    fn from(model: graph_data_nodes::Model) -> Self {
        Self {
            id: model.external_id,
            graph_id: model.graph_data_id,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            is_partition: model.is_partition,
            belongs_to: model.belongs_to,
            comment: model.comment,
            attrs: model.attributes.clone(),
            attributes: model.attributes,
            dataset_id: model.source_dataset_id,
            created_at: model.created_at,
        }
    }
}
