use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::database::entities::{
    data_sources, graph_edges, graph_nodes, graphs, layers, library_sources, plan_dag_edges,
    plan_dag_nodes, plans, project_collaborators, projects, user_sessions, users,
};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::graph::Graph;
use crate::graphql::types::plan::Plan;
use crate::graphql::types::plan_dag::DataSourceReference;
use crate::graphql::types::plan_dag::{
    DataSourceExecutionMetadata, GraphExecutionMetadata, PlanDag, PlanDagEdge, PlanDagInput,
    PlanDagMetadata, PlanDagNode, PlanDagNodeType, ValidationResult,
};
use crate::graphql::types::project::Project;
use crate::graphql::types::sample_project::SampleProject;
use crate::graphql::types::{
    DataSource, DataSourcePreview, GraphEdgePreview, GraphEdit, GraphNodePreview, GraphPreview,
    Layer, LibrarySource, ProjectCollaborator, TableColumn, TableRow, User, UserSession,
};
use crate::services::{
    graph_edit_service::GraphEditService, sample_project_service::SampleProjectService,
};

pub struct Query;

#[Object]
impl Query {
    /// List bundled sample projects
    async fn sample_projects(&self) -> Result<Vec<SampleProject>> {
        Ok(SampleProjectService::list_available_projects()
            .into_iter()
            .map(SampleProject::from)
            .collect())
    }

    /// Get all projects
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let projects = projects::Entity::find().all(&context.db).await?;

        Ok(projects.into_iter().map(Project::from).collect())
    }

    /// Get a specific project by ID
    async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(id).one(&context.db).await?;

        Ok(project.map(Project::from))
    }

    /// Get a specific plan by ID
    async fn plan(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plan = plans::Entity::find_by_id(id).one(&context.db).await?;

        Ok(plan.map(Plan::from))
    }

    // TODO: Fix this function after data model refactoring
    // /// Search nodes by label
    // async fn search_nodes(&self, ctx: &Context<'_>, project_id: i32, query: String) -> Result<Vec<Node>> {
    //     let context = ctx.data::<GraphQLContext>()?;
    //     let nodes = nodes::Entity::find()
    //         .filter(
    //             nodes::Column::ProjectId.eq(project_id)
    //                 .and(nodes::Column::Label.contains(&query))
    //         )
    //         .all(&context.db)
    //         .await?;

    //     Ok(nodes.into_iter().map(Node::from).collect())
    // }

    /// Get Plan DAG for a project
    async fn get_plan_dag(&self, ctx: &Context<'_>, project_id: i32) -> Result<Option<PlanDag>> {
        tracing::info!("getPlanDag called for project_id: {}", project_id);

        let context = ctx.data::<GraphQLContext>()?;
        tracing::debug!("Got GraphQL context");

        // Verify project exists
        tracing::debug!("Querying project by id...");
        let project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;
        tracing::debug!("Project found: {}", project.name);

        // Find or create a plan for this project
        let plan = match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
        {
            Some(plan) => plan,
            None => {
                // No plan exists yet - return empty Plan DAG
                tracing::debug!("No plan found for project, returning empty Plan DAG");
                let metadata = PlanDagMetadata {
                    version: "1.0".to_string(),
                    name: Some(format!("{} Plan DAG", project.name)),
                    description: project.description.clone(),
                    created: Some(project.created_at.to_rfc3339()),
                    last_modified: Some(project.updated_at.to_rfc3339()),
                    author: None,
                };
                return Ok(Some(PlanDag {
                    version: metadata.version.clone(),
                    nodes: vec![],
                    edges: vec![],
                    metadata,
                }));
            }
        };

        // Get Plan DAG nodes (not graph nodes!)
        tracing::debug!("Querying Plan DAG nodes for plan_id: {}...", plan.id);
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;
        tracing::debug!("Found {} Plan DAG nodes", dag_nodes.len());

        // Get Plan DAG edges (not graph edges!)
        tracing::debug!("Querying Plan DAG edges for plan_id: {}...", plan.id);
        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;
        tracing::debug!("Found {} Plan DAG edges", dag_edges.len());

        // Convert database models to GraphQL types and populate execution metadata
        tracing::debug!("Converting {} nodes to GraphQL format...", dag_nodes.len());
        let mut nodes: Vec<PlanDagNode> = Vec::new();

        for dag_node in dag_nodes {
            let mut node = PlanDagNode::from(dag_node.clone());

            // Populate execution metadata based on node type
            match node.node_type {
                PlanDagNodeType::DataSource => {
                    // Try to extract dataSourceId from config
                    if let Ok(config) =
                        serde_json::from_str::<serde_json::Value>(&dag_node.config_json)
                    {
                        if let Some(data_source_id) = config
                            .get("dataSourceId")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32)
                        {
                            // Query the data_source
                            if let Ok(Some(ds)) = data_sources::Entity::find_by_id(data_source_id)
                                .one(&context.db)
                                .await
                            {
                                // Map status to execution_state
                                let execution_state = match ds.status.as_str() {
                                    "active" => "completed",
                                    "processing" => "processing",
                                    "error" => "error",
                                    _ => "not_started",
                                }
                                .to_string();

                                node.datasource_execution = Some(DataSourceExecutionMetadata {
                                    data_source_id: ds.id,
                                    filename: ds.filename.clone(),
                                    status: ds.status.clone(),
                                    processed_at: ds.processed_at.map(|d| d.to_rfc3339()),
                                    execution_state,
                                    error_message: ds.error_message.clone(),
                                });
                            }
                        }
                    }
                }
                PlanDagNodeType::Graph => {
                    // Query the graph by node_id
                    if let Ok(Some(graph)) = graphs::Entity::find()
                        .filter(graphs::Column::ProjectId.eq(project_id))
                        .filter(graphs::Column::NodeId.eq(&dag_node.id))
                        .one(&context.db)
                        .await
                    {
                        node.graph_execution = Some(GraphExecutionMetadata {
                            graph_id: graph.id,
                            node_count: graph.node_count,
                            edge_count: graph.edge_count,
                            execution_state: graph.execution_state.clone(),
                            computed_date: graph.computed_date.map(|d| d.to_rfc3339()),
                            error_message: graph.error_message.clone(),
                        });
                    }
                }
                _ => {
                    // Other node types don't have execution metadata yet
                }
            }

            nodes.push(node);
        }
        tracing::debug!("Converted {} nodes with execution metadata", nodes.len());

        tracing::debug!("Converting {} edges to GraphQL format...", dag_edges.len());
        let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();
        tracing::debug!("Converted {} edges", edges.len());

        // Create metadata
        tracing::debug!("Creating metadata...");
        let metadata = PlanDagMetadata {
            version: plan.version.to_string(),
            name: Some(plan.name.clone()),
            description: None,
            created: Some(plan.created_at.to_rfc3339()),
            last_modified: Some(plan.updated_at.to_rfc3339()),
            author: None,
        };

        tracing::info!(
            "getPlanDag completed successfully for project_id: {}",
            project_id
        );
        Ok(Some(PlanDag {
            version: metadata.version.clone(),
            nodes,
            edges,
            metadata,
        }))
    }

    /// Validate a Plan DAG structure
    async fn validate_plan_dag(
        &self,
        _ctx: &Context<'_>,
        plan_dag: PlanDagInput,
    ) -> Result<ValidationResult> {
        // TODO: Implement comprehensive Plan DAG validation
        // For now, return a basic validation that always passes

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic validation: check for orphaned nodes, cycles, etc.
        let node_ids: std::collections::HashSet<String> =
            plan_dag.nodes.iter().filter_map(|n| n.id.clone()).collect();

        // Check for edges referencing non-existent nodes
        for edge in &plan_dag.edges {
            let edge_id_str = edge.id.clone().unwrap_or_else(|| "<unknown>".to_string());
            if !node_ids.contains(&edge.source) {
                errors.push(crate::graphql::types::ValidationError {
                    node_id: None,
                    edge_id: edge.id.clone(),
                    error_type: crate::graphql::types::ValidationErrorType::InvalidConnection,
                    message: format!(
                        "Edge {} references non-existent source node {}",
                        edge_id_str, edge.source
                    ),
                });
            }
            if !node_ids.contains(&edge.target) {
                errors.push(crate::graphql::types::ValidationError {
                    node_id: None,
                    edge_id: edge.id.clone(),
                    error_type: crate::graphql::types::ValidationErrorType::InvalidConnection,
                    message: format!(
                        "Edge {} references non-existent target node {}",
                        edge_id_str, edge.target
                    ),
                });
            }
        }

        // Check for isolated nodes (nodes with no connections)
        for node in &plan_dag.nodes {
            // Skip nodes without IDs (they will be generated)
            if let Some(ref node_id) = node.id {
                let has_connections = plan_dag
                    .edges
                    .iter()
                    .any(|e| &e.source == node_id || &e.target == node_id);
                if !has_connections && plan_dag.nodes.len() > 1 {
                    warnings.push(crate::graphql::types::ValidationWarning {
                        node_id: node.id.clone(),
                        edge_id: None,
                        warning_type: crate::graphql::types::ValidationWarningType::UnusedOutput,
                        message: format!("Node {} has no connections", node_id),
                    });
                }
            }
        }

        Ok(crate::graphql::types::plan_dag::ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    // Authentication and User Management Queries

    /// Get current user from session
    async fn me(&self, ctx: &Context<'_>, session_id: String) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find active session
        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::SessionId.eq(&session_id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .one(&context.db)
            .await?;

        if let Some(session) = session {
            // Check if session is not expired
            if session.expires_at > chrono::Utc::now() {
                let user = users::Entity::find_by_id(session.user_id)
                    .one(&context.db)
                    .await?;
                Ok(user.map(User::from))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get user by ID
    async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;
        let user = users::Entity::find_by_id(id).one(&context.db).await?;

        Ok(user.map(User::from))
    }

    /// Get user by username
    async fn user_by_username(&self, ctx: &Context<'_>, username: String) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;
        let user = users::Entity::find()
            .filter(users::Column::Username.eq(&username))
            .one(&context.db)
            .await?;

        Ok(user.map(User::from))
    }

    /// Get user by email
    async fn user_by_email(&self, ctx: &Context<'_>, email: String) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&email))
            .one(&context.db)
            .await?;

        Ok(user.map(User::from))
    }

    // Project Collaboration Queries

    /// Get all collaborators for a project
    async fn project_collaborators(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborators = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(collaborators
            .into_iter()
            .map(ProjectCollaborator::from)
            .collect())
    }

    /// Get specific collaborator
    async fn project_collaborator(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<Option<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborator = project_collaborators::Entity::find_by_id(id)
            .one(&context.db)
            .await?;

        Ok(collaborator.map(ProjectCollaborator::from))
    }

    /// Get user's collaborations (projects they have access to)
    async fn user_collaborations(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
    ) -> Result<Vec<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(collaborations
            .into_iter()
            .map(ProjectCollaborator::from)
            .collect())
    }

    /// Check if user has access to project
    async fn user_project_access(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&context.db)
            .await?;

        Ok(collaboration.map(ProjectCollaborator::from))
    }

    // User Presence Queries

    // REMOVED: project_online_users and user_presence queries - user presence now handled via WebSocket only
    // Real-time presence data is available through the WebSocket collaboration system at /ws/collaboration

    /// Get all active sessions for a user
    async fn user_sessions(&self, ctx: &Context<'_>, user_id: i32) -> Result<Vec<UserSession>> {
        let context = ctx.data::<GraphQLContext>()?;
        let sessions = user_sessions::Entity::find()
            .filter(user_sessions::Column::UserId.eq(user_id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(sessions.into_iter().map(UserSession::from).collect())
    }

    /// Get session by ID
    async fn session(&self, ctx: &Context<'_>, session_id: String) -> Result<Option<UserSession>> {
        let context = ctx.data::<GraphQLContext>()?;
        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::SessionId.eq(&session_id))
            .one(&context.db)
            .await?;

        Ok(session.map(UserSession::from))
    }

    /// Get DataSource by ID
    async fn data_source(&self, ctx: &Context<'_>, id: i32) -> Result<Option<DataSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source = data_sources::Entity::find_by_id(id)
            .one(&context.db)
            .await?;

        Ok(data_source.map(DataSource::from))
    }

    /// Get all DataSources for a project
    async fn data_sources(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<DataSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_sources_list = data_sources::Entity::find()
            .filter(data_sources::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;

        Ok(data_sources_list
            .into_iter()
            .map(DataSource::from)
            .collect())
    }

    /// Get all library sources
    async fn library_sources(&self, ctx: &Context<'_>) -> Result<Vec<LibrarySource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let sources = library_sources::Entity::find().all(&context.db).await?;

        Ok(sources.into_iter().map(LibrarySource::from).collect())
    }

    /// Get a single library source by ID
    async fn library_source(&self, ctx: &Context<'_>, id: i32) -> Result<Option<LibrarySource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = library_sources::Entity::find_by_id(id)
            .one(&context.db)
            .await?;

        Ok(source.map(LibrarySource::from))
    }

    /// Get all Graphs for a project
    async fn graphs(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Graph>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graphs_list = graphs::Entity::find()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;

        Ok(graphs_list.into_iter().map(Graph::from).collect())
    }

    /// Get Graph by ID
    async fn graph(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Graph>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph = graphs::Entity::find_by_id(id).one(&context.db).await?;

        Ok(graph.map(Graph::from))
    }

    /// Get available DataSources for selection in DAG editor
    async fn available_data_sources(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<DataSourceReference>> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_sources_list = data_sources::Entity::find()
            .filter(data_sources::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;

        Ok(data_sources_list
            .into_iter()
            .map(DataSourceReference::from)
            .collect())
    }

    /// Generate download URL for raw DataSource file
    async fn download_data_source_raw(&self, ctx: &Context<'_>, id: i32) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        let _data_source = data_sources::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("DataSource not found"))?;

        // Generate a temporary download URL (in a real implementation, this would be a signed URL)
        let download_url = format!("/api/data-sources/{}/download/raw", id);
        Ok(download_url)
    }

    /// Generate download URL for processed DataSource JSON
    async fn download_data_source_json(&self, ctx: &Context<'_>, id: i32) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        let _data_source = data_sources::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("DataSource not found"))?;

        // Generate a temporary download URL (in a real implementation, this would be a signed URL)
        let download_url = format!("/api/data-sources/{}/download/json", id);
        Ok(download_url)
    }

    // Pipeline Preview Queries

    /// Get DataSource preview with table data
    async fn datasource_preview(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
        #[graphql(default = 100)] limit: u64,
        #[graphql(default = 0)] offset: u64,
    ) -> Result<Option<DataSourcePreview>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan_dag_node to get the config
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Plan not found for project"))?;

        let dag_node = plan_dag_nodes::Entity::find_by_id(&node_id)
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Node not found"))?;

        // Parse config to get dataSourceId
        let config: serde_json::Value = serde_json::from_str(&dag_node.config_json)
            .map_err(|e| Error::new(format!("Failed to parse node config: {}", e)))?;

        let data_source_id = config
            .get("dataSourceId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| Error::new("Node config does not have dataSourceId"))?;

        // Query the data_sources table
        let data_source = data_sources::Entity::find_by_id(data_source_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("DataSource not found"))?;

        // Parse the graph_json field
        let graph_data: serde_json::Value = serde_json::from_str(&data_source.graph_json)
            .map_err(|e| Error::new(format!("Failed to parse graph JSON: {}", e)))?;

        // Determine what to extract based on data_type
        let (columns, rows, total_rows) = match data_source.data_type.as_str() {
            "nodes" => {
                let nodes_array = graph_data
                    .get("nodes")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| Error::new("Graph JSON does not contain nodes array"))?;

                // Build columns from first node's keys
                let columns =
                    if let Some(first_node) = nodes_array.first().and_then(|v| v.as_object()) {
                        first_node
                            .keys()
                            .map(|key| TableColumn {
                                name: key.clone(),
                                data_type: "string".to_string(),
                                nullable: true,
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                // Build rows from nodes with pagination
                let paginated_nodes: Vec<&serde_json::Value> = nodes_array
                    .iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect();

                let rows: Vec<TableRow> = paginated_nodes
                    .into_iter()
                    .enumerate()
                    .map(|(idx, node)| TableRow {
                        row_number: (offset as i32) + (idx as i32) + 1,
                        data: node.clone(),
                    })
                    .collect();

                (columns, rows, nodes_array.len() as i32)
            }
            "edges" => {
                let edges_array = graph_data
                    .get("edges")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| Error::new("Graph JSON does not contain edges array"))?;

                // Build columns from first edge's keys
                let columns =
                    if let Some(first_edge) = edges_array.first().and_then(|v| v.as_object()) {
                        first_edge
                            .keys()
                            .map(|key| TableColumn {
                                name: key.clone(),
                                data_type: "string".to_string(),
                                nullable: true,
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                // Build rows from edges with pagination
                let paginated_edges: Vec<&serde_json::Value> = edges_array
                    .iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect();

                let rows: Vec<TableRow> = paginated_edges
                    .into_iter()
                    .enumerate()
                    .map(|(idx, edge)| TableRow {
                        row_number: (offset as i32) + (idx as i32) + 1,
                        data: edge.clone(),
                    })
                    .collect();

                (columns, rows, edges_array.len() as i32)
            }
            _ => {
                return Err(Error::new(format!(
                    "Unsupported data type: {}",
                    data_source.data_type
                )));
            }
        };

        // Determine execution state based on data_source status
        let execution_state = match data_source.status.as_str() {
            "active" => "completed",
            "processing" => "processing",
            "error" => "error",
            _ => "not_started",
        };

        Ok(Some(DataSourcePreview {
            node_id,
            datasource_id: data_source.id,
            name: data_source.name.clone(),
            file_path: data_source.filename.clone(),
            file_type: data_source.data_type.clone(),
            total_rows,
            columns,
            rows,
            import_date: data_source.processed_at.map(|d| d.to_rfc3339()),
            execution_state: execution_state.to_string(),
            error_message: data_source.error_message.clone(),
        }))
    }

    /// Get Graph preview with nodes and edges
    async fn graph_preview(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
    ) -> Result<Option<GraphPreview>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find graph by project_id and node_id
        let graph = graphs::Entity::find()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .filter(graphs::Column::NodeId.eq(&node_id))
            .one(&context.db)
            .await?;

        let graph = match graph {
            Some(g) => g,
            None => return Ok(None),
        };

        // Get all nodes for this graph
        let nodes = graph_nodes::Entity::find()
            .filter(graph_nodes::Column::GraphId.eq(graph.id))
            .all(&context.db)
            .await?;

        // Get all edges for this graph
        let edges = graph_edges::Entity::find()
            .filter(graph_edges::Column::GraphId.eq(graph.id))
            .all(&context.db)
            .await?;

        // Get all layers for this graph
        let db_layers = layers::Entity::find()
            .filter(layers::Column::GraphId.eq(graph.id))
            .all(&context.db)
            .await?;

        // Convert to preview format
        let node_previews: Vec<GraphNodePreview> =
            nodes.into_iter().map(GraphNodePreview::from).collect();

        let edge_previews: Vec<GraphEdgePreview> =
            edges.into_iter().map(GraphEdgePreview::from).collect();

        let layer_previews: Vec<Layer> = db_layers.into_iter().map(Layer::from).collect();

        Ok(Some(GraphPreview {
            node_id,
            graph_id: graph.id,
            name: graph.name,
            nodes: node_previews,
            edges: edge_previews,
            layers: layer_previews,
            node_count: graph.node_count,
            edge_count: graph.edge_count,
            execution_state: graph.execution_state,
            computed_date: graph.computed_date.map(|d| d.to_rfc3339()),
            error_message: graph.error_message,
        }))
    }

    /// Get all edits for a graph
    async fn graph_edits(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        unapplied_only: Option<bool>,
    ) -> Result<Vec<GraphEdit>> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = GraphEditService::new(context.db.clone());

        let edits = service
            .get_edits_for_graph(graph_id, unapplied_only.unwrap_or(false))
            .await
            .map_err(|e| Error::new(format!("Failed to get graph edits: {}", e)))?;

        Ok(edits.into_iter().map(GraphEdit::from).collect())
    }

    /// Get edit count for a graph
    async fn graph_edit_count(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        unapplied_only: Option<bool>,
    ) -> Result<i32> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = GraphEditService::new(context.db.clone());

        let count = service
            .get_edit_count(graph_id, unapplied_only.unwrap_or(false))
            .await
            .map_err(|e| Error::new(format!("Failed to get edit count: {}", e)))?;

        Ok(count as i32)
    }
}
