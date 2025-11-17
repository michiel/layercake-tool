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
    PlantUmlWbs,
    MermaidMindmap,
    MermaidTreemap,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "RenderConfigInput")]
pub struct RenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<Orientation>,
    pub apply_layers: Option<bool>,
    pub built_in_styles: Option<RenderBuiltinStyle>,
    pub target_options: Option<RenderTargetOptions>,
    pub add_node_comments_as_notes: Option<bool>,
    pub note_position: Option<NotePosition>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum NotePosition {
    #[graphql(name = "Left")]
    #[serde(rename = "left")]
    Left,
    #[graphql(name = "Right")]
    #[serde(rename = "right")]
    Right,
    #[graphql(name = "Top")]
    #[serde(rename = "top")]
    Top,
    #[graphql(name = "Bottom")]
    #[serde(rename = "bottom")]
    Bottom,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Orientation {
    Lr,
    Tb,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum RenderBuiltinStyle {
    #[graphql(name = "NONE")]
    None,
    #[graphql(name = "LIGHT")]
    Light,
    #[graphql(name = "DARK")]
    Dark,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "RenderTargetOptionsInput")]
pub struct RenderTargetOptions {
    pub graphviz: Option<GraphvizRenderOptions>,
    pub mermaid: Option<MermaidRenderOptions>,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphvizRenderOptionsInput")]
pub struct GraphvizRenderOptions {
    pub layout: Option<GraphvizLayout>,
    pub overlap: Option<bool>,
    pub splines: Option<bool>,
    pub nodesep: Option<f32>,
    pub ranksep: Option<f32>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GraphvizLayout {
    #[graphql(name = "DOT")]
    #[serde(rename = "dot")]
    Dot,
    #[graphql(name = "NEATO")]
    #[serde(rename = "neato")]
    Neato,
    #[graphql(name = "FDP")]
    #[serde(rename = "fdp")]
    Fdp,
    #[graphql(name = "CIRCO")]
    #[serde(rename = "circo")]
    Circo,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "MermaidRenderOptionsInput")]
pub struct MermaidRenderOptions {
    pub look: Option<MermaidLook>,
    pub display: Option<MermaidDisplay>,
    pub theme: Option<MermaidTheme>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MermaidLook {
    #[graphql(name = "DEFAULT")]
    #[serde(rename = "default")]
    Default,
    #[graphql(name = "HAND_DRAWN")]
    #[serde(rename = "handDrawn")]
    HandDrawn,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MermaidDisplay {
    #[graphql(name = "FULL")]
    #[serde(rename = "full")]
    Full,
    #[graphql(name = "COMPACT")]
    #[serde(rename = "compact")]
    Compact,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MermaidTheme {
    #[graphql(name = "DEFAULT")]
    #[serde(rename = "default")]
    Default,
    #[graphql(name = "DARK")]
    #[serde(rename = "dark")]
    Dark,
    #[graphql(name = "NEUTRAL")]
    #[serde(rename = "neutral")]
    Neutral,
    #[graphql(name = "BASE")]
    #[serde(rename = "base")]
    Base,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphConfigInput")]
pub struct GraphConfig {
    pub generate_hierarchy: Option<bool>,
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub invert_graph: Option<bool>,
    pub aggregate_edges: Option<bool>,
    pub drop_unconnected_nodes: Option<bool>,
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
