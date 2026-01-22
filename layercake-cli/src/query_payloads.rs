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
