pub mod plan_dag_delta;

use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, ColumnTrait, QueryFilter};
use chrono::Utc;

use crate::database::entities::{projects, plans, nodes, edges, layers, plan_dag_nodes, plan_dag_edges, users, user_sessions, project_collaborators};
use crate::graphql::context::GraphQLContext;
use crate::services::auth_service::AuthService;
use crate::services::data_source_service::DataSourceService;
use crate::pipeline::DagExecutor;

use crate::graphql::types::{
    Project, Plan, Node, Edge, Layer,
    CreateProjectInput, UpdateProjectInput,
    CreatePlanInput, UpdatePlanInput,
    CreateNodeInput, CreateEdgeInput, CreateLayerInput,
    PlanDagInput, PlanDagNodeInput, PlanDagEdgeInput,
    PlanDagNodeUpdateInput, Position,
    PlanDag, PlanDagNode, PlanDagEdge,
    User, ProjectCollaborator,
    RegisterUserInput, LoginInput, UpdateUserInput, LoginResponse, RegisterResponse,
    InviteCollaboratorInput, UpdateCollaboratorRoleInput,
    DataSource, CreateDataSourceInput, UpdateDataSourceInput,
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
    ///
    /// **DEPRECATED**: This bulk replace operation conflicts with delta-based updates.
    /// Use individual node/edge mutations instead for better real-time collaboration.
    /// See PLAN.md Phase 2 for migration strategy.
    async fn update_plan_dag(&self, ctx: &Context<'_>, project_id: i32, plan_dag: PlanDagInput) -> Result<Option<PlanDag>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        // Clear existing Plan DAG nodes and edges for this project
        plan_dag_nodes::Entity::delete_many()
            .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
            .exec(&context.db)
            .await?;

        plan_dag_edges::Entity::delete_many()
            .filter(plan_dag_edges::Column::PlanId.eq(project_id))
            .exec(&context.db)
            .await?;

        // Insert new Plan DAG nodes
        for node in &plan_dag.nodes {
            let node_type_str = match node.node_type {
                crate::graphql::types::PlanDagNodeType::DataSource => "DataSourceNode",
                crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
                crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
                crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
                crate::graphql::types::PlanDagNodeType::Copy => "CopyNode",
                crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
            };

            let metadata_json = serde_json::to_string(&node.metadata)?;

            let dag_node = plan_dag_nodes::ActiveModel {
                id: Set(node.id.clone()),
                plan_id: Set(project_id), // Use project_id directly
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
                plan_id: Set(project_id), // Use project_id directly
                source_node_id: Set(edge.source.clone()),
                target_node_id: Set(edge.target.clone()),
                source_handle: Set(edge.source_handle.clone()),
                target_handle: Set(edge.target_handle.clone()),
                metadata_json: Set(metadata_json),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            dag_edge.insert(&context.db).await?;
        }

        // Return the updated Plan DAG
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
            .all(&context.db)
            .await?;

        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(project_id))
            .all(&context.db)
            .await?;

        let nodes: Vec<PlanDagNode> = dag_nodes.into_iter().map(PlanDagNode::from).collect();
        let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();

        Ok(Some(PlanDag {
            version: plan_dag.version,
            nodes,
            edges,
            metadata: plan_dag.metadata,
        }))
    }

    /// Add a single Plan DAG node
    async fn add_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node: PlanDagNodeInput) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await? {
            Some(plan) => plan,
            None => {
                // Auto-create a plan if one doesn't exist
                let now = chrono::Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(project_id),
                    name: Set(format!("Plan for Project {}", project_id)),
                    yaml_content: Set("".to_string()),
                    dependencies: Set(None),
                    status: Set("draft".to_string()),
                    version: Set(1),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                new_plan.insert(&context.db).await?
            }
        };

        // Fetch current state to determine node index
        let (current_nodes, _) = plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
        let node_index = current_nodes.len();

        let node_type_str = match node.node_type {
            crate::graphql::types::PlanDagNodeType::DataSource => "DataSourceNode",
            crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
            crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
            crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
            crate::graphql::types::PlanDagNodeType::Copy => "CopyNode",
            crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
        };

        let metadata_json = serde_json::to_string(&node.metadata)?;

        let dag_node = plan_dag_nodes::ActiveModel {
            id: Set(node.id.clone()),
            plan_id: Set(plan.id), // Use the actual plan ID instead of project ID
            node_type: Set(node_type_str.to_string()),
            position_x: Set(node.position.x),
            position_y: Set(node.position.y),
            metadata_json: Set(metadata_json),
            config_json: Set(node.config.clone()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_node = dag_node.insert(&context.db).await?;
        let result_node = PlanDagNode::from(inserted_node.clone());

        // Generate JSON Patch delta for node addition
        let patch_op = plan_dag_delta::generate_node_add_patch(&result_node, node_index);

        // Increment plan version
        let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;

        // Broadcast delta event
        let user_id = "demo_user".to_string(); // TODO: Get from auth context
        plan_dag_delta::publish_plan_dag_delta(
            project_id,
            new_version,
            user_id,
            vec![patch_op],
        ).await.ok(); // Non-fatal if broadcast fails

        // TODO: Trigger pipeline execution if node is configured
        // Will be implemented after Phase 2 completion
        // if should_execute_node(&inserted_node) {
        //     trigger_async_execution(context.db.clone(), project_id, result_node.id.clone());
        // }

        Ok(Some(result_node))
    }

    /// Update a Plan DAG node
    async fn update_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node_id: String, updates: PlanDagNodeUpdateInput) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await? {
            Some(plan) => plan,
            None => {
                // Auto-create a plan if one doesn't exist
                let now = chrono::Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(project_id),
                    name: Set(format!("Plan for Project {}", project_id)),
                    yaml_content: Set("".to_string()),
                    dependencies: Set(None),
                    status: Set("draft".to_string()),
                    version: Set(1),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                new_plan.insert(&context.db).await?
            }
        };

        // Fetch current state for delta generation
        let (current_nodes, _) = plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

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
        let mut patch_ops = Vec::new();
        let _config_updated = false;

        // Update position if provided
        if let Some(position) = updates.position {
            node_active.position_x = Set(position.x);
            node_active.position_y = Set(position.y);

            // Generate position delta
            patch_ops.extend(plan_dag_delta::generate_node_position_patch(
                &node_id,
                position.x,
                position.y,
                &current_nodes,
            ));
        }

        // Update metadata if provided
        if let Some(metadata) = updates.metadata {
            let metadata_json_str = serde_json::to_string(&metadata)?;
            node_active.metadata_json = Set(metadata_json_str.clone());

            // Generate metadata delta
            if let Some(patch) = plan_dag_delta::generate_node_update_patch(
                &node_id,
                "metadata",
                serde_json::from_str(&metadata_json_str).unwrap_or(serde_json::Value::Null),
                &current_nodes,
            ) {
                patch_ops.push(patch);
            }
        }

        // Update config if provided
        if let Some(config) = updates.config {
            // config_updated = true;
            node_active.config_json = Set(config.clone());

            // Generate config delta
            if let Some(patch) = plan_dag_delta::generate_node_update_patch(
                &node_id,
                "config",
                serde_json::from_str(&config).unwrap_or(serde_json::Value::Null),
                &current_nodes,
            ) {
                patch_ops.push(patch);
            }
        }

        node_active.updated_at = Set(Utc::now());
        let updated_node = node_active.update(&context.db).await?;
        let result_node = PlanDagNode::from(updated_node.clone());

        // Increment plan version and broadcast delta
        if !patch_ops.is_empty() {
            let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
            let user_id = "demo_user".to_string(); // TODO: Get from auth context
            plan_dag_delta::publish_plan_dag_delta(
                project_id,
                new_version,
                user_id,
                patch_ops,
            ).await.ok(); // Non-fatal if broadcast fails
        }

        // TODO: Trigger pipeline re-execution if config was updated
        // if config_updated && should_execute_node(&updated_node) {
        //     trigger_async_execution(context.db.clone(), project_id, result_node.id.clone());
        // }

        Ok(Some(result_node))
    }

    /// Delete a Plan DAG node
    async fn delete_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node_id: String) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Fetch current state for delta generation
        let (current_nodes, current_edges) = plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

        // Generate delta for node deletion
        let mut patch_ops = Vec::new();
        if let Some(patch) = plan_dag_delta::generate_node_delete_patch(&node_id, &current_nodes) {
            patch_ops.push(patch);
        }

        // Find and delete connected edges, generating deltas for each
        let connected_edges: Vec<&PlanDagEdge> = current_edges.iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect();

        for edge in &connected_edges {
            if let Some(patch) = plan_dag_delta::generate_edge_delete_patch(&edge.id, &current_edges) {
                patch_ops.push(patch);
            }
        }

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
            // Increment plan version and broadcast delta
            if !patch_ops.is_empty() {
                let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
                let user_id = "demo_user".to_string(); // TODO: Get from auth context
                plan_dag_delta::publish_plan_dag_delta(
                    project_id,
                    new_version,
                    user_id,
                    patch_ops,
                ).await.ok(); // Non-fatal if broadcast fails
            }

            Ok(None)
        } else {
            Err(Error::new("Node not found"))
        }
    }

    /// Add a Plan DAG edge
    async fn add_plan_dag_edge(&self, ctx: &Context<'_>, project_id: i32, edge: PlanDagEdgeInput) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await? {
            Some(plan) => plan,
            None => {
                // Auto-create a plan if one doesn't exist
                let now = chrono::Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(project_id),
                    name: Set(format!("Plan for Project {}", project_id)),
                    yaml_content: Set("".to_string()),
                    dependencies: Set(None),
                    status: Set("draft".to_string()),
                    version: Set(1),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                new_plan.insert(&context.db).await?
            }
        };

        // Fetch current state to determine edge index
        let (_, current_edges) = plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
        let edge_index = current_edges.len();

        let metadata_json = serde_json::to_string(&edge.metadata)?;

        let dag_edge = plan_dag_edges::ActiveModel {
            id: Set(edge.id.clone()),
            plan_id: Set(plan.id),
            source_node_id: Set(edge.source.clone()),
            target_node_id: Set(edge.target.clone()),
            source_handle: Set(edge.source_handle.clone()),
            target_handle: Set(edge.target_handle.clone()),
            metadata_json: Set(metadata_json),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_edge = dag_edge.insert(&context.db).await?;
        let result_edge = PlanDagEdge::from(inserted_edge);

        // Generate JSON Patch delta for edge addition
        let patch_op = plan_dag_delta::generate_edge_add_patch(&result_edge, edge_index);

        // Increment plan version
        let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;

        // Broadcast delta event
        let user_id = "demo_user".to_string(); // TODO: Get from auth context
        plan_dag_delta::publish_plan_dag_delta(
            project_id,
            new_version,
            user_id,
            vec![patch_op],
        ).await.ok(); // Non-fatal if broadcast fails

        // TODO: Trigger pipeline execution for affected nodes (target and downstream)
        // The target node needs to be recomputed because it has a new upstream dependency
        // trigger_async_affected_execution(context.db.clone(), project_id, result_edge.target.clone());

        Ok(Some(result_edge))
    }

    /// Delete a Plan DAG edge
    async fn delete_plan_dag_edge(&self, ctx: &Context<'_>, project_id: i32, edge_id: String) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Fetch current state for delta generation
        let (_, current_edges) = plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

        // Find the edge to get its target node before deletion
        let _deleted_edge_target = current_edges
            .iter()
            .find(|e| e.id == edge_id)
            .map(|e| e.target.clone());

        // Generate delta for edge deletion
        let mut patch_ops = Vec::new();
        if let Some(patch) = plan_dag_delta::generate_edge_delete_patch(&edge_id, &current_edges) {
            patch_ops.push(patch);
        }

        // Delete the edge
        let result = plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId.eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id))
            )
            .exec(&context.db)
            .await?;

        if result.rows_affected > 0 {
            // Increment plan version and broadcast delta
            if !patch_ops.is_empty() {
                let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
                let user_id = "demo_user".to_string(); // TODO: Get from auth context
                plan_dag_delta::publish_plan_dag_delta(
                    project_id,
                    new_version,
                    user_id,
                    patch_ops,
                ).await.ok(); // Non-fatal if broadcast fails
            }

            // TODO: Trigger pipeline re-execution for affected nodes (target and downstream)
            // The target node needs to be recomputed because it lost an upstream dependency
            // if let Some(target_node_id) = deleted_edge_target {
            //     trigger_async_affected_execution(context.db.clone(), project_id, target_node_id);
            // }

            Ok(None)
        } else {
            Err(Error::new("Edge not found"))
        }
    }

    /// Move a Plan DAG node (update position)
    async fn move_plan_dag_node(&self, ctx: &Context<'_>, project_id: i32, node_id: String, position: Position) -> Result<Option<PlanDagNode>> {
        let updates = PlanDagNodeUpdateInput {
            position: Some(position),
            metadata: None,
            config: None,
        };

        self.update_plan_dag_node(ctx, project_id, node_id, updates).await
    }

    // Authentication Mutations

    /// Register a new user
    async fn register(&self, ctx: &Context<'_>, input: RegisterUserInput) -> Result<RegisterResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Validate input
        AuthService::validate_email(&input.email)
            .map_err(|e| Error::new(format!("Email validation failed: {}", e)))?;
        AuthService::validate_username(&input.username)
            .map_err(|e| Error::new(format!("Username validation failed: {}", e)))?;
        AuthService::validate_display_name(&input.display_name)
            .map_err(|e| Error::new(format!("Display name validation failed: {}", e)))?;

        // Check if user already exists
        let existing_user = users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&context.db)
            .await?;

        if existing_user.is_some() {
            return Err(Error::new("User with this email already exists"));
        }

        let existing_username = users::Entity::find()
            .filter(users::Column::Username.eq(&input.username))
            .one(&context.db)
            .await?;

        if existing_username.is_some() {
            return Err(Error::new("Username already taken"));
        }

        // Hash password using bcrypt
        let password_hash = AuthService::hash_password(&input.password)
            .map_err(|e| Error::new(format!("Password hashing failed: {}", e)))?;

        // Create user
        let mut user = users::ActiveModel::new();
        user.email = Set(input.email);
        user.username = Set(input.username);
        user.display_name = Set(input.display_name);
        user.password_hash = Set(password_hash);
        user.avatar_color = Set(AuthService::generate_avatar_color());

        let user = user.insert(&context.db).await?;

        // Create session for the new user
        let session = user_sessions::ActiveModel::new(user.id, user.username.clone(), 1); // Assuming project ID 1 for now
        let session = session.insert(&context.db).await?;

        Ok(RegisterResponse {
            user: User::from(user),
            session_id: session.session_id,
            expires_at: session.expires_at,
        })
    }

    /// Login user
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find user by email
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Invalid email or password"))?;

        // Verify password using bcrypt
        let is_valid = AuthService::verify_password(&input.password, &user.password_hash)
            .map_err(|e| Error::new(format!("Password verification failed: {}", e)))?;

        if !is_valid {
            return Err(Error::new("Invalid email or password"));
        }

        // Check if user is active
        if !user.is_active {
            return Err(Error::new("Account is deactivated"));
        }

        // Create new session
        let session = user_sessions::ActiveModel::new(user.id, user.username.clone(), 1); // Assuming project ID 1 for now
        let session = session.insert(&context.db).await?;

        // Update last login
        let mut user_active: users::ActiveModel = user.clone().into();
        user_active.last_login_at = Set(Some(Utc::now()));
        user_active.update(&context.db).await?;

        Ok(LoginResponse {
            user: User::from(user),
            session_id: session.session_id,
            expires_at: session.expires_at,
        })
    }

    /// Logout user (deactivate session)
    async fn logout(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find and deactivate session
        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::SessionId.eq(&session_id))
            .one(&context.db)
            .await?;

        if let Some(session) = session {
            let mut session_active: user_sessions::ActiveModel = session.into();
            session_active = session_active.deactivate();
            session_active.update(&context.db).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update user profile
    async fn update_user(&self, ctx: &Context<'_>, user_id: i32, input: UpdateUserInput) -> Result<User> {
        let context = ctx.data::<GraphQLContext>()?;

        let user = users::Entity::find_by_id(user_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("User not found"))?;

        let mut user_active: users::ActiveModel = user.into();

        if let Some(display_name) = input.display_name {
            user_active.display_name = Set(display_name);
        }

        if let Some(email) = input.email {
            // Check if email is already taken by another user
            let existing = users::Entity::find()
                .filter(users::Column::Email.eq(&email))
                .filter(users::Column::Id.ne(user_id))
                .one(&context.db)
                .await?;

            if existing.is_some() {
                return Err(Error::new("Email already taken"));
            }

            user_active.email = Set(email);
        }

        user_active = user_active.set_updated_at();
        let updated_user = user_active.update(&context.db).await?;

        Ok(User::from(updated_user))
    }

    // Project Collaboration Mutations

    /// Invite a user to collaborate on a project
    async fn invite_collaborator(&self, ctx: &Context<'_>, input: InviteCollaboratorInput) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find user by email
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("User not found with this email"))?;

        // Check if user is already a collaborator
        let existing = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::ProjectId.eq(input.project_id))
            .filter(project_collaborators::Column::UserId.eq(user.id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&context.db)
            .await?;

        if existing.is_some() {
            return Err(Error::new("User is already a collaborator on this project"));
        }

        // Parse role
        let role = crate::database::entities::project_collaborators::ProjectRole::from_str(&input.role)
            .map_err(|_| Error::new("Invalid role"))?;

        // Create collaboration
        // Note: In a real app, you'd get invited_by from the authentication context
        let collaboration = project_collaborators::ActiveModel::new(
            input.project_id,
            user.id,
            role,
            Some(1), // TODO: Get from auth context
        );

        let collaboration = collaboration.insert(&context.db).await?;

        Ok(ProjectCollaborator::from(collaboration))
    }

    /// Accept collaboration invitation
    async fn accept_collaboration(&self, ctx: &Context<'_>, collaboration_id: i32) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Collaboration not found"))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.accept_invitation();
        let updated = collaboration_active.update(&context.db).await?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Decline collaboration invitation
    async fn decline_collaboration(&self, ctx: &Context<'_>, collaboration_id: i32) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Collaboration not found"))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.decline_invitation();
        let updated = collaboration_active.update(&context.db).await?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Update collaborator role
    async fn update_collaborator_role(&self, ctx: &Context<'_>, input: UpdateCollaboratorRoleInput) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(input.collaborator_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Collaboration not found"))?;

        // Parse new role
        let role = crate::database::entities::project_collaborators::ProjectRole::from_str(&input.role)
            .map_err(|_| Error::new("Invalid role"))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.update_role(role);
        let updated = collaboration_active.update(&context.db).await?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Remove collaborator from project
    async fn remove_collaborator(&self, ctx: &Context<'_>, collaboration_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Collaboration not found"))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.deactivate();
        collaboration_active.update(&context.db).await?;

        Ok(true)
    }

    // User Presence Mutations

    // REMOVED: update_user_presence, user_offline, and presence_heartbeat mutations
    // User presence is now handled via WebSocket only (memory-only storage) at /ws/collaboration
    // These GraphQL mutations have been replaced by real-time WebSocket communication for better performance

    // REMOVED: update_cursor_position mutation - replaced by WebSocket implementation
    // Cursor position updates are now handled via WebSocket at /ws/collaboration for better performance

    /// Join a project for collaboration
    async fn join_project_collaboration(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<bool> {
        let _context = ctx.data::<GraphQLContext>()?;

        // TODO: Extract from authenticated user context when authentication is implemented
        let (user_id, user_name, avatar_color) = {
            ("demo_user".to_string(), "Demo User".to_string(), "#3B82F6".to_string())
        };

        let plan_id = format!("project_{}", project_id);

        // Create user joined event data
        let user_data = crate::graphql::subscriptions::create_user_event_data(
            user_id.clone(),
            user_name,
            avatar_color,
        );

        // Create collaboration event
        let event_data = crate::graphql::subscriptions::CollaborationEventData {
            node_event: None,
            edge_event: None,
            user_event: Some(user_data),
            cursor_event: None,
        };
        let event = crate::graphql::subscriptions::create_collaboration_event(
            plan_id,
            user_id,
            crate::graphql::subscriptions::CollaborationEventType::UserJoined,
            event_data,
        );

        // Broadcast the event
        match crate::graphql::subscriptions::publish_collaboration_event(event).await {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Leave a project collaboration
    async fn leave_project_collaboration(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<bool> {
        let _context = ctx.data::<GraphQLContext>()?;

        // TODO: Extract from authenticated user context when authentication is implemented
        let (user_id, user_name, avatar_color) = {
            ("demo_user".to_string(), "Demo User".to_string(), "#3B82F6".to_string())
        };

        let plan_id = format!("project_{}", project_id);

        // Create user left event data
        let user_data = crate::graphql::subscriptions::create_user_event_data(
            user_id.clone(),
            user_name,
            avatar_color,
        );

        // Create collaboration event
        let event_data = crate::graphql::subscriptions::CollaborationEventData {
            node_event: None,
            edge_event: None,
            user_event: Some(user_data),
            cursor_event: None,
        };
        let event = crate::graphql::subscriptions::create_collaboration_event(
            plan_id,
            user_id,
            crate::graphql::subscriptions::CollaborationEventType::UserLeft,
            event_data,
        );

        // Broadcast the event
        match crate::graphql::subscriptions::publish_collaboration_event(event).await {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Create a new DataSource from uploaded file
    async fn create_data_source_from_file(
        &self,
        ctx: &Context<'_>,
        input: CreateDataSourceInput
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        // Decode the base64 file content
        use base64::Engine;
        let file_content = base64::engine::general_purpose::STANDARD.decode(&input.file_content)
            .map_err(|e| Error::new(format!("Failed to decode base64 file content: {}", e)))?;

        // Convert GraphQL enums to database enums
        let file_format = input.file_format.into();
        let data_type = input.data_type.into();

        let data_source = data_source_service
            .create_from_file(
                input.project_id,
                input.name,
                input.description,
                input.filename,
                file_format,
                data_type,
                file_content,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to create DataSource: {}", e)))?;

        Ok(DataSource::from(data_source))
    }

    /// Update DataSource metadata
    async fn update_data_source(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateDataSourceInput
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        let data_source = if let Some(file_content_b64) = input.file_content {
            // Update with new file - decode base64 content
            use base64::Engine;
            let file_content = base64::engine::general_purpose::STANDARD.decode(&file_content_b64)
                .map_err(|e| Error::new(format!("Failed to decode base64 file content: {}", e)))?;

            let filename = input.filename.unwrap_or_else(|| "updated_file".to_string());

            data_source_service
                .update_file(id, filename, file_content)
                .await
                .map_err(|e| Error::new(format!("Failed to update DataSource file: {}", e)))?
        } else {
            // Update metadata only
            data_source_service
                .update(id, input.name, input.description)
                .await
                .map_err(|e| Error::new(format!("Failed to update DataSource: {}", e)))?
        };

        Ok(DataSource::from(data_source))
    }

    /// Delete DataSource
    async fn delete_data_source(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        data_source_service
            .delete(id)
            .await
            .map_err(|e| Error::new(format!("Failed to delete DataSource: {}", e)))?;

        Ok(true)
    }

    /// Reprocess existing DataSource file
    async fn reprocess_data_source(&self, ctx: &Context<'_>, id: i32) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        let data_source = data_source_service
            .reprocess(id)
            .await
            .map_err(|e| Error::new(format!("Failed to reprocess DataSource: {}", e)))?;

        Ok(DataSource::from(data_source))
    }

    /// Execute a DAG node (builds graph from upstream data sources)
    async fn execute_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String
    ) -> Result<NodeExecutionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Get all nodes in the plan
        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;

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

        // Create executor and execute the node
        let executor = DagExecutor::new(context.db.clone());

        executor
            .execute_node(project_id, plan.id, &node_id, &nodes, &edges)
            .await
            .map_err(|e| Error::new(format!("Failed to execute node: {}", e)))?;

        Ok(NodeExecutionResult {
            success: true,
            message: format!("Node {} executed successfully", node_id),
            node_id,
        })
    }
}

#[derive(SimpleObject)]
pub struct PlanExecutionResult {
    pub success: bool,
    pub message: String,
    pub output_files: Vec<String>,
}

#[derive(SimpleObject)]
pub struct NodeExecutionResult {
    pub success: bool,
    pub message: String,
    pub node_id: String,
}
