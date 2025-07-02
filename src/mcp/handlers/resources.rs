//! MCP resources handlers

use crate::mcp::protocol::JsonRpcError;
use crate::mcp::resources;
use serde_json::{json, Value};
use sea_orm::DatabaseConnection;

/// Handle resources/list request
pub async fn handle_resources_list() -> Result<Value, JsonRpcError> {
    let resources_list = resources::get_resources();
    
    Ok(json!({
        "resources": resources_list
    }))
}

/// Handle resources/read request
pub async fn handle_resources_read(
    params: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters for resources/read".to_string(),
        data: None,
    })?;

    let uri = params.get("uri")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing 'uri' parameter".to_string(),
            data: None,
        })?;

    match resources::get_resource_content(uri, db).await {
        Ok(content) => Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&content).unwrap_or_else(|_| "{}".to_string())
            }]
        })),
        Err(error) => Err(JsonRpcError {
            code: -32603,
            message: format!("Failed to read resource '{}': {}", uri, error),
            data: None,
        }),
    }
}