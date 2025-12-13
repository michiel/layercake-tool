use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::database::entities::{graph_data_edges, graph_data_nodes, projections};
use crate::graphql::errors::StructuredError;

/// Projection metadata type
#[derive(SimpleObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct Projection {
    pub id: String,
    pub project_id: i32,
    pub graph_id: String,
    pub name: String,
    pub projection_type: String,
    pub settings: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<projections::Model> for Projection {
    fn from(model: projections::Model) -> Self {
        Self {
            id: model.id.to_string(),
            project_id: model.project_id,
            graph_id: model.graph_id.to_string(),
            name: model.name,
            projection_type: model.projection_type,
            settings: model.settings_json,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Node type for projection graph
#[derive(SimpleObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionGraphNode {
    pub id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub color: Option<String>,
    pub label_color: Option<String>,
}

/// Edge type for projection graph
#[derive(SimpleObject, Clone, Debug)]
pub struct ProjectionGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

/// Layer type for projection graph
#[derive(SimpleObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionGraphLayer {
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
}

/// Complete projection graph data (nodes, edges, layers)
#[derive(SimpleObject, Clone, Debug)]
pub struct ProjectionGraph {
    pub nodes: Vec<ProjectionGraphNode>,
    pub edges: Vec<ProjectionGraphEdge>,
    pub layers: Vec<ProjectionGraphLayer>,
}

/// Projection state for the 3D viewer
#[derive(SimpleObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionState {
    pub projection_id: String,
    pub projection_type: String,
    pub state_json: serde_json::Value,
}

/// Build projection graph from graph_data nodes and edges
pub async fn build_projection_graph(
    db: &sea_orm::DatabaseConnection,
    graph_data_id: i32,
) -> Result<ProjectionGraph> {
    // Load nodes
    let node_models = graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
        .all(db)
        .await
        .map_err(|e| StructuredError::database("graph_data_nodes::Entity::find", e))?;

    // Load edges
    let edge_models = graph_data_edges::Entity::find()
        .filter(graph_data_edges::Column::GraphDataId.eq(graph_data_id))
        .all(db)
        .await
        .map_err(|e| StructuredError::database("graph_data_edges::Entity::find", e))?;

    // Convert nodes to projection format
    let nodes: Vec<ProjectionGraphNode> = node_models
        .into_iter()
        .map(|node| {
            // Extract color from attributes if available
            let (color, label_color) = node
                .attributes
                .as_ref()
                .and_then(|attrs| attrs.as_object())
                .map(|obj| {
                    let color = obj
                        .get("color")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let label_color = obj
                        .get("labelColor")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    (color, label_color)
                })
                .unwrap_or((None, None));

            ProjectionGraphNode {
                id: node.external_id,
                label: node.label,
                layer: node.layer,
                color,
                label_color,
            }
        })
        .collect();

    // Convert edges to projection format
    let edges: Vec<ProjectionGraphEdge> = edge_models
        .into_iter()
        .map(|edge| ProjectionGraphEdge {
            id: edge.external_id,
            source: edge.source,
            target: edge.target,
        })
        .collect();

    // Build unique layer list from nodes
    let mut layer_map: std::collections::HashMap<String, ProjectionGraphLayer> =
        std::collections::HashMap::new();

    for node_model in graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
        .all(db)
        .await
        .map_err(|e| StructuredError::database("graph_data_nodes::Entity::find", e))?
    {
        if let Some(layer_id) = &node_model.layer {
            if !layer_map.contains_key(layer_id) {
                // Extract layer styling from node attributes
                let (bg_color, text_color, border_color) = node_model
                    .attributes
                    .as_ref()
                    .and_then(|attrs| attrs.as_object())
                    .map(|obj| {
                        let bg = obj
                            .get("backgroundColor")
                            .or_else(|| obj.get("color"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let text = obj
                            .get("textColor")
                            .or_else(|| obj.get("labelColor"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let border = obj
                            .get("borderColor")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        (bg, text, border)
                    })
                    .unwrap_or((None, None, None));

                layer_map.insert(
                    layer_id.clone(),
                    ProjectionGraphLayer {
                        layer_id: layer_id.clone(),
                        name: layer_id.clone(), // Use layer_id as name by default
                        background_color: bg_color,
                        text_color: text_color,
                        border_color: border_color,
                    },
                );
            }
        }
    }

    let layers: Vec<ProjectionGraphLayer> = layer_map.into_values().collect();

    Ok(ProjectionGraph {
        nodes,
        edges,
        layers,
    })
}
