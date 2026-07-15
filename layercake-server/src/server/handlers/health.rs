use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};

use crate::server::app::AppState;

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "layercake-server",
        "version": env!("CARGO_PKG_VERSION"),
        "database": state.database_path,
    })))
}
