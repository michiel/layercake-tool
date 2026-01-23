use serde::Deserialize;

use layercake_core::{
    plan::{ExportFileType, RenderConfig},
    plan_dag::Position,
    services::cli_graphql_helpers::{CliPlanEdgeUpdateInput, CliPlanNodeUpdateInput},
};

#[derive(Deserialize)]
pub struct NodeUpdatePayload {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(flatten)]
    pub update: CliPlanNodeUpdateInput,
}

#[derive(Deserialize)]
pub struct NodeDeletePayload {
    #[serde(rename = "nodeId")]
    pub node_id: String,
}

#[derive(Deserialize)]
pub struct NodeMovePayload {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub position: Position,
}

#[derive(Deserialize)]
pub struct EdgeUpdatePayload {
    #[serde(rename = "edgeId")]
    pub edge_id: String,
    #[serde(flatten)]
    pub update: CliPlanEdgeUpdateInput,
}

#[derive(Deserialize)]
pub struct EdgeDeletePayload {
    #[serde(rename = "edgeId")]
    pub edge_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub graph_id: i32,
    pub format: ExportFileType,
    pub render_config: Option<RenderConfig>,
    pub max_rows: Option<usize>,
}

// Phase 1.1: Node Query Filters
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeFilterPayload {
    pub node_type: Option<String>,
    pub label_pattern: Option<String>,
    pub execution_state: Option<String>,
    pub bounds: Option<BoundsFilter>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundsFilter {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

// Phase 1.2: Single Node GET
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeGetPayload {
    pub node_id: String,
}

// Phase 1.3: Graph Traversal
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraversePayload {
    pub start_node: String,
    pub direction: Option<String>, // "upstream", "downstream", "both"
    pub max_depth: Option<usize>,
    pub end_node: Option<String>, // For path finding
    pub find_path: Option<bool>,
    pub include_connections: Option<bool>,
    pub radius: Option<usize>,
}

// Phase 2.1: Batch Operations
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchPayload {
    pub operations: Vec<BatchOperation>,
    pub atomic: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOperation {
    pub op: String, // "createNode", "createEdge", "updateNode", etc.
    pub id: Option<String>, // Temporary ID for references
    pub data: serde_json::Value,
}

// Phase 2.2: Search and Discovery
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPayload {
    pub query: String,
    pub fields: Option<Vec<String>>, // ["label", "description"]
    pub edge_filter: Option<String>, // "noOutgoing", "noIncoming", "isolated"
}

// Phase 2.4: Annotations
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationCreatePayload {
    pub target_id: String,
    pub target_type: String, // "node" or "edge"
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationListPayload {
    pub target_id: Option<String>, // If provided, filter by target
    pub key: Option<String>,        // If provided, filter by key
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationGetPayload {
    pub id: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationUpdatePayload {
    pub id: i32,
    pub value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationDeletePayload {
    pub id: i32,
}

// Phase 2.5: Clone Operations
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClonePayload {
    pub node_id: String,
    pub position: Option<Position>,
    pub update_label: Option<String>,
}
