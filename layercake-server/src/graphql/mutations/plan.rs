use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::helpers::PlanExecutionResult;
use layercake_core::database::entities::{plan_dag_edges, plan_dag_nodes};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan::{CreatePlanInput, Plan, UpdatePlanInput};
use layercake_core::services::plan_service::{PlanCreateRequest, PlanUpdateRequest};

#[derive(Default)]
pub struct PlanMutation;

#[Object]
impl PlanMutation {
    /// Create a new plan
    async fn create_plan(&self, ctx: &Context<'_>, input: CreatePlanInput) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let request = PlanCreateRequest {
            project_id: input.project_id,
            name: input.name,
            description: input.description,
            tags: input.tags,
            yaml_content: input.yaml_content,
            dependencies: input.dependencies,
            status: None,
        };

        let summary = context
            .app
            .create_plan(&actor, request)
            .await
            .map_err(Error::from)?;
        Ok(Plan::from(summary))
    }

    /// Update an existing plan
    async fn update_plan(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdatePlanInput,
    ) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let UpdatePlanInput {
            name,
            description,
            tags,
            yaml_content,
            dependencies,
        } = input;
        let dependencies_is_set = dependencies.is_some();

        let update = PlanUpdateRequest {
            name,
            description,
            tags,
            yaml_content,
            dependencies,
            dependencies_is_set,
            status: None,
        };

        let summary = context
            .app
            .update_plan(&actor, id, update)
            .await
            .map_err(Error::from)?;
        Ok(Plan::from(summary))
    }

    /// Delete a plan
    async fn delete_plan(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        context
            .app
            .delete_plan(&actor, id)
            .await
            .map_err(Error::from)?;

        Ok(true)
    }

    /// Execute a plan (executes all nodes in the DAG in optimal topological order)
    async fn execute_plan(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
        #[graphql(name = "planId")] plan_id: Option<i32>,
    ) -> Result<PlanExecutionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        let plan_service = context.app.plan_service().clone();

        let plan = if let Some(plan_id) = plan_id {
            let plan = plan_service
                .get_plan(plan_id)
                .await
                .map_err(Error::from)?
                .ok_or_else(|| StructuredError::not_found("Plan", plan_id))?;

            if plan.project_id != project_id {
                return Err(StructuredError::validation(
                    "planId",
                    format!("Plan {} does not belong to project {}", plan_id, project_id),
                ));
            }

            plan
        } else {
            plan_service
                .get_default_plan(project_id)
                .await
                .map_err(Error::from)?
                .ok_or_else(|| StructuredError::not_found("Plan for project", project_id))?
        };

        // Get all nodes in the plan
        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_nodes::Entity::find (PlanId)", e))?;

        if nodes.is_empty() {
            return Ok(PlanExecutionResult {
                success: true,
                message: "No nodes to execute in this plan".to_string(),
                output_files: vec![],
            });
        }

        // Get all edges in the plan
        let edges_models = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_edges::Entity::find (PlanId)", e))?;

        // Convert edges to (source, target) tuples
        let edges: Vec<(String, String)> = edges_models
            .iter()
            .map(|e| (e.source_node_id.clone(), e.target_node_id.clone()))
            .collect();

        // Create executor and execute the entire DAG
        let executor = layercake_core::pipeline::DagExecutor::new(context.db.clone());

        executor
            .execute_dag(project_id, plan.id, &nodes, &edges)
            .await
            .map_err(|e| StructuredError::service("DagExecutor::execute_dag", e))?;

        Ok(PlanExecutionResult {
            success: true,
            message: format!("Executed {} nodes in topological order", nodes.len()),
            output_files: vec![],
        })
    }

    /// Duplicate a plan with all DAG nodes and edges
    async fn duplicate_plan(&self, ctx: &Context<'_>, id: i32, name: String) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let summary = context
            .app
            .duplicate_plan(&actor, id, name)
            .await
            .map_err(Error::from)?;

        Ok(Plan::from(summary))
    }
}
