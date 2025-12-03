use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    /// Deprecated: use attributes
    pub attrs: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
    #[graphql(name = "datasetId")]
    pub dataset_id: Option<i32>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::entities::graph_edges::Model> for GraphEdge {
    fn from(model: crate::database::entities::graph_edges::Model) -> Self {
        Self {
            id: model.id,
            graph_id: model.graph_id,
            source: model.source,
            target: model.target,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            attrs: model.attrs.clone(),
            attributes: model.attrs,
            dataset_id: model.dataset_id,
            created_at: model.created_at,
        }
    }
}
