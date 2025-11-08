use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Set};
use serde::{Deserialize, Serialize};

// Re-export common types for convenience
pub use super::common_types::{DataType, FileFormat};

/// LibrarySource entity for managing reusable datasource files outside of projects
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "library_sources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,

    pub file_format: String,
    pub data_type: String,
    pub filename: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub blob: Vec<u8>,
    #[sea_orm(column_type = "Text")]
    pub graph_json: String,
    pub status: String,
    pub error_message: Option<String>,
    pub file_size: i64,
    pub processed_at: Option<ChronoDateTimeUtc>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            name: ActiveValue::NotSet,
            description: ActiveValue::NotSet,
            file_format: ActiveValue::NotSet,
            data_type: ActiveValue::NotSet,
            filename: ActiveValue::NotSet,
            blob: ActiveValue::NotSet,
            graph_json: Set("{}".to_string()),
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
}

impl Model {
    pub fn get_file_format(&self) -> Option<FileFormat> {
        self.file_format.parse().ok()
    }

    pub fn get_data_type(&self) -> Option<DataType> {
        self.data_type.parse().ok()
    }

    pub fn is_ready(&self) -> bool {
        self.status == "active" && !self.graph_json.is_empty()
    }

    pub fn has_error(&self) -> bool {
        self.status == "error"
    }

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
}
