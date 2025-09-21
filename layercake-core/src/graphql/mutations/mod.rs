use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, ActiveValue, ColumnTrait, QueryFilter};
use chrono::Utc;

use crate::database::entities::{projects, plans, nodes, edges, layers, plan_dag_nodes, plan_dag_edges};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{
    Project, Plan, Node, Edge, Layer,
    CreateProjectInput, UpdateProjectInput,
    CreatePlanInput, UpdatePlanInput,
    CreateNodeInput, CreateEdgeInput, CreateLayerInput,
    PlanDagInput, PlanDagResponse, PlanDagNodeInput, PlanDagEdgeInput,
    PlanDagNodeUpdateInput, NodeResponse, EdgeResponse, Position,
    PlanDag, PlanDagNode, PlanDagEdge, PlanDagMetadata,
};

pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new project
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        
        let mut project = projects::ActiveModel::new();
        project.name = Set(input.name);
        project.description = Set(input.description);

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
            yaml_content: Set(input.yaml_content),
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
        plan.yaml_content = Set(input.yaml_content);
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

    /// Update a complete Plan DAG
    async fn update_plan_dag(&self, ctx: &Context<'_>, project_id: i32, plan_dag: PlanDagInput) -> Result<PlanDagResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find or create the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?;

        let plan = match plan {
            Some(p) => p,
            None => {
                // Create a new plan if none exists
                let new_plan = plans::ActiveModel {
                    project_id: Set(project_id),
                    name: Set(plan_dag.metadata.name.clone().unwrap_or_else(|| "Plan DAG".to_string())),
                    yaml_content: Set("".to_string()), // Empty YAML for now
                    dependencies: Set(None),
                    status: Set("active".to_string()),
                    plan_dag_json: Set(Some(serde_json::to_string(&plan_dag.metadata)?)),
                    ..Default::default()
                };
                new_plan.insert(&context.db).await?
            }
        };

        // Clear existing Plan DAG nodes and edges
        plan_dag_nodes::Entity::delete_many()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .exec(&context.db)
            .await?;

        plan_dag_edges::Entity::delete_many()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .exec(&context.db)
            .await?;

        // Insert new Plan DAG nodes
        for node in &plan_dag.nodes {
            let node_type_str = match node.node_type {
                crate::graphql::types::PlanDagNodeType::Input => "InputNode",
                crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
                crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
                crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
                crate::graphql::types::PlanDagNodeType::Copy => "CopyNode",
                crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
            };

            let metadata_json = serde_json::to_string(&node.metadata)?;

            let dag_node = plan_dag_nodes::ActiveModel {
                id: Set(node.id.clone()),
                plan_id: Set(plan.id),
                node_type: Set(node_type_str.to_string()),
                position_x: Set(node.position.x),
                position_y: Set(node.position.y),
                metadata_json: Set(metadata_json),
                config_json: Set(node.config.clone()),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            dag_node.insert(&context.db).await?;
        }

        // Insert new Plan DAG edges
        for edge in &plan_dag.edges {
            let metadata_json = serde_json::to_string(&edge.metadata)?;

            let dag_edge = plan_dag_edges::ActiveModel {
                id: Set(edge.id.clone()),
                plan_id: Set(plan.id),
                source_node_id: Set(edge.source.clone()),
                target_node_id: Set(edge.target.clone()),
                metadata_json: Set(metadata_json),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            dag_edge.insert(&context.db).await?;
        }

        // Update plan with new metadata
        let mut plan_active: plans::ActiveModel = plan.into();
        plan_active.plan_dag_json = Set(Some(serde_json::to_string(&plan_dag.metadata)?));
        plan_active.updated_at = Set(Utc::now());
        let updated_plan = plan_active.update(&context.db).await?;

        // Return the updated Plan DAG
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(updated_plan.id))
            .all(&context.db)
            .await?;

        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(updated_plan.id))
            .all(&context.db)
            .await?;

        let nodes: Vec<PlanDagNode> = dag_nodes.into_iter().map(PlanDagNode::from).collect();
        let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();

        Ok(PlanDagResponse {
            success: true,
            errors: vec![],
            plan_dag: Some(PlanDag {
                version: plan_dag.version,
                nodes,
                edges,
                metadata: plan_dag.metadata,
            }),
        })
    }

    /// Add a single Plan DAG node
    async fn add_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node: PlanDagNodeInput) -> Result<NodeResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        let node_type_str = match node.node_type {
            crate::graphql::types::PlanDagNodeType::Input => "InputNode",
            crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
            crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
            crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
            crate::graphql::types::PlanDagNodeType::Copy => "CopyNode",
            crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
        };

        let metadata_json = serde_json::to_string(&node.metadata)?;

        let dag_node = plan_dag_nodes::ActiveModel {
            id: Set(node.id.clone()),
            plan_id: Set(plan.id),
            node_type: Set(node_type_str.to_string()),
            position_x: Set(node.position.x),
            position_y: Set(node.position.y),
            metadata_json: Set(metadata_json),
            config_json: Set(node.config.clone()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_node = dag_node.insert(&context.db).await?;

        Ok(NodeResponse {
            success: true,
            errors: vec![],
            node: Some(PlanDagNode::from(inserted_node)),
        })
    }

    /// Update a Plan DAG node
    async fn update_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node_id: String, updates: PlanDagNodeUpdateInput) -> Result<NodeResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Find the node
        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId.eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id))
            )
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Node not found"))?;

        let mut node_active: plan_dag_nodes::ActiveModel = node.into();

        // Update position if provided
        if let Some(position) = updates.position {
            node_active.position_x = Set(position.x);
            node_active.position_y = Set(position.y);
        }

        // Update metadata if provided
        if let Some(metadata) = updates.metadata {
            node_active.metadata_json = Set(serde_json::to_string(&metadata)?);
        }

        // Update config if provided
        if let Some(config) = updates.config {
            node_active.config_json = Set(config);
        }

        node_active.updated_at = Set(Utc::now());
        let updated_node = node_active.update(&context.db).await?;

        Ok(NodeResponse {
            success: true,
            errors: vec![],
            node: Some(PlanDagNode::from(updated_node)),
        })
    }

    /// Delete a Plan DAG node
    async fn delete_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node_id: String) -> Result<NodeResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Delete edges connected to this node first
        plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId.eq(plan.id)
                    .and(
                        plan_dag_edges::Column::SourceNodeId.eq(&node_id)
                            .or(plan_dag_edges::Column::TargetNodeId.eq(&node_id))
                    )
            )
            .exec(&context.db)
            .await?;

        // Delete the node
        let result = plan_dag_nodes::Entity::delete_many()
            .filter(
                plan_dag_nodes::Column::PlanId.eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id))
            )
            .exec(&context.db)
            .await?;

        if result.rows_affected > 0 {
            Ok(NodeResponse {
                success: true,
                errors: vec![],
                node: None,
            })
        } else {
            Ok(NodeResponse {
                success: false,
                errors: vec!["Node not found".to_string()],
                node: None,
            })
        }
    }

    /// Add a Plan DAG edge
    async fn add_plan_dag_edge(&self, ctx: &Context<'_>, project_id: i32, edge: PlanDagEdgeInput) -> Result<EdgeResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        let metadata_json = serde_json::to_string(&edge.metadata)?;

        let dag_edge = plan_dag_edges::ActiveModel {
            id: Set(edge.id.clone()),
            plan_id: Set(plan.id),
            source_node_id: Set(edge.source.clone()),
            target_node_id: Set(edge.target.clone()),
            metadata_json: Set(metadata_json),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_edge = dag_edge.insert(&context.db).await?;

        Ok(EdgeResponse {
            success: true,
            errors: vec![],
            edge: Some(PlanDagEdge::from(inserted_edge)),
        })
    }

    /// Delete a Plan DAG edge
    async fn delete_plan_dag_edge(&self, ctx: &Context<'_>, project_id: i32, edge_id: String) -> Result<EdgeResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Delete the edge
        let result = plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId.eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id))
            )
            .exec(&context.db)
            .await?;

        if result.rows_affected > 0 {
            Ok(EdgeResponse {
                success: true,
                errors: vec![],
                edge: None,
            })
        } else {
            Ok(EdgeResponse {
                success: false,
                errors: vec!["Edge not found".to_string()],
                edge: None,
            })
        }
    }

    /// Move a Plan DAG node (update position)
    async fn move_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node_id: String, position: Position) -> Result<NodeResponse> {
        let updates = PlanDagNodeUpdateInput {
            position: Some(position),
            metadata: None,
            config: None,
        };

        self.update_plan_dag_node(ctx, project_id, node_id, updates).await
    }
}

#[derive(SimpleObject)]
pub struct PlanExecutionResult {
    pub success: bool,
    pub message: String,
    pub output_files: Vec<String>,
}