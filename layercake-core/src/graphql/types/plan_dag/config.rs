use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app_context::DataSetSummary;
use crate::database::entities::data_sets;

// Data Source Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "DataSetNodeConfigInput")]
pub struct DataSetNodeConfig {
    #[graphql(name = "dataSetId")]
    pub data_set_id: Option<i32>, // Reference to DataSet entity (new)
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
pub struct DataSetReference {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub data_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<data_sets::Model> for DataSetReference {
    fn from(model: data_sets::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            data_type: model.data_type,
            created_at: model.created_at,
        }
    }
}

impl From<DataSetSummary> for DataSetReference {
    fn from(summary: DataSetSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            description: summary.description,
            data_type: summary.data_type,
            created_at: summary.created_at,
        }
    }
}

// Graph Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphNodeConfigInput")]
pub struct GraphNodeConfig {
    // Removed: graph_id - graph connections handled by visual edges in DAG
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

// Graph Artefact Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphArtefactNodeConfigInput")]
pub struct GraphArtefactNodeConfig {
    // Removed: source_graph_ref - source comes from incoming edge connection
    pub render_target: GraphArtefactRenderTarget,
    pub output_path: String,
    pub render_config: Option<RenderConfig>,
    pub graph_config: Option<GraphConfig>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GraphArtefactRenderTarget {
    Dot,
    Gml,
    Json,
    PlantUml,
    CsvNodes,
    CsvEdges,
    Mermaid,
    Custom,
}

// Tree Artefact Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "TreeArtefactNodeConfigInput")]
pub struct TreeArtefactNodeConfig {
    pub render_target: TreeArtefactRenderTarget,
    pub output_path: String,
    pub render_config: Option<RenderConfig>,
    pub graph_config: Option<GraphConfig>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum TreeArtefactRenderTarget {
    PlantUmlMindmap,
    MermaidMindmap,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "RenderConfigInput")]
pub struct RenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<Orientation>,
    pub use_default_styling: Option<bool>,
    pub theme: Option<RenderTheme>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Orientation {
    Lr,
    Tb,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum RenderTheme {
    Light,
    Dark,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphConfigInput")]
pub struct GraphConfig {
    pub generate_hierarchy: Option<bool>,
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub invert_graph: Option<bool>,
    pub aggregate_edges: Option<bool>,
    pub node_label_max_length: Option<i32>,
    pub node_label_insert_newlines_at: Option<i32>,
    pub edge_label_max_length: Option<i32>,
    pub edge_label_insert_newlines_at: Option<i32>,
}

// Union type for all node configurations
#[derive(Union, Clone, Debug, Serialize, Deserialize)]
pub enum NodeConfig {
    DataSet(DataSetNodeConfig),
    Graph(GraphNodeConfig),
    Transform(super::transforms::TransformNodeConfig),
    Filter(super::filter::FilterNodeConfig),
    Merge(MergeNodeConfig),
    GraphArtefact(GraphArtefactNodeConfig),
    TreeArtefact(TreeArtefactNodeConfig),
}
