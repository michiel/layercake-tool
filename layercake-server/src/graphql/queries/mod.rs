use async_graphql::*;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Statement,
};

use layercake_core::database::entities::{
    data_sets, graph_data, graph_data_edges, graph_data_nodes, layer_aliases, plan_dag_edges,
    plan_dag_nodes, plans, project_collaborators, projections, sequences, stories, user_sessions,
    users,
};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::code_analysis::CodeAnalysisProfile;
use crate::graphql::types::graph::Graph;
use crate::graphql::types::plan::Plan;
use crate::graphql::types::plan_dag::DataSetReference;
use crate::graphql::types::plan_dag::{PlanDag, PlanDagInput, ValidationResult};
use crate::graphql::types::project::Project;
use crate::graphql::types::sample_project::SampleProject;
use crate::graphql::types::{
    DataSet, DataSetPreview, GraphData, GraphEdgePreview, GraphEdit, GraphNodePreview,
    GraphPreview, Layer, LayerAlias, LibraryItem, LibraryItemFilterInput, ProjectCollaborator,
    ProjectLayer, Sequence, Story, SystemSetting, TableColumn, TableRow, User, UserFilter,
    UserSession,
};
use crate::graphql::types::{GraphPage, GraphSummary};
use layercake_core::services::{
    graph_edit_service::GraphEditService, library_item_service::LibraryItemFilter,
    library_item_service::LibraryItemService, sample_project_service::SampleProjectService,
    GraphService,
};
use layercake_genai::entities::tags as acquisition_tags;
use std::collections::HashMap;

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
    async fn projects(&self, ctx: &Context<'_>, tags: Option<Vec<String>>) -> Result<Vec<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let projects = context
            .app
            .list_projects_filtered(tags)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(projects.into_iter().map(Project::from).collect())
    }

    /// Get a specific project by ID
    async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = context
            .app
            .get_project(id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(project.map(Project::from))
    }

    async fn code_analysis_profiles(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<CodeAnalysisProfile>> {
        let context = ctx.data::<GraphQLContext>()?;
        let profiles = context
            .app
            .code_analysis_service()
            .list(project_id)
            .await
            .map_err(|e| StructuredError::service("CodeAnalysisService::list", e))?;
        Ok(profiles
            .into_iter()
            .map(CodeAnalysisProfile::from)
            .collect())
    }

    async fn code_analysis_profile(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<CodeAnalysisProfile>> {
        let context = ctx.data::<GraphQLContext>()?;
        let profile = context
            .app
            .code_analysis_service()
            .get(id)
            .await
            .map_err(|e| StructuredError::service("CodeAnalysisService::get", e))?;
        Ok(profile.map(CodeAnalysisProfile::from))
    }

    async fn graph_summary(&self, ctx: &Context<'_>, dataset_id: i32) -> Result<GraphSummary> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .data_set_service()
            .get_graph_summary(dataset_id)
            .await
            .map_err(StructuredError::from_core_error)?;
        Ok(GraphSummary::from(summary))
    }

    async fn graph_page(
        &self,
        ctx: &Context<'_>,
        dataset_id: i32,
        limit: i32,
        offset: i32,
        layers: Option<Vec<String>>,
    ) -> Result<GraphPage> {
        let context = ctx.data::<GraphQLContext>()?;
        let page = context
            .app
            .data_set_service()
            .get_graph_page(
                dataset_id,
                limit.max(0) as usize,
                offset.max(0) as usize,
                layers,
            )
            .await
            .map_err(StructuredError::from_core_error)?;
        Ok(GraphPage::from(page))
    }

    /// Get aggregate statistics for a project (for overview page)
    async fn project_stats(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<crate::graphql::types::ProjectStats> {
        let context = ctx.data::<GraphQLContext>()?;

        // Get document stats
        let files = context
            .app
            .data_acquisition_service()
            .list_files(project_id)
            .await
            .map_err(|e| StructuredError::service("DataAcquisitionService::list_files", e))?;

        let total_files = files.len() as i32;
        let mut indexed_count = 0;

        for file in &files {
            let file_id = uuid::Uuid::parse_str(&file.id.to_string()).map_err(|e| {
                StructuredError::validation("fileId", format!("Invalid UUID: {}", e))
            })?;
            if context
                .app
                .data_acquisition_service()
                .is_file_indexed(project_id, file_id)
                .await
                .unwrap_or(false)
            {
                indexed_count += 1;
            }
        }

        let document_stats = crate::graphql::types::DocumentStats {
            total: total_files,
            indexed: indexed_count,
            not_indexed: total_files - indexed_count,
        };

        // Get knowledge base stats
        let kb_status = context
            .app
            .data_acquisition_service()
            .knowledge_base_status(project_id)
            .await
            .map_err(|e| {
                StructuredError::service("DataAcquisitionService::knowledge_base_status", e)
            })?;

        let kb_stats = crate::graphql::types::KnowledgeBaseStats {
            file_count: kb_status.file_count as i32,
            chunk_count: kb_status.chunk_count as i32,
            last_indexed_at: kb_status.last_indexed_at,
        };

        // Get dataset stats
        let datasets = context
            .app
            .list_data_sets(project_id)
            .await
            .map_err(StructuredError::from_core_error)?;

        let total_datasets = datasets.len() as i32;
        let mut by_type = std::collections::HashMap::new();

        for ds in datasets {
            if ds.node_count.unwrap_or(0) > 0 {
                *by_type.entry("nodes".to_string()).or_insert(0) += 1;
            }
            if ds.edge_count.unwrap_or(0) > 0 {
                *by_type.entry("edges".to_string()).or_insert(0) += 1;
            }
            if ds.layer_count.unwrap_or(0) > 0 {
                *by_type.entry("layers".to_string()).or_insert(0) += 1;
            }
            if ds.node_count.unwrap_or(0) == 0
                && ds.edge_count.unwrap_or(0) == 0
                && ds.layer_count.unwrap_or(0) == 0
            {
                *by_type.entry("empty".to_string()).or_insert(0) += 1;
            }
        }

        let dataset_stats = crate::graphql::types::DatasetStats {
            total: total_datasets,
            by_type,
        };

        Ok(crate::graphql::types::ProjectStats {
            project_id,
            documents: document_stats,
            knowledge_base: kb_stats,
            datasets: dataset_stats,
        })
    }

    /// Get a specific plan by ID
    async fn plan(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plan = context
            .app
            .get_plan(id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(plan.map(Plan::from))
    }

    /// List all plans for a project
    async fn plans(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plans = context
            .app
            .list_plans(Some(project_id))
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(plans.into_iter().map(Plan::from).collect())
    }

    // Story and Sequence Queries

    /// Get all stories for a project
    async fn stories(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Story>> {
        let context = ctx.data::<GraphQLContext>()?;
        let story_list = stories::Entity::find()
            .filter(stories::Column::ProjectId.eq(project_id))
            .order_by_desc(stories::Column::UpdatedAt)
            .all(&context.db)
            .await?;

        Ok(story_list.into_iter().map(Story::from).collect())
    }

    /// Get a specific story by ID
    async fn story(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Story>> {
        let context = ctx.data::<GraphQLContext>()?;
        let story = stories::Entity::find_by_id(id).one(&context.db).await?;

        Ok(story.map(Story::from))
    }

    /// Get all sequences for a story
    async fn sequences(&self, ctx: &Context<'_>, story_id: i32) -> Result<Vec<Sequence>> {
        let context = ctx.data::<GraphQLContext>()?;
        let sequence_list = sequences::Entity::find()
            .filter(sequences::Column::StoryId.eq(story_id))
            .order_by_desc(sequences::Column::UpdatedAt)
            .all(&context.db)
            .await?;

        Ok(sequence_list.into_iter().map(Sequence::from).collect())
    }

    /// Get a specific sequence by ID
    async fn sequence(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Sequence>> {
        let context = ctx.data::<GraphQLContext>()?;
        let sequence = sequences::Entity::find_by_id(id).one(&context.db).await?;

        Ok(sequence.map(Sequence::from))
    }

    /// List project-wide layers (project palette)
    #[graphql(name = "projectLayers")]
    async fn project_layers(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<ProjectLayer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let layers = context
            .app
            .graph_service()
            .list_project_layers(project_id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(layers.into_iter().map(ProjectLayer::from).collect())
    }

    /// List layers referenced in nodes/edges that are missing from the project palette
    #[graphql(name = "missingLayers")]
    async fn missing_layers(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<String>> {
        let context = ctx.data::<GraphQLContext>()?;
        let missing = context
            .app
            .graph_service()
            .missing_layers(project_id)
            .await
            .map_err(StructuredError::from_core_error)?;
        Ok(missing)
    }

    /// List all layer aliases for a project
    #[graphql(name = "listLayerAliases")]
    async fn list_layer_aliases(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<LayerAlias>> {
        let context = ctx.data::<GraphQLContext>()?;
        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;

        Ok(aliases.into_iter().map(LayerAlias::from).collect())
    }

    /// Get layer aliases for a specific target layer
    #[graphql(name = "getLayerAliases")]
    async fn get_layer_aliases(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        target_layer_id: i32,
    ) -> Result<Vec<LayerAlias>> {
        let context = ctx.data::<GraphQLContext>()?;
        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .filter(layer_aliases::Column::TargetLayerId.eq(target_layer_id))
            .all(&context.db)
            .await?;

        Ok(aliases.into_iter().map(LayerAlias::from).collect())
    }

    /// List runtime-editable system settings
    async fn system_settings(&self, ctx: &Context<'_>) -> Result<Vec<SystemSetting>> {
        let context = ctx.data::<GraphQLContext>()?;
        let settings = context
            .system_settings
            .list_settings()
            .await
            .map_err(StructuredError::from_core_error)?;
        Ok(settings.into_iter().map(SystemSetting::from).collect())
    }

    /// Fetch a single runtime setting by key
    async fn system_setting(&self, ctx: &Context<'_>, key: String) -> Result<SystemSetting> {
        let context = ctx.data::<GraphQLContext>()?;
        let setting = context
            .system_settings
            .get_setting(&key)
            .await
            .map_err(StructuredError::from_core_error)?;
        Ok(SystemSetting::from(setting))
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
    async fn get_plan_dag(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> Result<Option<PlanDag>> {
        let context = ctx.data::<GraphQLContext>()?;
        let snapshot = context
            .app
            .load_plan_dag(project_id, plan_id)
            .await
            .map_err(StructuredError::from_core_error)?
            .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

        Ok(Some(PlanDag::from(snapshot)))
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

    /// Find user by various filters (id, email, username, or session_id)
    async fn find_user(&self, ctx: &Context<'_>, filter: UserFilter) -> Result<Option<User>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Handle session_id lookup (equivalent to old 'me' query)
        if let Some(session_id) = filter.session_id {
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
                    return Ok(user.map(User::from));
                }
            }
            return Ok(None);
        }

        // Build query based on provided filters
        let mut query = users::Entity::find();

        if let Some(id) = filter.id {
            query = query.filter(users::Column::Id.eq(id));
        }
        if let Some(email) = filter.email {
            query = query.filter(users::Column::Email.eq(email));
        }
        if let Some(username) = filter.username {
            query = query.filter(users::Column::Username.eq(username));
        }

        let user = query.one(&context.db).await?;
        Ok(user.map(User::from))
    }

    /// Get current user from session
    #[graphql(
        deprecation = "Use find_user(filter: { session_id: \"...\" }) instead for better flexibility and consistency."
    )]
    async fn me(&self, ctx: &Context<'_>, session_id: String) -> Result<Option<User>> {
        self.find_user(
            ctx,
            UserFilter {
                id: None,
                email: None,
                username: None,
                session_id: Some(session_id),
            },
        )
        .await
    }

    /// Get user by ID
    #[graphql(
        deprecation = "Use find_user(filter: { id: ... }) instead for better flexibility and consistency."
    )]
    async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<Option<User>> {
        self.find_user(
            ctx,
            UserFilter {
                id: Some(id),
                email: None,
                username: None,
                session_id: None,
            },
        )
        .await
    }

    /// Get user by username
    #[graphql(
        deprecation = "Use find_user(filter: { username: \"...\" }) instead for better flexibility and consistency."
    )]
    async fn user_by_username(&self, ctx: &Context<'_>, username: String) -> Result<Option<User>> {
        self.find_user(
            ctx,
            UserFilter {
                id: None,
                email: None,
                username: Some(username),
                session_id: None,
            },
        )
        .await
    }

    /// Get user by email
    #[graphql(
        deprecation = "Use find_user(filter: { email: \"...\" }) instead for better flexibility and consistency."
    )]
    async fn user_by_email(&self, ctx: &Context<'_>, email: String) -> Result<Option<User>> {
        self.find_user(
            ctx,
            UserFilter {
                id: None,
                email: Some(email),
                username: None,
                session_id: None,
            },
        )
        .await
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

    /// Get DataSet by ID
    async fn data_set(&self, ctx: &Context<'_>, id: i32) -> Result<Option<DataSet>> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .get_data_set(id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(summary.map(DataSet::from))
    }

    /// Get all DataSets for a project
    async fn data_sets(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<DataSet>> {
        let context = ctx.data::<GraphQLContext>()?;
        let summaries = context
            .app
            .list_data_sets(project_id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(summaries.into_iter().map(DataSet::from).collect())
    }

    /// Get GraphData by ID (unified query for datasets and computed graphs)
    async fn graph_data(&self, ctx: &Context<'_>, id: i32) -> Result<Option<GraphData>> {
        let context = ctx.data::<GraphQLContext>()?;

        use sea_orm::EntityTrait;
        let model = graph_data::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data::Entity::find_by_id", e))?;

        Ok(model.map(GraphData::from))
    }

    /// Get all GraphData for a project (both datasets and computed graphs)
    async fn graph_data_list(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        source_type: Option<String>,
    ) -> Result<Vec<GraphData>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Legacy rows may contain non-JSON annotation blobs that break deserialization.
        // Repair invalid entries to keep the listing query resilient for projections and other consumers.
        repair_invalid_graph_data_annotations(&context.db, project_id)
            .await
            .map_err(|e| StructuredError::database("repair_invalid_graph_data_annotations", e))?;

        use sea_orm::{EntityTrait, QueryFilter};
        let mut query =
            graph_data::Entity::find().filter(graph_data::Column::ProjectId.eq(project_id));

        // Optionally filter by source_type ("dataset" or "computed")
        if let Some(st) = source_type {
            query = query.filter(graph_data::Column::SourceType.eq(st));
        }

        let models = query
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data::Entity::find", e))?;

        Ok(models.into_iter().map(GraphData::from).collect())
    }

    /// Get GraphData by DAG node ID (for computed graphs)
    async fn graph_data_by_dag_node(
        &self,
        ctx: &Context<'_>,
        dag_node_id: String,
    ) -> Result<Option<GraphData>> {
        let context = ctx.data::<GraphQLContext>()?;

        use sea_orm::{EntityTrait, QueryFilter};
        let model = graph_data::Entity::find()
            .filter(graph_data::Column::DagNodeId.eq(dag_node_id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data::Entity::find", e))?;

        Ok(model.map(GraphData::from))
    }

    /// Get all library items with optional filtering
    async fn library_items(
        &self,
        ctx: &Context<'_>,
        filter: Option<LibraryItemFilterInput>,
    ) -> Result<Vec<LibraryItem>> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());

        let params = filter
            .map(|f| LibraryItemFilter {
                item_types: f
                    .types
                    .map(|types| types.into_iter().map(|t| t.as_str().to_string()).collect()),
                tags: f.tags,
                search: f.search_query,
            })
            .unwrap_or_default();

        let items = service
            .list(params)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(items.into_iter().map(LibraryItem::from).collect())
    }

    /// Get a single library item by ID
    async fn library_item(&self, ctx: &Context<'_>, id: i32) -> Result<Option<LibraryItem>> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());
        let item = service
            .get(id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(item.map(LibraryItem::from))
    }

    /// Get all Graphs for a project
    async fn graphs(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Graph>> {
        let context = ctx.data::<GraphQLContext>()?;
        let mut results = Vec::new();

        // Repair any invalid annotations stored as plain strings to avoid JSON decode errors
        repair_invalid_graph_data_annotations(&context.db, project_id)
            .await
            .map_err(|e| StructuredError::database("repair_invalid_graph_data_annotations", e))?;

        let gd_list = graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::SourceType.eq("computed"))
            .all(&context.db)
            .await?;
        results.extend(gd_list.into_iter().map(Graph::from));

        Ok(results)
    }

    /// Get Graph by ID
    async fn graph(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Graph>> {
        let context = ctx.data::<GraphQLContext>()?;
        let gd = graph_data::Entity::find_by_id(id).one(&context.db).await?;
        Ok(gd.map(Graph::from))
    }

    /// Get available DataSets for selection in DAG editor
    async fn available_data_sets(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<DataSetReference>> {
        let context = ctx.data::<GraphQLContext>()?;
        let summaries = context
            .app
            .available_data_sets(project_id)
            .await
            .map_err(StructuredError::from_core_error)?;

        Ok(summaries.into_iter().map(DataSetReference::from).collect())
    }

    /// Generate download URL for raw DataSet file

    async fn download_data_set_raw(&self, ctx: &Context<'_>, id: i32) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        let _data_set = data_sets::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("data_sets::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("DataSet", id))?;

        // Generate a temporary download URL (in a real implementation, this would be a signed URL)
        let download_url = format!("/api/data-sources/{}/download/raw", id);
        Ok(download_url)
    }

    /// Generate download URL for processed DataSet JSON
    async fn download_data_set_json(&self, ctx: &Context<'_>, id: i32) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        let _data_set = data_sets::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("data_sets::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("DataSet", id))?;

        // Generate a temporary download URL (in a real implementation, this would be a signed URL)
        let download_url = format!("/api/data-sources/{}/download/json", id);
        Ok(download_url)
    }

    // Pipeline Preview Queries

    /// Get DataSet preview with table data
    async fn dataset_preview(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
        #[graphql(default = 100)] limit: u64,
        #[graphql(default = 0)] offset: u64,
    ) -> Result<Option<DataSetPreview>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find the plan_dag_node to get the config
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("plans::Entity::find (ProjectId)", e))?
            .ok_or_else(|| StructuredError::not_found("Plan for project", project_id))?;

        let dag_node = plan_dag_nodes::Entity::find_by_id(&node_id)
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_nodes::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Plan DAG node", &node_id))?;

        // Parse config to get dataSetId
        let config: serde_json::Value =
            serde_json::from_str(&dag_node.config_json).map_err(|e| {
                StructuredError::bad_request(format!("Failed to parse node config: {}", e))
            })?;

        let data_set_id = config
            .get("dataSetId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| {
                StructuredError::bad_request("Node config does not have dataSetId".to_string())
            })?;

        // Query the data_sets table
        let data_set = data_sets::Entity::find_by_id(data_set_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("data_sets::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("DataSet", data_set_id))?;

        // Parse the graph_json field
        let graph_data: serde_json::Value =
            serde_json::from_str(&data_set.graph_json).map_err(|e| {
                StructuredError::bad_request(format!("Failed to parse graph JSON: {}", e))
            })?;

        enum PreviewSection<'a> {
            Nodes(&'a Vec<serde_json::Value>),
            Edges(&'a Vec<serde_json::Value>),
            Layers(&'a Vec<serde_json::Value>),
            Empty,
        }

        let nodes_array = graph_data.get("nodes").and_then(|v| v.as_array());
        let edges_array = graph_data.get("edges").and_then(|v| v.as_array());
        let layers_array = graph_data.get("layers").and_then(|v| v.as_array());

        let section = if nodes_array.is_some_and(|arr| !arr.is_empty()) {
            PreviewSection::Nodes(nodes_array.unwrap())
        } else if edges_array.is_some_and(|arr| !arr.is_empty()) {
            PreviewSection::Edges(edges_array.unwrap())
        } else if layers_array.is_some_and(|arr| !arr.is_empty()) {
            PreviewSection::Layers(layers_array.unwrap())
        } else if let Some(nodes) = nodes_array {
            PreviewSection::Nodes(nodes)
        } else if let Some(edges) = edges_array {
            PreviewSection::Edges(edges)
        } else if let Some(layers) = layers_array {
            PreviewSection::Layers(layers)
        } else {
            PreviewSection::Empty
        };

        let (section_label, section_records) = match section {
            PreviewSection::Nodes(records) => ("nodes", Some(records)),
            PreviewSection::Edges(records) => ("edges", Some(records)),
            PreviewSection::Layers(records) => ("layers", Some(records)),
            PreviewSection::Empty => ("nodes", None),
        };

        let columns = section_records
            .and_then(|records| records.first())
            .and_then(|value| value.as_object())
            .map(|object| {
                object
                    .keys()
                    .map(|key| TableColumn {
                        name: key.clone(),
                        data_type: "string".to_string(),
                        nullable: true,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let total_rows = section_records
            .map(|records| records.len() as i32)
            .unwrap_or(0);

        let paginated_records: Vec<&serde_json::Value> = section_records
            .map(|records| {
                records
                    .iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect()
            })
            .unwrap_or_default();

        let rows: Vec<TableRow> = paginated_records
            .into_iter()
            .enumerate()
            .map(|(idx, value)| TableRow {
                row_number: (offset as i32) + (idx as i32) + 1,
                data: value.clone(),
            })
            .collect();

        // Determine execution state based on data_set status
        let execution_state = match data_set.status.as_str() {
            "active" => "completed",
            "processing" => "processing",
            "error" => "error",
            _ => "not_started",
        };

        Ok(Some(DataSetPreview {
            node_id,
            dataset_id: data_set.id,
            name: data_set.name.clone(),
            file_path: data_set.filename.clone(),
            file_type: section_label.to_string(),
            total_rows,
            columns,
            rows,
            import_date: data_set.processed_at.map(|d| d.to_rfc3339()),
            execution_state: execution_state.to_string(),
            error_message: data_set.error_message.clone(),
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

        // Try graph_data first by dag_node_id
        let graph_data_model = graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::DagNodeId.eq(node_id.clone()))
            .one(&context.db)
            .await?;

        let (
            graph_id,
            name,
            base_annotations,
            node_previews,
            edge_previews,
            layer_previews,
            node_count,
            edge_count,
            execution_state,
            computed_date,
            error_message,
        ) = if let Some(gd) = graph_data_model {
            let nodes = graph_data_nodes::Entity::find()
                .filter(graph_data_nodes::Column::GraphDataId.eq(gd.id))
                .all(&context.db)
                .await?;
            let edges = graph_data_edges::Entity::find()
                .filter(graph_data_edges::Column::GraphDataId.eq(gd.id))
                .all(&context.db)
                .await?;
            let palette_layers = GraphService::new(context.db.clone())
                .get_all_resolved_layers(project_id)
                .await
                .unwrap_or_default();
            let palette_map: HashMap<String, layercake_core::graph::Layer> = palette_layers
                .into_iter()
                .map(|layer| (layer.id.clone(), layer))
                .collect();

            let mut layers = Vec::new();
            for (idx, layer_id) in nodes
                .iter()
                .filter_map(|n| n.layer.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .enumerate()
            {
                let palette_entry = palette_map.get(&layer_id);
                layers.push(Layer {
                    id: -(idx as i32 + 1),
                    graph_id: gd.id,
                    layer_id: layer_id.clone(),
                    name: palette_entry
                        .map(|p| p.label.clone())
                        .unwrap_or_else(|| layer_id.clone()),
                    background_color: palette_entry.map(|p| p.background_color.clone()),
                    text_color: palette_entry.map(|p| p.text_color.clone()),
                    border_color: palette_entry.map(|p| p.border_color.clone()),
                    alias: palette_entry.and_then(|p| p.alias.clone()),
                    comment: None,
                    properties: None,
                    dataset_id: palette_entry.and_then(|p| p.dataset),
                });
            }

            let annotations = gd
                .annotations
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string()));

            (
                gd.id,
                gd.name,
                annotations,
                nodes.into_iter().map(GraphNodePreview::from).collect(),
                edges.into_iter().map(GraphEdgePreview::from).collect(),
                layers,
                gd.node_count,
                gd.edge_count,
                gd.status,
                gd.computed_date.map(|d| d.to_rfc3339()),
                gd.error_message,
            )
        } else {
            return Ok(None);
        };

        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![node_id.clone()];
        let mut annotation_history = Vec::new();
        if let Some(text) = base_annotations.clone() {
            annotation_history.push(text);
        }

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }

            if let Some(gd) = graph_data::Entity::find()
                .filter(graph_data::Column::DagNodeId.eq(&current))
                .one(&context.db)
                .await?
            {
                if let Some(text) = gd
                    .annotations
                    .as_ref()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                {
                    annotation_history.push(text);
                }
            }

            let upstream = plan_dag_edges::Entity::find()
                .filter(plan_dag_edges::Column::TargetNodeId.eq(&current))
                .all(&context.db)
                .await?;

            for edge in upstream {
                stack.push(edge.source_node_id);
            }
        }
        let annotations = if annotation_history.is_empty() {
            None
        } else {
            Some(annotation_history.join("\n\n"))
        };

        Ok(Some(GraphPreview {
            node_id,
            graph_id,
            name,
            annotations,
            nodes: node_previews,
            edges: edge_previews,
            layers: layer_previews,
            node_count,
            edge_count,
            execution_state,
            computed_date,
            error_message,
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
            .map_err(|e| StructuredError::service("GraphEditService::get_edits_for_graph", e))?;

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
            .map_err(|e| StructuredError::service("GraphEditService::get_edit_count", e))?;

        Ok(count as i32)
    }

    // Chat History Queries

    /// Get chat sessions for a project
    async fn chat_sessions(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        #[graphql(default = false)] include_archived: bool,
        #[graphql(default = 50)] limit: u64,
        #[graphql(default = 0)] offset: u64,
    ) -> Result<Vec<crate::graphql::types::ChatSession>> {
        use layercake_core::services::chat_history_service::ChatHistoryService;
        let context = ctx.data::<GraphQLContext>()?;
        let service = ChatHistoryService::new(context.db.clone());

        let sessions = service
            .list_sessions(project_id, None, include_archived, limit, offset)
            .await
            .map_err(|e| StructuredError::service("ChatHistoryService::list_sessions", e))?;

        Ok(sessions
            .into_iter()
            .map(crate::graphql::types::ChatSession::from)
            .collect())
    }

    /// Get a specific chat session
    async fn chat_session(
        &self,
        ctx: &Context<'_>,
        session_id: String,
    ) -> Result<Option<crate::graphql::types::ChatSession>> {
        use layercake_core::services::chat_history_service::ChatHistoryService;
        let context = ctx.data::<GraphQLContext>()?;
        let service = ChatHistoryService::new(context.db.clone());

        let session = service
            .get_session(&session_id)
            .await
            .map_err(|e| StructuredError::service("ChatHistoryService::get_session", e))?;

        Ok(session.map(crate::graphql::types::ChatSession::from))
    }

    /// Get message history for a chat session
    async fn chat_history(
        &self,
        ctx: &Context<'_>,
        session_id: String,
        #[graphql(default = 100)] limit: u64,
        #[graphql(default = 0)] offset: u64,
    ) -> Result<Vec<crate::graphql::types::ChatMessage>> {
        use layercake_core::services::chat_history_service::ChatHistoryService;
        let context = ctx.data::<GraphQLContext>()?;
        let service = ChatHistoryService::new(context.db.clone());

        let messages = service
            .get_history(&session_id, limit, offset)
            .await
            .map_err(|e| StructuredError::service("ChatHistoryService::get_history", e))?;

        Ok(messages
            .into_iter()
            .map(crate::graphql::types::ChatMessage::from)
            .collect())
    }

    /// Get message count for a session
    async fn chat_message_count(&self, ctx: &Context<'_>, session_id: String) -> Result<i32> {
        use layercake_core::services::chat_history_service::ChatHistoryService;
        let context = ctx.data::<GraphQLContext>()?;
        let service = ChatHistoryService::new(context.db.clone());

        let count = service
            .get_message_count(&session_id)
            .await
            .map_err(|e| StructuredError::service("ChatHistoryService::get_message_count", e))?;

        Ok(count as i32)
    }

    // MCP Agent Queries

    /// List MCP agents for a project
    async fn mcp_agents(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<crate::graphql::types::McpAgent>> {
        use layercake_core::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let service = McpAgentService::new(context.db.clone());

        let agents = service
            .list_agents(project_id)
            .await
            .map_err(|e| StructuredError::service("McpAgentService::list_agents", e))?;

        Ok(agents
            .into_iter()
            .map(crate::graphql::types::McpAgent::from)
            .collect())
    }
    /// Fetch per-project knowledge base status
    async fn knowledge_base_status(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<crate::graphql::types::KnowledgeBaseStatus> {
        let context = ctx.data::<GraphQLContext>()?;
        let status = context
            .app
            .data_acquisition_service()
            .knowledge_base_status(project_id)
            .await
            .map_err(|e| {
                StructuredError::service("DataAcquisitionService::knowledge_base_status", e)
            })?;

        Ok(crate::graphql::types::KnowledgeBaseStatus {
            project_id: status.project_id,
            file_count: status.file_count,
            chunk_count: status.chunk_count,
            status: status.status,
            last_indexed_at: status.last_indexed_at,
            embedding_provider: status.embedding_provider,
            embedding_model: status.embedding_model,
        })
    }

    /// List stored files for a project
    async fn data_acquisition_files(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Vec<crate::graphql::types::ProjectFile>> {
        let context = ctx.data::<GraphQLContext>()?;
        let files = context
            .app
            .data_acquisition_service()
            .list_files(project_id)
            .await
            .map_err(|e| StructuredError::service("DataAcquisitionService::list_files", e))?;

        let mut result = Vec::new();
        for file in files {
            let file_id = uuid::Uuid::parse_str(&file.id.to_string()).map_err(|e| {
                StructuredError::validation("fileId", format!("Invalid UUID: {}", e))
            })?;
            let indexed = context
                .app
                .data_acquisition_service()
                .is_file_indexed(project_id, file_id)
                .await
                .unwrap_or(false);

            result.push(crate::graphql::types::ProjectFile {
                id: file.id.to_string(),
                filename: file.filename,
                media_type: file.media_type,
                size_bytes: file.size_bytes,
                checksum: file.checksum,
                created_at: file.created_at,
                tags: file.tags,
                indexed,
            });
        }

        Ok(result)
    }

    /// List tags optionally filtered by scope
    async fn data_acquisition_tags(
        &self,
        ctx: &Context<'_>,
        scope: Option<String>,
    ) -> Result<Vec<crate::graphql::types::TagView>> {
        let context = ctx.data::<GraphQLContext>()?;
        let db = context.app.db();
        let mut query =
            acquisition_tags::Entity::find().order_by_asc(acquisition_tags::Column::Name);
        if let Some(scope_value) = scope {
            query = query.filter(acquisition_tags::Column::Scope.eq(scope_value));
        }
        let tags = query
            .all(db)
            .await
            .map_err(|e| StructuredError::database("tags", e))?;

        Ok(tags
            .into_iter()
            .map(|model| crate::graphql::types::TagView {
                id: model.id.to_string(),
                name: model.name,
                scope: model.scope,
                color: model.color,
            })
            .collect())
    }

    // Projection Queries

    /// Get all projections for a project
    async fn projections(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: ID,
    ) -> Result<Vec<crate::graphql::types::Projection>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project_id_int = project_id
            .parse::<i32>()
            .map_err(|_| StructuredError::validation("projectId", "Invalid project ID"))?;

        let projections_list = projections::Entity::find()
            .filter(projections::Column::ProjectId.eq(project_id_int))
            .order_by_desc(projections::Column::UpdatedAt)
            .all(&context.db)
            .await?;

        Ok(projections_list
            .into_iter()
            .map(crate::graphql::types::Projection::from)
            .collect())
    }

    /// Get a specific projection by ID
    async fn projection(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<crate::graphql::types::Projection>> {
        let context = ctx.data::<GraphQLContext>()?;
        let projection_id = id
            .parse::<i32>()
            .map_err(|_| StructuredError::validation("id", "Invalid projection ID"))?;

        let projection = projections::Entity::find_by_id(projection_id)
            .one(&context.db)
            .await?;

        Ok(projection.map(crate::graphql::types::Projection::from))
    }

    /// Get projection graph data (nodes, edges, layers)
    #[graphql(name = "projectionGraph")]
    async fn projection_graph(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<crate::graphql::types::ProjectionGraph> {
        let context = ctx.data::<GraphQLContext>()?;
        let projection_id = id
            .parse::<i32>()
            .map_err(|_| StructuredError::validation("id", "Invalid projection ID"))?;

        // Get the projection
        let projection = projections::Entity::find_by_id(projection_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| StructuredError::not_found("Projection", projection_id))?;

        // Build and return the projection graph using the helper
        crate::graphql::types::projection::build_projection_graph(&context.db, projection.graph_id)
            .await
    }

    /// Get projection state for the 3D viewer
    #[graphql(name = "projectionState")]
    async fn projection_state(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<crate::graphql::types::ProjectionState>> {
        let context = ctx.data::<GraphQLContext>()?;
        let projection_id_str = id.to_string();
        let projection_id = id
            .parse::<i32>()
            .map_err(|_| StructuredError::validation("id", "Invalid projection ID"))?;

        // Get the projection
        let projection = projections::Entity::find_by_id(projection_id)
            .one(&context.db)
            .await?;

        Ok(projection.map(|p| crate::graphql::types::ProjectionState {
            projection_id: projection_id_str,
            projection_type: p.projection_type,
            state_json: p.settings_json.unwrap_or_else(|| serde_json::json!({})),
        }))
    }
}

/// Normalize legacy annotation payloads to valid JSON arrays so queries don't fail deserializing
async fn repair_invalid_graph_data_annotations(
    db: &DatabaseConnection,
    project_id: i32,
) -> Result<(), DbErr> {
    let backend = db.get_database_backend();

    // Use backend-specific JSON helpers to coerce invalid annotation payloads into valid arrays.
    // This avoids deserialization errors when legacy markdown strings were stored directly.
    let (update_sql, values) = match backend {
        DatabaseBackend::Postgres => {
            // Postgres enforces JSON validity at write time; nothing to repair.
            return Ok(());
        }
        DatabaseBackend::MySql => (
            "update graph_data set annotations = JSON_ARRAY(annotations) where project_id = ? and annotations is not null and JSON_VALID(annotations) = 0",
            vec![project_id.into()],
        ),
        DatabaseBackend::Sqlite => (
            "update graph_data set annotations = json_array(annotations) where project_id = ? and annotations is not null and json_valid(annotations) = 0",
            vec![project_id.into()],
        ),
    };

    db.execute(Statement::from_sql_and_values(backend, update_sql, values))
        .await?;

    Ok(())
}
