use async_graphql::*;
use serde::{Deserialize, Serialize};
use crate::database::entities::plan_nodes;

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PlanNode {
    pub id: String,
    pub plan_id: i32,
    pub node_type: String,
    pub name: String,
    pub description: Option<String>,
    pub configuration: String, // JSON configuration
    pub graph_id: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<plan_nodes::Model> for PlanNode {
    fn from(model: plan_nodes::Model) -> Self {
        Self {
            id: model.id,
            plan_id: model.plan_id,
            node_type: model.node_type,
            name: model.name,
            description: model.description,
            configuration: model.configuration,
            graph_id: model.graph_id,
            position_x: model.position_x,
            position_y: model.position_y,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[derive(InputObject)]
pub struct CreatePlanNodeInput {
    pub plan_id: i32,
    pub node_type: String,
    pub name: String,
    pub description: Option<String>,
    pub configuration: String,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(InputObject)]
pub struct UpdatePlanNodeInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub configuration: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(SimpleObject)]
pub struct DagPlan {
    pub nodes: Vec<PlanNode>,
    pub edges: Vec<DagEdge>,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct DagEdge {
    pub source: String,
    pub target: String,
}

#[derive(SimpleObject)]
pub struct GraphArtifact {
    pub id: String,
    pub plan_id: i32,
    pub plan_node_id: String,
    pub name: String,
    pub description: Option<String>,
    pub graph_data: String, // JSON graph data
    pub metadata: Option<String>, // JSON metadata
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::database::entities::graphs::Model> for GraphArtifact {
    fn from(model: crate::database::entities::graphs::Model) -> Self {
        Self {
            id: model.id,
            plan_id: model.plan_id,
            plan_node_id: model.plan_node_id,
            name: model.name,
            description: model.description,
            graph_data: model.graph_data,
            metadata: model.metadata,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GraphStatistics {
    pub node_count: i32,
    pub edge_count: i32,
    pub layer_count: i32,
    pub nodes_per_layer: Vec<LayerNodeCount>,
    pub edges_per_layer: Vec<LayerEdgeCount>,
    pub connected_components: i32,
    pub density: f64,
}

#[derive(SimpleObject)]
pub struct LayerNodeCount {
    pub layer: String,
    pub count: i32,
}

#[derive(SimpleObject)]
pub struct LayerEdgeCount {
    pub layer: String,
    pub count: i32,
}

impl From<crate::services::GraphStatistics> for GraphStatistics {
    fn from(stats: crate::services::GraphStatistics) -> Self {
        Self {
            node_count: stats.node_count as i32,
            edge_count: stats.edge_count as i32,
            layer_count: stats.layer_count as i32,
            nodes_per_layer: stats.nodes_per_layer.into_iter()
                .map(|(layer, count)| LayerNodeCount { layer, count: count as i32 })
                .collect(),
            edges_per_layer: stats.edges_per_layer.into_iter()
                .map(|(layer, count)| LayerEdgeCount { layer, count: count as i32 })
                .collect(),
            connected_components: stats.connected_components as i32,
            density: stats.density,
        }
    }
}

#[derive(SimpleObject)]
pub struct GraphValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl From<crate::services::GraphValidationResult> for GraphValidationResult {
    fn from(result: crate::services::GraphValidationResult) -> Self {
        Self {
            is_valid: result.is_valid,
            errors: result.errors,
            warnings: result.warnings,
        }
    }
}

#[derive(SimpleObject)]
pub struct GraphDiff {
    pub added_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub added_edges: Vec<String>,
    pub removed_edges: Vec<String>,
}

impl From<crate::services::GraphDiff> for GraphDiff {
    fn from(diff: crate::services::GraphDiff) -> Self {
        Self {
            added_nodes: diff.added_nodes,
            removed_nodes: diff.removed_nodes,
            added_edges: diff.added_edges.into_iter()
                .map(|(source, target)| format!("{}->{}", source, target))
                .collect(),
            removed_edges: diff.removed_edges.into_iter()
                .map(|(source, target)| format!("{}->{}", source, target))
                .collect(),
        }
    }
}