use async_graphql::*;
use serde::{Deserialize, Serialize};

// Input type for bulk node updates
#[derive(InputObject, Clone, Debug)]
pub struct GraphNodeUpdateInput {
    #[graphql(name = "nodeId")]
    pub node_id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub attrs: Option<serde_json::Value>,
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
    pub attrs: Option<serde_json::Value>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::entities::graph_nodes::Model> for GraphNode {
    fn from(model: crate::database::entities::graph_nodes::Model) -> Self {
        Self {
            id: model.id,
            graph_id: model.graph_id,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            is_partition: model.is_partition,
            belongs_to: model.belongs_to,
            attrs: model.attrs,
            created_at: model.created_at,
        }
    }
}
