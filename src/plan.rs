use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// ## Structure
/// This module contains the data structures for the configuration file.
///
/// ```text
/// Plan
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

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ExportProfileGraphConfig {
    pub generate_hierarchy: Option<bool>,
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ExportProfileRenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<RenderConfigOrientation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub enum RenderConfigOrientation {
    LR,
    TB,
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
    CSVNodes,
    CSVEdges,
    Mermaid,
    JSGraph,
    Custom(CustomExportProfile),
}

impl Default for ExportProfileRenderConfig {
    fn default() -> Self {
        Self {
            contain_nodes: Some(true),
            orientation: Some(RenderConfigOrientation::TB),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct RenderConfig {
    pub contain_nodes: bool,
    pub orientation: RenderConfigOrientation,
}

impl ExportProfileItem {
    pub fn get_render_config(&self) -> RenderConfig {
        let render_config = match self.render_config {
            Some(config) => config,
            None => ExportProfileRenderConfig::default(),
        };
        let orientation = render_config
            .orientation
            .unwrap_or(RenderConfigOrientation::TB);
        let contain_nodes = render_config.contain_nodes.unwrap_or(true);
        RenderConfig {
            contain_nodes,
            orientation,
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
