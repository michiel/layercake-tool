use async_graphql::*;
use serde::{Deserialize, Serialize};

use super::PlanDagNodeType;

// Node metadata
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "NodeMetadataInput")]
pub struct NodeMetadata {
    pub label: String,
    pub description: Option<String>,
}

// Edge metadata
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "EdgeMetadataInput")]
pub struct EdgeMetadata {
    pub label: Option<String>,
    #[graphql(name = "dataType")]
    pub data_type: DataType,
}

// Data type for edges
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DataType {
    GraphData,
    GraphReference,
}

// Execution metadata for DataSet nodes
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DataSetExecutionMetadata {
    #[graphql(name = "dataSetId")]
    pub data_set_id: i32,
    pub filename: String,
    pub status: String,
    #[graphql(name = "processedAt")]
    pub processed_at: Option<String>,
    #[graphql(name = "executionState")]
    pub execution_state: String,
    #[graphql(name = "errorMessage")]
    pub error_message: Option<String>,
}

// Execution metadata for Graph nodes
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct GraphExecutionMetadata {
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    #[graphql(name = "nodeCount")]
    pub node_count: i32,
    #[graphql(name = "edgeCount")]
    pub edge_count: i32,
    #[graphql(name = "executionState")]
    pub execution_state: String,
    #[graphql(name = "computedDate")]
    pub computed_date: Option<String>,
    #[graphql(name = "errorMessage")]
    pub error_message: Option<String>,
}

// Node execution status change event for subscriptions
#[derive(Clone, Debug, SimpleObject)]
pub struct NodeExecutionStatusEvent {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "nodeId")]
    pub node_id: String,
    #[graphql(name = "nodeType")]
    pub node_type: PlanDagNodeType,
    #[graphql(name = "datasetExecution")]
    pub dataset_execution: Option<DataSetExecutionMetadata>,
    #[graphql(name = "graphExecution")]
    pub graph_execution: Option<GraphExecutionMetadata>,
    pub timestamp: String,
}

// Plan DAG Metadata
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PlanDagMetadataInput")]
pub struct PlanDagMetadata {
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: Option<String>,
    #[graphql(name = "lastModified")]
    pub last_modified: Option<String>,
    pub author: Option<String>,
}
