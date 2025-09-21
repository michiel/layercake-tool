use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::database::entities::{
    nodes, nodes::Entity as Nodes,
    edges, edges::Entity as Edges,
    layers, layers::Entity as Layers,
    projects::Entity as Projects,
};
use crate::server::app::AppState;
use crate::services::{ImportService, ExportService, CsvImportData};

#[derive(Serialize, Deserialize)]
pub struct CreateNodeRequest {
    pub node_id: String,
    pub label: String,
    pub layer_id: Option<String>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateEdgeRequest {
    pub source_node_id: String,
    pub target_node_id: String,
    pub properties: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateLayerRequest {
    pub layer_id: String,
    pub name: String,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
}

// Node handlers
pub async fn list_nodes(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<nodes::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let nodes = Nodes::find()
        .filter(nodes::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(nodes))
}

pub async fn create_nodes(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Vec<CreateNodeRequest>>,
) -> Result<Json<Vec<nodes::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut created_nodes = Vec::new();

    for node_req in payload {
        let properties_json = match node_req.properties {
            Some(props) => Some(serde_json::to_string(&props).map_err(|_| StatusCode::BAD_REQUEST)?),
            None => None,
        };

        let node = nodes::ActiveModel {
            project_id: Set(project_id),
            node_id: Set(node_req.node_id),
            label: Set(node_req.label),
            layer_id: Set(node_req.layer_id),
            properties: Set(properties_json),
            ..Default::default()
        };

        let node = node
            .insert(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        created_nodes.push(node);
    }

    Ok(Json(created_nodes))
}

pub async fn delete_nodes(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Nodes::delete_many()
        .filter(nodes::Column::ProjectId.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// Edge handlers
pub async fn list_edges(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<edges::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let edges = Edges::find()
        .filter(edges::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(edges))
}

pub async fn create_edges(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Vec<CreateEdgeRequest>>,
) -> Result<Json<Vec<edges::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut created_edges = Vec::new();

    for edge_req in payload {
        let properties_json = match edge_req.properties {
            Some(props) => Some(serde_json::to_string(&props).map_err(|_| StatusCode::BAD_REQUEST)?),
            None => None,
        };

        let edge = edges::ActiveModel {
            project_id: Set(project_id),
            source_node_id: Set(edge_req.source_node_id),
            target_node_id: Set(edge_req.target_node_id),
            properties: Set(properties_json),
            ..Default::default()
        };

        let edge = edge
            .insert(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        created_edges.push(edge);
    }

    Ok(Json(created_edges))
}

pub async fn delete_edges(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Edges::delete_many()
        .filter(edges::Column::ProjectId.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// Layer handlers
pub async fn list_layers(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<layers::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let layers = Layers::find()
        .filter(layers::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(layers))
}

pub async fn create_layers(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Vec<CreateLayerRequest>>,
) -> Result<Json<Vec<layers::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut created_layers = Vec::new();

    for layer_req in payload {
        let properties_json = match layer_req.properties {
            Some(props) => Some(serde_json::to_string(&props).map_err(|_| StatusCode::BAD_REQUEST)?),
            None => None,
        };

        let layer = layers::ActiveModel {
            project_id: Set(project_id),
            layer_id: Set(layer_req.layer_id),
            name: Set(layer_req.name),
            color: Set(layer_req.color),
            properties: Set(properties_json),
            ..Default::default()
        };

        let layer = layer
            .insert(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        created_layers.push(layer);
    }

    Ok(Json(created_layers))
}

pub async fn delete_layers(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Layers::delete_many()
        .filter(layers::Column::ProjectId.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize)]
pub struct CsvImportRequest {
    pub nodes_csv: Option<String>,
    pub edges_csv: Option<String>,
    pub layers_csv: Option<String>,
}

// Import/Export handlers
pub async fn import_csv(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Json(payload): Json<CsvImportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let import_service = ImportService::new(state.db.clone());
    let import_data = CsvImportData {
        nodes_csv: payload.nodes_csv,
        edges_csv: payload.edges_csv,
        layers_csv: payload.layers_csv,
    };

    match import_service.import_csv_data(project_id, import_data).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "status": "success",
            "nodes_imported": result.nodes_imported,
            "edges_imported": result.edges_imported,
            "layers_imported": result.layers_imported,
            "errors": result.errors
        }))),
        Err(e) => {
            tracing::error!("CSV import failed: {}", e);
            Ok(Json(serde_json::json!({
                "status": "error",
                "message": format!("Import failed: {}", e)
            })))
        }
    }
}

pub async fn export_graph(
    State(state): State<AppState>,
    Path((project_id, format)): Path<(i32, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let export_service = ExportService::new(state.db.clone());

    match export_service.export_graph(project_id, &format).await {
        Ok(content) => Ok(Json(serde_json::json!({
            "status": "success",
            "format": format,
            "content": content
        }))),
        Err(e) => {
            tracing::error!("Graph export failed: {}", e);
            Ok(Json(serde_json::json!({
                "status": "error",
                "message": format!("Export failed: {}", e)
            })))
        }
    }
}