use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

#[cfg(feature = "server")]
use utoipa::OpenApi;

#[cfg(feature = "server")]
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check successful", body = Value)
    )
)]
pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "layercake-server",
        "version": env!("CARGO_PKG_VERSION")
    })))
}