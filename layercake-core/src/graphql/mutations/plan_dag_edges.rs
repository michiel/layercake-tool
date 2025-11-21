use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait};

use crate::app_context::{PlanDagEdgeRequest, PlanDagEdgeUpdateRequest};
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
            .create_plan_dag_edge(project_id, plan_id, request)
            .await
            .map_err(|e| StructuredError::service("AppContext::create_plan_dag_edge", e))?;

        Ok(Some(created))
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

        let deleted = context
            .app
            .delete_plan_dag_edge(project_id, plan_id, edge_id)
            .await
            .map_err(|e| StructuredError::service("AppContext::delete_plan_dag_edge", e))?;

        Ok(Some(deleted))
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
            .update_plan_dag_edge(project_id, plan_id, edge_id, request)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_plan_dag_edge", e))?;

        Ok(Some(updated))
    }
}
