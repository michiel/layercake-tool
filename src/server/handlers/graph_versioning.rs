use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::database::entities::{graph_snapshots, graph_versions, snapshot_data};
use crate::services::GraphVersioningService;
use crate::server::app::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSnapshotRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_automatic: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub version_string: String,
    pub is_automatic: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub node_count: i32,
    pub edge_count: i32,
    pub layer_count: i32,
    pub total_entities: i32,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResponse {
    pub id: i32,
    pub change_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub changed_at: String,
    pub changed_by: Option<String>,
    pub change_summary: String,
    pub has_data_changes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotDataResponse {
    pub id: i32,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_data: serde_json::Value,
    pub data_summary: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangesQuery {
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotDataQuery {
    pub entity_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub restored_by: Option<String>,
}

impl From<graph_snapshots::Model> for SnapshotResponse {
    fn from(snapshot: graph_snapshots::Model) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name.clone(),
            description: snapshot.description.clone(),
            version: snapshot.version,
            version_string: snapshot.version_string(),
            is_automatic: snapshot.is_automatic,
            created_at: snapshot.created_at.to_rfc3339(),
            created_by: snapshot.created_by.clone(),
            node_count: snapshot.node_count,
            edge_count: snapshot.edge_count,
            layer_count: snapshot.layer_count,
            total_entities: snapshot.total_entities(),
            summary: snapshot.summary(),
        }
    }
}

impl From<graph_versions::Model> for VersionResponse {
    fn from(version: graph_versions::Model) -> Self {
        Self {
            id: version.id,
            change_type: version.change_type.clone(),
            entity_type: version.entity_type.clone(),
            entity_id: version.entity_id.clone(),
            changed_at: version.changed_at.to_rfc3339(),
            changed_by: version.changed_by.clone(),
            change_summary: version.get_change_summary(),
            has_data_changes: version.has_data_changes(),
        }
    }
}

impl From<snapshot_data::Model> for SnapshotDataResponse {
    fn from(data: snapshot_data::Model) -> Self {
        Self {
            id: data.id,
            entity_type: data.entity_type.clone(),
            entity_id: data.entity_id.clone(),
            entity_data: data.entity_data.clone(),
            data_summary: data.data_summary(),
            created_at: data.created_at.to_rfc3339(),
        }
    }
}

/// Create a new graph snapshot
pub async fn create_snapshot(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Json(request): Json<CreateSnapshotRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Creating snapshot '{}' for project {}", request.name, project_id);

    let versioning_service = GraphVersioningService::new(state.db.clone());

    match versioning_service
        .create_snapshot(
            project_id,
            request.name,
            request.description,
            request.is_automatic.unwrap_or(false),
            None, // TODO: Get from authentication context
        )
        .await
    {
        Ok(snapshot) => {
            let response = SnapshotResponse::from(snapshot);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create snapshot: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all snapshots for a project
pub async fn list_snapshots(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let versioning_service = GraphVersioningService::new(state.db.clone());

    match versioning_service.list_snapshots(project_id).await {
        Ok(snapshots) => {
            let response: Vec<SnapshotResponse> = snapshots
                .into_iter()
                .map(SnapshotResponse::from)
                .collect();
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list snapshots: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific snapshot
pub async fn get_snapshot(
    State(state): State<AppState>,
    Path((_project_id, snapshot_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, StatusCode> {
    let versioning_service = GraphVersioningService::new(state.db.clone());

    match versioning_service.get_snapshot(snapshot_id).await {
        Ok(Some(snapshot)) => {
            let response = SnapshotResponse::from(snapshot);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get snapshot: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a snapshot
pub async fn delete_snapshot(
    State(state): State<AppState>,
    Path((_project_id, snapshot_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Deleting snapshot {}", snapshot_id);

    let versioning_service = GraphVersioningService::new(state.db.clone());

    match versioning_service.delete_snapshot(snapshot_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete snapshot: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Restore from a snapshot
pub async fn restore_from_snapshot(
    State(state): State<AppState>,
    Path((project_id, snapshot_id)): Path<(i32, i32)>,
    Json(request): Json<RestoreRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Restoring project {} from snapshot {}", project_id, snapshot_id);

    let versioning_service = GraphVersioningService::new(state.db.clone());

    match versioning_service
        .restore_from_snapshot(project_id, snapshot_id, request.restored_by)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to restore from snapshot: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get project changes/version history
pub async fn get_changes(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Query(query): Query<ChangesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let versioning_service = GraphVersioningService::new(state.db.clone());

    match versioning_service.get_changes(project_id, query.limit).await {
        Ok(changes) => {
            let response: Vec<VersionResponse> = changes
                .into_iter()
                .map(VersionResponse::from)
                .collect();
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get changes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get snapshot data
pub async fn get_snapshot_data(
    State(state): State<AppState>,
    Path((_project_id, snapshot_id)): Path<(i32, i32)>,
    Query(query): Query<SnapshotDataQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let versioning_service = GraphVersioningService::new(state.db.clone());

    let entity_type = query.entity_type.map(|et| {
        crate::database::entities::graph_versions::EntityType::from(et)
    });

    match versioning_service.get_snapshot_data(snapshot_id, entity_type).await {
        Ok(data) => {
            let response: Vec<SnapshotDataResponse> = data
                .into_iter()
                .map(SnapshotDataResponse::from)
                .collect();
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get snapshot data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
