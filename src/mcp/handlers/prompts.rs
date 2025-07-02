//! MCP prompts handlers

use crate::mcp::protocol::JsonRpcError;
use crate::mcp::prompts;
use serde_json::{json, Value};
use sea_orm::DatabaseConnection;

/// Handle prompts/list request
pub async fn handle_prompts_list() -> Result<Value, JsonRpcError> {
    let prompts_list = prompts::get_prompts();
    
    Ok(json!({
        "prompts": prompts_list
    }))
}

/// Handle prompts/get request
pub async fn handle_prompts_get(
    params: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters for prompts/get".to_string(),
        data: None,
    })?;

    let prompt_name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing 'name' parameter".to_string(),
            data: None,
        })?;

    let arguments = params.get("arguments").cloned();

    match prompts::get_prompt_content(prompt_name, arguments, db).await {
        Ok(content) => {
            let prompt_text = content.get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("No prompt content available");
                
            Ok(json!({
                "description": format!("Analysis prompt for {}", prompt_name),
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_text
                    }
                }]
            }))
        },
        Err(error) => Err(JsonRpcError {
            code: -32603,
            message: format!("Failed to get prompt '{}': {}", prompt_name, error),
            data: None,
        }),
    }
}