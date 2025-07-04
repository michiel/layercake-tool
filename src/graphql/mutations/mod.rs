use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::database::entities::{projects, plans, nodes, edges, layers};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{
    Project, Plan, Node, Edge, Layer,
    CreateProjectInput, UpdateProjectInput,
    CreatePlanInput, UpdatePlanInput,
    CreateNodeInput, CreateEdgeInput, CreateLayerInput,
};

pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new project
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let project = projects::ActiveModel {
            name: Set(input.name),
            description: Set(input.description),
            ..Default::default()
        };

        let project = project.insert(&context.db).await?;
        Ok(Project::from(project))
    }

    /// Update an existing project
    async fn update_project(&self, ctx: &Context<'_>, id: i32, input: UpdateProjectInput) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        let mut project: projects::ActiveModel = project.into();
        project.name = Set(input.name);
        project.description = Set(input.description);

        let project = project.update(&context.db).await?;
        Ok(Project::from(project))
    }

    /// Delete a project
    async fn delete_project(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        projects::Entity::delete_by_id(project.id)
            .exec(&context.db)
            .await?;

        Ok(true)
    }

    /// Create a new plan
    async fn create_plan(&self, ctx: &Context<'_>, input: CreatePlanInput) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let dependencies_json = input.dependencies
            .map(|deps| serde_json::to_string(&deps))
            .transpose()?;
        
        let plan = plans::ActiveModel {
            project_id: Set(input.project_id),
            name: Set(input.name),
            plan_content: Set(input.plan_content),
            plan_format: Set(input.plan_format.unwrap_or_else(|| "json".to_string())),
            plan_schema_version: Set("1.0.0".to_string()),
            dependencies: Set(dependencies_json),
            status: Set("pending".to_string()),
            ..Default::default()
        };

        let plan = plan.insert(&context.db).await?;
        Ok(Plan::from(plan))
    }

    /// Update an existing plan
    async fn update_plan(&self, ctx: &Context<'_>, id: i32, input: UpdatePlanInput) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found"))?;

        let dependencies_json = input.dependencies
            .map(|deps| serde_json::to_string(&deps))
            .transpose()?;

        let mut plan: plans::ActiveModel = plan.into();
        plan.name = Set(input.name);
        plan.plan_content = Set(input.plan_content);
        plan.plan_format = Set(input.plan_format.unwrap_or_else(|| "json".to_string()));
        plan.dependencies = Set(dependencies_json);

        let plan = plan.update(&context.db).await?;
        Ok(Plan::from(plan))
    }

    /// Delete a plan
    async fn delete_plan(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
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

    /// Execute a plan
    async fn execute_plan(&self, ctx: &Context<'_>, id: i32) -> Result<PlanExecutionResult> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let _plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found"))?;

        // TODO: Implement plan execution using existing export service
        // For now, return a success result
        Ok(PlanExecutionResult {
            success: true,
            message: "Plan execution not yet implemented".to_string(),
            output_files: vec![],
        })
    }

    /// Create multiple nodes
    async fn create_nodes(&self, ctx: &Context<'_>, project_id: i32, nodes: Vec<CreateNodeInput>) -> Result<Vec<Node>> {
        let context = ctx.data::<GraphQLContext>()?;
        let mut results = Vec::new();

        for node_input in nodes {
            let properties_json = node_input.properties
                .map(|props| serde_json::to_string(&props))
                .transpose()?;
            
            let node = nodes::ActiveModel {
                project_id: Set(project_id),
                node_id: Set(node_input.node_id),
                label: Set(node_input.label),
                layer_id: Set(node_input.layer_id),
                properties: Set(properties_json),
                ..Default::default()
            };

            let node = node.insert(&context.db).await?;
            results.push(Node::from(node));
        }

        Ok(results)
    }

    /// Create multiple edges
    async fn create_edges(&self, ctx: &Context<'_>, project_id: i32, edges: Vec<CreateEdgeInput>) -> Result<Vec<Edge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let mut results = Vec::new();

        for edge_input in edges {
            let properties_json = edge_input.properties
                .map(|props| serde_json::to_string(&props))
                .transpose()?;
            
            let edge = edges::ActiveModel {
                project_id: Set(project_id),
                source_node_id: Set(edge_input.source_node_id),
                target_node_id: Set(edge_input.target_node_id),
                properties: Set(properties_json),
                ..Default::default()
            };

            let edge = edge.insert(&context.db).await?;
            results.push(Edge::from(edge));
        }

        Ok(results)
    }

    /// Create multiple layers
    async fn create_layers(&self, ctx: &Context<'_>, project_id: i32, layers: Vec<CreateLayerInput>) -> Result<Vec<Layer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let mut results = Vec::new();

        for layer_input in layers {
            let properties_json = layer_input.properties
                .map(|props| serde_json::to_string(&props))
                .transpose()?;
            
            let layer = layers::ActiveModel {
                project_id: Set(project_id),
                layer_id: Set(layer_input.layer_id),
                name: Set(layer_input.name),
                color: Set(layer_input.color),
                properties: Set(properties_json),
                ..Default::default()
            };

            let layer = layer.insert(&context.db).await?;
            results.push(Layer::from(layer));
        }

        Ok(results)
    }
}

#[derive(SimpleObject)]
pub struct PlanExecutionResult {
    pub success: bool,
    pub message: String,
    pub output_files: Vec<String>,
}