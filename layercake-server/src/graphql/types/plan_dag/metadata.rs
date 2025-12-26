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
    SequenceData,
}

impl From<layercake_core::plan_dag::DataType> for DataType {
    fn from(data_type: layercake_core::plan_dag::DataType) -> Self {
        match data_type {
            layercake_core::plan_dag::DataType::GraphData => DataType::GraphData,
            layercake_core::plan_dag::DataType::GraphReference => DataType::GraphReference,
            layercake_core::plan_dag::DataType::SequenceData => DataType::SequenceData,
        }
    }
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

impl From<layercake_core::plan_dag::DataSetExecutionMetadata> for DataSetExecutionMetadata {
    fn from(metadata: layercake_core::plan_dag::DataSetExecutionMetadata) -> Self {
        Self {
            data_set_id: metadata.data_set_id,
            filename: metadata.filename,
            status: metadata.status,
            processed_at: metadata.processed_at,
            execution_state: metadata.execution_state,
            error_message: metadata.error_message,
        }
    }
}

// Execution metadata for Graph nodes
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct GraphExecutionMetadata {
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    #[graphql(name = "graphDataId")]
    pub graph_data_id: Option<i32>,
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
    pub annotations: Option<String>,
}

impl From<layercake_core::plan_dag::GraphExecutionMetadata> for GraphExecutionMetadata {
    fn from(metadata: layercake_core::plan_dag::GraphExecutionMetadata) -> Self {
        Self {
            graph_id: metadata.graph_id,
            graph_data_id: metadata.graph_data_id,
            node_count: metadata.node_count,
            edge_count: metadata.edge_count,
            execution_state: metadata.execution_state,
            computed_date: metadata.computed_date,
            error_message: metadata.error_message,
            annotations: metadata.annotations,
        }
    }
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

impl From<layercake_core::plan_dag::PlanDagMetadata> for PlanDagMetadata {
    fn from(metadata: layercake_core::plan_dag::PlanDagMetadata) -> Self {
        Self {
            version: metadata.version,
            name: metadata.name,
            description: metadata.description,
            created: metadata.created,
            last_modified: metadata.last_modified,
            author: metadata.author,
        }
    }
}

impl From<layercake_core::plan_dag::NodeMetadata> for NodeMetadata {
    fn from(metadata: layercake_core::plan_dag::NodeMetadata) -> Self {
        Self {
            label: metadata.label,
            description: metadata.description,
        }
    }
}

impl From<layercake_core::plan_dag::EdgeMetadata> for EdgeMetadata {
    fn from(metadata: layercake_core::plan_dag::EdgeMetadata) -> Self {
        Self {
            label: metadata.label,
            data_type: metadata.data_type.into(),
        }
    }
}
