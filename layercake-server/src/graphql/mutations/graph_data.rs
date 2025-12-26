use async_graphql::*;

use layercake_core::database::entities::graph_data;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{GraphData, UpdateGraphDataInput};
use layercake_core::services::GraphDataService;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

#[derive(Default)]
pub struct GraphDataMutation;

#[Object]
impl GraphDataMutation {
    /// Update graph_data metadata (name and/or metadata JSON)
    async fn update_graph_data(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateGraphDataInput,
    ) -> Result<GraphData> {
        let context = ctx.data::<GraphQLContext>()?;

        // Get existing graph_data
        let existing = graph_data::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("GraphData", id))?;

        // Update fields if provided
        let mut active: graph_data::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(metadata) = input.metadata {
            active.metadata = Set(Some(metadata));
        }
        active.updated_at = Set(chrono::Utc::now());

        let updated = active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data::ActiveModel::update", e))?;

        Ok(GraphData::from(updated))
    }

    /// Replay edits for a graph_data item
    async fn replay_graph_data_edits(
        &self,
        ctx: &Context<'_>,
        graph_data_id: i32,
    ) -> Result<GraphData> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = GraphDataService::new(context.db.clone());

        // Replay edits
        let _summary = service
            .replay_edits(graph_data_id)
            .await
            .map_err(StructuredError::from_core_error)?;

        // Fetch updated graph_data
        let graph_data = service
            .get_by_id(graph_data_id)
            .await
            .map_err(StructuredError::from_core_error)?
            .ok_or_else(|| StructuredError::not_found("GraphData", graph_data_id))?;

        Ok(GraphData::from(graph_data))
    }

    /// Clear all edits for a graph_data item
    async fn clear_graph_data_edits(&self, ctx: &Context<'_>, graph_data_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = GraphDataService::new(context.db.clone());

        service
            .clear_edits(graph_data_id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(true)
    }
}
