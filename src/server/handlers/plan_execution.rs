use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, IntoResponse, Sse, sse::Event},
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, Stream, StreamExt};
use tracing::{info, error};

use crate::database::entities::{plan_executions, execution_logs, execution_outputs};
use crate::services::AsyncPlanExecutionService;
use crate::server::app::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutePlanRequest {
    pub plan_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutePlanResponse {
    pub execution_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStatusResponse {
    pub execution_id: String,
    pub plan_id: i32,
    pub status: String,
    pub progress: Option<i32>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionLogResponse {
    pub id: i32,
    pub level: String,
    pub message: String,
    pub details: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionOutputResponse {
    pub id: i32,
    pub file_name: String,
    pub file_type: String,
    pub file_path: Option<String>,
    pub file_size: Option<i32>,
    pub formatted_size: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub event_type: String,
    pub execution_id: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

/// Start async plan execution
pub async fn execute_plan_async(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Starting async execution for plan {} in project {}", plan_id, project_id);

    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    match execution_service.execute_plan_async(plan_id).await {
        Ok(execution_id) => {
            let response = ExecutePlanResponse {
                execution_id,
                status: "queued".to_string(),
                message: "Plan execution started".to_string(),
            };
            Ok((StatusCode::ACCEPTED, Json(response)))
        }
        Err(e) => {
            error!("Failed to start plan execution: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get execution status
pub async fn get_execution_status(
    State(state): State<AppState>,
    Path((project_id, plan_id, execution_id)): Path<(i32, i32, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    match execution_service.get_execution_status(&execution_id).await {
        Ok(Some(execution)) => {
            let response = ExecutionStatusResponse {
                execution_id: execution.execution_id.clone(),
                plan_id: execution.plan_id,
                status: execution.status.clone(),
                progress: execution.progress,
                started_at: execution.started_at.map(|dt| dt.to_rfc3339()),
                completed_at: execution.completed_at.map(|dt| dt.to_rfc3339()),
                duration_seconds: execution.duration_seconds(),
                error: execution.error.clone(),
                created_at: execution.created_at.to_rfc3339(),
                updated_at: execution.updated_at.to_rfc3339(),
            };
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get execution status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get execution logs
pub async fn get_execution_logs(
    State(state): State<AppState>,
    Path((project_id, plan_id, execution_id)): Path<(i32, i32, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    match execution_service.get_execution_logs(&execution_id).await {
        Ok(logs) => {
            let response: Vec<ExecutionLogResponse> = logs
                .into_iter()
                .map(|log| ExecutionLogResponse {
                    id: log.id,
                    level: log.level,
                    message: log.message,
                    details: log.details,
                    timestamp: log.timestamp.to_rfc3339(),
                })
                .collect();
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get execution logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get execution outputs
pub async fn get_execution_outputs(
    State(state): State<AppState>,
    Path((project_id, plan_id, execution_id)): Path<(i32, i32, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    match execution_service.get_execution_outputs(&execution_id).await {
        Ok(outputs) => {
            let response: Vec<ExecutionOutputResponse> = outputs
                .into_iter()
                .map(|output| {
                    let formatted_size = output.formatted_size();
                    ExecutionOutputResponse {
                        id: output.id,
                        file_name: output.file_name,
                        file_type: output.file_type,
                        file_path: output.file_path,
                        file_size: output.file_size,
                        formatted_size,
                        created_at: output.created_at.to_rfc3339(),
                    }
                })
                .collect();
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get execution outputs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Cancel execution
pub async fn cancel_execution(
    State(state): State<AppState>,
    Path((project_id, plan_id, execution_id)): Path<(i32, i32, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    match execution_service.cancel_execution(&execution_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to cancel execution: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List plan executions
pub async fn list_plan_executions(
    State(state): State<AppState>,
    Path((project_id, plan_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, StatusCode> {
    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    match execution_service.list_plan_executions(plan_id).await {
        Ok(executions) => {
            let response: Vec<ExecutionStatusResponse> = executions
                .into_iter()
                .map(|execution| ExecutionStatusResponse {
                    execution_id: execution.execution_id.clone(),
                    plan_id: execution.plan_id,
                    status: execution.status.clone(),
                    progress: execution.progress,
                    started_at: execution.started_at.map(|dt| dt.to_rfc3339()),
                    completed_at: execution.completed_at.map(|dt| dt.to_rfc3339()),
                    duration_seconds: execution.duration_seconds(),
                    error: execution.error.clone(),
                    created_at: execution.created_at.to_rfc3339(),
                    updated_at: execution.updated_at.to_rfc3339(),
                })
                .collect();
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list plan executions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Stream execution progress via Server-Sent Events
pub async fn stream_execution_progress(
    State(state): State<AppState>,
    Path((project_id, plan_id, execution_id)): Path<(i32, i32, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Starting SSE stream for execution: {}", execution_id);

    let execution_service = AsyncPlanExecutionService::new(state.db.clone());

    // Verify execution exists
    match execution_service.get_execution_status(&execution_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    let stream = create_progress_stream(execution_service, execution_id);

    Ok(Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text"),
        ))
}

/// Create progress event stream
fn create_progress_stream(
    execution_service: AsyncPlanExecutionService,
    execution_id: String,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let interval = interval(Duration::from_millis(1000)); // Poll every second
    let stream = IntervalStream::new(interval);

    stream.map(move |_| {
        let execution_service = execution_service.clone();
        let execution_id = execution_id.clone();

        async move {
            // Get latest status
            if let Ok(Some(execution)) = execution_service.get_execution_status(&execution_id).await {
                let event_data = serde_json::json!({
                    "status": execution.status,
                    "progress": execution.progress,
                    "started_at": execution.started_at.map(|dt| dt.to_rfc3339()),
                    "completed_at": execution.completed_at.map(|dt| dt.to_rfc3339()),
                    "error": execution.error
                });

                let progress_event = ProgressEvent {
                    event_type: "progress".to_string(),
                    execution_id: execution_id.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    data: event_data,
                };

                if let Ok(json_data) = serde_json::to_string(&progress_event) {
                    return Event::default().data(json_data);
                }
            }

            // Return heartbeat if no data
            Event::default().data("heartbeat")
        }
    })
    .then(|fut| fut)
    .map(|event| Ok(event))
}

