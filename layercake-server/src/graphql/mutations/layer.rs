use async_graphql::*;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};

use layercake_core::database::entities::{layer_aliases, project_layers};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::layer::{LayerAlias, ProjectLayer, ProjectLayerInput};

#[derive(Default)]
pub struct LayerMutation;

#[Object]
impl LayerMutation {
    /// Create or update a project-level layer entry
    #[graphql(name = "upsertProjectLayer")]
    async fn upsert_project_layer(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        input: ProjectLayerInput,
    ) -> Result<ProjectLayer> {
        let context = ctx.data::<GraphQLContext>()?;
        let model = context
            .app
            .graph_service()
            .upsert_project_layer(
                project_id,
                input.layer_id.clone(),
                input.name.clone(),
                input
                    .background_color
                    .unwrap_or_else(|| "FFFFFF".to_string()),
                input.text_color.unwrap_or_else(|| "000000".to_string()),
                input.border_color.unwrap_or_else(|| "000000".to_string()),
                input.alias.clone(),
                input.source_dataset_id,
                input.enabled.unwrap_or(true),
            )
            .await
            .map_err(|e| StructuredError::service("GraphService::upsert_project_layer", e))?;

        Ok(ProjectLayer::from(model))
    }

    /// Delete a project layer entry (optionally scoped to a source dataset row)
    #[graphql(name = "deleteProjectLayer")]
    async fn delete_project_layer(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        layer_id: String,
        source_dataset_id: Option<i32>,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let rows = context
            .app
            .graph_service()
            .delete_project_layer(project_id, layer_id, source_dataset_id)
            .await
            .map_err(|e| StructuredError::service("GraphService::delete_project_layer", e))?;

        Ok(rows > 0)
    }

    /// Enable or disable layers originating from a dataset
    #[graphql(name = "setLayerDatasetEnabled")]
    async fn set_layer_dataset_enabled(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        data_set_id: i32,
        enabled: bool,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        context
            .app
            .graph_service()
            .set_layer_dataset_enabled(project_id, data_set_id, enabled)
            .await
            .map_err(|e| StructuredError::service("GraphService::set_layer_dataset_enabled", e))?;

        Ok(true)
    }

    /// Reset all project layer configuration (manual layers, aliases, dataset entries)
    #[graphql(name = "resetProjectLayers")]
    async fn reset_project_layers(&self, ctx: &Context<'_>, project_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        context
            .app
            .graph_service()
            .reset_project_layers(project_id)
            .await
            .map_err(|e| StructuredError::service("GraphService::reset_project_layers", e))?;

        Ok(true)
    }

    /// Create an alias from a missing layer to an existing project layer
    #[graphql(name = "createLayerAlias")]
    async fn create_layer_alias(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        alias_layer_id: String,
        target_layer_id: i32,
    ) -> Result<LayerAlias> {
        let context = ctx.data::<GraphQLContext>()?;

        // Validate that target layer exists and belongs to project
        let _target_layer = project_layers::Entity::find_by_id(target_layer_id)
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| {
                StructuredError::validation(
                    "targetLayerId",
                    format!(
                        "Target layer {} not found in project {}",
                        target_layer_id, project_id
                    ),
                )
            })?;

        // Create alias
        let alias = layer_aliases::ActiveModel {
            id: NotSet,
            project_id: Set(project_id),
            alias_layer_id: Set(alias_layer_id.clone()),
            target_layer_id: Set(target_layer_id),
            created_at: Set(chrono::Utc::now()),
        };

        let result = alias
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("layer_aliases::insert", e))?;

        Ok(LayerAlias::from(result))
    }

    /// Remove a layer alias
    #[graphql(name = "removeLayerAlias")]
    async fn remove_layer_alias(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        alias_layer_id: String,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let result = layer_aliases::Entity::delete_many()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .filter(layer_aliases::Column::AliasLayerId.eq(alias_layer_id))
            .exec(&context.db)
            .await?;

        Ok(result.rows_affected > 0)
    }

    /// Remove all aliases for a target layer
    #[graphql(name = "removeLayerAliases")]
    async fn remove_layer_aliases(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        target_layer_id: i32,
    ) -> Result<i32> {
        let context = ctx.data::<GraphQLContext>()?;

        let result = layer_aliases::Entity::delete_many()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .filter(layer_aliases::Column::TargetLayerId.eq(target_layer_id))
            .exec(&context.db)
            .await?;

        Ok(result.rows_affected as i32)
    }
}
