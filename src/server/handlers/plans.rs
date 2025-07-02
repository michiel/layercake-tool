use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

use crate::database::entities::{plans, plans::Entity as Plans, projects::Entity as Projects};
use crate::server::app::AppState;
use crate::services::ExportService;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreatePlanRequest {
    pub name: String,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdatePlanRequest {
    pub name: String,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
}

#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/plans",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "List all plans for project", body = [plans::Model]),
        (status = 404, description = "Project not found")
    )
)]
pub async fn list_plans(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<plans::Model>>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plans = Plans::find()
        .filter(plans::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plans))
}

pub async fn create_plan(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<Json<plans::Model>, StatusCode> {
    // Verify project exists
    Projects::find_by_id(project_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let dependencies_json = match payload.dependencies {
        Some(deps) => Some(serde_json::to_string(&deps).map_err(|_| StatusCode::BAD_REQUEST)?),
        None => None,
    };

    let plan = plans::ActiveModel {
        project_id: Set(project_id),
        name: Set(payload.name),
        yaml_content: Set(payload.yaml_content),
        dependencies: Set(dependencies_json),
        ..Default::default()
    };

    let plan = plan
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plan))
}

pub async fn get_plan(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
) -> Result<Json<plans::Model>, StatusCode> {
    let plan = Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(plan))
}

pub async fn update_plan(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
    Json(payload): Json<UpdatePlanRequest>,
) -> Result<Json<plans::Model>, StatusCode> {
    let plan = Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let dependencies_json = match payload.dependencies {
        Some(deps) => Some(serde_json::to_string(&deps).map_err(|_| StatusCode::BAD_REQUEST)?),
        None => None,
    };

    let mut plan: plans::ActiveModel = plan.into();
    plan.name = Set(payload.name);
    plan.yaml_content = Set(payload.yaml_content);
    plan.dependencies = Set(dependencies_json);

    let plan = plan
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plan))
}

pub async fn delete_plan(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
) -> Result<StatusCode, StatusCode> {
    let plan = Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Plans::delete_by_id(plan.id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn execute_plan(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    let plan = Plans::find_by_id(plan_id)
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let export_service = ExportService::new(state.db.clone());

    match export_service.execute_plan_exports(project_id, &plan.yaml_content).await {
        Ok(outputs) => Ok(Json(serde_json::json!({
            "status": "completed",
            "plan_id": plan_id,
            "plan_name": plan.name,
            "outputs": outputs,
            "message": format!("Plan executed successfully, generated {} outputs", outputs.len())
        }))),
        Err(e) => {
            tracing::error!("Plan execution failed: {}", e);
            Ok(Json(serde_json::json!({
                "status": "failed",
                "plan_id": plan_id,
                "message": format!("Plan execution failed: {}", e)
            })))
        }
    }
}