#![allow(dead_code)]

use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

use crate::app_context::PlanDagSnapshot;

// Module declarations
pub mod config;
pub mod edge;
pub mod filter;
pub mod input_types;
pub mod metadata;
pub mod node;
pub mod position;
pub mod transforms;

// Re-export commonly used types
pub use config::*;
pub use edge::*;
pub use filter::*;
pub use input_types::{
    PlanDagEdgeInput, PlanDagEdgeUpdateInput, PlanDagNodeInput, PlanDagNodeUpdateInput,
    ValidationError, ValidationErrorType, ValidationResult, ValidationWarning,
    ValidationWarningType,
};
pub use metadata::*;
pub use node::*;
pub use position::*;
pub use transforms::*;

// Plan DAG Node Types (matching frontend enum)
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum PlanDagNodeType {
    #[graphql(name = "DataSetNode")]
    DataSet,
    #[graphql(name = "GraphNode")]
    Graph,
    #[graphql(name = "TransformNode")]
    Transform,
    #[graphql(name = "FilterNode")]
    Filter,
    #[graphql(name = "MergeNode")]
    Merge,
    #[graphql(name = "OutputNode")]
    Output,
}

// Complete Plan DAG Structure
#[derive(SimpleObject, Clone, Debug)]
pub struct PlanDag {
    pub version: String,
    pub nodes: Vec<PlanDagNode>,
    pub edges: Vec<PlanDagEdge>,
    pub metadata: PlanDagMetadata,
}

// Input types for mutations
#[derive(InputObject, Clone, Debug)]
pub struct PlanDagInput {
    pub version: String,
    pub nodes: Vec<PlanDagNodeInput>,
    pub edges: Vec<PlanDagEdgeInput>,
    pub metadata: PlanDagMetadata,
}

impl From<PlanDagSnapshot> for PlanDag {
    fn from(snapshot: PlanDagSnapshot) -> Self {
        Self {
            version: snapshot.version,
            nodes: snapshot.nodes,
            edges: snapshot.edges,
            metadata: snapshot.metadata,
        }
    }
}
