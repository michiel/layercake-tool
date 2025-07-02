//! MCP tools handlers

use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tools;
use serde_json::{json, Value};
use sea_orm::DatabaseConnection;

/// Handle tools/list request
pub async fn handle_tools_list() -> Result<Value, JsonRpcError> {
    let tools_list = tools::get_tools();
    
    Ok(json!({
        "tools": tools_list
    }))
}

/// Handle tools/call request
pub async fn handle_tools_call(
    params: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters for tools/call".to_string(),
        data: None,
    })?;

    let tool_name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing 'name' parameter".to_string(),
            data: None,
        })?;

    let arguments = params.get("arguments").cloned();

    match tools::execute_tool(tool_name, arguments, db).await {
        Ok(result) => Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }],
            "isError": false
        })),
        Err(error) => Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Error executing tool '{}': {}", tool_name, error)
            }],
            "isError": true
        })),
    }
}