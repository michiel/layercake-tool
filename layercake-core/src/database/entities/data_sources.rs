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
    pub source_type: String, // 'csv_nodes', 'csv_edges', 'csv_layers', 'json_graph'
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
            source_type: ActiveValue::NotSet,
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
    /// Get the DataSource type as an enum for type safety
    pub fn get_source_type(&self) -> Option<DataSourceType> {
        match self.source_type.as_str() {
            "csv_nodes" => Some(DataSourceType::CsvNodes),
            "csv_edges" => Some(DataSourceType::CsvEdges),
            "csv_layers" => Some(DataSourceType::CsvLayers),
            "json_graph" => Some(DataSourceType::JsonGraph),
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
}

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

    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower_filename = filename.to_lowercase();
        if lower_filename.contains("node") && lower_filename.ends_with(".csv") {
            Some(DataSourceType::CsvNodes)
        } else if lower_filename.contains("edge") && lower_filename.ends_with(".csv") {
            Some(DataSourceType::CsvEdges)
        } else if lower_filename.contains("layer") && lower_filename.ends_with(".csv") {
            Some(DataSourceType::CsvLayers)
        } else if lower_filename.ends_with(".json") {
            Some(DataSourceType::JsonGraph)
        } else {
            None
        }
    }

    pub fn get_expected_headers(&self) -> Vec<&'static str> {
        match self {
            DataSourceType::CsvNodes => vec!["id", "label"],
            DataSourceType::CsvEdges => vec!["id", "source", "target"],
            DataSourceType::CsvLayers => vec!["id", "label"],
            DataSourceType::JsonGraph => vec![], // JSON doesn't have headers
        }
    }

    pub fn get_optional_headers(&self) -> Vec<&'static str> {
        match self {
            DataSourceType::CsvNodes => vec!["layer", "x", "y", "description", "color"],
            DataSourceType::CsvEdges => vec!["label", "description", "weight", "color"],
            DataSourceType::CsvLayers => vec!["color", "description", "z_index"],
            DataSourceType::JsonGraph => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_type_from_filename() {
        assert_eq!(DataSourceType::from_filename("nodes.csv"), Some(DataSourceType::CsvNodes));
        assert_eq!(DataSourceType::from_filename("edges.csv"), Some(DataSourceType::CsvEdges));
        assert_eq!(DataSourceType::from_filename("layers.csv"), Some(DataSourceType::CsvLayers));
        assert_eq!(DataSourceType::from_filename("graph.json"), Some(DataSourceType::JsonGraph));
        assert_eq!(DataSourceType::from_filename("unknown.txt"), None);
    }

    #[test]
    fn test_data_source_type_headers() {
        let node_type = DataSourceType::CsvNodes;
        assert_eq!(node_type.get_expected_headers(), vec!["id", "label"]);
        assert!(node_type.get_optional_headers().contains(&"layer"));
    }

    #[test]
    fn test_model_status_methods() {
        let mut model = Model {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            description: None,
            source_type: "csv_nodes".to_string(),
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
            source_type: "csv_nodes".to_string(),
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