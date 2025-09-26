use async_graphql::*;
use serde::{Deserialize, Serialize};

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Project;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DataSource {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub filename: String,
    pub graph_json: String,
    pub status: String,
    pub error_message: Option<String>,
    pub file_size: i64,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl DataSource {
    async fn project(&self, ctx: &Context<'_>) -> Result<Project> {
        let graphql_ctx = ctx.data::<GraphQLContext>()
            .map_err(|_| Error::new("GraphQL context not found"))?;

        use sea_orm::EntityTrait;
        use crate::database::entities::projects;

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
            format!("{:.1} GB", self.file_size as f64 / (1024.0 * 1024.0 * 1024.0))
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
            source_type: model.source_type,
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
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub file: Upload,
}

#[derive(InputObject)]
pub struct UpdateDataSourceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub file: Option<Upload>,
}

// Data source type enum
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DataSourceType {
    #[graphql(name = "csv_nodes")]
    CsvNodes,
    #[graphql(name = "csv_edges")]
    CsvEdges,
    #[graphql(name = "csv_layers")]
    CsvLayers,
    #[graphql(name = "json_graph")]
    JsonGraph,
}

impl From<crate::database::entities::data_sources::DataSourceType> for DataSourceType {
    fn from(db_type: crate::database::entities::data_sources::DataSourceType) -> Self {
        match db_type {
            crate::database::entities::data_sources::DataSourceType::CsvNodes => DataSourceType::CsvNodes,
            crate::database::entities::data_sources::DataSourceType::CsvEdges => DataSourceType::CsvEdges,
            crate::database::entities::data_sources::DataSourceType::CsvLayers => DataSourceType::CsvLayers,
            crate::database::entities::data_sources::DataSourceType::JsonGraph => DataSourceType::JsonGraph,
        }
    }
}

// Upload scalar for file uploads
#[derive(Clone)]
pub struct Upload {
    pub filename: String,
    pub content_type: Option<String>,
    pub content: Vec<u8>,
}

#[Scalar]
impl ScalarType for Upload {
    fn parse(_value: Value) -> InputValueResult<Self> {
        // File uploads are handled by multipart/form-data, not JSON
        Err(InputValueError::custom("Upload must be provided as multipart/form-data"))
    }

    fn to_value(&self) -> Value {
        Value::Null
    }
}

// Response types for download URLs
#[derive(SimpleObject)]
pub struct DownloadUrl {
    pub url: String,
    pub filename: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}