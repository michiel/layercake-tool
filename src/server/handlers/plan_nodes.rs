use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "server")]
use utoipa::ToSchema;

use crate::database::entities::{
    plan_nodes, plans, graphs,
    plan_nodes::Entity as PlanNodes,
    plans::Entity as Plans,
    graphs::Entity as Graphs,
};
use crate::server::app::AppState;
use crate::services::{GraphService, GraphStatistics, GraphValidationResult, GraphDiff};
use crate::plan::DagPlan;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreatePlanNodeRequest {
    pub name: String,
    pub node_type: String,
    pub configuration: serde_json::Value,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdatePlanNodeRequest {
    pub name: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PlanNodeResponse {
    #[serde(flatten)]
    pub node: plan_nodes::Model,
    pub graph: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct NodeQueryParams {
    pub include_graph: Option<bool>,
    pub execution_id: Option<String>,
}

/// Get all plan nodes for a plan
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans/{plan_id}/plan-nodes",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("plan_id" = i32, Path, description = "Plan ID")
    ),
    responses(
        (status = 200, description = "List all plan nodes", body = [plan_nodes::Model]),
        (status = 404, description = "Plan not found")
    )
)]
pub async fn list_plan_nodes(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<plan_nodes::Model>>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plan_nodes = PlanNodes::find()
        .filter(plan_nodes::Column::PlanId.eq(plan_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plan_nodes))
}

/// Create a new plan node
pub async fn create_plan_node(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
    Json(payload): Json<CreatePlanNodeRequest>,
) -> Result<Json<plan_nodes::Model>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let now = chrono::Utc::now();
    let plan_node = plan_nodes::ActiveModel {
        plan_id: Set(plan_id),
        name: Set(payload.name),
        node_type: Set(payload.node_type),
        configuration: Set(payload.configuration.to_string()),
        position_x: Set(payload.position_x),
        position_y: Set(payload.position_y),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let plan_node = plan_node
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plan_node))
}

/// Get a specific plan node
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans/{plan_id}/plan-nodes/{node_id}",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("plan_id" = i32, Path, description = "Plan ID"),
        ("node_id" = String, Path, description = "Plan Node ID")
    ),
    responses(
        (status = 200, description = "Plan node details", body = PlanNodeResponse),
        (status = 404, description = "Plan node not found")
    )
)]
pub async fn get_plan_node(
    State(state): State<AppState>,
    Path((project_id, plan_id, node_id)): Path<(i32, i32, String)>,
    Query(params): Query<NodeQueryParams>,
) -> Result<Json<PlanNodeResponse>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plan_node = PlanNodes::find_by_id(&node_id)
        .filter(plan_nodes::Column::PlanId.eq(plan_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let graph = if params.include_graph.unwrap_or(false) {
        let graph_service = GraphService::new(state.db.clone());
        match graph_service.get_graph_by_plan_node(&node_id).await {
            Ok(Some(graph)) => Some(serde_json::to_value(graph).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?),
            Ok(None) => None,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        None
    };

    Ok(Json(PlanNodeResponse {
        node: plan_node,
        graph,
    }))
}

/// Get graph for a specific plan node
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans/{plan_id}/plan-nodes/{node_id}/graph",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("plan_id" = i32, Path, description = "Plan ID"),
        ("node_id" = String, Path, description = "Plan Node ID")
    ),
    responses(
        (status = 200, description = "Graph data for plan node", body = serde_json::Value),
        (status = 404, description = "Graph not found")
    )
)]
pub async fn get_plan_node_graph(
    State(state): State<AppState>,
    Path((project_id, plan_id, node_id)): Path<(i32, i32, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify plan node exists
    PlanNodes::find_by_id(&node_id)
        .filter(plan_nodes::Column::PlanId.eq(plan_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let graph_service = GraphService::new(state.db.clone());
    let graph = graph_service.get_graph_by_plan_node(&node_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(graph).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

/// Update a plan node
pub async fn update_plan_node(
    State(state): State<AppState>,
    Path((project_id, plan_id, node_id)): Path<(i32, i32, String)>,
    Json(payload): Json<UpdatePlanNodeRequest>,
) -> Result<Json<plan_nodes::Model>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plan_node = PlanNodes::find_by_id(&node_id)
        .filter(plan_nodes::Column::PlanId.eq(plan_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut plan_node: plan_nodes::ActiveModel = plan_node.into();
    
    // Only update fields that are provided
    if let Some(name) = payload.name {
        plan_node.name = Set(name);
    }
    if let Some(configuration) = payload.configuration {
        plan_node.configuration = Set(configuration.to_string());
    }
    if let Some(position_x) = payload.position_x {
        plan_node.position_x = Set(Some(position_x));
    }
    if let Some(position_y) = payload.position_y {
        plan_node.position_y = Set(Some(position_y));
    }
    plan_node.updated_at = Set(chrono::Utc::now());

    let plan_node = plan_node
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plan_node))
}

/// Delete a plan node
pub async fn delete_plan_node(
    State(state): State<AppState>,
    Path((project_id, plan_id, node_id)): Path<(i32, i32, String)>,
) -> Result<StatusCode, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plan_node = PlanNodes::find_by_id(&node_id)
        .filter(plan_nodes::Column::PlanId.eq(plan_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    PlanNodes::delete_by_id(plan_node.id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get DAG structure for a plan
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/plans/{plan_id}/dag",
    params(
        ("plan_id" = i32, Path, description = "Plan ID")
    ),
    responses(
        (status = 200, description = "DAG structure", body = serde_json::Value),
        (status = 404, description = "Plan not found")
    )
)]
pub async fn get_plan_dag(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let plan = Plans::find_by_id(plan_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Try to parse as DAG plan
    match serde_json::from_str::<DagPlan>(&plan.plan_content) {
        Ok(dag_plan) => Ok(Json(serde_json::to_value(dag_plan).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)),
        Err(_) => {
            // Return error for non-DAG plans
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get all graphs for a plan execution
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans/{plan_id}/execution/{exec_id}/graphs",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("plan_id" = i32, Path, description = "Plan ID"),
        ("exec_id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Graphs from execution", body = Vec<serde_json::Value>),
        (status = 404, description = "Execution not found")
    )
)]
pub async fn get_execution_graphs(
    State(state): State<AppState>,
    Path((project_id, plan_id, _exec_id)): Path<(i32, i32, String)>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let graph_service = GraphService::new(state.db.clone());
    let graph_artifacts = graph_service.get_plan_graph_artifacts(plan_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let graphs: Result<Vec<_>, _> = graph_artifacts.into_iter()
        .map(|(graph, model)| {
            Ok(serde_json::json!({
                "id": model.id,
                "plan_node_id": model.plan_node_id,
                "name": model.name,
                "description": model.description,
                "created_at": model.created_at,
                "graph": serde_json::to_value(graph)?
            }))
        })
        .collect();

    Ok(Json(graphs.map_err(|_: serde_json::Error| StatusCode::INTERNAL_SERVER_ERROR)?))
}

/// Get graph statistics
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans/{plan_id}/plan-nodes/{node_id}/graph/stats",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("plan_id" = i32, Path, description = "Plan ID"),
        ("node_id" = String, Path, description = "Plan Node ID")
    ),
    responses(
        (status = 200, description = "Graph statistics", body = GraphStatistics),
        (status = 404, description = "Graph not found")
    )
)]
pub async fn get_plan_node_graph_stats(
    State(state): State<AppState>,
    Path((project_id, plan_id, node_id)): Path<(i32, i32, String)>,
) -> Result<Json<GraphStatistics>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Find graph for this plan node
    let graph = Graphs::find()
        .filter(graphs::Column::PlanId.eq(plan_id))
        .filter(graphs::Column::PlanNodeId.eq(&node_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let graph_service = GraphService::new(state.db.clone());
    let stats = graph_service.get_graph_statistics(&graph.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(stats))
}

/// Validate graph integrity
#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans/{plan_id}/plan-nodes/{node_id}/graph/validate",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("plan_id" = i32, Path, description = "Plan ID"),
        ("node_id" = String, Path, description = "Plan Node ID")
    ),
    responses(
        (status = 200, description = "Graph validation result", body = GraphValidationResult),
        (status = 404, description = "Graph not found")
    )
)]
pub async fn validate_plan_node_graph(
    State(state): State<AppState>,
    Path((project_id, plan_id, node_id)): Path<(i32, i32, String)>,
) -> Result<Json<GraphValidationResult>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Find graph for this plan node
    let graph = Graphs::find()
        .filter(graphs::Column::PlanId.eq(plan_id))
        .filter(graphs::Column::PlanNodeId.eq(&node_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let graph_service = GraphService::new(state.db.clone());
    let validation = graph_service.validate_graph(&graph.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(validation))
}

#[derive(Serialize, Deserialize)]
pub struct CompareGraphsRequest {
    pub graph_id1: String,
    pub graph_id2: String,
}

/// Compare two graphs
pub async fn compare_graphs(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
    Json(payload): Json<CompareGraphsRequest>,
) -> Result<Json<GraphDiff>, StatusCode> {
    // Verify plan exists and belongs to project
    Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let graph_service = GraphService::new(state.db.clone());
    let diff = graph_service.compare_graphs(&payload.graph_id1, &payload.graph_id2)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(diff))
}