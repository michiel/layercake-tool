use serde::{Deserialize, Serialize};

use super::PlanDagNodeType;

// Node metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub label: String,
    pub description: Option<String>,
}

// Edge metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub label: Option<String>,
    pub data_type: DataType,
}

// Data type for edges
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DataType {
    GraphData,
    GraphReference,
    SequenceData,
}

// Execution metadata for DataSet nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSetExecutionMetadata {
    pub data_set_id: i32,
    pub filename: String,
    pub status: String,
    pub processed_at: Option<String>,
    pub execution_state: String,
    pub error_message: Option<String>,
}

// Execution metadata for Graph nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphExecutionMetadata {
    pub graph_id: i32,
    pub graph_data_id: Option<i32>,
    pub node_count: i32,
    pub edge_count: i32,
    pub execution_state: String,
    pub computed_date: Option<String>,
    pub error_message: Option<String>,
    pub annotations: Option<String>,
}

// Node execution status change event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeExecutionStatusEvent {
    pub project_id: i32,
    pub node_id: String,
    pub node_type: PlanDagNodeType,
    pub dataset_execution: Option<DataSetExecutionMetadata>,
    pub graph_execution: Option<GraphExecutionMetadata>,
    pub timestamp: String,
}

// Plan DAG Metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagMetadata {
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: Option<String>,
    pub last_modified: Option<String>,
    pub author: Option<String>,
}
