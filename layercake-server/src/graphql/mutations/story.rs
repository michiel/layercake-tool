use async_graphql::*;
use base64::Engine;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use layercake_core::database::entities::{sequences, stories};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    CreateStoryInput, Story, StoryExport, StoryExportFormat, StoryImportFormat,
    StoryImportResult as GqlStoryImportResult, StoryImportSummary as GqlStoryImportSummary,
    UpdateStoryInput,
};

#[derive(Default)]
pub struct StoryMutation;

#[Object]
impl StoryMutation {
    /// Create a new story
    async fn create_story(&self, ctx: &Context<'_>, input: CreateStoryInput) -> Result<Story> {
        let context = ctx.data::<GraphQLContext>()?;
        let now = Utc::now();

        let tags_json = serde_json::to_string(&input.tags.unwrap_or_default()).map_err(|e| {
            StructuredError::bad_request(format!("Failed to serialize tags: {}", e))
        })?;

        let enabled_dataset_ids_json =
            serde_json::to_string(&input.enabled_dataset_ids.unwrap_or_default()).map_err(|e| {
                StructuredError::bad_request(format!(
                    "Failed to serialize enabled_dataset_ids: {}",
                    e
                ))
            })?;

        let layer_config_json = serde_json::to_string(&input.layer_config.unwrap_or_default())
            .map_err(|e| {
                StructuredError::bad_request(format!("Failed to serialize layer_config: {}", e))
            })?;

        let story = stories::ActiveModel {
            project_id: Set(input.project_id),
            name: Set(input.name),
            description: Set(input.description),
            tags: Set(tags_json),
            enabled_dataset_ids: Set(enabled_dataset_ids_json),
            layer_config: Set(layer_config_json),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let model = story
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("stories::ActiveModel::insert", e))?;

        Ok(Story::from(model))
    }

    /// Update an existing story
    async fn update_story(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateStoryInput,
    ) -> Result<Story> {
        let context = ctx.data::<GraphQLContext>()?;

        let existing = stories::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("stories::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Story", id))?;

        let mut story: stories::ActiveModel = existing.into();
        story.updated_at = Set(Utc::now());

        if let Some(name) = input.name {
            story.name = Set(name);
        }

        if let Some(description) = input.description {
            story.description = Set(Some(description));
        }

        if let Some(tags) = input.tags {
            let tags_json = serde_json::to_string(&tags).map_err(|e| {
                StructuredError::bad_request(format!("Failed to serialize tags: {}", e))
            })?;
            story.tags = Set(tags_json);
        }

        if let Some(enabled_dataset_ids) = input.enabled_dataset_ids {
            let json = serde_json::to_string(&enabled_dataset_ids).map_err(|e| {
                StructuredError::bad_request(format!(
                    "Failed to serialize enabled_dataset_ids: {}",
                    e
                ))
            })?;
            story.enabled_dataset_ids = Set(json);
        }

        if let Some(layer_config) = input.layer_config {
            let json = serde_json::to_string(&layer_config).map_err(|e| {
                StructuredError::bad_request(format!("Failed to serialize layer_config: {}", e))
            })?;
            story.layer_config = Set(json);
        }

        let model = story
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("stories::ActiveModel::update", e))?;

        Ok(Story::from(model))
    }

    /// Delete a story (cascades to delete sequences)
    async fn delete_story(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        // First delete all sequences for this story
        sequences::Entity::delete_many()
            .filter(sequences::Column::StoryId.eq(id))
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("sequences::Entity::delete_many", e))?;

        // Then delete the story
        let result = stories::Entity::delete_by_id(id)
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("stories::Entity::delete_by_id", e))?;

        Ok(result.rows_affected > 0)
    }

    /// Export a story to CSV or JSON format
    async fn export_story(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "storyId")] story_id: i32,
        format: StoryExportFormat,
    ) -> Result<StoryExport> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let export_result = match format {
            StoryExportFormat::Csv => context
                .app
                .export_story_csv(&actor, story_id)
                .await
                .map_err(Error::from)?,
            StoryExportFormat::Json => context
                .app
                .export_story_json(&actor, story_id)
                .await
                .map_err(Error::from)?,
        };

        let content_base64 =
            base64::engine::general_purpose::STANDARD.encode(&export_result.content);

        Ok(StoryExport {
            filename: export_result.filename,
            content: content_base64,
            mime_type: export_result.mime_type,
        })
    }

    /// Import stories from CSV or JSON format
    async fn import_story(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
        format: StoryImportFormat,
        content: String,
    ) -> Result<GqlStoryImportResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let import_result = match format {
            StoryImportFormat::Csv => context
                .app
                .import_story_csv(&actor, project_id, &content)
                .await
                .map_err(Error::from)?,
            StoryImportFormat::Json => context
                .app
                .import_story_json(&actor, project_id, &content)
                .await
                .map_err(Error::from)?,
        };

        Ok(GqlStoryImportResult {
            imported_stories: import_result
                .imported_stories
                .into_iter()
                .map(|s| GqlStoryImportSummary {
                    id: s.id,
                    name: s.name,
                    sequence_count: s.sequence_count,
                })
                .collect(),
            created_count: import_result.created_count,
            updated_count: import_result.updated_count,
            errors: import_result.errors,
        })
    }
}
