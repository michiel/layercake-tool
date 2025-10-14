use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::{data_sources, plan_dag_edges, plan_dag_nodes};

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

// Plan DAG Node Types (matching frontend enum)
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum PlanDagNodeType {
    #[graphql(name = "DataSourceNode")]
    DataSource,
    #[graphql(name = "GraphNode")]
    Graph,
    #[graphql(name = "TransformNode")]
    Transform,
    #[graphql(name = "MergeNode")]
    Merge,
    #[graphql(name = "CopyNode")]
    Copy,
    #[graphql(name = "OutputNode")]
    Output,
}

// Data type for edges
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DataType {
    GraphData,
    GraphReference,
}

// Data Source Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "DataSourceNodeConfigInput")]
pub struct DataSourceNodeConfig {
    #[graphql(name = "dataSourceId")]
    pub data_source_id: Option<i32>, // Reference to DataSource entity (new)
    pub display_mode: Option<String>, // 'summary' | 'detailed' | 'preview'

    // Legacy fields (for backward compatibility)
    pub input_type: Option<InputType>,
    pub source: Option<String>,
    pub data_type: Option<InputDataType>,
    // Removed: output_graph_ref - output connections handled by visual edges in DAG
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum InputType {
    CsvNodesFromFile,
    CsvEdgesFromFile,
    CsvLayersFromFile,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum InputDataType {
    Nodes,
    Edges,
    Layers,
}

// Data Source Reference for dropdown selection
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceReference {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub data_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<data_sources::Model> for DataSourceReference {
    fn from(model: data_sources::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            data_type: model.data_type,
            created_at: model.created_at,
        }
    }
}

// Graph Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphNodeConfigInput")]
pub struct GraphNodeConfig {
    // Removed: graph_id - graph connections handled by visual edges in DAG
    pub is_reference: bool,
    pub metadata: GraphNodeMetadata,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphNodeMetadataInput")]
pub struct GraphNodeMetadata {
    pub node_count: Option<i32>,
    pub edge_count: Option<i32>,
    #[graphql(name = "lastModified")]
    pub last_modified: Option<String>,
}

// Transform Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "TransformNodeConfigInput")]
pub struct TransformNodeConfig {
    // Removed: input_graph_ref - input connections handled by incoming edges
    // Removed: output_graph_ref - output connections handled by outgoing edges
    pub transform_type: TransformType,
    pub transform_config: TransformConfig,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum TransformType {
    PartitionDepthLimit,
    InvertGraph,
    FilterNodes,
    FilterEdges,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "TransformConfigInput")]
pub struct TransformConfig {
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub generate_hierarchy: Option<bool>,
    pub invert_graph: Option<bool>,
    pub node_filter: Option<String>,
    pub edge_filter: Option<String>,
}

// Merge Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "MergeNodeConfigInput")]
pub struct MergeNodeConfig {
    // Removed: input_refs - inputs come from incoming edges
    // Removed: output_graph_ref - output goes to outgoing edge
    pub merge_strategy: MergeStrategy,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MergeStrategy {
    Union,
    Intersection,
    Difference,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConflictResolution {
    PreferFirst,
    PreferLast,
    Manual,
}

// Copy Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "CopyNodeConfigInput")]
pub struct CopyNodeConfig {
    // Removed: source_graph_ref - source comes from incoming edge
    // Removed: output_graph_ref - output goes to outgoing edge
    pub copy_type: CopyType,
    pub preserve_metadata: bool,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum CopyType {
    DeepCopy,
    ShallowCopy,
    Reference,
}

// Output Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "OutputNodeConfigInput")]
pub struct OutputNodeConfig {
    // Removed: source_graph_ref - source comes from incoming edge connection
    pub render_target: RenderTarget,
    pub output_path: String,
    pub render_config: Option<RenderConfig>,
    pub graph_config: Option<GraphConfig>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum RenderTarget {
    Dot,
    Gml,
    Json,
    PlantUml,
    CsvNodes,
    CsvEdges,
    Mermaid,
    Custom,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "RenderConfigInput")]
pub struct RenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<Orientation>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Orientation {
    Lr,
    Tb,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphConfigInput")]
pub struct GraphConfig {
    pub generate_hierarchy: Option<bool>,
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub invert_graph: Option<bool>,
    pub node_label_max_length: Option<i32>,
    pub node_label_insert_newlines_at: Option<i32>,
    pub edge_label_max_length: Option<i32>,
    pub edge_label_insert_newlines_at: Option<i32>,
}

// Union type for all node configurations
#[derive(Union, Clone, Debug, Serialize, Deserialize)]
pub enum NodeConfig {
    DataSource(DataSourceNodeConfig),
    Graph(GraphNodeConfig),
    Transform(TransformNodeConfig),
    Merge(MergeNodeConfig),
    Copy(CopyNodeConfig),
    Output(OutputNodeConfig),
}

// Execution metadata for DataSource nodes
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceExecutionMetadata {
    #[graphql(name = "dataSourceId")]
    pub data_source_id: i32,
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

// Plan DAG Node Structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagNode {
    pub id: String,
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub source_position: Option<String>,
    pub target_position: Option<String>,
    pub metadata: NodeMetadata,
    pub config: String, // JSON string for now, will be parsed as NodeConfig
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Optional execution metadata populated by query resolver
    pub datasource_execution: Option<DataSourceExecutionMetadata>,
    pub graph_execution: Option<GraphExecutionMetadata>,
}

#[Object]
impl PlanDagNode {
    async fn id(&self) -> &str {
        &self.id
    }

    #[graphql(name = "nodeType")]
    async fn node_type(&self) -> PlanDagNodeType {
        self.node_type
    }

    async fn position(&self) -> &Position {
        &self.position
    }

    #[graphql(name = "sourcePosition")]
    async fn source_position(&self) -> Option<&String> {
        self.source_position.as_ref()
    }

    #[graphql(name = "targetPosition")]
    async fn target_position(&self) -> Option<&String> {
        self.target_position.as_ref()
    }

    async fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }

    async fn config(&self) -> &str {
        &self.config
    }

    #[graphql(name = "createdAt")]
    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[graphql(name = "updatedAt")]
    async fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(name = "datasourceExecution")]
    async fn datasource_execution(&self) -> Option<&DataSourceExecutionMetadata> {
        self.datasource_execution.as_ref()
    }

    #[graphql(name = "graphExecution")]
    async fn graph_execution(&self) -> Option<&GraphExecutionMetadata> {
        self.graph_execution.as_ref()
    }

    /// Parse the config JSON into a specific node configuration type
    async fn parsed_config(&self) -> Result<String> {
        // For now, return the raw JSON. In the future, this could parse to specific types
        Ok(self.config.clone())
    }
}

// Plan DAG Edge Structure
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[graphql(name = "sourceHandle")]
    pub source_handle: Option<String>,
    #[graphql(name = "targetHandle")]
    pub target_handle: Option<String>,
    pub metadata: EdgeMetadata,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
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

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagNodeInput {
    pub id: String,
    #[graphql(name = "nodeType")]
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub config: String, // JSON string
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagEdgeInput {
    pub id: String,
    pub source: String,
    pub target: String,
    #[graphql(name = "sourceHandle")]
    pub source_handle: Option<String>,
    #[graphql(name = "targetHandle")]
    pub target_handle: Option<String>,
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
    #[graphql(name = "sourceHandle")]
    pub source_handle: Option<String>,
    #[graphql(name = "targetHandle")]
    pub target_handle: Option<String>,
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

// Conversions from database entities
impl From<plan_dag_nodes::Model> for PlanDagNode {
    fn from(model: plan_dag_nodes::Model) -> Self {
        let node_type = match model.node_type.as_str() {
            "DataSourceNode" => PlanDagNodeType::DataSource,
            "GraphNode" => PlanDagNodeType::Graph,
            "TransformNode" => PlanDagNodeType::Transform,
            "MergeNode" => PlanDagNodeType::Merge,
            "CopyNode" => PlanDagNodeType::Copy,
            "OutputNode" => PlanDagNodeType::Output,
            _ => PlanDagNodeType::DataSource, // Default fallback
        };

        let metadata: NodeMetadata =
            serde_json::from_str(&model.metadata_json).unwrap_or_else(|_| NodeMetadata {
                label: "Unnamed Node".to_string(),
                description: None,
            });

        Self {
            id: model.id,
            node_type,
            position: Position {
                x: model.position_x,
                y: model.position_y,
            },
            source_position: model.source_position,
            target_position: model.target_position,
            metadata,
            config: model.config_json,
            created_at: model.created_at,
            updated_at: model.updated_at,
            datasource_execution: None,
            graph_execution: None,
        }
    }
}

impl From<plan_dag_edges::Model> for PlanDagEdge {
    fn from(model: plan_dag_edges::Model) -> Self {
        let metadata: EdgeMetadata =
            serde_json::from_str(&model.metadata_json).unwrap_or(EdgeMetadata {
                label: None,
                data_type: DataType::GraphData,
            });

        Self {
            id: model.id,
            source: model.source_node_id,
            target: model.target_node_id,
            source_handle: model.source_handle,
            target_handle: model.target_handle,
            metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
