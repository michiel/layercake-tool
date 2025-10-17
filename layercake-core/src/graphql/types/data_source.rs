use async_graphql::*;
use serde::{Deserialize, Serialize};

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Project;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DataSource {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
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
impl DataSource {
    async fn project(&self, ctx: &Context<'_>) -> Result<Project> {
        let graphql_ctx = ctx
            .data::<GraphQLContext>()
            .map_err(|_| Error::new("GraphQL context not found"))?;

        use crate::database::entities::projects;
        use sea_orm::EntityTrait;

        let project = projects::Entity::find_by_id(self.project_id)
            .one(&graphql_ctx.db)
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Project not found"))?;

        Ok(Project {
            id: project.id,
            name: project.name,
            description: project.description,
            created_at: project.created_at,
            updated_at: project.updated_at,
        })
    }

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

impl From<crate::database::entities::data_sources::Model> for DataSource {
    fn from(model: crate::database::entities::data_sources::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
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
pub struct CreateDataSourceInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded file content
    #[graphql(name = "fileFormat")]
    pub file_format: FileFormat,
    #[graphql(name = "dataType")]
    pub data_type: DataSourceDataType,
}

#[derive(InputObject)]
pub struct UpdateDataSourceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filename: Option<String>,
    #[graphql(name = "fileContent")]
    pub file_content: Option<String>, // Base64 encoded file content
}

/// Input for bulk upload - file format and data type are auto-detected
#[derive(InputObject)]
pub struct BulkUploadDataSourceInput {
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded file content
}

// File format enum (physical representation)
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum FileFormat {
    CSV,
    TSV,
    JSON,
}

impl From<crate::database::entities::data_sources::FileFormat> for FileFormat {
    fn from(db_format: crate::database::entities::data_sources::FileFormat) -> Self {
        match db_format {
            crate::database::entities::data_sources::FileFormat::Csv => FileFormat::CSV,
            crate::database::entities::data_sources::FileFormat::Tsv => FileFormat::TSV,
            crate::database::entities::data_sources::FileFormat::Json => FileFormat::JSON,
        }
    }
}

impl From<FileFormat> for crate::database::entities::data_sources::FileFormat {
    fn from(gql_format: FileFormat) -> Self {
        match gql_format {
            FileFormat::CSV => crate::database::entities::data_sources::FileFormat::Csv,
            FileFormat::TSV => crate::database::entities::data_sources::FileFormat::Tsv,
            FileFormat::JSON => crate::database::entities::data_sources::FileFormat::Json,
        }
    }
}

// Data type enum (semantic meaning)
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(name = "DataSourceDataType")]
pub enum DataSourceDataType {
    NODES,
    EDGES,
    LAYERS,
    GRAPH,
}

impl From<crate::database::entities::data_sources::DataType> for DataSourceDataType {
    fn from(db_type: crate::database::entities::data_sources::DataType) -> Self {
        match db_type {
            crate::database::entities::data_sources::DataType::Nodes => DataSourceDataType::NODES,
            crate::database::entities::data_sources::DataType::Edges => DataSourceDataType::EDGES,
            crate::database::entities::data_sources::DataType::Layers => DataSourceDataType::LAYERS,
            crate::database::entities::data_sources::DataType::Graph => DataSourceDataType::GRAPH,
        }
    }
}

impl From<DataSourceDataType> for crate::database::entities::data_sources::DataType {
    fn from(gql_type: DataSourceDataType) -> Self {
        match gql_type {
            DataSourceDataType::NODES => crate::database::entities::data_sources::DataType::Nodes,
            DataSourceDataType::EDGES => crate::database::entities::data_sources::DataType::Edges,
            DataSourceDataType::LAYERS => crate::database::entities::data_sources::DataType::Layers,
            DataSourceDataType::GRAPH => crate::database::entities::data_sources::DataType::Graph,
        }
    }
}

// Response types for download URLs
#[derive(SimpleObject)]
pub struct DownloadUrl {
    pub url: String,
    pub filename: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// Export/Import types
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum SpreadsheetFormat {
    XLSX,
    ODS,
}

#[derive(InputObject)]
pub struct ExportDataSourcesInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "dataSourceIds")]
    pub data_source_ids: Vec<i32>,
    pub format: SpreadsheetFormat,
}

#[derive(SimpleObject)]
pub struct ExportDataSourcesResult {
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded spreadsheet file
    pub filename: String,
    pub format: String,
}

#[derive(InputObject)]
pub struct ImportDataSourcesInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded spreadsheet file
    pub filename: String,
}

#[derive(SimpleObject)]
pub struct ImportDataSourcesResult {
    #[graphql(name = "dataSources")]
    pub data_sources: Vec<DataSource>,
    #[graphql(name = "createdCount")]
    pub created_count: i32,
    #[graphql(name = "updatedCount")]
    pub updated_count: i32,
}
