#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ## Structure
/// This module contains the data structures for the configuration file.
///
/// ```text
/// Plan
///   ├── meta: Option<Meta>
///   │   └── name: Option<String>
///   ├── import: ImportConfig
///   │   └── profiles: Vec<ImportProfile>
///   │       ├── filename: String
///   │       └── filetype: ImportFileType
///   │           ├── Edges
///   │           ├── Nodes
///   │           └── Layers
///   └── export: ExportProfile
///       └── profiles: Vec<ExportProfileItem>
///           ├── filename: String
///           ├── exporter: ExportFileType
///           │   ├── GML
///           │   ├── DOT
///           │   ├── DOTHierarchy
///           │   ├── JSON
///           │   ├── PlantUML
///           │   ├── CSVNodes
///           │   ├── CSVEdges
///           │   ├── Mermaid
///           │   └── Custom(CustomExportProfile)
///           └── graph_config: Option<ExportProfileGraphConfig>
///               ├── generate_hierarchy: Option<bool>
///               ├── max_partition_depth: Option<i32>
///               └── max_partition_width: Option<i32>

//
// Import configuration
//

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Meta {
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Plan {
    pub meta: Option<Meta>,
    pub import: ImportConfig,
    pub export: ExportProfile,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ImportConfig {
    pub profiles: Vec<ImportProfile>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileImportProfile {
    CSV(CSVImportParams),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CSVImportParams {
    pub skiprows: Option<usize>,
    pub separator: Option<char>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ImportFileType {
    Edges,
    Nodes,
    Layers,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImportProfile {
    pub filename: String,
    pub filetype: ImportFileType,
}

//
// Export configuration
//

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExportProfile {
    pub profiles: Vec<ExportProfileItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportProfileItem {
    pub filename: String,
    pub exporter: ExportFileType,
    pub render_config: Option<ExportProfileRenderConfig>,
    pub graph_config: Option<ExportProfileGraphConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
pub struct ExportProfileGraphConfig {
    pub generate_hierarchy: Option<bool>,
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub invert_graph: Option<bool>,
    pub aggregate_edges: Option<bool>,
    pub drop_unconnected_nodes: Option<bool>,
    pub node_label_max_length: Option<usize>,
    pub node_label_insert_newlines_at: Option<usize>,
    pub edge_label_max_length: Option<usize>,
    pub edge_label_insert_newlines_at: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportProfileRenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<RenderConfigOrientation>,
    pub apply_layers: Option<bool>,
    pub built_in_styles: Option<RenderConfigBuiltInStyle>,
    pub target_options: Option<RenderTargetOptions>,
    #[serde(rename = "use_default_styling")]
    pub legacy_use_default_styling: Option<bool>,
    #[serde(rename = "theme")]
    pub legacy_theme: Option<RenderConfigTheme>,
    pub add_node_comments_as_notes: Option<bool>,
    pub note_position: Option<NotePosition>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub enum RenderConfigOrientation {
    LR,
    TB,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum NotePosition {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "bottom")]
    Bottom,
}

impl Default for NotePosition {
    fn default() -> Self {
        NotePosition::Left
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum RenderConfigBuiltInStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub enum RenderConfigTheme {
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomExportProfile {
    pub template: String,
    pub partials: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExportFileType {
    GML,
    DOT,
    DOTHierarchy,
    JSON,
    PlantUML,
    PlantUmlMindmap,
    PlantUmlWbs,
    CSVNodes,
    CSVEdges,
    CSVMatrix,
    Mermaid,
    MermaidMindmap,
    MermaidTreemap,
    JSGraph,
    Custom(CustomExportProfile),
}

impl Default for ExportProfileRenderConfig {
    fn default() -> Self {
        Self {
            contain_nodes: Some(true),
            orientation: Some(RenderConfigOrientation::TB),
            apply_layers: Some(true),
            built_in_styles: Some(RenderConfigBuiltInStyle::Light),
            target_options: Some(RenderTargetOptions::default()),
            legacy_use_default_styling: Some(true),
            legacy_theme: Some(RenderConfigTheme::Light),
            add_node_comments_as_notes: Some(false),
            note_position: Some(NotePosition::Left),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenderConfig {
    pub contain_nodes: bool,
    pub orientation: RenderConfigOrientation,
    pub apply_layers: bool,
    pub built_in_styles: RenderConfigBuiltInStyle,
    #[serde(default)]
    pub target_options: RenderTargetOptions,
    #[serde(default)]
    pub add_node_comments_as_notes: bool,
    #[serde(default)]
    pub note_position: NotePosition,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenderTargetOptions {
    pub graphviz: Option<GraphvizRenderOptions>,
    pub mermaid: Option<MermaidRenderOptions>,
}

impl Default for RenderTargetOptions {
    fn default() -> Self {
        Self {
            graphviz: Some(GraphvizRenderOptions::default()),
            mermaid: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphvizRenderOptions {
    pub layout: GraphvizLayout,
    pub overlap: bool,
    pub splines: bool,
    pub nodesep: f32,
    pub ranksep: f32,
    #[serde(default)]
    pub comment_style: GraphvizCommentStyle,
}

impl Default for GraphvizRenderOptions {
    fn default() -> Self {
        Self {
            layout: GraphvizLayout::Dot,
            overlap: false,
            splines: true,
            nodesep: 0.3,
            ranksep: 1.3,
            comment_style: GraphvizCommentStyle::Label,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GraphvizLayout {
    #[serde(rename = "dot")]
    Dot,
    #[serde(rename = "neato")]
    Neato,
    #[serde(rename = "fdp")]
    Fdp,
    #[serde(rename = "circo")]
    Circo,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GraphvizCommentStyle {
    #[serde(rename = "label")]
    Label,
    #[serde(rename = "tooltip")]
    Tooltip,
}

impl Default for GraphvizCommentStyle {
    fn default() -> Self {
        GraphvizCommentStyle::Label
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MermaidRenderOptions {
    pub look: MermaidLook,
    pub display: MermaidDisplay,
    pub theme: MermaidTheme,
}

impl Default for MermaidRenderOptions {
    fn default() -> Self {
        Self {
            look: MermaidLook::Default,
            display: MermaidDisplay::Full,
            theme: MermaidTheme::Default,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum MermaidLook {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "handDrawn")]
    HandDrawn,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum MermaidTheme {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "neutral")]
    Neutral,
    #[serde(rename = "base")]
    Base,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum MermaidDisplay {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "compact")]
    Compact,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct GraphConfig {
    pub generate_hierarchy: bool,
    pub max_partition_depth: i32,
    pub max_partition_width: i32,
    pub invert_graph: bool,
    pub aggregate_edges: bool,
    pub drop_unconnected_nodes: bool,
    pub node_label_max_length: usize,
    pub node_label_insert_newlines_at: usize,
    pub edge_label_max_length: usize,
    pub edge_label_insert_newlines_at: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            generate_hierarchy: false,
            max_partition_depth: 0,
            max_partition_width: 0,
            invert_graph: false,
            aggregate_edges: true,
            drop_unconnected_nodes: false,
            node_label_max_length: 0,
            node_label_insert_newlines_at: 0,
            edge_label_max_length: 0,
            edge_label_insert_newlines_at: 0,
        }
    }
}

impl ExportProfileItem {
    pub fn get_graph_config(&self) -> GraphConfig {
        let graph_config = self.graph_config.unwrap_or_default();

        let generate_hierarchy = graph_config.generate_hierarchy.unwrap_or(false);
        let max_partition_depth = graph_config.max_partition_depth.unwrap_or(0);
        let max_partition_width = graph_config.max_partition_width.unwrap_or(0);
        let invert_graph = graph_config.invert_graph.unwrap_or(false);
        let aggregate_edges = graph_config.aggregate_edges.unwrap_or(true);
        let drop_unconnected_nodes = graph_config.drop_unconnected_nodes.unwrap_or(false);
        let node_label_max_length = graph_config.node_label_max_length.unwrap_or(0);
        let node_label_insert_newlines_at = graph_config.node_label_insert_newlines_at.unwrap_or(0);
        let edge_label_max_length = graph_config.edge_label_max_length.unwrap_or(0);
        let edge_label_insert_newlines_at = graph_config.edge_label_insert_newlines_at.unwrap_or(0);

        GraphConfig {
            generate_hierarchy,
            max_partition_depth,
            max_partition_width,
            invert_graph,
            aggregate_edges,
            drop_unconnected_nodes,
            node_label_max_length,
            node_label_insert_newlines_at,
            edge_label_max_length,
            edge_label_insert_newlines_at,
        }
    }
    pub fn get_render_config(&self) -> RenderConfig {
        let render_config = self.render_config.clone().unwrap_or_default();
        let orientation = render_config
            .orientation
            .unwrap_or(RenderConfigOrientation::TB);
        let contain_nodes = render_config.contain_nodes.unwrap_or(true);
        let apply_layers = render_config
            .apply_layers
            .or_else(|| render_config.legacy_use_default_styling.map(|_| true))
            .unwrap_or(true);

        let built_in_styles = render_config
            .built_in_styles
            .or_else(|| {
                match (
                    render_config.legacy_use_default_styling,
                    render_config.legacy_theme,
                ) {
                    (Some(false), _) => Some(RenderConfigBuiltInStyle::None),
                    (Some(true), Some(RenderConfigTheme::Dark)) => {
                        Some(RenderConfigBuiltInStyle::Dark)
                    }
                    (Some(true), _) => Some(RenderConfigBuiltInStyle::Light),
                    (None, Some(RenderConfigTheme::Dark)) => Some(RenderConfigBuiltInStyle::Dark),
                    (None, _) => None,
                }
            })
            .unwrap_or(RenderConfigBuiltInStyle::Light);
        let mut target_options = render_config
            .target_options
            .unwrap_or_else(RenderTargetOptions::default);
        if target_options.graphviz.is_none() {
            target_options.graphviz = Some(GraphvizRenderOptions::default());
        }
        let add_node_comments_as_notes = render_config.add_node_comments_as_notes.unwrap_or(false);
        let note_position = render_config.note_position.unwrap_or(NotePosition::Left);

        RenderConfig {
            contain_nodes,
            orientation,
            apply_layers,
            built_in_styles,
            target_options,
            add_node_comments_as_notes,
            note_position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let config = ImportConfig {
            profiles: vec![ImportProfile {
                filetype: ImportFileType::Nodes,
                filename: "data.csv".to_string(),
            }],
        };

        let yaml_str = serde_yaml::to_string(&config).unwrap();
        println!("{}", yaml_str);
        assert!(yaml_str.contains("profiles"));
    }
    #[test]
    fn test_deserialization() {
        let yaml_str = r#"
profiles:
  - filename: data.csv
    filetype: Nodes
"#;

        let config: ImportConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].filename, "data.csv");
    }
    #[test]
    fn test_planfile_deserialization() {
        let yaml_str = r#"
import:
  profiles:
    - filename: data.csv
      filetype: Nodes
export:
  profiles:
    - filename: output.gml
      exporter: GML
    - filename: output.dot
      exporter: DOT
    - filename: nodes-full.csv
      exporter: CSVNodes
    - filename: nodes-full.csv
      exporter: CSVEdges
"#;

        let _config: Plan = serde_yaml::from_str(yaml_str).unwrap();
    }
}
