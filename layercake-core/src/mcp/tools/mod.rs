//! Tool implementations for Layercake MCP server

pub mod analysis;
pub mod auth;
pub mod graph_data;
pub mod plan_dag;
pub mod plans;
pub mod projects;

// Helper functions for parameter extraction and response formatting
use axum_mcp::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn get_required_param<'a>(arguments: &'a Option<Value>, key: &str) -> McpResult<&'a Value> {
    arguments
        .as_ref()
        .and_then(|args| args.get(key))
        .ok_or_else(|| McpError::Validation {
            message: format!("Missing required parameter: {}", key),
        })
}

pub fn get_optional_param<'a>(arguments: &'a Option<Value>, key: &str) -> Option<&'a Value> {
    arguments.as_ref().and_then(|args| args.get(key))
}

/// Create a successful MCP tool response with JSON content
pub fn create_success_response(result: &Value) -> McpResult<ToolsCallResult> {
    let json_string = serde_json::to_string_pretty(result).map_err(|e| McpError::Internal {
        message: format!("Failed to serialize response: {}", e),
    })?;

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text { text: json_string }],
        is_error: false,
        metadata: HashMap::new(),
    })
}
