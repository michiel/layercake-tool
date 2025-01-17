use serde::{Deserialize, Serialize};

/// ## Structure
/// This module contains the data structures for the configuration file.
///
/// ```text
/// Plan
///   ├── import: ImportConfig
///   │     └── profiles: Vec<ImportProfile>
///   │            ├── filename: String
///   │            └── tablename: String
///   └── export: ExportProfile
///         └── profiles: Vec<ExportProfileItem>
///                ├── filename: String
///                └── exporter: Exporter
///                       ├── GML
///                       ├── DOT
///                       ├── CSVNodes
///                       └── CSVEdges
/// ```
///

//
// Import configuration
//

#[derive(Serialize, Deserialize, Debug)]
pub struct Plan {
    pub import: ImportConfig,
    pub export: ExportProfile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportConfig {
    pub profiles: Vec<ImportProfile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FileImportProfile {
    CSV(CSVImportParams),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSVImportParams {
    pub skiprows: Option<usize>,
    pub separator: Option<char>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ImportFileType {
    Edges,
    Nodes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportProfile {
    pub filename: String,
    pub tablename: String,
    pub filetype: ImportFileType,
}

//
// Export configuration
//

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportProfile {
    pub profiles: Vec<ExportProfileItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportProfileItem {
    pub filename: String,
    pub exporter: Exporter,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Exporter {
    GML,
    DOT,
    CSVNodes,
    CSVEdges,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;
    #[test]
    fn test_serialization() {
        let config = ImportConfig {
            profiles: vec![ImportProfile {
                filetype: ImportFileType::Nodes,
                filename: "data.csv".to_string(),
                tablename: "table1".to_string(),
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
    tablename: table1
"#;

        let config: ImportConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].filename, "data.csv");
        assert_eq!(config.profiles[0].tablename, "table1");
    }
    #[test]
    fn test_planfile_deserialization() {
        let yaml_str = r#"
import:
  profiles:
    - filename: data.csv
      filetype: Nodes
      tablename: table1
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
