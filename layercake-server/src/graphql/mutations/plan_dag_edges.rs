use async_graphql::*;

use layercake_core::app_context::{PlanDagEdgeRequest, PlanDagEdgeUpdateRequest};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan_dag::{PlanDagEdge, PlanDagEdgeInput, PlanDagEdgeUpdateInput};

#[derive(Default)]
pub struct PlanDagEdgesMutation;

#[Object]
impl PlanDagEdgesMutation {
    /// Add a Plan DAG edge
    async fn add_plan_dag_edge(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        edge: PlanDagEdgeInput,
    ) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let PlanDagEdgeInput {
            source,
            target,
            metadata,
            ..
        } = edge;

        let metadata_value = serde_json::to_value(metadata)
            .map_err(|e| StructuredError::bad_request(format!("Invalid edge metadata: {}", e)))?;

        let request = PlanDagEdgeRequest {
            source,
            target,
            metadata: metadata_value,
        };

        let created = context
            .app
            .create_plan_dag_edge(&actor, project_id, plan_id, request)
            .await
            .map_err(Error::from)?;

        Ok(Some(PlanDagEdge::from(created)))
    }

    /// Delete a Plan DAG edge
    async fn delete_plan_dag_edge(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
    ) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let deleted = context
            .app
            .delete_plan_dag_edge(&actor, project_id, plan_id, edge_id)
            .await
            .map_err(Error::from)?;

        Ok(Some(PlanDagEdge::from(deleted)))
    }

    /// Update a Plan DAG edge
    async fn update_plan_dag_edge(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
        updates: PlanDagEdgeUpdateInput,
    ) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let metadata_value = if let Some(metadata) = updates.metadata {
            Some(serde_json::to_value(metadata).map_err(|e| {
                StructuredError::bad_request(format!("Invalid edge metadata: {}", e))
            })?)
        } else {
            None
        };

        let request = PlanDagEdgeUpdateRequest {
            metadata: metadata_value,
        };

        let updated = context
            .app
            .update_plan_dag_edge(&actor, project_id, plan_id, edge_id, request)
            .await
            .map_err(Error::from)?;

        Ok(Some(PlanDagEdge::from(updated)))
    }
}
