use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::Utc;

#[cfg(feature = "server")]
use utoipa::ToSchema;

use crate::database::entities::{projects, projects::Entity as Projects};
use crate::server::app::AppState;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects",
    responses(
        (status = 200, description = "List all projects", body = [crate::database::entities::projects::Model])
    )
)]
pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<projects::Model>>, StatusCode> {
    let projects = Projects::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(projects))
}

#[cfg(feature = "server")]
#[utoipa::path(
    post,
    path = "/api/v1/projects",
    request_body = crate::server::handlers::projects::CreateProjectRequest,
    responses(
        (status = 200, description = "Project created successfully", body = crate::database::entities::projects::Model)
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<projects::Model>, StatusCode> {
    let now = Utc::now();
    let project = projects::ActiveModel {
        name: Set(payload.name),
        description: Set(payload.description),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let project = project
        .insert(&state.db)
        .await
        .map_err(|err| {
            eprintln!("Database error creating project: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(project))
}

#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}",
    params(
        ("id" = i32, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project found", body = crate::database::entities::projects::Model),
        (status = 404, description = "Project not found")
    )
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<projects::Model>, StatusCode> {
    let project = Projects::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(project))
}

#[cfg(feature = "server")]
#[utoipa::path(
    put,
    path = "/api/v1/projects/{id}",
    params(
        ("id" = i32, Path, description = "Project ID")
    ),
    request_body = crate::server::handlers::projects::UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated successfully", body = crate::database::entities::projects::Model),
        (status = 404, description = "Project not found")
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<projects::Model>, StatusCode> {
    let project = Projects::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut project: projects::ActiveModel = project.into();
    project.name = Set(payload.name);
    project.description = Set(payload.description);
    project.updated_at = Set(Utc::now());

    let project = project
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(project))
}

#[cfg(feature = "server")]
#[utoipa::path(
    delete,
    path = "/api/v1/projects/{id}",
    params(
        ("id" = i32, Path, description = "Project ID")
    ),
    responses(
        (status = 204, description = "Project deleted successfully"),
        (status = 404, description = "Project not found")
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let project = Projects::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Projects::delete_by_id(project.id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}