use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::database::entities::{projects, plans, nodes, edges, layers, plan_dag_nodes, plan_dag_edges};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, Plan, Node, Edge, Layer, PlanDag, PlanDagNode, PlanDagEdge, PlanDagMetadata, ValidationResult, PlanDagInput};

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

        // First, find the plan for this project (assuming one plan per project for now)
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&context.db)
            .await?;

        let plan = match plan {
            Some(p) => p,
            None => return Ok(None),
        };

        // Get Plan DAG nodes for this plan
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;

        // Get Plan DAG edges for this plan
        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await?;

        // Convert to GraphQL types
        let nodes: Vec<PlanDagNode> = dag_nodes.into_iter().map(PlanDagNode::from).collect();
        let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();

        // Parse Plan DAG metadata from plan_dag_json field or create default
        let metadata = if let Some(json_str) = &plan.plan_dag_json {
            serde_json::from_str::<PlanDagMetadata>(json_str)
                .unwrap_or_else(|_| PlanDagMetadata {
                    version: "1.0".to_string(),
                    name: Some(plan.name.clone()),
                    description: None,
                    created: Some(plan.created_at.to_rfc3339()),
                    last_modified: Some(plan.updated_at.to_rfc3339()),
                    author: None,
                })
        } else {
            PlanDagMetadata {
                version: "1.0".to_string(),
                name: Some(plan.name.clone()),
                description: None,
                created: Some(plan.created_at.to_rfc3339()),
                last_modified: Some(plan.updated_at.to_rfc3339()),
                author: None,
            }
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
}

#[derive(SimpleObject)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
}