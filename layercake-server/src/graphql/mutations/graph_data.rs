use async_graphql::*;

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{GraphData, UpdateGraphDataInput};

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
        let actor = context.actor_for_request(ctx).await;

        let updated = context
            .app
            .update_graph_data_metadata(&actor, id, input.name, input.metadata)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(GraphData::from(updated))
    }

    /// Delete computed graphs whose originating plan DAG node no longer exists.
    /// Returns the ids of the graphs that were pruned. Dataset/manual graphs are
    /// never affected.
    #[graphql(name = "pruneOrphanedGraphs")]
    async fn prune_orphaned_graphs(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
    ) -> Result<Vec<i32>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        context
            .app
            .prune_orphaned_graphs(&actor, project_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)
    }

    /// Replay edits for a graph_data item
    async fn replay_graph_data_edits(
        &self,
        ctx: &Context<'_>,
        graph_data_id: i32,
    ) -> Result<GraphData> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let graph_data = context
            .app
            .replay_graph_data_edits(&actor, graph_data_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(GraphData::from(graph_data))
    }

    /// Clear all edits for a graph_data item
    async fn clear_graph_data_edits(&self, ctx: &Context<'_>, graph_data_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        context
            .app
            .clear_graph_data_edits(&actor, graph_data_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(true)
    }
}
