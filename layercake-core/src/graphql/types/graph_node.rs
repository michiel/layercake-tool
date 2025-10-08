use async_graphql::*;
use serde::{Deserialize, Serialize};

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
            attrs: model.attrs,
            created_at: model.created_at,
        }
    }
}
