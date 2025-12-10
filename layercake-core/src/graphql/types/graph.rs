use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::database::entities::{graph_edges, graph_layers, graph_nodes};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::graph_edge::GraphEdge;
use crate::graphql::types::graph_node::GraphNode;
use crate::graphql::types::{Layer, Project};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Graph {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub node_id: String,
    pub execution_state: String,
    pub computed_date: Option<chrono::DateTime<chrono::Utc>>,
    pub source_hash: Option<String>,
    pub node_count: i32,
    pub edge_count: i32,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub annotations: Option<String>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl Graph {
    async fn project(&self, ctx: &Context<'_>) -> Result<Project> {
        let graphql_ctx = ctx
            .data::<GraphQLContext>()
            .map_err(|_| StructuredError::internal("GraphQL context not found"))?;

        use crate::database::entities::projects;
        use sea_orm::EntityTrait;

        let project = projects::Entity::find_by_id(self.project_id)
            .one(&graphql_ctx.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", self.project_id))?;

        Ok(Project::from(project))
    }

    #[graphql(name = "layers")]
    async fn graph_layers(&self, ctx: &Context<'_>) -> Result<Vec<Layer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_layers = graph_layers::Entity::find()
            .filter(graph_layers::Column::GraphId.eq(self.id))
            .all(&context.db)
            .await?;

        Ok(graph_layers.into_iter().map(Layer::from).collect())
    }

    async fn graph_nodes(&self, ctx: &Context<'_>) -> Result<Vec<GraphNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let nodes = graph_nodes::Entity::find()
            .filter(graph_nodes::Column::GraphId.eq(self.id))
            .all(&context.db)
            .await?;

        Ok(nodes.into_iter().map(GraphNode::from).collect())
    }

    async fn graph_edges(&self, ctx: &Context<'_>) -> Result<Vec<GraphEdge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let edges = graph_edges::Entity::find()
            .filter(graph_edges::Column::GraphId.eq(self.id))
            .all(&context.db)
            .await?;

        Ok(edges.into_iter().map(GraphEdge::from).collect())
    }
}

impl From<crate::database::entities::graphs::Model> for Graph {
    fn from(model: crate::database::entities::graphs::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            node_id: model.node_id,
            execution_state: model.execution_state,
            computed_date: model.computed_date,
            source_hash: model.source_hash,
            node_count: model.node_count,
            edge_count: model.edge_count,
            error_message: model.error_message,
            metadata: model.metadata,
            annotations: model.annotations,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Facade: Convert graph_data (with source_type="computed") to Graph
/// This allows Graph GraphQL type to work with the unified graph_data table
impl From<crate::database::entities::graph_data::Model> for Graph {
    fn from(model: crate::database::entities::graph_data::Model) -> Self {
        // Map graph_data status to legacy execution_state
        let execution_state = match model.status.as_str() {
            "pending" => "pending".to_string(),
            "processing" => "running".to_string(),
            "active" => "completed".to_string(),
            "error" => "failed".to_string(),
            _ => model.status.clone(),
        };

        // Serialize annotations back to JSON string for legacy compatibility
        let annotations = model.annotations.as_ref().and_then(|val| {
            serde_json::to_string(val).ok()
        });

        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            node_id: model.dag_node_id.unwrap_or_else(|| format!("graph-{}", model.id)),
            execution_state,
            computed_date: model.computed_date,
            source_hash: model.source_hash,
            node_count: model.node_count,
            edge_count: model.edge_count,
            error_message: model.error_message,
            metadata: model.metadata,
            annotations,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(InputObject)]
pub struct CreateGraphInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
}

#[derive(InputObject)]
pub struct UpdateGraphInput {
    pub name: Option<String>,
}

#[derive(InputObject)]
pub struct CreateLayerInput {
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    #[graphql(name = "layerId")]
    pub layer_id: String,
    pub name: String,
}

#[derive(SimpleObject)]
#[graphql(name = "GraphValidationResult")]
pub struct GraphValidationResult {
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "isValid")]
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    #[graphql(name = "nodeCount")]
    pub node_count: i32,
    #[graphql(name = "edgeCount")]
    pub edge_count: i32,
    #[graphql(name = "layerCount")]
    pub layer_count: i32,
    #[graphql(name = "checkedAt")]
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::app_context::GraphValidationSummary> for GraphValidationResult {
    fn from(summary: crate::app_context::GraphValidationSummary) -> Self {
        Self {
            graph_id: summary.graph_id,
            project_id: summary.project_id,
            is_valid: summary.is_valid,
            errors: summary.errors,
            warnings: summary.warnings,
            node_count: summary.node_count as i32,
            edge_count: summary.edge_count as i32,
            layer_count: summary.layer_count as i32,
            checked_at: summary.checked_at,
        }
    }
}
