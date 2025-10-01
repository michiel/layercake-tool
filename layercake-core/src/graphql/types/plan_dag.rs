use async_graphql::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::database::entities::{plan_dag_nodes, plan_dag_edges, data_sources};

// Position for ReactFlow nodes
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
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
    pub input_type: InputType,
    pub source: String,
    pub data_type: InputDataType,
    pub output_graph_ref: String,
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
            data_type: model.source_type,
            created_at: model.created_at,
        }
    }
}

// Graph Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphNodeConfigInput")]
pub struct GraphNodeConfig {
    pub graph_id: i32,
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
    pub input_graph_ref: String,
    pub output_graph_ref: String,
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
    pub input_refs: Vec<String>,
    pub output_graph_ref: String,
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
    pub source_graph_ref: String,
    pub output_graph_ref: String,
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
    pub source_graph_ref: String,
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

// Plan DAG Node Structure
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(complex)]
pub struct PlanDagNode {
    pub id: String,
    #[graphql(name = "nodeType")]
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub config: String, // JSON string for now, will be parsed as NodeConfig
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

// Plan DAG Edge Structure
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
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
    pub metadata: EdgeMetadata,
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagNodeUpdateInput {
    pub position: Option<Position>,
    pub metadata: Option<NodeMetadata>,
    pub config: Option<String>,
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

        let metadata: NodeMetadata = serde_json::from_str(&model.metadata_json)
            .unwrap_or_else(|_| NodeMetadata {
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
            metadata,
            config: model.config_json,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<plan_dag_edges::Model> for PlanDagEdge {
    fn from(model: plan_dag_edges::Model) -> Self {
        let metadata: EdgeMetadata = serde_json::from_str(&model.metadata_json)
            .unwrap_or_else(|_| EdgeMetadata {
                label: None,
                data_type: DataType::GraphData,
            });

        Self {
            id: model.id,
            source: model.source_node_id,
            target: model.target_node_id,
            metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

// Complex object implementations for additional resolvers
#[ComplexObject]
impl PlanDagNode {
    /// Parse the config JSON into a specific node configuration type
    async fn parsed_config(&self) -> Result<String> {
        // For now, return the raw JSON. In the future, this could parse to specific types
        Ok(self.config.clone())
    }
}