use async_graphql::*;
use serde::{Deserialize, Serialize};

/// DataSet preview with table data
#[derive(Clone, Debug, SimpleObject)]
pub struct DataSetPreview {
    pub node_id: String,
    pub dataset_id: i32,
    pub name: String,
    pub file_path: String,
    pub file_type: String,
    pub total_rows: i32,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub import_date: Option<String>,
    pub execution_state: String,
    pub error_message: Option<String>,
}

/// Column metadata for table preview
#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
pub struct TableColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Row data for table preview
#[derive(Clone, Debug, SimpleObject)]
pub struct TableRow {
    pub row_number: i32,
    pub data: serde_json::Value,
}

/// Graph preview with nodes and edges
#[derive(Clone, Debug, SimpleObject)]
pub struct GraphPreview {
    pub node_id: String,
    pub graph_id: i32,
    pub name: String,
    pub annotations: Option<String>,
    pub nodes: Vec<GraphNodePreview>,
    pub edges: Vec<GraphEdgePreview>,
    pub layers: Vec<crate::graphql::types::layer::Layer>,
    pub node_count: i32,
    pub edge_count: i32,
    pub execution_state: String,
    pub computed_date: Option<String>,
    pub error_message: Option<String>,
}

/// Graph node for preview
#[derive(Clone, Debug, SimpleObject)]
pub struct GraphNodePreview {
    pub id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub is_partition: bool,
    /// Deprecated: use attributes
    pub attrs: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
}

/// Graph edge for preview
#[derive(Clone, Debug, SimpleObject)]
pub struct GraphEdgePreview {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    /// Deprecated: use attributes
    pub attrs: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
}

impl From<crate::database::entities::graph_nodes::Model> for GraphNodePreview {
    fn from(model: crate::database::entities::graph_nodes::Model) -> Self {
        Self {
            id: model.id,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            is_partition: model.is_partition,
            attrs: model.attrs.clone(),
            attributes: model.attrs,
        }
    }
}

impl From<crate::database::entities::graph_data_nodes::Model> for GraphNodePreview {
    fn from(model: crate::database::entities::graph_data_nodes::Model) -> Self {
        Self {
            id: model.external_id,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            is_partition: model.is_partition,
            attrs: model.attributes.clone(),
            attributes: model.attributes,
        }
    }
}

impl From<crate::database::entities::graph_edges::Model> for GraphEdgePreview {
    fn from(model: crate::database::entities::graph_edges::Model) -> Self {
        Self {
            id: model.id,
            source: model.source,
            target: model.target,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            attrs: model.attrs.clone(),
            attributes: model.attrs,
        }
    }
}

impl From<crate::database::entities::graph_data_edges::Model> for GraphEdgePreview {
    fn from(model: crate::database::entities::graph_data_edges::Model) -> Self {
        Self {
            id: model.external_id,
            source: model.source,
            target: model.target,
            label: model.label,
            layer: model.layer,
            weight: model.weight,
            attrs: model.attributes.clone(),
            attributes: model.attributes,
        }
    }
}
