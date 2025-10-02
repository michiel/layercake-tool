use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "data_sources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub source_type: String, // DEPRECATED: kept for migration compatibility
    pub file_format: String, // 'csv', 'tsv', 'json'
    pub data_type: String, // 'nodes', 'edges', 'layers', 'graph'
    pub filename: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub blob: Vec<u8>,
    #[sea_orm(column_type = "Text")]
    pub graph_json: String,
    pub status: String, // 'active', 'processing', 'error'
    pub error_message: Option<String>,
    pub file_size: i64,
    pub processed_at: Option<ChronoDateTimeUtc>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            project_id: ActiveValue::NotSet,
            name: ActiveValue::NotSet,
            description: ActiveValue::NotSet,
            source_type: Set("".to_string()), // DEPRECATED
            file_format: ActiveValue::NotSet,
            data_type: ActiveValue::NotSet,
            filename: ActiveValue::NotSet,
            blob: ActiveValue::NotSet,
            graph_json: Set("{}".to_string()), // Default empty JSON
            status: Set("processing".to_string()),
            error_message: ActiveValue::NotSet,
            file_size: ActiveValue::NotSet,
            processed_at: ActiveValue::NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        }
    }

    pub fn set_updated_at(mut self) -> Self {
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_processed_now(mut self) -> Self {
        self.processed_at = Set(Some(chrono::Utc::now()));
        self.status = Set("active".to_string());
        self
    }

    pub fn set_error(mut self, error_msg: String) -> Self {
        self.status = Set("error".to_string());
        self.error_message = Set(Some(error_msg));
        self
    }
}

// Helper methods for the Model
impl Model {
    /// Get the file format as an enum for type safety
    pub fn get_file_format(&self) -> Option<FileFormat> {
        match self.file_format.as_str() {
            "csv" => Some(FileFormat::Csv),
            "tsv" => Some(FileFormat::Tsv),
            "json" => Some(FileFormat::Json),
            _ => None,
        }
    }

    /// Get the data type as an enum for type safety
    pub fn get_data_type(&self) -> Option<DataType> {
        match self.data_type.as_str() {
            "nodes" => Some(DataType::Nodes),
            "edges" => Some(DataType::Edges),
            "layers" => Some(DataType::Layers),
            "graph" => Some(DataType::Graph),
            _ => None,
        }
    }

    /// Check if the DataSource is ready for use
    pub fn is_ready(&self) -> bool {
        self.status == "active" && !self.graph_json.is_empty()
    }

    /// Check if the DataSource has an error
    pub fn has_error(&self) -> bool {
        self.status == "error"
    }

    /// Get file size in a human-readable format
    pub fn get_file_size_formatted(&self) -> String {
        if self.file_size < 1024 {
            format!("{} B", self.file_size)
        } else if self.file_size < 1024 * 1024 {
            format!("{:.1} KB", self.file_size as f64 / 1024.0)
        } else if self.file_size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.file_size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.file_size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Validate that the format and type combination is valid
    pub fn validate_format_type_combination(&self) -> Result<(), String> {
        match (self.file_format.as_str(), self.data_type.as_str()) {
            ("csv", "nodes") | ("csv", "edges") | ("csv", "layers") => Ok(()),
            ("tsv", "nodes") | ("tsv", "edges") | ("tsv", "layers") => Ok(()),
            ("json", "graph") => Ok(()),
            (format, dtype) => Err(format!(
                "Invalid combination: {} format cannot contain {} data",
                format, dtype
            )),
        }
    }
}

// File format enum (physical representation)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileFormat {
    Csv,
    Tsv,
    Json,
}

impl FileFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileFormat::Csv => "csv",
            FileFormat::Tsv => "tsv",
            FileFormat::Json => "json",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(FileFormat::Csv),
            "tsv" => Some(FileFormat::Tsv),
            "json" => Some(FileFormat::Json),
            _ => None,
        }
    }

    pub fn from_extension(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        if lower.ends_with(".csv") {
            Some(FileFormat::Csv)
        } else if lower.ends_with(".tsv") {
            Some(FileFormat::Tsv)
        } else if lower.ends_with(".json") {
            Some(FileFormat::Json)
        } else {
            None
        }
    }

    pub fn get_delimiter(&self) -> Option<u8> {
        match self {
            FileFormat::Csv => Some(b','),
            FileFormat::Tsv => Some(b'\t'),
            FileFormat::Json => None,
        }
    }
}

// Data type enum (semantic meaning)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Nodes,
    Edges,
    Layers,
    Graph,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::Nodes => "nodes",
            DataType::Edges => "edges",
            DataType::Layers => "layers",
            DataType::Graph => "graph",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nodes" => Some(DataType::Nodes),
            "edges" => Some(DataType::Edges),
            "layers" => Some(DataType::Layers),
            "graph" => Some(DataType::Graph),
            _ => None,
        }
    }

    pub fn get_expected_headers(&self) -> Vec<&'static str> {
        match self {
            DataType::Nodes => vec!["id", "label"],
            DataType::Edges => vec!["id", "source", "target"],
            DataType::Layers => vec!["id", "label"],
            DataType::Graph => vec![], // JSON doesn't have headers
        }
    }

    pub fn get_optional_headers(&self) -> Vec<&'static str> {
        match self {
            DataType::Nodes => vec!["layer", "x", "y", "description", "color"],
            DataType::Edges => vec!["label", "description", "weight", "color"],
            DataType::Layers => vec!["color", "description", "z_index"],
            DataType::Graph => vec![],
        }
    }

    pub fn is_compatible_with_format(&self, format: &FileFormat) -> bool {
        match (format, self) {
            (FileFormat::Csv, DataType::Nodes) |
            (FileFormat::Csv, DataType::Edges) |
            (FileFormat::Csv, DataType::Layers) |
            (FileFormat::Tsv, DataType::Nodes) |
            (FileFormat::Tsv, DataType::Edges) |
            (FileFormat::Tsv, DataType::Layers) |
            (FileFormat::Json, DataType::Graph) => true,
            _ => false,
        }
    }
}

// DEPRECATED: Keep for backward compatibility during migration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataSourceType {
    CsvNodes,
    CsvEdges,
    CsvLayers,
    JsonGraph,
}

impl DataSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSourceType::CsvNodes => "csv_nodes",
            DataSourceType::CsvEdges => "csv_edges",
            DataSourceType::CsvLayers => "csv_layers",
            DataSourceType::JsonGraph => "json_graph",
        }
    }

    pub fn to_format_and_type(&self) -> (FileFormat, DataType) {
        match self {
            DataSourceType::CsvNodes => (FileFormat::Csv, DataType::Nodes),
            DataSourceType::CsvEdges => (FileFormat::Csv, DataType::Edges),
            DataSourceType::CsvLayers => (FileFormat::Csv, DataType::Layers),
            DataSourceType::JsonGraph => (FileFormat::Json, DataType::Graph),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_from_extension() {
        assert_eq!(FileFormat::from_extension("test.csv"), Some(FileFormat::Csv));
        assert_eq!(FileFormat::from_extension("test.tsv"), Some(FileFormat::Tsv));
        assert_eq!(FileFormat::from_extension("test.json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_extension("test.txt"), None);
    }

    #[test]
    fn test_data_type_headers() {
        let node_type = DataType::Nodes;
        assert_eq!(node_type.get_expected_headers(), vec!["id", "label"]);
        assert!(node_type.get_optional_headers().contains(&"layer"));
    }

    #[test]
    fn test_format_type_compatibility() {
        assert!(DataType::Nodes.is_compatible_with_format(&FileFormat::Csv));
        assert!(DataType::Edges.is_compatible_with_format(&FileFormat::Tsv));
        assert!(DataType::Graph.is_compatible_with_format(&FileFormat::Json));
        assert!(!DataType::Graph.is_compatible_with_format(&FileFormat::Csv));
        assert!(!DataType::Nodes.is_compatible_with_format(&FileFormat::Json));
    }

    #[test]
    fn test_model_validation() {
        let model = Model {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            description: None,
            source_type: "".to_string(),
            file_format: "csv".to_string(),
            data_type: "nodes".to_string(),
            filename: "test.csv".to_string(),
            blob: vec![],
            graph_json: "{}".to_string(),
            status: "active".to_string(),
            error_message: None,
            file_size: 1024,
            processed_at: Some(chrono::Utc::now()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(model.validate_format_type_combination().is_ok());

        let invalid_model = Model {
            file_format: "json".to_string(),
            data_type: "nodes".to_string(),
            ..model.clone()
        };

        assert!(invalid_model.validate_format_type_combination().is_err());
    }

    #[test]
    fn test_model_status_methods() {
        let mut model = Model {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            description: None,
            source_type: "".to_string(),
            file_format: "csv".to_string(),
            data_type: "nodes".to_string(),
            filename: "test.csv".to_string(),
            blob: vec![],
            graph_json: "{}".to_string(),
            status: "active".to_string(),
            error_message: None,
            file_size: 1024,
            processed_at: Some(chrono::Utc::now()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(model.is_ready());
        assert!(!model.has_error());

        model.status = "error".to_string();
        assert!(!model.is_ready());
        assert!(model.has_error());
    }

    #[test]
    fn test_file_size_formatting() {
        let model = Model {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            description: None,
            source_type: "".to_string(),
            file_format: "csv".to_string(),
            data_type: "nodes".to_string(),
            filename: "test.csv".to_string(),
            blob: vec![],
            graph_json: "{}".to_string(),
            status: "active".to_string(),
            error_message: None,
            file_size: 1536, // 1.5 KB
            processed_at: Some(chrono::Utc::now()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(model.get_file_size_formatted(), "1.5 KB");
    }
}