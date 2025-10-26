use async_graphql::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans, projects, ExecutionState};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::plan::{CreatePlanInput, Plan, UpdatePlanInput};
use crate::graphql::types::PlanDagNode;

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

        let dependencies_json = input
            .dependencies
            .map(|deps| serde_json::to_string(&deps))
            .transpose()?;

        let plan = plans::ActiveModel {
            project_id: Set(input.project_id),
            name: Set(input.name),
            yaml_content: Set(input.yaml_content),
            dependencies: Set(dependencies_json),
            status: Set("pending".to_string()),
            ..Default::default()
        };

        let plan = plan.insert(&context.db).await?;
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

        let plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found"))?;

        let dependencies_json = input
            .dependencies
            .map(|deps| serde_json::to_string(&deps))
            .transpose()?;

        let mut plan: plans::ActiveModel = plan.into();
        plan.name = Set(input.name);
        plan.yaml_content = Set(input.yaml_content);
        plan.dependencies = Set(dependencies_json);

        let plan = plan.update(&context.db).await?;
        Ok(Plan::from(plan))
    }

    /// Delete a plan
    async fn delete(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found"))?;

        plans::Entity::delete_by_id(plan.id)
            .exec(&context.db)
            .await?;

        Ok(true)
    }

    /// Execute a plan (executes all nodes in the DAG in optimal topological order)
    async fn execute(&self, ctx: &Context<'_>, id: i32) -> Result<PlanExecutionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan (id is actually the project_id)
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Get all nodes in the plan
        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;

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
            .await?;

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
            .map_err(|e| Error::new(format!("Failed to execute plan: {}", e)))?;

        Ok(PlanExecutionResult {
            success: true,
            message: format!("Executed {} nodes in topological order", nodes.len()),
            output_files: vec![],
        })
    }
}
