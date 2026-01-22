use async_graphql::*;
use layercake_core::database::entities::graph_data_edges;
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
    pub comment: Option<String>,
    /// Deprecated: use attributes
    pub attrs: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
    #[graphql(name = "datasetId")]
    pub dataset_id: Option<i32>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<layercake_core::database::entities::graph_edges::Model> for GraphEdge {
    fn from(model: layercake_core::database::entities::graph_edges::Model) -> Self {
        Self {
            id: model.id,
            graph_id: model.graph_id,
            source: model.source,
            target: model.target,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            comment: model.comment,
            attrs: model.attrs.clone(),
            attributes: model.attrs,
            dataset_id: model.dataset_id,
            created_at: model.created_at,
        }
    }
}

impl From<graph_data_edges::Model> for GraphEdge {
    fn from(model: graph_data_edges::Model) -> Self {
        Self {
            id: model.external_id,
            graph_id: model.graph_data_id,
            source: model.source,
            target: model.target,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            comment: model.comment,
            attrs: model.attributes.clone(),
            attributes: model.attributes,
            dataset_id: model.source_dataset_id,
            created_at: model.created_at,
        }
    }
}
