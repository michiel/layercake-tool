use async_graphql::*;
use serde::{Deserialize, Serialize};

// Position for ReactFlow nodes
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

// Batch node move input
#[derive(InputObject, Clone, Debug)]
pub struct NodePositionInput {
    pub node_id: String,
    pub position: Position,
    #[graphql(name = "sourcePosition")]
    pub source_position: Option<String>,
    #[graphql(name = "targetPosition")]
    pub target_position: Option<String>,
}
