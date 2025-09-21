use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::database::entities::{projects, plans, nodes, edges, layers};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, Plan, Node, Edge, Layer};

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
}

#[derive(SimpleObject)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
}