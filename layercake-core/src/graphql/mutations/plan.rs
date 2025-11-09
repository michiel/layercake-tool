use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::helpers::PlanExecutionResult;
use crate::app_context::{PlanCreateRequest, PlanUpdateRequest};
use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan::{CreatePlanInput, Plan, UpdatePlanInput};

#[derive(Default)]
pub struct PlanMutation;

#[Object]
impl PlanMutation {
    /// Create a new plan
    async fn create_plan(&self, ctx: &Context<'_>, input: CreatePlanInput) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;

        let request = PlanCreateRequest {
            project_id: input.project_id,
            name: input.name,
            yaml_content: input.yaml_content,
            dependencies: input.dependencies,
            status: None,
        };

        let summary = context
            .app
            .create_plan(request)
            .await
            .map_err(|e| StructuredError::service("AppContext::create_plan", e))?;
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

        let update = PlanUpdateRequest {
            name: Some(input.name),
            yaml_content: Some(input.yaml_content),
            dependencies: input.dependencies,
            dependencies_is_set: true,
            status: None,
        };

        let summary = context
            .app
            .update_plan(id, update)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_plan", e))?;
        Ok(Plan::from(summary))
    }

    /// Delete a plan
    async fn delete_plan(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Plan", id))?;

        plans::Entity::delete_by_id(plan.id)
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::delete_by_id", e))?;

        Ok(true)
    }

    /// Execute a plan (executes all nodes in the DAG in optimal topological order)
    async fn execute_plan(&self, ctx: &Context<'_>, id: i32) -> Result<PlanExecutionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan (id is actually the project_id)
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::find (ProjectId)", e))?
            .ok_or_else(|| StructuredError::not_found("Plan for project", id))?;

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
        let executor = crate::pipeline::DagExecutor::new(context.db.clone());

        executor
            .execute_dag(id, plan.id, &nodes, &edges)
            .await
            .map_err(|e| StructuredError::service("DagExecutor::execute_dag", e))?;

        Ok(PlanExecutionResult {
            success: true,
            message: format!("Executed {} nodes in topological order", nodes.len()),
            output_files: vec![],
        })
    }
}
