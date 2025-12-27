use async_graphql::*;
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
        let actor = context.actor_for_request(ctx).await;
        let model = context
            .app
            .upsert_project_layer(
                &actor,
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
            .map_err(Error::from)?;

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
        let actor = context.actor_for_request(ctx).await;
        let rows = context
            .app
            .delete_project_layer(&actor, project_id, layer_id, source_dataset_id)
            .await
            .map_err(Error::from)?;

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
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .set_layer_dataset_enabled(&actor, project_id, data_set_id, enabled)
            .await
            .map_err(Error::from)?;

        Ok(true)
    }

    /// Reset all project layer configuration (manual layers, aliases, dataset entries)
    #[graphql(name = "resetProjectLayers")]
    async fn reset_project_layers(&self, ctx: &Context<'_>, project_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .reset_project_layers(&actor, project_id)
            .await
            .map_err(Error::from)?;

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
        let actor = context.actor_for_request(ctx).await;

        let result = context
            .app
            .create_layer_alias(&actor, project_id, alias_layer_id, target_layer_id)
            .await
            .map_err(Error::from)?;

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
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .remove_layer_alias(&actor, project_id, alias_layer_id)
            .await
            .map_err(Error::from)
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
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .remove_layer_aliases(&actor, project_id, target_layer_id)
            .await
            .map_err(Error::from)
    }
}
