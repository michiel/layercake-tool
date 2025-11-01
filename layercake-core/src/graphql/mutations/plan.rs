use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::app_context::{PlanCreateRequest, PlanUpdateRequest};
use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::plan::{CreatePlanInput, Plan, UpdatePlanInput};
use crate::graphql::types::PlanDagNode;
use crate::graphql::errors::StructuredError;

#[derive(SimpleObject)]
pub struct PlanExecutionResult {
    pub success: bool,
    pub message: String,
    pub output_files: Vec<String>,
}

pub struct PlanMutations;

#[Object]
impl PlanMutations {
    /// Create a new plan
    async fn create(&self, ctx: &Context<'_>, input: CreatePlanInput) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;

        let CreatePlanInput {
            project_id,
            name,
            yaml_content,
            dependencies,
        } = input;

        let request = PlanCreateRequest {
            project_id,
            name,
            yaml_content,
            dependencies,
            status: None,
        };

        let plan = context
            .app
            .create_plan(request)
            .await
            .map_err(|e| StructuredError::service("AppContext::create_plan", e))?;
        Ok(Plan::from(plan))
    }

    /// Update an existing plan
    async fn update(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdatePlanInput,
    ) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;

        let UpdatePlanInput {
            name,
            yaml_content,
            dependencies,
        } = input;

        let update = PlanUpdateRequest {
            name: Some(name),
            yaml_content: Some(yaml_content),
            dependencies,
            dependencies_is_set: true,
            status: None,
        };

        let plan = context
            .app
            .update_plan(id, update)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_plan", e))?;
        Ok(Plan::from(plan))
    }

    /// Delete a plan
    async fn delete(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        context
            .app
            .delete_plan(id)
            .await
            .map_err(|e| StructuredError::service("AppContext::delete_plan", e))?;

        Ok(true)
    }

    /// Execute a plan (executes all nodes in the DAG in optimal topological order)
    async fn execute(&self, ctx: &Context<'_>, id: i32) -> Result<PlanExecutionResult> {
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
            .map_err(|e| StructuredError::database("plan_dag_nodes::Entity::find", e))?;

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
            .map_err(|e| StructuredError::database("plan_dag_edges::Entity::find", e))?;

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
