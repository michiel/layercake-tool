use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::database::entities::{graph_edges, graph_nodes, layers};
use crate::graphql::context::GraphQLContext;
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
            .map_err(|_| Error::new("GraphQL context not found"))?;

        use crate::database::entities::projects;
        use sea_orm::EntityTrait;

        let project = projects::Entity::find_by_id(self.project_id)
            .one(&graphql_ctx.db)
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Project not found"))?;

        Ok(Project::from(project))
    }

    async fn layers(&self, ctx: &Context<'_>) -> Result<Vec<Layer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let layers = layers::Entity::find()
            .filter(layers::Column::GraphId.eq(self.id))
            .all(&context.db)
            .await?;

        Ok(layers.into_iter().map(Layer::from).collect())
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
