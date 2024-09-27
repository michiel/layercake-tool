use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ImportConfig {
    profiles: Vec<ImportProfile>,
}

#[derive(Serialize, Deserialize)]
enum Transformation {
    AddSQLColumn(String, String),
    FillColumnForward(String),
}

#[derive(Serialize, Deserialize)]
enum FileImportProfile {
    CSV(CSVImportParams),
}

#[derive(Serialize, Deserialize)]
struct CSVImportParams {
    skiprows: Option<usize>,
    separator: Option<char>,
}

#[derive(Serialize, Deserialize)]
struct ImportProfile {
    filename: String,
    tablename: String,
    transformations: Vec<Transformation>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_serialization() {
        let config = ImportConfig {
            profiles: vec![
                ImportProfile {
                    filename: "data.csv".to_string(),
                    tablename: "table1".to_string(),
                    transformations: vec![
                        Transformation::AddSQLColumn("col1".to_string(), "value1".to_string()),
                        Transformation::FillColumnForward("col2".to_string()),
                    ],
                },
            ],
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        println!("{}", yaml);
        assert!(yaml.contains("profiles"));
    }

    #[test]
    fn test_deserialization() {
        let yaml = r#"
profiles:
  - filename: "data.csv"
    tablename: "table1"
    transformations:
      - AddSQLColumn: ["col1", "value1"]
      - FillColumnForward: "col2"
"#;

        let config: ImportConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].filename, "data.csv");
        assert_eq!(config.profiles[0].tablename, "table1");
        assert_eq!(config.profiles[0].transformations.len(), 2);
    }
}