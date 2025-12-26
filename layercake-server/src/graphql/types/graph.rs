use layercake_core::database::entities::{graph_data_edges, graph_data_nodes};
use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::graph_edge::GraphEdge;
use crate::graphql::types::graph_node::GraphNode;
use crate::graphql::types::{Layer, Project};
use layercake_core::services::GraphService;

/// DEPRECATED: Use GraphData type instead.
/// This type is maintained for backward compatibility but will be removed in a future version.
///
/// Migration: Replace `graph` queries with `graphData` queries and filter by `sourceType: "computed"`.
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Graph {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    /// Unified graph_data id when available
    #[graphql(name = "graphDataId")]
    pub graph_data_id: Option<i32>,
    /// Legacy graphs.id when a legacy record exists
    #[graphql(name = "legacyGraphId")]
    pub legacy_graph_id: Option<i32>,
    /// Type discriminator, mirrors graph_data.source_type ("computed" or "dataset")
    #[graphql(name = "sourceType")]
    pub source_type: Option<String>,
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

        use layercake_core::database::entities::projects;
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
        let palette_layers = GraphService::new(context.db.clone())
            .get_all_resolved_layers(self.project_id)
            .await
            .unwrap_or_default();
        let palette_map: HashMap<String, layercake_core::graph::Layer> = palette_layers
            .into_iter()
            .map(|layer| (layer.id.clone(), layer))
            .collect();
        let nodes = graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(self.id))
            .all(&context.db)
            .await?;

        let mut layers = Vec::new();
        let unique_layers: HashSet<String> = nodes.iter().filter_map(|n| n.layer.clone()).collect();
        for (idx, layer_id) in unique_layers.into_iter().enumerate() {
            let palette_entry = palette_map.get(&layer_id);
            layers.push(Layer {
                id: -(idx as i32 + 1),
                graph_id: self.id,
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
        Ok(layers)
    }

    async fn graph_nodes(&self, ctx: &Context<'_>) -> Result<Vec<GraphNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let nodes = graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(self.id))
            .all(&context.db)
            .await?;
        Ok(nodes.into_iter().map(GraphNode::from).collect())
    }

    async fn graph_edges(&self, ctx: &Context<'_>) -> Result<Vec<GraphEdge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let edges = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(self.id))
            .all(&context.db)
            .await?;
        Ok(edges.into_iter().map(GraphEdge::from).collect())
    }
}

impl From<layercake_core::database::entities::graphs::Model> for Graph {
    fn from(model: layercake_core::database::entities::graphs::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            graph_data_id: None,
            legacy_graph_id: Some(model.id),
            source_type: Some("computed".to_string()),
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
impl From<layercake_core::database::entities::graph_data::Model> for Graph {
    fn from(model: layercake_core::database::entities::graph_data::Model) -> Self {
        // Map graph_data status to legacy execution_state
        let execution_state = match model.status.as_str() {
            "pending" => "pending".to_string(),
            "processing" => "running".to_string(),
            "active" => "completed".to_string(),
            "error" => "failed".to_string(),
            _ => model.status.clone(),
        };

        // Serialize annotations back to JSON string for legacy compatibility
        let annotations = model
            .annotations
            .as_ref()
            .and_then(|val| serde_json::to_string(val).ok());

        Self {
            id: model.id,
            project_id: model.project_id,
            graph_data_id: Some(model.id),
            legacy_graph_id: None,
            source_type: Some(model.source_type.clone()),
            name: model.name,
            node_id: model
                .dag_node_id
                .unwrap_or_else(|| format!("graph-{}", model.id)),
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

impl From<layercake_core::app_context::GraphValidationSummary> for GraphValidationResult {
    fn from(summary: layercake_core::app_context::GraphValidationSummary) -> Self {
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
