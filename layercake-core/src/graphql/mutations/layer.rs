use async_graphql::*;

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::layer::{ProjectLayer, ProjectLayerInput};

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
}
