use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Set};
use serde::{Deserialize, Serialize};

// Re-export common types for backwards compatibility
pub use super::common_types::{DataSetOrigin, DataType, FileFormat};

/// DataSet entity for uploaded file data (CSV/TSV/JSON)
///
/// This entity stores the actual uploaded data files and their metadata. Each record
/// represents a file that has been uploaded to a project, containing the raw binary
/// data, parsed graph JSON, and processing status.
///
/// This is distinct from the `datasets` table, which tracks dataset node
/// execution in the DAG pipeline. Think of it as:
/// - `data_sets` = the library of available data files
/// - `datasets` = references to those files being used in pipeline execution
///
/// Related entities:
/// - `projects`: The project this data source belongs to
/// - `datasets`: DAG execution nodes that reference this data source
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "data_sets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,

    pub file_format: String, // 'csv', 'tsv', 'json'
    pub data_type: String,   // 'nodes', 'edges', 'layers', 'graph'
    pub origin: String,      // 'file_upload', 'manual_edit', 'rag_agent'
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

            file_format: ActiveValue::NotSet,
            data_type: ActiveValue::NotSet,
            origin: ActiveValue::NotSet,
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
        self.file_format.parse().ok()
    }

    /// Get the data type as an enum for type safety
    pub fn get_data_type(&self) -> Option<DataType> {
        self.data_type.parse().ok()
    }

    /// Get the origin as an enum for type safety
    pub fn get_origin(&self) -> Option<DataSetOrigin> {
        self.origin.parse().ok()
    }

    /// Check if the DataSet is ready for use
    pub fn is_ready(&self) -> bool {
        self.status == "active" && !self.graph_json.is_empty()
    }

    /// Check if the DataSet has an error
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
            format!(
                "{:.1} GB",
                self.file_size as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        }
    }

    /// Validate that the format and type combination is valid
    pub fn validate_format_type_combination(&self) -> Result<(), String> {
        let format = self
            .get_file_format()
            .ok_or_else(|| format!("Invalid file format: {}", self.file_format))?;
        let dtype = self
            .get_data_type()
            .ok_or_else(|| format!("Invalid data type: {}", self.data_type))?;

        if dtype.is_compatible_with_format(&format) {
            Ok(())
        } else {
            Err(format!(
                "Invalid combination: {} format cannot contain {} data",
                format, dtype
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_validation() {
        let model = Model {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            description: None,
            file_format: "csv".to_string(),
            data_type: "nodes".to_string(),
            origin: "file_upload".to_string(),
            filename: "test.csv".to_string(),
            blob: vec![],
            graph_json: "{}".to_string(),
            status: "active".to_string(),
            error_message: None,
            file_size: 0,
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
            file_format: "csv".to_string(),
            data_type: "nodes".to_string(),
            origin: "file_upload".to_string(),
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
            file_format: "csv".to_string(),
            data_type: "nodes".to_string(),
            origin: "file_upload".to_string(),
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
