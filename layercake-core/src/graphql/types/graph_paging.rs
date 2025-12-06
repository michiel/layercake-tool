use async_graphql::SimpleObject;

use crate::graph::{Edge, Node};
use crate::services::data_set_service::{GraphPageData, GraphSummaryData};

#[derive(SimpleObject, Clone)]
pub struct GraphSummary {
    pub node_count: i32,
    pub edge_count: i32,
    pub layer_count: i32,
    pub layers: Vec<String>,
}

impl From<GraphSummaryData> for GraphSummary {
    fn from(data: GraphSummaryData) -> Self {
        Self {
            node_count: data.node_count as i32,
            edge_count: data.edge_count as i32,
            layer_count: data.layer_count as i32,
            layers: data.layers,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct GraphPage {
    pub nodes: Vec<GraphNodeSlice>,
    pub edges: Vec<GraphEdgeSlice>,
    pub layers: Vec<LayerSlice>,
    pub has_more: bool,
}

impl From<GraphPageData> for GraphPage {
    fn from(data: GraphPageData) -> Self {
        Self {
            nodes: data.nodes.into_iter().map(GraphNodeSlice::from).collect(),
            edges: data.edges.into_iter().map(GraphEdgeSlice::from).collect(),
            layers: data.layers.into_iter().map(LayerSlice::from).collect(),
            has_more: data.has_more,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct GraphNodeSlice {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub belongs_to: Option<String>,
    pub weight: i32,
    pub is_partition: bool,
    pub comment: Option<String>,
    pub dataset: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

impl From<Node> for GraphNodeSlice {
    fn from(n: Node) -> Self {
        GraphNodeSlice {
            id: n.id,
            label: n.label,
            layer: n.layer,
            belongs_to: n.belongs_to,
            weight: n.weight,
            is_partition: n.is_partition,
            comment: n.comment,
            dataset: n.dataset.map(|d| d.to_string()),
            attributes: n.attributes,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct GraphEdgeSlice {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub weight: i32,
    pub comment: Option<String>,
    pub dataset: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

impl From<Edge> for GraphEdgeSlice {
    fn from(e: Edge) -> Self {
        GraphEdgeSlice {
            id: e.id,
            source: e.source,
            target: e.target,
            label: e.label,
            layer: e.layer,
            weight: e.weight,
            comment: e.comment,
            dataset: e.dataset.map(|d| d.to_string()),
            attributes: e.attributes,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct LayerSlice {
    pub id: String,
    pub label: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
}

impl From<crate::graph::Layer> for LayerSlice {
    fn from(l: crate::graph::Layer) -> Self {
        LayerSlice {
            id: l.id,
            label: l.label,
            background_color: Some(l.background_color),
            text_color: Some(l.text_color),
            border_color: Some(l.border_color),
        }
    }
}
