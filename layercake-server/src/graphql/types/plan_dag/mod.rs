#![allow(dead_code)]

use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

use layercake_core::app_context::PlanDagSnapshot;

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
    #[graphql(name = "GraphArtefactNode")]
    #[serde(rename = "GraphArtefactNode", alias = "OutputNode", alias = "Output")]
    GraphArtefact,
    #[graphql(name = "TreeArtefactNode")]
    #[serde(rename = "TreeArtefactNode")]
    TreeArtefact,
    #[graphql(name = "ProjectionNode")]
    #[serde(rename = "ProjectionNode")]
    Projection,
    #[graphql(name = "StoryNode")]
    #[serde(rename = "StoryNode")]
    Story,
    #[graphql(name = "SequenceArtefactNode")]
    #[serde(rename = "SequenceArtefactNode")]
    SequenceArtefact,
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

#[derive(SimpleObject, Clone, Debug)]
pub struct PlanDagMigrationDetail {
    #[graphql(name = "nodeId")]
    pub node_id: String,
    #[graphql(name = "fromType")]
    pub from_type: String,
    #[graphql(name = "toType")]
    pub to_type: String,
    pub note: Option<String>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct PlanDagMigrationResult {
    #[graphql(name = "checkedNodes")]
    pub checked_nodes: i32,
    #[graphql(name = "updatedNodes")]
    pub updated_nodes: Vec<PlanDagMigrationDetail>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl From<layercake_core::plan_dag::PlanDagNodeType> for PlanDagNodeType {
    fn from(node_type: layercake_core::plan_dag::PlanDagNodeType) -> Self {
        match node_type {
            layercake_core::plan_dag::PlanDagNodeType::DataSet => PlanDagNodeType::DataSet,
            layercake_core::plan_dag::PlanDagNodeType::Graph => PlanDagNodeType::Graph,
            layercake_core::plan_dag::PlanDagNodeType::Transform => PlanDagNodeType::Transform,
            layercake_core::plan_dag::PlanDagNodeType::Filter => PlanDagNodeType::Filter,
            layercake_core::plan_dag::PlanDagNodeType::Merge => PlanDagNodeType::Merge,
            layercake_core::plan_dag::PlanDagNodeType::GraphArtefact => {
                PlanDagNodeType::GraphArtefact
            }
            layercake_core::plan_dag::PlanDagNodeType::TreeArtefact => {
                PlanDagNodeType::TreeArtefact
            }
            layercake_core::plan_dag::PlanDagNodeType::Projection => PlanDagNodeType::Projection,
            layercake_core::plan_dag::PlanDagNodeType::Story => PlanDagNodeType::Story,
            layercake_core::plan_dag::PlanDagNodeType::SequenceArtefact => {
                PlanDagNodeType::SequenceArtefact
            }
        }
    }
}

impl From<PlanDagNodeType> for layercake_core::plan_dag::PlanDagNodeType {
    fn from(node_type: PlanDagNodeType) -> Self {
        match node_type {
            PlanDagNodeType::DataSet => layercake_core::plan_dag::PlanDagNodeType::DataSet,
            PlanDagNodeType::Graph => layercake_core::plan_dag::PlanDagNodeType::Graph,
            PlanDagNodeType::Transform => layercake_core::plan_dag::PlanDagNodeType::Transform,
            PlanDagNodeType::Filter => layercake_core::plan_dag::PlanDagNodeType::Filter,
            PlanDagNodeType::Merge => layercake_core::plan_dag::PlanDagNodeType::Merge,
            PlanDagNodeType::GraphArtefact => {
                layercake_core::plan_dag::PlanDagNodeType::GraphArtefact
            }
            PlanDagNodeType::TreeArtefact => {
                layercake_core::plan_dag::PlanDagNodeType::TreeArtefact
            }
            PlanDagNodeType::Projection => layercake_core::plan_dag::PlanDagNodeType::Projection,
            PlanDagNodeType::Story => layercake_core::plan_dag::PlanDagNodeType::Story,
            PlanDagNodeType::SequenceArtefact => {
                layercake_core::plan_dag::PlanDagNodeType::SequenceArtefact
            }
        }
    }
}

impl From<PlanDagSnapshot> for PlanDag {
    fn from(snapshot: PlanDagSnapshot) -> Self {
        Self {
            version: snapshot.version,
            nodes: snapshot.nodes.into_iter().map(PlanDagNode::from).collect(),
            edges: snapshot.edges.into_iter().map(PlanDagEdge::from).collect(),
            metadata: snapshot.metadata.into(),
        }
    }
}
