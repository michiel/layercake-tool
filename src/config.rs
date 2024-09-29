use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlanConfig {
    pub import: ImportConfig,
    pub export: ExportProfile,
}

//
// Import configuration
//

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
    use toml;

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

        let toml_str = toml::to_string(&config).unwrap();
        println!("{}", toml_str);
        assert!(toml_str.contains("profiles"));
    }

    #[test]
    fn test_deserialization() {
        let toml_str = r#"
[[profiles]]
filename = "data.csv"
tablename = "table1"

[[profiles.transformations]]
AddSQLColumn = ["repo_id", "SELECT repo_1, repo_2, repo_1 || '-' || repo_2 AS repo_id FROM df"]

[[profiles.transformations]]
FillColumnForward = "col2"
"#;

        let config: ImportConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].filename, "data.csv");
        assert_eq!(config.profiles[0].tablename, "table1");
        assert_eq!(config.profiles[0].transformations.len(), 2);
    }

    #[test]
    fn test_planfile_deserialization() {
        let toml_str = r#"
[import]
[[import.profiles]]
filename = "data.csv"
tablename = "table1"

[[import.profiles.transformations]]
AddSQLColumn = ["repo_id", "SELECT repo_1, repo_2, repo_1 || '-' || repo_2 AS repo_id FROM df"]

[[import.profiles.transformations]]
FillColumnForward = "col2"

[export]
[[export.profiles]]
filename = "output.gml"
exporter = "GML"

[[export.profiles]]
filename = "output.dot"
exporter = "DOT"

[[export.profiles]]
filename = "nodes-full.csv"
exporter = "CSVNodes"

[[export.profiles]]
filename = "nodes-full.csv"
exporter = "CSVEdges"
"#;

        let _config: PlanConfig = toml::from_str(toml_str).unwrap();
    }
}
