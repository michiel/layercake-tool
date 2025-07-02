//! MCP request handlers
//! 
//! This module contains handlers for different types of MCP requests.

pub mod tools;
pub mod resources; 
pub mod prompts;

use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use serde_json::Value;
use sea_orm::DatabaseConnection;

/// Handle incoming MCP request
pub async fn handle_request(
    request: JsonRpcRequest,
    db: &DatabaseConnection,
) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "initialize" => handle_initialize(request.params).await,
        "tools/list" => tools::handle_tools_list().await,
        "tools/call" => tools::handle_tools_call(request.params, db).await,
        "resources/list" => resources::handle_resources_list().await,
        "resources/read" => resources::handle_resources_read(request.params, db).await,
        "prompts/list" => prompts::handle_prompts_list().await,
        "prompts/get" => prompts::handle_prompts_get(request.params, db).await,
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(result_value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result_value),
            error: None,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(error),
        },
    }
}

/// Handle initialization request
async fn handle_initialize(params: Option<Value>) -> Result<Value, JsonRpcError> {
    use crate::mcp::protocol::{InitializeRequest, InitializeResponse, ServerCapabilities, ServerInfo, PromptsCapability, ResourcesCapability, ToolsCapability};
    
    let _init_request: InitializeRequest = serde_json::from_value(params.unwrap_or_default())
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid initialize params: {}", e),
            data: None,
        })?;

    let response = InitializeResponse {
        protocol_version: crate::mcp::protocol::MCP_VERSION.to_string(),
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
        },
        server_info: ServerInfo {
            name: "layercake-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    serde_json::to_value(response)
        .map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Internal error: {}", e),
            data: None,
        })
}