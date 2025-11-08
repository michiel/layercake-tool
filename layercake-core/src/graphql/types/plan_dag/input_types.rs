use async_graphql::*;

use super::metadata::{EdgeMetadata, NodeMetadata};
use super::position::Position;
use super::PlanDagNodeType;

// Input types for mutations
#[derive(InputObject, Clone, Debug)]
pub struct PlanDagNodeInput {
    /// Optional ID - if not provided, backend will generate one
    pub id: Option<String>,
    #[graphql(name = "nodeType")]
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub config: String, // JSON string
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagEdgeInput {
    /// Optional ID - if not provided, backend will generate one
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    // Removed source_handle and target_handle for floating edges
    pub metadata: EdgeMetadata,
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagNodeUpdateInput {
    pub position: Option<Position>,
    pub metadata: Option<NodeMetadata>,
    pub config: Option<String>,
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagEdgeUpdateInput {
    // Removed source_handle and target_handle for floating edges
    pub metadata: Option<EdgeMetadata>,
}

// Validation types
#[derive(SimpleObject, Clone, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ValidationError {
    #[graphql(name = "nodeId")]
    pub node_id: Option<String>,
    #[graphql(name = "edgeId")]
    pub edge_id: Option<String>,
    pub error_type: ValidationErrorType,
    pub message: String,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ValidationWarning {
    #[graphql(name = "nodeId")]
    pub node_id: Option<String>,
    #[graphql(name = "edgeId")]
    pub edge_id: Option<String>,
    pub warning_type: ValidationWarningType,
    pub message: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ValidationErrorType {
    MissingInput,
    InvalidConnection,
    CyclicDependency,
    InvalidConfig,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ValidationWarningType {
    UnusedOutput,
    PerformanceImpact,
    ConfigurationSuggestion,
}
