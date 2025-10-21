use async_graphql::*;

use super::{DataSourceDataType, FileFormat};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct LibrarySource {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[graphql(name = "fileFormat")]
    pub file_format: String,
    #[graphql(name = "dataType")]
    pub data_type: String,
    pub filename: String,
    #[graphql(name = "graphJson")]
    pub graph_json: String,
    pub status: String,
    #[graphql(name = "errorMessage")]
    pub error_message: Option<String>,
    #[graphql(name = "fileSize")]
    pub file_size: i64,
    #[graphql(name = "processedAt")]
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl LibrarySource {
    async fn file_size_formatted(&self) -> String {
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

    async fn is_ready(&self) -> bool {
        self.status == "active" && !self.graph_json.is_empty()
    }

    async fn has_error(&self) -> bool {
        self.status == "error"
    }
}

impl From<crate::database::entities::library_sources::Model> for LibrarySource {
    fn from(model: crate::database::entities::library_sources::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            file_format: model.file_format,
            data_type: model.data_type,
            filename: model.filename,
            graph_json: model.graph_json,
            status: model.status,
            error_message: model.error_message,
            file_size: model.file_size,
            processed_at: model.processed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(InputObject)]
pub struct CreateLibrarySourceInput {
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    #[graphql(name = "fileContent")]
    pub file_content: String,
    #[graphql(name = "fileFormat")]
    pub file_format: FileFormat,
    #[graphql(name = "dataType")]
    pub data_type: DataSourceDataType,
}

#[derive(InputObject)]
pub struct UpdateLibrarySourceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filename: Option<String>,
    #[graphql(name = "fileContent")]
    pub file_content: Option<String>,
}

#[derive(InputObject)]
pub struct ImportLibrarySourcesInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "librarySourceIds")]
    pub library_source_ids: Vec<i32>,
}
