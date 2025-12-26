use serde::{Deserialize, Serialize};

// Position for ReactFlow nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
