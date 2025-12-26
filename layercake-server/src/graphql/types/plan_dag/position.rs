use async_graphql::*;
use serde::{Deserialize, Serialize};

// Position for ReactFlow nodes
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl From<layercake_core::plan_dag::Position> for Position {
    fn from(position: layercake_core::plan_dag::Position) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }
}

impl From<Position> for layercake_core::plan_dag::Position {
    fn from(position: Position) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }
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
