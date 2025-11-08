use async_graphql::*;

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::graph_edit::{CreateGraphEditInput, GraphEdit, EditResult, ReplaySummary};
use crate::services::graph_edit_service::GraphEditService;

#[derive(Default)]
pub struct GraphEditMutation;

#[Object]
impl GraphEditMutation {
/// Create a new graph edit
async fn create_graph_edit(
    &self,
    ctx: &Context<'_>,
    input: CreateGraphEditInput,
) -> Result<GraphEdit> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = GraphEditService::new(context.db.clone());

    let edit = service
        .create_edit(
            input.graph_id,
            input.target_type,
            input.target_id,
            input.operation,
            input.field_name,
            input.old_value,
            input.new_value,
            input.created_by,
            false, // External edit - not yet applied, will be replayed
        )
        .await
        .map_err(|e| StructuredError::service("GraphEditService::create_edit", e))?;

    Ok(GraphEdit::from(edit))
}

/// Replay all unapplied edits for a graph
async fn replay_graph_edits(&self, ctx: &Context<'_>, graph_id: i32) -> Result<ReplaySummary> {
    let context = ctx.data::<GraphQLContext>()?;

    let summary = context
        .app
        .replay_graph_edits(graph_id)
        .await
        .map_err(|e| StructuredError::service("AppContext::replay_graph_edits", e))?;

    Ok(ReplaySummary {
        total: summary.total as i32,
        applied: summary.applied as i32,
        skipped: summary.skipped as i32,
        failed: summary.failed as i32,
        details: summary
            .details
            .into_iter()
            .map(|d| EditResult {
                sequence_number: d.sequence_number,
                target_type: d.target_type,
                target_id: d.target_id,
                operation: d.operation,
                result: d.result,
                message: d.message,
            })
            .collect(),
    })
}

/// Clear all edits for a graph
async fn clear_graph_edits(&self, ctx: &Context<'_>, graph_id: i32) -> Result<bool> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = GraphEditService::new(context.db.clone());

    service
        .clear_graph_edits(graph_id)
        .await
        .map_err(|e| StructuredError::service("GraphEditService::clear_graph_edits", e))?;

    Ok(true)
}
}
