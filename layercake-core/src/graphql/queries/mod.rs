use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::database::entities::{projects, plans, nodes, edges, layers, plan_dag_nodes, plan_dag_edges, users, user_sessions, project_collaborators, data_sources};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, Plan, Node, Edge, Layer, PlanDag, PlanDagNode, PlanDagEdge, ValidationResult, PlanDagInput, User, ProjectCollaborator, DataSource};
use crate::graphql::types::plan_dag::DataSourceReference;

pub struct Query;

#[Object]
impl Query {
    /// Get all projects
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let projects = projects::Entity::find()
            .all(&context.db)
            .await?;
        
        Ok(projects.into_iter().map(Project::from).collect())
    }

    /// Get a specific project by ID
    async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await?;
        
        Ok(project.map(Project::from))
    }

    /// Get all plans for a project
    async fn plans(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plans = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;
        
        Ok(plans.into_iter().map(Plan::from).collect())
    }

    /// Get a specific plan by ID
    async fn plan(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plan = plans::Entity::find_by_id(id)
            .one(&context.db)
            .await?;
        
        Ok(plan.map(Plan::from))
    }

    /// Get all nodes for a project
    async fn nodes(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Node>> {
        let context = ctx.data::<GraphQLContext>()?;
        let nodes = nodes::Entity::find()
            .filter(nodes::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;
        
        Ok(nodes.into_iter().map(Node::from).collect())
    }

    /// Get all edges for a project
    async fn edges(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Edge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let edges = edges::Entity::find()
            .filter(edges::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;
        
        Ok(edges.into_iter().map(Edge::from).collect())
    }

    /// Get all layers for a project
    async fn layers(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Layer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let layers = layers::Entity::find()
            .filter(layers::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;
        
        Ok(layers.into_iter().map(Layer::from).collect())
    }

    /// Get complete graph data for a project
    async fn graph_data(&self, ctx: &Context<'_>, project_id: i32) -> Result<GraphData> {
        let nodes = self.nodes(ctx, project_id).await?;
        let edges = self.edges(ctx, project_id).await?;
        let layers = self.layers(ctx, project_id).await?;
        
        Ok(GraphData { nodes, edges, layers })
    }

    /// Search nodes by label
    async fn search_nodes(&self, ctx: &Context<'_>, project_id: i32, query: String) -> Result<Vec<Node>> {
        let context = ctx.data::<GraphQLContext>()?;
        let nodes = nodes::Entity::find()
            .filter(
                nodes::Column::ProjectId.eq(project_id)
                    .and(nodes::Column::Label.contains(&query))
            )
            .all(&context.db)
            .await?;

        Ok(nodes.into_iter().map(Node::from).collect())
    }

    /// Get Plan DAG for a project
    async fn get_plan_dag(&self, ctx: &Context<'_>, project_id: i32) -> Result<Option<PlanDag>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        // Get Plan DAG nodes for this project
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
            .all(&context.db)
            .await?;

        // Get Plan DAG edges for this project
        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(project_id))
            .all(&context.db)
            .await?;

        // Convert to GraphQL types
        let nodes: Vec<PlanDagNode> = dag_nodes.into_iter().map(PlanDagNode::from).collect();
        let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();

        // Create default metadata using project information
        let metadata = PlanDagMetadata {
            version: "1.0".to_string(),
            name: Some(format!("{} Plan DAG", project.name)),
            description: project.description.clone(),
            created: Some(project.created_at.to_rfc3339()),
            last_modified: Some(project.updated_at.to_rfc3339()),
            author: None,
        };

        Ok(Some(PlanDag {
            version: metadata.version.clone(),
            nodes,
            edges,
            metadata,
        }))
    }

    /// Validate a Plan DAG structure
    async fn validate_plan_dag(&self, _ctx: &Context<'_>, plan_dag: PlanDagInput) -> Result<ValidationResult> {
        // TODO: Implement comprehensive Plan DAG validation
        // For now, return a basic validation that always passes

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic validation: check for orphaned nodes, cycles, etc.
        let node_ids: std::collections::HashSet<String> = plan_dag.nodes.iter().map(|n| n.id.clone()).collect();

        // Check for edges referencing non-existent nodes
        for edge in &plan_dag.edges {
            if !node_ids.contains(&edge.source) {
                errors.push(crate::graphql::types::ValidationError {
                    node_id: None,
                    edge_id: Some(edge.id.clone()),
                    error_type: crate::graphql::types::ValidationErrorType::InvalidConnection,
                    message: format!("Edge {} references non-existent source node {}", edge.id, edge.source),
                });
            }
            if !node_ids.contains(&edge.target) {
                errors.push(crate::graphql::types::ValidationError {
                    node_id: None,
                    edge_id: Some(edge.id.clone()),
                    error_type: crate::graphql::types::ValidationErrorType::InvalidConnection,
                    message: format!("Edge {} references non-existent target node {}", edge.id, edge.target),
                });
            }
        }

        // Check for isolated nodes (nodes with no connections)
        for node in &plan_dag.nodes {
            let has_connections = plan_dag.edges.iter().any(|e| e.source == node.id || e.target == node.id);
            if !has_connections && plan_dag.nodes.len() > 1 {
                warnings.push(crate::graphql::types::ValidationWarning {
                    node_id: Some(node.id.clone()),
                    edge_id: None,
                    warning_type: crate::graphql::types::ValidationWarningType::UnusedOutput,
                    message: format!("Node {} has no connections", node.id),
                });
            }
        }

        Ok(ValidationResult {
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
        let user = users::Entity::find_by_id(id)
            .one(&context.db)
            .await?;

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
    async fn project_collaborators(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborators = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(collaborators.into_iter().map(ProjectCollaborator::from).collect())
    }

    /// Get specific collaborator
    async fn project_collaborator(&self, ctx: &Context<'_>, id: i32) -> Result<Option<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborator = project_collaborators::Entity::find_by_id(id)
            .one(&context.db)
            .await?;

        Ok(collaborator.map(ProjectCollaborator::from))
    }

    /// Get user's collaborations (projects they have access to)
    async fn user_collaborations(&self, ctx: &Context<'_>, user_id: i32) -> Result<Vec<ProjectCollaborator>> {
        let context = ctx.data::<GraphQLContext>()?;
        let collaborations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .all(&context.db)
            .await?;

        Ok(collaborations.into_iter().map(ProjectCollaborator::from).collect())
    }

    /// Check if user has access to project
    async fn user_project_access(&self, ctx: &Context<'_>, user_id: i32, project_id: i32) -> Result<Option<ProjectCollaborator>> {
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

        Ok(data_sources_list.into_iter().map(DataSource::from).collect())
    }

    /// Get available DataSources for selection in DAG editor
    async fn available_data_sources(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<DataSourceReference>> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_sources_list = data_sources::Entity::find()
            .filter(data_sources::Column::ProjectId.eq(project_id))
            .all(&context.db)
            .await?;

        Ok(data_sources_list.into_iter().map(DataSourceReference::from).collect())
    }

    /// Generate download URL for raw DataSource file
    async fn download_data_source_raw(&self, ctx: &Context<'_>, id: i32) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        let data_source = data_sources::Entity::find_by_id(id)
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
        let data_source = data_sources::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("DataSource not found"))?;

        // Generate a temporary download URL (in a real implementation, this would be a signed URL)
        let download_url = format!("/api/data-sources/{}/download/json", id);
        Ok(download_url)
    }
}

#[derive(SimpleObject)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
}