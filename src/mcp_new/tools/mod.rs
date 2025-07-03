//! Tool implementations for Layercake MCP server

pub mod projects;
pub mod plans;
pub mod graph_data;

// Helper functions for parameter extraction
use axum_mcp::prelude::*;
use serde_json::Value;

pub fn get_required_param<'a>(arguments: &'a Option<Value>, key: &str) -> McpResult<&'a Value> {
    arguments
        .as_ref()
        .and_then(|args| args.get(key))
        .ok_or_else(|| McpError::Validation {
            message: format!("Missing required parameter: {}", key),
        })
}

pub fn get_optional_param<'a>(arguments: &'a Option<Value>, key: &str) -> Option<&'a Value> {
    arguments
        .as_ref()
        .and_then(|args| args.get(key))
}