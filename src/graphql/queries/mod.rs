use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::database::entities::{projects, plans, nodes, edges, layers, plan_nodes, graphs};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, Plan, Node, Edge, Layer, PlanNode, DagPlan, DagEdge, GraphArtifact, GraphStatistics};

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

    /// Get DAG structure for a plan
    async fn plan_dag(&self, ctx: &Context<'_>, plan_id: i32) -> Result<Option<DagPlan>> {
        let context = ctx.data::<GraphQLContext>()?;
        
        // Get all plan nodes for this plan
        let plan_nodes = plan_nodes::Entity::find()
            .filter(plan_nodes::Column::PlanId.eq(plan_id))
            .all(&context.db)
            .await?;
        
        if plan_nodes.is_empty() {
            return Ok(None);
        }
        
        let nodes: Vec<PlanNode> = plan_nodes.into_iter().map(PlanNode::from).collect();
        
        // For now, return empty edges - will be implemented when we add plan edges table
        let edges: Vec<DagEdge> = vec![];
        
        Ok(Some(DagPlan { nodes, edges }))
    }

    /// Get all plan nodes for a plan
    async fn plan_nodes(&self, ctx: &Context<'_>, plan_id: i32) -> Result<Vec<PlanNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plan_nodes = plan_nodes::Entity::find()
            .filter(plan_nodes::Column::PlanId.eq(plan_id))
            .all(&context.db)
            .await?;
        
        Ok(plan_nodes.into_iter().map(PlanNode::from).collect())
    }

    /// Get a specific plan node
    async fn plan_node(&self, ctx: &Context<'_>, id: String) -> Result<Option<PlanNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plan_node = plan_nodes::Entity::find_by_id(id)
            .one(&context.db)
            .await?;
        
        Ok(plan_node.map(PlanNode::from))
    }

    /// Get graph artifact at a specific plan node
    async fn graph_artifact(&self, ctx: &Context<'_>, plan_node_id: String) -> Result<Option<GraphArtifact>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph = graphs::Entity::find()
            .filter(graphs::Column::PlanNodeId.eq(plan_node_id))
            .one(&context.db)
            .await?;
        
        Ok(graph.map(GraphArtifact::from))
    }

    /// Get all graph artifacts for a plan
    async fn graph_artifacts(&self, ctx: &Context<'_>, plan_id: i32) -> Result<Vec<GraphArtifact>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graphs = graphs::Entity::find()
            .filter(graphs::Column::PlanId.eq(plan_id))
            .all(&context.db)
            .await?;
        
        Ok(graphs.into_iter().map(GraphArtifact::from).collect())
    }
}

#[derive(SimpleObject)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
}