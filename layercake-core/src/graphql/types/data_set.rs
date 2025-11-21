#![allow(dead_code)]

use async_graphql::*;
use serde::{Deserialize, Serialize};

use crate::app_context::DataSetSummary;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::Project;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DataSet {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,

    #[graphql(name = "fileFormat")]
    pub file_format: String,
    pub origin: String,
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
impl DataSet {
    async fn project(&self, ctx: &Context<'_>) -> Result<Project> {
        let graphql_ctx = ctx
            .data::<GraphQLContext>()
            .map_err(|_| StructuredError::internal("GraphQL context not found"))?;

        use crate::database::entities::projects;
        use sea_orm::EntityTrait;

        let project = projects::Entity::find_by_id(self.project_id)
            .one(&graphql_ctx.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", self.project_id))?;

        Ok(Project {
            id: project.id,
            name: project.name,
            description: project.description,
            tags: serde_json::from_str(&project.tags).unwrap_or_default(),
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

impl From<crate::database::entities::data_sets::Model> for DataSet {
    fn from(model: crate::database::entities::data_sets::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            description: model.description,

            file_format: model.file_format,
            origin: model.origin,
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

impl From<DataSetSummary> for DataSet {
    fn from(summary: DataSetSummary) -> Self {
        Self {
            id: summary.id,
            project_id: summary.project_id,
            name: summary.name,
            description: summary.description,
            file_format: summary.file_format,
            origin: summary.origin,
            filename: summary.filename,
            graph_json: summary.graph_json,
            status: summary.status,
            error_message: summary.error_message,
            file_size: summary.file_size,
            processed_at: summary.processed_at,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
        }
    }
}

#[derive(InputObject)]
pub struct CreateDataSetInput {
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
    pub data_type: DataSetDataType,
}

#[derive(InputObject)]
pub struct CreateEmptyDataSetInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateDataSetInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filename: Option<String>,
    #[graphql(name = "fileContent")]
    pub file_content: Option<String>, // Base64 encoded file content
}

/// Input for bulk upload - file format and data type are auto-detected
#[derive(InputObject)]
pub struct BulkUploadDataSetInput {
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

impl From<crate::database::entities::common_types::FileFormat> for FileFormat {
    fn from(db_format: crate::database::entities::common_types::FileFormat) -> Self {
        match db_format {
            crate::database::entities::common_types::FileFormat::Csv => FileFormat::CSV,
            crate::database::entities::common_types::FileFormat::Tsv => FileFormat::TSV,
            crate::database::entities::common_types::FileFormat::Json => FileFormat::JSON,
            _ => panic!("Unsupported file format for GraphQL conversion"),
        }
    }
}

impl From<FileFormat> for crate::database::entities::common_types::FileFormat {
    fn from(gql_format: FileFormat) -> Self {
        match gql_format {
            FileFormat::CSV => crate::database::entities::common_types::FileFormat::Csv,
            FileFormat::TSV => crate::database::entities::common_types::FileFormat::Tsv,
            FileFormat::JSON => crate::database::entities::common_types::FileFormat::Json,
        }
    }
}

// Data type enum (semantic meaning)
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(name = "DataSetDataType")]
pub enum DataSetDataType {
    NODES,
    EDGES,
    LAYERS,
    GRAPH,
}

impl From<crate::database::entities::common_types::DataType> for DataSetDataType {
    fn from(db_type: crate::database::entities::common_types::DataType) -> Self {
        match db_type {
            crate::database::entities::common_types::DataType::Nodes => DataSetDataType::NODES,
            crate::database::entities::common_types::DataType::Edges => DataSetDataType::EDGES,
            crate::database::entities::common_types::DataType::Layers => DataSetDataType::LAYERS,
            crate::database::entities::common_types::DataType::Graph => DataSetDataType::GRAPH,
        }
    }
}

impl From<DataSetDataType> for crate::database::entities::common_types::DataType {
    fn from(gql_type: DataSetDataType) -> Self {
        match gql_type {
            DataSetDataType::NODES => crate::database::entities::common_types::DataType::Nodes,
            DataSetDataType::EDGES => crate::database::entities::common_types::DataType::Edges,
            DataSetDataType::LAYERS => crate::database::entities::common_types::DataType::Layers,
            DataSetDataType::GRAPH => crate::database::entities::common_types::DataType::Graph,
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
pub struct ExportDataSetsInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "dataSetIds")]
    pub data_set_ids: Vec<i32>,
    pub format: SpreadsheetFormat,
}

#[derive(SimpleObject)]
pub struct ExportDataSetsResult {
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded spreadsheet file
    pub filename: String,
    pub format: String,
}

#[derive(InputObject)]
pub struct ImportDataSetsInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded spreadsheet file
    pub filename: String,
}

#[derive(SimpleObject)]
pub struct ImportDataSetsResult {
    #[graphql(name = "dataSets")]
    pub data_sets: Vec<DataSet>,
    #[graphql(name = "createdCount")]
    pub created_count: i32,
    #[graphql(name = "updatedCount")]
    pub updated_count: i32,
}
