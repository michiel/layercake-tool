use serde::{Deserialize, Serialize};

/// ## Structure
/// This module contains the data structures for the configuration file.
/// 
/// ```text
/// Plan
///   ├── import: ImportConfig
///   │     └── profiles: Vec<ImportProfile>
///   │            ├── filename: String
///   │            ├── tablename: String
///   │            └── transformations: Vec<Transformation>
///   │                   ├── AddSQLColumn(String, String)
///   │                   └── FillColumnForward(String)
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

#[derive(Serialize, Deserialize)]                                                                                                                                                                                                                                                                                                                                                                     
pub struct Plan {                                                                                                                                                                                                                                                                                                                                                                               
    pub import: ImportConfig,                                                                                                                                                                                                                                                                                                                                                                         
    pub export: ExportProfile,                                                                                                                                                                                                                                                                                                                                                                        
} 

#[derive(Serialize, Deserialize)]
pub struct ImportConfig {
    pub profiles: Vec<ImportProfile>,
}

#[derive(Serialize, Deserialize)]
pub enum Transformation {
    AddSQLColumn(String, String),
    FillColumnForward(String),
}

#[derive(Serialize, Deserialize)]
pub enum FileImportProfile {
    CSV(CSVImportParams),
}

#[derive(Serialize, Deserialize)]
pub struct CSVImportParams {
    pub skiprows: Option<usize>,
    pub separator: Option<char>,
}

#[derive(Serialize, Deserialize)]
pub struct ImportProfile {
    pub filename: String,
    pub tablename: String,
    pub transformations: Vec<Transformation>,
}

//
// Export configuration
//

#[derive(Serialize, Deserialize)]
pub struct ExportProfile {
    pub profiles: Vec<ExportProfileItem>,
}

#[derive(Serialize, Deserialize)]
pub struct ExportProfileItem {
    pub filename: String,
    pub exporter: Exporter,
}

#[derive(Serialize, Deserialize)]
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
                filename: "data.csv".to_string(),
                tablename: "table1".to_string(),
                transformations: vec![
                    Transformation::AddSQLColumn("col1".to_string(), "value1".to_string()),
                    Transformation::FillColumnForward("col2".to_string()),
                ],
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
    tablename: table1
    transformations:
      - !AddSQLColumn
          - "repo_id"
          - "SELECT repo_1, repo_2, repo_1 || '-' || repo_2 AS repo_id FROM df"
      - !FillColumnForward "col2"
"#;

        let config: ImportConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].filename, "data.csv");
        assert_eq!(config.profiles[0].tablename, "table1");
        assert_eq!(config.profiles[0].transformations.len(), 2);
    }
    #[test]
    fn test_planfile_deserialization() {
        let yaml_str = r#"
import:
  profiles:
    - filename: data.csv
      tablename: table1
      transformations:
        - !AddSQLColumn
            - repo_id
            - "SELECT repo_1, repo_2, repo_1 || '-' || repo_2 AS repo_id FROM df"
        - !FillColumnForward col2
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
