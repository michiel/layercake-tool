use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::database::entities::{projects, sequences, stories};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Project;

/// Layer source configuration for a story
/// Controls how layer sources are rendered - if a source is in this list,
/// it's disabled and uses the fallback style mode instead of project layer colours
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "StoryLayerConfigInput")]
pub struct StoryLayerConfig {
    #[graphql(name = "sourceDatasetId")]
    pub source_dataset_id: Option<i32>, // None = manual layers
    pub mode: String, // 'default' | 'light' | 'dark' - fallback style when source is disabled
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Story {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[graphql(name = "enabledDatasetIds")]
    pub enabled_dataset_ids: Vec<i32>,
    #[graphql(name = "layerConfig")]
    pub layer_config: Vec<StoryLayerConfig>,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl From<stories::Model> for Story {
    fn from(model: stories::Model) -> Self {
        let tags: Vec<String> = serde_json::from_str(&model.tags).unwrap_or_default();
        let enabled_dataset_ids: Vec<i32> =
            serde_json::from_str(&model.enabled_dataset_ids).unwrap_or_default();
        let layer_config: Vec<StoryLayerConfig> =
            serde_json::from_str(&model.layer_config).unwrap_or_default();

        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            description: model.description,
            tags,
            enabled_dataset_ids,
            layer_config,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[ComplexObject]
impl Story {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(self.project_id)
            .one(&context.db)
            .await?;

        Ok(project.map(Project::from))
    }

    #[graphql(name = "sequenceCount")]
    async fn sequence_count(&self, ctx: &Context<'_>) -> Result<i32> {
        use sea_orm::{ColumnTrait, QueryFilter};

        let context = ctx.data::<GraphQLContext>()?;
        let count = sequences::Entity::find()
            .filter(sequences::Column::StoryId.eq(self.id))
            .count(&context.db)
            .await?;

        Ok(count as i32)
    }
}

#[derive(InputObject)]
pub struct CreateStoryInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[graphql(name = "enabledDatasetIds")]
    pub enabled_dataset_ids: Option<Vec<i32>>,
    #[graphql(name = "layerConfig")]
    pub layer_config: Option<Vec<StoryLayerConfig>>,
}

#[derive(InputObject)]
pub struct UpdateStoryInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[graphql(name = "enabledDatasetIds")]
    pub enabled_dataset_ids: Option<Vec<i32>>,
    #[graphql(name = "layerConfig")]
    pub layer_config: Option<Vec<StoryLayerConfig>>,
}

/// Story export/import enums and types

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum StoryExportFormat {
    #[graphql(name = "CSV")]
    Csv,
    #[graphql(name = "JSON")]
    Json,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum StoryImportFormat {
    #[graphql(name = "CSV")]
    Csv,
    #[graphql(name = "JSON")]
    Json,
}

#[derive(SimpleObject)]
pub struct StoryExport {
    pub filename: String,
    /// Base64-encoded file content
    pub content: String,
    #[graphql(name = "mimeType")]
    pub mime_type: String,
}

#[derive(SimpleObject)]
pub struct StoryImportResult {
    #[graphql(name = "importedStories")]
    pub imported_stories: Vec<StoryImportSummary>,
    #[graphql(name = "createdCount")]
    pub created_count: i32,
    #[graphql(name = "updatedCount")]
    pub updated_count: i32,
    pub errors: Vec<String>,
}

#[derive(SimpleObject)]
pub struct StoryImportSummary {
    pub id: i32,
    pub name: String,
    #[graphql(name = "sequenceCount")]
    pub sequence_count: i32,
}
