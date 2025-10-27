pub mod plan_dag_delta;

use async_graphql::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::database::entities::{
    data_sources, datasources, graphs, plan_dag_edges, plan_dag_nodes, plans,
    project_collaborators, projects, user_sessions, users, ExecutionState,
};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::services::auth_service::AuthService;
use serde_json::{Map, Value};

use crate::pipeline::DagExecutor;
use crate::services::data_source_service::DataSourceService;
use crate::services::datasource_bulk_service::DataSourceBulkService;
use crate::services::export_service::ExportService;
use crate::services::graph_edit_service::GraphEditService;
use crate::services::graph_service::GraphService;
use crate::services::library_source_service::LibrarySourceService;
use crate::services::sample_project_service::SampleProjectService;

use crate::graphql::types::graph::{CreateGraphInput, CreateLayerInput, Graph, UpdateGraphInput};
use crate::graphql::types::graph_edit::{
    CreateGraphEditInput, EditResult, GraphEdit, ReplaySummary,
};
use crate::graphql::types::plan::{CreatePlanInput, Plan, UpdatePlanInput};
use crate::graphql::types::plan_dag::{
    NodePositionInput, PlanDag, PlanDagEdge, PlanDagEdgeInput, PlanDagEdgeUpdateInput,
    PlanDagInput, PlanDagNode, PlanDagNodeInput, PlanDagNodeUpdateInput, Position,
};
use crate::graphql::types::project::{CreateProjectInput, Project, UpdateProjectInput};
use crate::graphql::types::{
    BulkUploadDataSourceInput, CreateDataSourceInput, CreateEmptyDataSourceInput,
    CreateLibrarySourceInput, DataSource, ExportDataSourcesInput, ExportDataSourcesResult,
    ImportDataSourcesInput, ImportDataSourcesResult, ImportLibrarySourcesInput,
    InviteCollaboratorInput, LibrarySource, LoginInput, LoginResponse, ProjectCollaborator,
    RegisterResponse, RegisterUserInput, SeedLibrarySourcesResult, UpdateCollaboratorRoleInput,
    UpdateDataSourceInput, UpdateLibrarySourceInput, UpdateUserInput, User,
};

pub struct Mutation;

/// Generate a unique node ID based on node type and existing nodes
fn generate_node_id(
    node_type: &crate::graphql::types::PlanDagNodeType,
    existing_nodes: &[PlanDagNode],
) -> String {
    generate_node_id_from_ids(
        node_type,
        &existing_nodes
            .iter()
            .map(|n| n.id.as_str())
            .collect::<Vec<_>>(),
    )
}

/// Generate a unique node ID based on node type and existing node IDs
fn generate_node_id_from_ids(
    node_type: &crate::graphql::types::PlanDagNodeType,
    existing_node_ids: &[&str],
) -> String {
    use crate::graphql::types::PlanDagNodeType;

    // Get type prefix
    let type_prefix = match node_type {
        PlanDagNodeType::DataSource => "datasource",
        PlanDagNodeType::Graph => "graph",
        PlanDagNodeType::Transform => "transform",
        PlanDagNodeType::Filter => "filter",
        PlanDagNodeType::Merge => "merge",
        PlanDagNodeType::Copy => "copy",
        PlanDagNodeType::Output => "output",
    };

    // Extract all numeric suffixes from existing node IDs
    let number_pattern = regex::Regex::new(r"_(\d+)$").unwrap();
    let existing_numbers: Vec<i32> = existing_node_ids
        .iter()
        .filter_map(|id| {
            number_pattern
                .captures(id)
                .and_then(|cap| cap.get(1))
                .and_then(|m| m.as_str().parse::<i32>().ok())
        })
        .collect();

    // Find the max number and increment
    let max_number = existing_numbers.iter().max().copied().unwrap_or(0);
    let next_number = max_number + 1;

    // Format with leading zeros (3 digits)
    format!("{}_{:03}", type_prefix, next_number)
}

/// Generate a unique edge ID based on source and target
fn generate_edge_id(source: &str, target: &str) -> String {
    use uuid::Uuid;
    // Use a UUID suffix to ensure uniqueness even if same source/target combination
    format!("edge-{}-{}-{}", source, target, Uuid::new_v4().simple())
}

#[Object]
impl Mutation {
    /// Create a new project
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;

        let mut project = projects::ActiveModel::new();
        project.name = Set(input.name);
        project.description = Set(input.description);

        let project = project.insert(&context.db).await?;
        Ok(Project::from(project))
    }

    /// Create a project from a bundled sample definition
    async fn create_sample_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "sampleKey")] sample_key: String,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = SampleProjectService::new(context.db.clone());

        let project = service
            .create_sample_project(&sample_key)
            .await
            .map_err(|e| {
                StructuredError::service("SampleProjectService::create_sample_project", e)
            })?;

        Ok(Project::from(project))
    }

    /// Update an existing project
    async fn update_project(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;

        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", id))?;

        let mut project: projects::ActiveModel = project.into();
        project.name = Set(input.name);
        project.description = Set(input.description);

        let project = project
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::update", e))?;
        Ok(Project::from(project))
    }

    /// Delete a project
    async fn delete_project(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", id))?;

        projects::Entity::delete_by_id(project.id)
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::delete_by_id", e))?;

        Ok(true)
    }

    /// Create a new plan
    async fn create_plan(&self, ctx: &Context<'_>, input: CreatePlanInput) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;

        let dependencies_json = input
            .dependencies
            .map(|deps| serde_json::to_string(&deps))
            .transpose()
            .map_err(|e| {
                StructuredError::bad_request(format!("Invalid plan dependencies JSON: {}", e))
            })?;

        let plan = plans::ActiveModel {
            project_id: Set(input.project_id),
            name: Set(input.name),
            yaml_content: Set(input.yaml_content),
            dependencies: Set(dependencies_json),
            status: Set("pending".to_string()),
            ..Default::default()
        };

        let plan = plan
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::insert", e))?;
        Ok(Plan::from(plan))
    }

    /// Update an existing plan
    async fn update_plan(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdatePlanInput,
    ) -> Result<Plan> {
        let context = ctx.data::<GraphQLContext>()?;

        let plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Plan", id))?;

        let dependencies_json = input
            .dependencies
            .map(|deps| serde_json::to_string(&deps))
            .transpose()
            .map_err(|e| {
                StructuredError::bad_request(format!("Invalid plan dependencies JSON: {}", e))
            })?;

        let mut plan: plans::ActiveModel = plan.into();
        plan.name = Set(input.name);
        plan.yaml_content = Set(input.yaml_content);
        plan.dependencies = Set(dependencies_json);

        let plan = plan
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::update", e))?;
        Ok(Plan::from(plan))
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

    /// Update a complete Plan DAG
    ///
    /// **DEPRECATED**: This bulk replace operation conflicts with delta-based updates.
    /// Use individual node/edge mutations instead for better real-time collaboration.
    /// See PLAN.md Phase 2 for migration strategy.
    #[graphql(
        deprecation = "This bulk replace operation conflicts with delta-based real-time updates. Use addPlanDagNode, updatePlanDagNode, deletePlanDagNode, addPlanDagEdge, and deletePlanDagEdge mutations instead for better collaboration support."
    )]
    async fn update_plan_dag(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_dag: PlanDagInput,
    ) -> Result<Option<PlanDag>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

        // Clear existing Plan DAG nodes and edges for this project
        plan_dag_nodes::Entity::delete_many()
            .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
            .exec(&context.db)
            .await?;

        plan_dag_edges::Entity::delete_many()
            .filter(plan_dag_edges::Column::PlanId.eq(project_id))
            .exec(&context.db)
            .await?;

        // Collect existing node IDs for ID generation
        let mut existing_node_ids: Vec<String> = Vec::new();

        // Insert new Plan DAG nodes
        for node in &plan_dag.nodes {
            // Generate ID if not provided
            let node_id = node.id.clone().unwrap_or_else(|| {
                let id_refs: Vec<&str> = existing_node_ids.iter().map(|s| s.as_str()).collect();
                generate_node_id_from_ids(&node.node_type, &id_refs)
            });
            existing_node_ids.push(node_id.clone());

            let node_type_str = match node.node_type {
                crate::graphql::types::PlanDagNodeType::DataSource => "DataSourceNode",
                crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
                crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
                crate::graphql::types::PlanDagNodeType::Filter => "FilterNode",
                crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
                crate::graphql::types::PlanDagNodeType::Copy => "CopyNode",
                crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
            };

            let metadata_json = serde_json::to_string(&node.metadata)?;

            let dag_node = plan_dag_nodes::ActiveModel {
                id: Set(node_id),
                plan_id: Set(project_id), // Use project_id directly
                node_type: Set(node_type_str.to_string()),
                position_x: Set(node.position.x),
                position_y: Set(node.position.y),
                source_position: Set(None),
                target_position: Set(None),
                metadata_json: Set(metadata_json),
                config_json: Set(node.config.clone()),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            dag_node.insert(&context.db).await?;
        }

        // Insert new Plan DAG edges
        for edge in &plan_dag.edges {
            // Generate ID if not provided
            let edge_id = edge
                .id
                .clone()
                .unwrap_or_else(|| generate_edge_id(&edge.source, &edge.target));

            let metadata_json = serde_json::to_string(&edge.metadata)?;

            let dag_edge = plan_dag_edges::ActiveModel {
                id: Set(edge_id),
                plan_id: Set(project_id), // Use project_id directly
                source_node_id: Set(edge.source.clone()),
                target_node_id: Set(edge.target.clone()),
                // Removed source_handle and target_handle for floating edges
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
    async fn add_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node: PlanDagNodeInput,
    ) -> Result<Option<PlanDagNode>> {
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
            .await?
        {
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

        // Fetch current state to determine node index and generate unique ID
        let (current_nodes, _) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
        let node_index = current_nodes.len();

        // Generate unique ID on backend - ignore frontend-provided ID
        let generated_id = generate_node_id(&node.node_type, &current_nodes);

        let node_type_str = match node.node_type {
            crate::graphql::types::PlanDagNodeType::DataSource => "DataSourceNode",
            crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
            crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
            crate::graphql::types::PlanDagNodeType::Filter => "FilterNode",
            crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
            crate::graphql::types::PlanDagNodeType::Copy => "CopyNode",
            crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
        };

        let metadata_json = serde_json::to_string(&node.metadata)?;

        let dag_node = plan_dag_nodes::ActiveModel {
            id: Set(generated_id),
            plan_id: Set(plan.id), // Use the actual plan ID instead of project ID
            node_type: Set(node_type_str.to_string()),
            position_x: Set(node.position.x),
            position_y: Set(node.position.y),
            source_position: Set(None),
            target_position: Set(None),
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
        plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, vec![patch_op])
            .await
            .ok(); // Non-fatal if broadcast fails

        // TODO: Trigger pipeline execution if node is configured
        // Will be implemented after Phase 2 completion
        // if should_execute_node(&inserted_node) {
        //     trigger_async_execution(context.db.clone(), project_id, result_node.id.clone());
        // }

        Ok(Some(result_node))
    }

    /// Update a Plan DAG node
    async fn update_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
        updates: PlanDagNodeUpdateInput,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
        {
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
        let (current_nodes, _) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

        // Find the node
        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Node not found"))?;

        let node_type = node.node_type.clone();
        let mut metadata_json_current = node.metadata_json.clone();

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
            metadata_json_current = metadata_json_str.clone();

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

            if node_type == "DataSourceNode" {
                if let Ok(config_value) = serde_json::from_str::<Value>(&config) {
                    if let Some(data_source_id) =
                        config_value.get("dataSourceId").and_then(|v| v.as_i64())
                    {
                        if let Some(data_source) =
                            data_sources::Entity::find_by_id(data_source_id as i32)
                                .one(&context.db)
                                .await?
                        {
                            let mut metadata_obj =
                                serde_json::from_str::<Value>(&metadata_json_current)
                                    .ok()
                                    .and_then(|value| value.as_object().cloned())
                                    .unwrap_or_else(|| Map::new());

                            let needs_update = match metadata_obj.get("label") {
                                Some(Value::String(current_label))
                                    if current_label == &data_source.name =>
                                {
                                    false
                                }
                                _ => true,
                            };

                            if needs_update {
                                metadata_obj.insert(
                                    "label".to_string(),
                                    Value::String(data_source.name.clone()),
                                );
                                let metadata_value = Value::Object(metadata_obj);
                                metadata_json_current = metadata_value.to_string();
                                node_active.metadata_json = Set(metadata_json_current.clone());

                                if let Some(patch) = plan_dag_delta::generate_node_update_patch(
                                    &node_id,
                                    "metadata",
                                    serde_json::from_str(&metadata_json_current)
                                        .unwrap_or(Value::Null),
                                    &current_nodes,
                                ) {
                                    patch_ops.push(patch);
                                }
                            }
                        }
                    }
                }
            }
        }

        node_active.updated_at = Set(Utc::now());
        let updated_node = node_active.update(&context.db).await?;
        let result_node = PlanDagNode::from(updated_node.clone());

        // Increment plan version and broadcast delta
        if !patch_ops.is_empty() {
            let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
            let user_id = "demo_user".to_string(); // TODO: Get from auth context
            plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, patch_ops)
                .await
                .ok(); // Non-fatal if broadcast fails
        }

        // TODO: Trigger pipeline re-execution if config was updated
        // if config_updated && should_execute_node(&updated_node) {
        //     trigger_async_execution(context.db.clone(), project_id, result_node.id.clone());
        // }

        Ok(Some(result_node))
    }

    /// Delete a Plan DAG node
    async fn delete_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Fetch current state for delta generation
        let (current_nodes, current_edges) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

        // Generate delta for node deletion
        let mut patch_ops = Vec::new();
        if let Some(patch) = plan_dag_delta::generate_node_delete_patch(&node_id, &current_nodes) {
            patch_ops.push(patch);
        }

        // Find and delete connected edges, generating deltas for each
        let connected_edges: Vec<&PlanDagEdge> = current_edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect();

        for edge in &connected_edges {
            if let Some(patch) =
                plan_dag_delta::generate_edge_delete_patch(&edge.id, &current_edges)
            {
                patch_ops.push(patch);
            }
        }

        // Delete edges connected to this node first
        plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId.eq(plan.id).and(
                    plan_dag_edges::Column::SourceNodeId
                        .eq(&node_id)
                        .or(plan_dag_edges::Column::TargetNodeId.eq(&node_id)),
                ),
            )
            .exec(&context.db)
            .await?;

        // Delete the node
        let result = plan_dag_nodes::Entity::delete_many()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .exec(&context.db)
            .await?;

        if result.rows_affected > 0 {
            // Increment plan version and broadcast delta
            if !patch_ops.is_empty() {
                let new_version =
                    plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
                let user_id = "demo_user".to_string(); // TODO: Get from auth context
                plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, patch_ops)
                    .await
                    .ok(); // Non-fatal if broadcast fails
            }

            Ok(None)
        } else {
            Err(Error::new("Node not found"))
        }
    }

    /// Add a Plan DAG edge
    async fn add_plan_dag_edge(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        edge: PlanDagEdgeInput,
    ) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
        {
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
        let (_, current_edges) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
        let edge_index = current_edges.len();

        // Generate unique ID on backend - ignore frontend-provided ID
        let generated_id = generate_edge_id(&edge.source, &edge.target);

        let metadata_json = serde_json::to_string(&edge.metadata)?;

        let dag_edge = plan_dag_edges::ActiveModel {
            id: Set(generated_id),
            plan_id: Set(plan.id),
            source_node_id: Set(edge.source.clone()),
            target_node_id: Set(edge.target.clone()),
            // Removed source_handle and target_handle for floating edges
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
        plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, vec![patch_op])
            .await
            .ok(); // Non-fatal if broadcast fails

        // TODO: Trigger pipeline execution for affected nodes (target and downstream)
        // The target node needs to be recomputed because it has a new upstream dependency
        // trigger_async_affected_execution(context.db.clone(), project_id, result_edge.target.clone());

        Ok(Some(result_edge))
    }

    /// Delete a Plan DAG edge
    async fn delete_plan_dag_edge(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        edge_id: String,
    ) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Fetch current state for delta generation
        let (_, current_edges) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

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
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .exec(&context.db)
            .await?;

        if result.rows_affected > 0 {
            // Increment plan version and broadcast delta
            if !patch_ops.is_empty() {
                let new_version =
                    plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
                let user_id = "demo_user".to_string(); // TODO: Get from auth context
                plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, patch_ops)
                    .await
                    .ok(); // Non-fatal if broadcast fails
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

    /// Update a Plan DAG edge
    async fn update_plan_dag_edge(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        edge_id: String,
        updates: PlanDagEdgeUpdateInput,
    ) -> Result<Option<PlanDagEdge>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Find the edge
        let edge = plan_dag_edges::Entity::find()
            .filter(
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Edge not found"))?;

        let mut edge_active: plan_dag_edges::ActiveModel = edge.into();

        // Removed source_handle and target_handle updates for floating edges

        // Update metadata if provided
        if let Some(metadata) = updates.metadata {
            let metadata_json = serde_json::to_string(&metadata)?;
            edge_active.metadata_json = Set(metadata_json);
        }

        edge_active.updated_at = Set(Utc::now());
        let updated_edge = edge_active.update(&context.db).await?;

        Ok(Some(PlanDagEdge::from(updated_edge)))
    }

    /// Move a Plan DAG node (update position)
    async fn move_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
        position: Position,
    ) -> Result<Option<PlanDagNode>> {
        let updates = PlanDagNodeUpdateInput {
            position: Some(position),
            metadata: None,
            config: None,
        };

        self.update_plan_dag_node(ctx, project_id, node_id, updates)
            .await
    }

    /// Batch move multiple nodes at once (optimized for layout operations)
    async fn batch_move_plan_dag_nodes(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_positions: Vec<NodePositionInput>,
    ) -> Result<Vec<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
        {
            Some(plan) => plan,
            None => {
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

        // Fetch current state once for all nodes
        let (current_nodes, _) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;

        let mut updated_nodes = Vec::new();
        let mut all_patch_ops = Vec::new();

        // Update all nodes
        for node_pos in node_positions {
            // Find the node
            let node = plan_dag_nodes::Entity::find()
                .filter(
                    plan_dag_nodes::Column::PlanId
                        .eq(plan.id)
                        .and(plan_dag_nodes::Column::Id.eq(&node_pos.node_id)),
                )
                .one(&context.db)
                .await?;

            if let Some(node) = node {
                let mut node_active: plan_dag_nodes::ActiveModel = node.clone().into();

                // Update position
                node_active.position_x = Set(node_pos.position.x);
                node_active.position_y = Set(node_pos.position.y);
                if let Some(source_pos) = node_pos.source_position.clone() {
                    node_active.source_position = Set(Some(source_pos));
                }
                if let Some(target_pos) = node_pos.target_position.clone() {
                    node_active.target_position = Set(Some(target_pos));
                }
                node_active.updated_at = Set(Utc::now());

                // Save to database
                let updated_node = node_active.update(&context.db).await?;

                // Generate position delta
                all_patch_ops.extend(plan_dag_delta::generate_node_position_patch(
                    &node_pos.node_id,
                    node_pos.position.x,
                    node_pos.position.y,
                    &current_nodes,
                ));

                updated_nodes.push(PlanDagNode::from(updated_node));
            }
        }

        // Publish all deltas in a single batch
        if !all_patch_ops.is_empty() {
            let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;
            let user_id = "demo_user".to_string(); // TODO: Get from auth context
            plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, all_patch_ops)
                .await?;
        }

        Ok(updated_nodes)
    }

    // Authentication Mutations

    /// Register a new user
    async fn register(
        &self,
        ctx: &Context<'_>,
        input: RegisterUserInput,
    ) -> Result<RegisterResponse> {
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
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        input: UpdateUserInput,
    ) -> Result<User> {
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
    async fn invite_collaborator(
        &self,
        ctx: &Context<'_>,
        input: InviteCollaboratorInput,
    ) -> Result<ProjectCollaborator> {
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
        let role =
            crate::database::entities::project_collaborators::ProjectRole::from_str(&input.role)
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
    async fn accept_collaboration(
        &self,
        ctx: &Context<'_>,
        collaboration_id: i32,
    ) -> Result<ProjectCollaborator> {
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
    async fn decline_collaboration(
        &self,
        ctx: &Context<'_>,
        collaboration_id: i32,
    ) -> Result<ProjectCollaborator> {
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
    async fn update_collaborator_role(
        &self,
        ctx: &Context<'_>,
        input: UpdateCollaboratorRoleInput,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(input.collaborator_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Collaboration not found"))?;

        // Parse new role
        let role =
            crate::database::entities::project_collaborators::ProjectRole::from_str(&input.role)
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
    async fn join_project_collaboration(&self, ctx: &Context<'_>, project_id: i32) -> Result<bool> {
        let _context = ctx.data::<GraphQLContext>()?;

        // TODO: Extract from authenticated user context when authentication is implemented
        let (user_id, user_name, avatar_color) = {
            (
                "demo_user".to_string(),
                "Demo User".to_string(),
                "#3B82F6".to_string(),
            )
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
            (
                "demo_user".to_string(),
                "Demo User".to_string(),
                "#3B82F6".to_string(),
            )
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
        input: CreateDataSourceInput,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        // Decode the base64 file content
        use base64::Engine;
        let file_content = base64::engine::general_purpose::STANDARD
            .decode(&input.file_content)
            .map_err(|e| Error::new(format!("Failed to decode base64 file content: {}", e)))?;

        // Convert GraphQL enums to database enums
        let file_format = input.file_format.into();
        let data_type = input.data_type.into();

        let data_source = data_source_service
            .create_from_file(
                input.project_id,
                input.name.clone(),
                input.description,
                input.filename.clone(),
                file_format,
                data_type,
                file_content,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to create DataSource: {}", e)))?;

        // Automatically add a DataSourceNode to the Plan DAG
        let timestamp = chrono::Utc::now().timestamp_millis();
        let node_id = format!("datasourcenode_{}_{:08x}", timestamp, data_source.id);

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(input.project_id))
            .one(&context.db)
            .await?
        {
            Some(plan) => plan,
            None => {
                // Auto-create a plan if one doesn't exist
                let now = chrono::Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(input.project_id),
                    name: Set(format!("Plan for Project {}", input.project_id)),
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

        // Fetch current state to determine node index and position
        let (current_nodes, _) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
        let node_index = current_nodes.len();

        // Calculate position: stack vertically with some spacing
        let position_x = 100.0;
        let position_y = 100.0 + (node_index as f64 * 120.0);

        // Create the DAG node metadata and config
        let metadata_json = serde_json::to_string(&serde_json::json!({
            "label": input.name
        }))?;

        let config_json = serde_json::to_string(&serde_json::json!({
            "dataSourceId": data_source.id,
            "filename": input.filename,
            "dataType": data_source.data_type.to_lowercase()
        }))?;

        let dag_node = plan_dag_nodes::ActiveModel {
            id: Set(node_id.clone()),
            plan_id: Set(plan.id),
            node_type: Set("DataSourceNode".to_string()),
            position_x: Set(position_x),
            position_y: Set(position_y),
            source_position: Set(None),
            target_position: Set(None),
            metadata_json: Set(metadata_json),
            config_json: Set(config_json),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_node = dag_node.insert(&context.db).await?;
        let result_node = PlanDagNode::from(inserted_node);

        // Generate JSON Patch delta for node addition
        let patch_op = plan_dag_delta::generate_node_add_patch(&result_node, node_index);

        // Increment plan version
        let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;

        // Broadcast delta event
        let user_id = "demo_user".to_string(); // TODO: Get from auth context
        plan_dag_delta::publish_plan_dag_delta(
            input.project_id,
            new_version,
            user_id,
            vec![patch_op],
        )
        .await
        .ok(); // Non-fatal if broadcast fails

        Ok(DataSource::from(data_source))
    }

    /// Create a new empty DataSource (without file upload)
    async fn create_empty_data_source(
        &self,
        ctx: &Context<'_>,
        input: CreateEmptyDataSourceInput,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        // Convert GraphQL enum to database enum
        let data_type = input.data_type.into();

        let data_source = data_source_service
            .create_empty(
                input.project_id,
                input.name.clone(),
                input.description,
                data_type,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to create empty DataSource: {}", e)))?;

        // Automatically add a DataSourceNode to the Plan DAG
        let timestamp = chrono::Utc::now().timestamp_millis();
        let node_id = format!("datasourcenode_{}_{:08x}", timestamp, data_source.id);

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(input.project_id))
            .one(&context.db)
            .await?
        {
            Some(plan) => plan,
            None => {
                // Auto-create a plan if one doesn't exist
                let now = chrono::Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(input.project_id),
                    name: Set(format!("Plan for Project {}", input.project_id)),
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

        // Fetch current state to determine node index and position
        let (current_nodes, _) =
            plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
        let node_index = current_nodes.len();

        // Calculate position: stack vertically with some spacing
        let position_x = 100.0;
        let position_y = 100.0 + (node_index as f64 * 120.0);

        // Create the DAG node metadata and config
        let metadata_json = serde_json::to_string(&serde_json::json!({
            "label": input.name
        }))?;

        let config_json = serde_json::to_string(&serde_json::json!({
            "dataSourceId": data_source.id,
            "filename": data_source.filename,
            "dataType": data_source.data_type.to_lowercase()
        }))?;

        let dag_node = plan_dag_nodes::ActiveModel {
            id: Set(node_id.clone()),
            plan_id: Set(plan.id),
            node_type: Set("DataSourceNode".to_string()),
            position_x: Set(position_x),
            position_y: Set(position_y),
            source_position: Set(None),
            target_position: Set(None),
            metadata_json: Set(metadata_json),
            config_json: Set(config_json),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let inserted_node = dag_node.insert(&context.db).await?;
        let result_node = PlanDagNode::from(inserted_node);

        // Generate JSON Patch delta for node addition
        let patch_op = plan_dag_delta::generate_node_add_patch(&result_node, node_index);

        // Increment plan version
        let new_version = plan_dag_delta::increment_plan_version(&context.db, plan.id).await?;

        // Broadcast delta event
        let user_id = "demo_user".to_string(); // TODO: Get from auth context
        plan_dag_delta::publish_plan_dag_delta(
            input.project_id,
            new_version,
            user_id,
            vec![patch_op],
        )
        .await
        .ok(); // Non-fatal if broadcast fails

        Ok(DataSource::from(data_source))
    }

    /// Bulk upload multiple DataSources with automatic file type detection
    async fn bulk_upload_data_sources(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        files: Vec<BulkUploadDataSourceInput>,
    ) -> Result<Vec<DataSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        let mut created_data_sources = Vec::new();

        for file_input in files {
            // Decode the base64 file content
            use base64::Engine;
            let file_content = base64::engine::general_purpose::STANDARD
                .decode(&file_input.file_content)
                .map_err(|e| {
                    Error::new(format!(
                        "Failed to decode base64 file content for {}: {}",
                        file_input.filename, e
                    ))
                })?;

            // Use auto-detection to create the data source
            let data_source = data_source_service
                .create_with_auto_detect(
                    project_id,
                    file_input.name.clone(),
                    file_input.description,
                    file_input.filename.clone(),
                    file_content,
                )
                .await
                .map_err(|e| {
                    Error::new(format!(
                        "Failed to create DataSource for {}: {}",
                        file_input.filename, e
                    ))
                })?;

            // Automatically add a DataSourceNode to the Plan DAG
            let timestamp = chrono::Utc::now().timestamp_millis();
            let node_id = format!("datasourcenode_{}_{:08x}", timestamp, data_source.id);

            // Find or create a plan for this project
            let plan = match plans::Entity::find()
                .filter(plans::Column::ProjectId.eq(project_id))
                .one(&context.db)
                .await?
            {
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

            // Fetch current state to determine node index and position
            let (current_nodes, _) =
                plan_dag_delta::fetch_current_plan_dag(&context.db, plan.id).await?;
            let node_index = current_nodes.len();

            // Calculate position: stack vertically with some spacing
            let position_x = 100.0;
            let position_y = 100.0 + (node_index as f64 * 120.0);

            // Create the DAG node metadata and config
            let metadata_json = serde_json::to_string(&serde_json::json!({
                "label": file_input.name
            }))?;

            let config_json = serde_json::to_string(&serde_json::json!({
                "dataSourceId": data_source.id,
                "filename": file_input.filename,
                "dataType": data_source.data_type.to_lowercase()
            }))?;

            let dag_node = plan_dag_nodes::ActiveModel {
                id: Set(node_id.clone()),
                plan_id: Set(plan.id),
                node_type: Set("DataSourceNode".to_string()),
                position_x: Set(position_x),
                position_y: Set(position_y),
                source_position: Set(None),
                target_position: Set(None),
                metadata_json: Set(metadata_json),
                config_json: Set(config_json),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            let inserted_node = dag_node.insert(&context.db).await?;
            let result_node = PlanDagNode::from(inserted_node);

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
            )
            .await
            .ok(); // Non-fatal if broadcast fails

            created_data_sources.push(DataSource::from(data_source));
        }

        Ok(created_data_sources)
    }

    /// Update DataSource metadata
    async fn update_data_source(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateDataSourceInput,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        let data_source = if let Some(file_content_b64) = input.file_content {
            // Update with new file - decode base64 content
            use base64::Engine;
            let file_content = base64::engine::general_purpose::STANDARD
                .decode(&file_content_b64)
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

    /// Update DataSource graph data directly
    async fn update_data_source_graph_data(
        &self,
        ctx: &Context<'_>,
        id: i32,
        graph_json: String,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source_service = DataSourceService::new(context.db.clone());

        let data_source = data_source_service
            .update_graph_data(id, graph_json)
            .await
            .map_err(|e| Error::new(format!("Failed to update DataSource graph data: {}", e)))?;

        Ok(DataSource::from(data_source))
    }

    /// Export data sources as spreadsheet (XLSX or ODS)
    async fn export_data_sources(
        &self,
        ctx: &Context<'_>,
        input: ExportDataSourcesInput,
    ) -> Result<ExportDataSourcesResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let bulk_service = DataSourceBulkService::new(context.db.clone());

        let format_str = match input.format {
            crate::graphql::types::SpreadsheetFormat::XLSX => "xlsx",
            crate::graphql::types::SpreadsheetFormat::ODS => "ods",
        };

        // Export to the requested format
        let file_bytes = match input.format {
            crate::graphql::types::SpreadsheetFormat::XLSX => bulk_service
                .export_to_xlsx(&input.data_source_ids)
                .await
                .map_err(|e| {
                    StructuredError::service("DataSourceBulkService::export_to_xlsx", e)
                })?,
            crate::graphql::types::SpreadsheetFormat::ODS => bulk_service
                .export_to_ods(&input.data_source_ids)
                .await
                .map_err(|e| StructuredError::service("DataSourceBulkService::export_to_ods", e))?,
        };

        // Encode as base64
        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(&file_bytes);

        Ok(ExportDataSourcesResult {
            file_content: encoded,
            filename: format!(
                "datasources_export_{}.{}",
                chrono::Utc::now().timestamp(),
                format_str
            ),
            format: format_str.to_string(),
        })
    }

    /// Import data sources from spreadsheet (XLSX or ODS)
    async fn import_data_sources(
        &self,
        ctx: &Context<'_>,
        input: ImportDataSourcesInput,
    ) -> Result<ImportDataSourcesResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let bulk_service = DataSourceBulkService::new(context.db.clone());

        // Decode base64 file content
        use base64::{engine::general_purpose, Engine as _};
        tracing::info!(
            "Importing datasources from file: {} (base64 length: {})",
            input.filename,
            input.file_content.len()
        );

        let file_bytes = general_purpose::STANDARD
            .decode(&input.file_content)
            .map_err(|e| StructuredError::bad_request(format!("Invalid base64 content: {}", e)))?;

        tracing::info!("Decoded {} bytes from base64", file_bytes.len());

        // Import from XLSX or ODS (check file extension)
        let result = if input.filename.to_lowercase().ends_with(".xlsx") {
            bulk_service
                .import_from_xlsx(input.project_id, &file_bytes)
                .await
                .map_err(|e| {
                    StructuredError::service("DataSourceBulkService::import_from_xlsx", e)
                })?
        } else if input.filename.to_lowercase().ends_with(".ods") {
            bulk_service
                .import_from_ods(input.project_id, &file_bytes)
                .await
                .map_err(|e| {
                    StructuredError::service("DataSourceBulkService::import_from_ods", e)
                })?
        } else {
            return Err(StructuredError::bad_request(
                "Only XLSX and ODS formats are supported for import",
            ));
        };

        // Fetch the imported datasources to return
        use crate::database::entities::data_sources;
        use sea_orm::EntityTrait;

        let datasources = data_sources::Entity::find()
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("data_sources::Entity::find().all", e))?
            .into_iter()
            .filter(|ds| result.imported_ids.contains(&ds.id))
            .map(DataSource::from)
            .collect();

        Ok(ImportDataSourcesResult {
            data_sources: datasources,
            created_count: result.created_count,
            updated_count: result.updated_count,
        })
    }

    /// Create a new LibrarySource from uploaded file
    async fn create_library_source(
        &self,
        ctx: &Context<'_>,
        input: CreateLibrarySourceInput,
    ) -> Result<LibrarySource> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibrarySourceService::new(context.db.clone());

        let CreateLibrarySourceInput {
            name,
            description,
            filename,
            file_content,
            file_format,
            data_type,
        } = input;

        use base64::Engine;
        let file_bytes = base64::engine::general_purpose::STANDARD
            .decode(&file_content)
            .map_err(|e| {
                StructuredError::bad_request(format!("Failed to decode base64 file content: {}", e))
            })?;

        let file_format: crate::database::entities::data_sources::FileFormat = file_format.into();
        let data_type: crate::database::entities::data_sources::DataType = data_type.into();

        let model = service
            .create_from_file(
                name,
                description,
                filename,
                file_format,
                data_type,
                file_bytes,
            )
            .await
            .map_err(|e| StructuredError::service("LibrarySourceService::create_from_file", e))?;

        Ok(LibrarySource::from(model))
    }

    /// Update an existing LibrarySource metadata and optionally replace its file
    async fn update_library_source(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateLibrarySourceInput,
    ) -> Result<LibrarySource> {
        if input.file_content.is_none() && input.filename.is_some() {
            return Err(StructuredError::validation(
                "filename",
                "filename can only be changed when fileContent is provided",
            ));
        }

        let context = ctx.data::<GraphQLContext>()?;
        let service = LibrarySourceService::new(context.db.clone());

        let mut current = if let Some(file_content) = &input.file_content {
            use base64::Engine;
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(file_content)
                .map_err(|e| {
                    StructuredError::bad_request(format!(
                        "Failed to decode base64 file content: {}",
                        e
                    ))
                })?;

            let filename = if let Some(filename) = &input.filename {
                filename.clone()
            } else {
                service
                    .get_by_id(id)
                    .await
                    .map_err(|e| StructuredError::service("LibrarySourceService::get_by_id", e))?
                    .ok_or_else(|| StructuredError::not_found("LibrarySource", id))?
                    .filename
            };

            service
                .update_file(id, filename, file_bytes)
                .await
                .map_err(|e| StructuredError::service("LibrarySourceService::update_file", e))?
        } else {
            service
                .get_by_id(id)
                .await
                .map_err(|e| StructuredError::service("LibrarySourceService::get_by_id", e))?
                .ok_or_else(|| StructuredError::not_found("LibrarySource", id))?
        };

        if input.name.is_some() || input.description.is_some() {
            current = service
                .update(id, input.name.clone(), input.description.clone())
                .await
                .map_err(|e| StructuredError::service("LibrarySourceService::update", e))?;
        }

        Ok(LibrarySource::from(current))
    }

    /// Delete a LibrarySource
    async fn delete_library_source(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibrarySourceService::new(context.db.clone());

        service
            .delete(id)
            .await
            .map_err(|e| StructuredError::service("LibrarySourceService::delete", e))?;

        Ok(true)
    }

    /// Reprocess the stored file for a LibrarySource
    async fn reprocess_library_source(&self, ctx: &Context<'_>, id: i32) -> Result<LibrarySource> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibrarySourceService::new(context.db.clone());

        let model = service
            .reprocess(id)
            .await
            .map_err(|e| StructuredError::service("LibrarySourceService::reprocess", e))?;

        Ok(LibrarySource::from(model))
    }

    /// Import one or more LibrarySources into a project as project-scoped DataSources
    async fn import_library_sources(
        &self,
        ctx: &Context<'_>,
        input: ImportLibrarySourcesInput,
    ) -> Result<Vec<DataSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibrarySourceService::new(context.db.clone());

        let models = service
            .import_many_into_project(input.project_id, &input.library_source_ids)
            .await
            .map_err(|e| {
                StructuredError::service("LibrarySourceService::import_many_into_project", e)
            })?;

        Ok(models.into_iter().map(DataSource::from).collect())
    }

    /// Seed the shared library with the canonical GitHub resources bundle
    async fn seed_library_sources(&self, ctx: &Context<'_>) -> Result<SeedLibrarySourcesResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibrarySourceService::new(context.db.clone());

        let result = service.seed_from_github_library().await.map_err(|e| {
            StructuredError::service("LibrarySourceService::seed_from_github_library", e)
        })?;

        Ok(SeedLibrarySourcesResult::from(result))
    }

    /// Create a new Graph
    async fn create_graph(&self, ctx: &Context<'_>, input: CreateGraphInput) -> Result<Graph> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        let graph = graph_service
            .create_graph(input.project_id, input.name, None)
            .await
            .map_err(|e| Error::new(format!("Failed to create Graph: {}", e)))?;

        Ok(Graph::from(graph))
    }

    /// Update Graph metadata
    async fn update_graph(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateGraphInput,
    ) -> Result<Graph> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        let graph = graph_service
            .update_graph(id, input.name)
            .await
            .map_err(|e| Error::new(format!("Failed to update Graph: {}", e)))?;

        Ok(Graph::from(graph))
    }

    /// Delete Graph
    async fn delete_graph(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        graph_service
            .delete_graph(id)
            .await
            .map_err(|e| Error::new(format!("Failed to delete Graph: {}", e)))?;

        Ok(true)
    }

    /// Create a new Layer
    async fn create_layer(
        &self,
        ctx: &Context<'_>,
        input: CreateLayerInput,
    ) -> Result<crate::graphql::types::Layer> {
        let context = ctx.data::<GraphQLContext>()?;

        use crate::database::entities::graph_layers;

        let layer = graph_layers::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_id: Set(input.graph_id),
            layer_id: Set(input.layer_id),
            name: Set(input.name),
            background_color: Set(None),
            text_color: Set(None),
            border_color: Set(None),
            comment: Set(None),
            properties: Set(None),
            datasource_id: Set(None),
        };

        let inserted_layer = layer
            .insert(&context.db)
            .await
            .map_err(|e| Error::new(format!("Failed to create layer: {}", e)))?;

        Ok(crate::graphql::types::Layer::from(inserted_layer))
    }

    /// Update a graph node's properties
    async fn update_graph_node(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attrs: Option<crate::graphql::types::scalars::JSON>,
        belongs_to: Option<String>,
    ) -> Result<crate::graphql::types::graph_node::GraphNode> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        // Fetch current node to get old values
        use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as GraphNodes};
        use sea_orm::{ColumnTrait, QueryFilter};

        let old_node = GraphNodes::find()
            .filter(NodeColumn::GraphId.eq(graph_id))
            .filter(NodeColumn::Id.eq(&node_id))
            .one(&context.db)
            .await?;

        // Convert belongs_to to Option<Option<String>> for service call
        let belongs_to_param =
            belongs_to
                .as_ref()
                .map(|b| if b.is_empty() { None } else { Some(b.clone()) });

        // Update the node
        let node = graph_service
            .update_graph_node(
                graph_id,
                node_id.clone(),
                label.clone(),
                layer.clone(),
                attrs.clone(),
                belongs_to_param.clone(),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to update graph node: {}", e)))?;

        // Create graph edits for each changed field
        if let Some(old_node) = old_node {
            if let Some(new_label) = &label {
                if old_node.label.as_ref() != Some(new_label) {
                    let _ = edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("label".to_string()),
                            old_node.label.as_ref().map(|l| serde_json::json!(l)),
                            Some(serde_json::json!(new_label)),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_layer) = &layer {
                let old_layer_value = old_node.layer.clone().unwrap_or_default();
                if &old_layer_value != new_layer {
                    let _ = edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("layer".to_string()),
                            if old_layer_value.is_empty() {
                                None
                            } else {
                                Some(serde_json::json!(old_layer_value))
                            },
                            Some(serde_json::json!(new_layer)),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_attrs) = &attrs {
                if old_node.attrs.as_ref() != Some(new_attrs) {
                    let _ = edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("attrs".to_string()),
                            old_node.attrs.clone(),
                            Some(new_attrs.clone()),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_belongs_to) = belongs_to_param.clone() {
                if old_node.belongs_to != new_belongs_to {
                    let _ = edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("belongsTo".to_string()),
                            old_node.belongs_to.as_ref().map(|b| serde_json::json!(b)),
                            new_belongs_to.as_ref().map(|b| serde_json::json!(b)),
                            None,
                            true,
                        )
                        .await;
                }
            }
        }

        Ok(crate::graphql::types::graph_node::GraphNode::from(node))
    }

    /// Update layer properties (name, colors, etc.)
    async fn update_layer_properties(
        &self,
        ctx: &Context<'_>,
        id: i32,
        name: Option<String>,
        properties: Option<crate::graphql::types::scalars::JSON>,
    ) -> Result<crate::graphql::types::layer::Layer> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        // Fetch current layer to get old values
        use crate::database::entities::graph_layers::Entity as Layers;

        let old_layer = Layers::find_by_id(id).one(&context.db).await?;

        // Update the layer
        let layer = graph_service
            .update_layer_properties(id, name.clone(), properties.clone())
            .await
            .map_err(|e| Error::new(format!("Failed to update layer properties: {}", e)))?;

        // Create graph edits for changed fields
        if let Some(old_layer) = old_layer {
            if let Some(new_name) = &name {
                if &old_layer.name != new_name {
                    let _ = edit_service
                        .create_edit(
                            old_layer.graph_id,
                            "layer".to_string(),
                            old_layer.layer_id.clone(),
                            "update".to_string(),
                            Some("name".to_string()),
                            Some(serde_json::json!(old_layer.name)),
                            Some(serde_json::json!(new_name)),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_properties) = &properties {
                let old_props = old_layer
                    .properties
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok());

                tracing::debug!(
                    "Layer properties update - old_props: {:?}, new_properties: {:?}",
                    old_props,
                    new_properties
                );
                tracing::debug!(
                    "Properties are equal: {}",
                    old_props.as_ref() == Some(new_properties)
                );

                if old_props.as_ref() != Some(new_properties) {
                    tracing::info!("Creating edit for layer properties change");
                    let _ = edit_service
                        .create_edit(
                            old_layer.graph_id,
                            "layer".to_string(),
                            old_layer.layer_id.clone(),
                            "update".to_string(),
                            Some("properties".to_string()),
                            old_props,
                            Some(new_properties.clone()),
                            None,
                            true,
                        )
                        .await;
                }
            }
        }

        Ok(crate::graphql::types::layer::Layer::from(layer))
    }

    /// Add a new node to a graph
    async fn add_graph_node(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        id: String,
        label: Option<String>,
        layer: Option<String>,
        is_partition: bool,
        belongs_to: Option<String>,
        weight: Option<f64>,
        attrs: Option<crate::graphql::types::scalars::JSON>,
    ) -> Result<crate::graphql::types::graph_node::GraphNode> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        // Create the new node
        let node = graph_service
            .add_graph_node(
                graph_id,
                id.clone(),
                label.clone(),
                layer.clone(),
                is_partition,
                belongs_to.clone(),
                weight,
                attrs.clone(),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to add graph node: {}", e)))?;

        // Create edit record for the new node
        let node_data = serde_json::json!({
            "id": id,
            "label": label,
            "layer": layer,
            "is_partition": is_partition,
            "belongs_to": belongs_to,
            "weight": weight,
            "attrs": attrs,
        });

        let _ = edit_service
            .create_edit(
                graph_id,
                "node".to_string(),
                id.clone(),
                "create".to_string(),
                None,
                None,
                Some(node_data),
                None,
                true,
            )
            .await;

        Ok(crate::graphql::types::graph_node::GraphNode::from(node))
    }

    /// Add a new edge to a graph
    async fn add_graph_edge(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        id: String,
        source: String,
        target: String,
        label: Option<String>,
        layer: Option<String>,
        weight: Option<f64>,
        attrs: Option<crate::graphql::types::scalars::JSON>,
    ) -> Result<crate::graphql::types::graph_edge::GraphEdge> {
        let context = ctx.data::<GraphQLContext>()?;
        let edit_service = GraphEditService::new(context.db.clone());

        use crate::database::entities::graph_edges::{
            ActiveModel as GraphEdgeActiveModel, Entity as GraphEdges,
        };
        use sea_orm::{ActiveValue::Set, EntityTrait};

        // Create the new edge
        let now = chrono::Utc::now();
        let edge_model = GraphEdgeActiveModel {
            id: Set(id.clone()),
            graph_id: Set(graph_id),
            source: Set(source.clone()),
            target: Set(target.clone()),
            label: Set(label.clone()),
            layer: Set(layer.clone()),
            weight: Set(weight),
            attrs: Set(attrs.clone()),
            datasource_id: Set(None),
            comment: Set(None),
            created_at: Set(now),
        };

        GraphEdges::insert(edge_model)
            .exec_without_returning(&context.db)
            .await
            .map_err(|e| Error::new(format!("Failed to insert graph edge: {}", e)))?;

        // Create edit record for the new edge
        let edge_data = serde_json::json!({
            "id": id,
            "source": source,
            "target": target,
            "label": label,
            "layer": layer,
            "weight": weight,
            "attrs": attrs,
        });

        let _ = edit_service
            .create_edit(
                graph_id,
                "edge".to_string(),
                id.clone(),
                "create".to_string(),
                None,
                None,
                Some(edge_data),
                None,
                true,
            )
            .await;

        // Fetch the inserted edge to return
        use crate::database::entities::graph_edges::Column as EdgeColumn;
        use sea_orm::{ColumnTrait, QueryFilter};

        let edge = GraphEdges::find()
            .filter(EdgeColumn::GraphId.eq(graph_id))
            .filter(EdgeColumn::Id.eq(&id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Failed to fetch inserted edge"))?;

        Ok(crate::graphql::types::graph_edge::GraphEdge::from(edge))
    }

    /// Delete an edge from a graph
    async fn delete_graph_edge(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        edge_id: String,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let edit_service = GraphEditService::new(context.db.clone());

        use crate::database::entities::graph_edges::{Column as EdgeColumn, Entity as GraphEdges};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        // Fetch current edge to get old values for edit record
        let old_edge = GraphEdges::find()
            .filter(EdgeColumn::GraphId.eq(graph_id))
            .filter(EdgeColumn::Id.eq(&edge_id))
            .one(&context.db)
            .await?;

        if let Some(old_edge) = old_edge {
            // Create edit record for the deletion
            let edge_data = serde_json::json!({
                "id": old_edge.id,
                "source": old_edge.source,
                "target": old_edge.target,
                "label": old_edge.label,
                "layer": old_edge.layer,
                "weight": old_edge.weight,
                "attrs": old_edge.attrs,
            });

            let _ = edit_service
                .create_edit(
                    graph_id,
                    "edge".to_string(),
                    edge_id.clone(),
                    "delete".to_string(),
                    None,
                    Some(edge_data),
                    None,
                    None,
                    true,
                )
                .await;

            // Delete the edge
            GraphEdges::delete_many()
                .filter(EdgeColumn::GraphId.eq(graph_id))
                .filter(EdgeColumn::Id.eq(&edge_id))
                .exec(&context.db)
                .await
                .map_err(|e| Error::new(format!("Failed to delete graph edge: {}", e)))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a node from a graph
    async fn delete_graph_node(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        node_id: String,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        // Fetch current node to get old values for edit record
        let old_node = graph_service
            .delete_graph_node(graph_id, node_id.clone())
            .await
            .map_err(|e| Error::new(format!("Failed to delete graph node: {}", e)))?;

        // Create edit record for the deletion
        let node_data = serde_json::json!({
            "id": old_node.id,
            "label": old_node.label,
            "layer": old_node.layer,
            "is_partition": old_node.is_partition,
            "belongs_to": old_node.belongs_to,
            "weight": old_node.weight,
            "attrs": old_node.attrs,
        });

        let _ = edit_service
            .create_edit(
                graph_id,
                "node".to_string(),
                node_id,
                "delete".to_string(),
                None,
                Some(node_data),
                None,
                None,
                true,
            )
            .await;

        Ok(true)
    }

    /// Bulk update graph nodes and layers in a single transaction
    async fn bulk_update_graph_data(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        nodes: Option<Vec<crate::graphql::types::graph_node::GraphNodeUpdateInput>>,
        layers: Option<Vec<crate::graphql::types::layer::LayerUpdateInput>>,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        use crate::database::entities::graph_layers::Entity as Layers;
        use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as GraphNodes};
        use sea_orm::{ColumnTrait, QueryFilter};

        // Update nodes
        if let Some(node_updates) = nodes {
            for node_update in node_updates {
                // Fetch current node to get old values
                let old_node = GraphNodes::find()
                    .filter(NodeColumn::GraphId.eq(graph_id))
                    .filter(NodeColumn::Id.eq(&node_update.node_id))
                    .one(&context.db)
                    .await?;

                // Update the node
                let _ = graph_service
                    .update_graph_node(
                        graph_id,
                        node_update.node_id.clone(),
                        node_update.label.clone(),
                        node_update.layer.clone(),
                        node_update.attrs.clone(),
                        None, // belongs_to not supported in bulk update
                    )
                    .await
                    .map_err(|e| Error::new(format!("Failed to update graph node: {}", e)))?;

                // Create graph edits for each changed field
                if let Some(old_node) = old_node {
                    if let Some(new_label) = &node_update.label {
                        if old_node.label.as_ref() != Some(new_label) {
                            let _ = edit_service
                                .create_edit(
                                    graph_id,
                                    "node".to_string(),
                                    node_update.node_id.clone(),
                                    "update".to_string(),
                                    Some("label".to_string()),
                                    old_node.label.as_ref().map(|l| serde_json::json!(l)),
                                    Some(serde_json::json!(new_label)),
                                    None,
                                    true,
                                )
                                .await;
                        }
                    }

                    if let Some(new_layer) = &node_update.layer {
                        let old_layer_value = old_node.layer.clone().unwrap_or_default();
                        if &old_layer_value != new_layer {
                            let _ = edit_service
                                .create_edit(
                                    graph_id,
                                    "node".to_string(),
                                    node_update.node_id.clone(),
                                    "update".to_string(),
                                    Some("layer".to_string()),
                                    if old_layer_value.is_empty() {
                                        None
                                    } else {
                                        Some(serde_json::json!(old_layer_value))
                                    },
                                    Some(serde_json::json!(new_layer)),
                                    None,
                                    true,
                                )
                                .await;
                        }
                    }

                    if let Some(new_attrs) = &node_update.attrs {
                        if old_node.attrs.as_ref() != Some(new_attrs) {
                            let _ = edit_service
                                .create_edit(
                                    graph_id,
                                    "node".to_string(),
                                    node_update.node_id.clone(),
                                    "update".to_string(),
                                    Some("attrs".to_string()),
                                    old_node.attrs.clone(),
                                    Some(new_attrs.clone()),
                                    None,
                                    true,
                                )
                                .await;
                        }
                    }
                }
            }
        }

        // Update layers
        if let Some(layer_updates) = layers {
            for layer_update in layer_updates {
                // Fetch current layer to get old values
                let old_layer = Layers::find_by_id(layer_update.id).one(&context.db).await?;

                // Update the layer
                let _ = graph_service
                    .update_layer_properties(
                        layer_update.id,
                        layer_update.name.clone(),
                        layer_update.properties.clone(),
                    )
                    .await
                    .map_err(|e| Error::new(format!("Failed to update layer properties: {}", e)))?;

                // Create graph edits for changed fields
                if let Some(old_layer) = old_layer {
                    if let Some(new_name) = &layer_update.name {
                        if &old_layer.name != new_name {
                            let _ = edit_service
                                .create_edit(
                                    old_layer.graph_id,
                                    "layer".to_string(),
                                    old_layer.layer_id.clone(),
                                    "update".to_string(),
                                    Some("name".to_string()),
                                    Some(serde_json::json!(old_layer.name)),
                                    Some(serde_json::json!(new_name)),
                                    None,
                                    true,
                                )
                                .await;
                        }
                    }

                    if let Some(new_properties) = &layer_update.properties {
                        let old_props = old_layer
                            .properties
                            .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok());

                        if old_props.as_ref() != Some(new_properties) {
                            let _ = edit_service
                                .create_edit(
                                    old_layer.graph_id,
                                    "layer".to_string(),
                                    old_layer.layer_id.clone(),
                                    "update".to_string(),
                                    Some("properties".to_string()),
                                    old_props,
                                    Some(new_properties.clone()),
                                    None,
                                    true,
                                )
                                .await;
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    /// Execute a DAG node (builds graph from upstream data sources)
    async fn execute_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
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

        // Create executor and execute the node with all its upstream dependencies
        let executor = DagExecutor::new(context.db.clone());

        executor
            .execute_with_dependencies(project_id, plan.id, &node_id, &nodes, &edges)
            .await
            .map_err(|e| Error::new(format!("Failed to execute node: {}", e)))?;

        executor
            .execute_affected_nodes(project_id, plan.id, &node_id, &nodes, &edges)
            .await
            .map_err(|e| Error::new(format!("Failed to execute downstream nodes: {}", e)))?;

        Ok(NodeExecutionResult {
            success: true,
            message: format!(
                "Node {} executed successfully; downstream graphs refreshed",
                node_id
            ),
            node_id,
        })
    }

    /// Export a node's output (graph export to various formats)
    async fn export_node_output(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
    ) -> Result<ExportNodeOutputResult> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan for this project
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        // Get the output node
        let output_node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Output node not found"))?;

        // Parse node config to get renderTarget and outputPath
        let config: serde_json::Value = serde_json::from_str(&output_node.config_json)
            .map_err(|e| Error::new(format!("Failed to parse node config: {}", e)))?;

        let render_target = config
            .get("renderTarget")
            .and_then(|v| v.as_str())
            .unwrap_or("GML");

        let output_path = config.get("outputPath").and_then(|v| v.as_str());

        // Get project name for default filename
        let project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        // Generate filename
        let extension = get_extension_for_format(render_target);
        let filename = output_path
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.{}", project.name, extension));

        // Find the upstream GraphNode connected to this OutputNode
        let edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;

        let upstream_node_id = edges
            .iter()
            .find(|e| e.target_node_id == node_id)
            .map(|e| e.source_node_id.clone())
            .ok_or_else(|| Error::new("No upstream graph connected to output node"))?;

        // Get all nodes and edges for DAG execution
        let all_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;

        let edge_tuples: Vec<(String, String)> = edges
            .iter()
            .map(|e| (e.source_node_id.clone(), e.target_node_id.clone()))
            .collect();

        // Execute the upstream GraphNode and its dependencies to ensure graph is built
        let executor = DagExecutor::new(context.db.clone());
        executor
            .execute_with_dependencies(
                project_id,
                plan.id,
                &upstream_node_id,
                &all_nodes,
                &edge_tuples,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to execute graph: {}", e)))?;

        // Get the built graph from the graphs table using the GraphNode's ID
        use crate::database::entities::graphs;
        let graph_model = graphs::Entity::find()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .filter(graphs::Column::NodeId.eq(&upstream_node_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new(format!("Graph not found for node {}", upstream_node_id)))?;

        // Build Graph object from the DAG-built graph
        use crate::services::GraphService;
        let graph_service = GraphService::new(context.db.clone());
        let graph = graph_service
            .build_graph_from_dag_graph(graph_model.id)
            .await
            .map_err(|e| Error::new(format!("Failed to build graph: {}", e)))?;

        // Export the graph
        let export_service = ExportService::new(context.db.clone());
        let content = export_service
            .export_to_string(&graph, &parse_export_format(render_target)?)
            .map_err(|e| Error::new(format!("Export failed: {}", e)))?;

        // Encode as base64
        use base64::Engine;
        let encoded_content = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());

        // Get MIME type
        let mime_type = get_mime_type_for_format(render_target);

        Ok(ExportNodeOutputResult {
            success: true,
            message: format!("Successfully exported {} as {}", filename, render_target),
            content: encoded_content,
            filename,
            mime_type,
        })
    }

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
            .map_err(|e| Error::new(format!("Failed to create graph edit: {}", e)))?;

        Ok(GraphEdit::from(edit))
    }

    /// Replay all unapplied edits for a graph
    async fn replay_graph_edits(&self, ctx: &Context<'_>, graph_id: i32) -> Result<ReplaySummary> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = GraphEditService::new(context.db.clone());

        let summary = service
            .replay_graph_edits(graph_id)
            .await
            .map_err(|e| Error::new(format!("Failed to replay graph edits: {}", e)))?;

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
            .map_err(|e| Error::new(format!("Failed to clear graph edits: {}", e)))?;

        Ok(true)
    }

    /// Clear execution state for all nodes in a project (keeps edits, config, and datasources)
    async fn clear_project_execution(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<ExecutionActionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        // Delete all graphs generated by this project's plan nodes
        // This will cascade delete graph_nodes, graph_edges, and graph_layers
        let delete_result = graphs::Entity::delete_many()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .exec(&context.db)
            .await?;

        let graphs_deleted = delete_result.rows_affected;

        // Delete all datasources imported for this project
        let datasources_result = datasources::Entity::delete_many()
            .filter(datasources::Column::ProjectId.eq(project_id))
            .exec(&context.db)
            .await?;

        let datasources_deleted = datasources_result.rows_affected;

        Ok(ExecutionActionResult {
            success: true,
            message: format!(
                "Cleared execution state: deleted {} graphs and {} datasources. Configuration and edits preserved.",
                graphs_deleted, datasources_deleted
            ),
        })
    }

    /// Stop plan execution in progress
    async fn stop_plan_execution(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<ExecutionActionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        // Find all graphs in processing or pending state for this project
        let processing_graphs = graphs::Entity::find()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .filter(
                graphs::Column::ExecutionState
                    .eq(ExecutionState::Processing.as_str())
                    .or(graphs::Column::ExecutionState.eq(ExecutionState::Pending.as_str())),
            )
            .all(&context.db)
            .await?;

        let mut stopped_count = 0;

        // Update each graph to not_started state
        for graph in processing_graphs {
            let mut active: graphs::ActiveModel = graph.into();
            active = active.set_state(ExecutionState::NotStarted);
            active.error_message = Set(Some("Execution stopped by user".to_string()));
            active.update(&context.db).await?;
            stopped_count += 1;
        }

        // Find all datasources in processing or pending state for this project
        let processing_datasources = datasources::Entity::find()
            .filter(datasources::Column::ProjectId.eq(project_id))
            .filter(
                datasources::Column::ExecutionState
                    .eq(ExecutionState::Processing.as_str())
                    .or(datasources::Column::ExecutionState.eq(ExecutionState::Pending.as_str())),
            )
            .all(&context.db)
            .await?;

        // Update each datasource to not_started state
        for datasource in processing_datasources {
            let mut active: datasources::ActiveModel = datasource.into();
            active.execution_state = Set(ExecutionState::NotStarted.as_str().to_string());
            active.error_message = Set(Some("Execution stopped by user".to_string()));
            active.update(&context.db).await?;
            stopped_count += 1;
        }

        let message = if stopped_count > 0 {
            format!("Stopped {} running executions", stopped_count)
        } else {
            "No executions were in progress".to_string()
        };

        Ok(ExecutionActionResult {
            success: true,
            message,
        })
    }
}

// Helper function to get file extension for render target
fn get_extension_for_format(format: &str) -> &str {
    match format {
        "DOT" => "dot",
        "GML" => "gml",
        "JSON" => "json",
        "CSV" => "csv",
        "CSVNodes" => "csv",
        "CSVEdges" => "csv",
        "PlantUML" => "puml",
        "Mermaid" => "mermaid",
        _ => "txt",
    }
}

// Helper function to get MIME type for render target
fn get_mime_type_for_format(format: &str) -> String {
    match format {
        "DOT" => "text/vnd.graphviz",
        "GML" => "text/plain",
        "JSON" => "application/json",
        "CSV" | "CSVNodes" | "CSVEdges" => "text/csv",
        "PlantUML" => "text/plain",
        "Mermaid" => "text/plain",
        _ => "text/plain",
    }
    .to_string()
}

// Helper function to parse render target string to ExportFileType enum
fn parse_export_format(format: &str) -> Result<crate::plan::ExportFileType> {
    use crate::plan::ExportFileType;
    match format {
        "DOT" => Ok(ExportFileType::DOT),
        "GML" => Ok(ExportFileType::GML),
        "JSON" => Ok(ExportFileType::JSON),
        "PlantUML" => Ok(ExportFileType::PlantUML),
        "Mermaid" => Ok(ExportFileType::Mermaid),
        "CSVNodes" => Ok(ExportFileType::CSVNodes),
        "CSVEdges" => Ok(ExportFileType::CSVEdges),
        "CSV" => Ok(ExportFileType::CSVNodes), // Default CSV to nodes
        _ => Err(Error::new(format!("Unsupported export format: {}", format))),
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

#[derive(SimpleObject)]
pub struct ExecutionActionResult {
    pub success: bool,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct ExportNodeOutputResult {
    pub success: bool,
    pub message: String,
    pub content: String, // Base64 encoded
    pub filename: String,
    pub mime_type: String,
}
