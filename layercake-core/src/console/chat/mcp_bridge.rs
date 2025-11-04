#![cfg(feature = "console")]

use anyhow::Result;
use axum_mcp::prelude::*;
use axum_mcp::protocol::messages::{Tool, ToolContent, ToolsCallResult};
use axum_mcp::server::registry::ToolExecutionContext;
use sea_orm::DatabaseConnection;
use serde_json::json;
use std::sync::Arc;

use crate::app_context::AppContext;
use crate::mcp::{
    prompts::LayercakePromptRegistry,
    resources::LayercakeResourceRegistry,
    server::{LayercakeAuth, LayercakeServerState, LayercakeToolRegistry},
};

/// Convenience wrapper exposing the MCP server state to the chat loop.
pub struct McpBridge {
    state: LayercakeServerState,
}

impl McpBridge {
    pub fn new(db: DatabaseConnection) -> Self {
        let app = Arc::new(AppContext::new(db.clone()));
        let tools = LayercakeToolRegistry::new(app.clone());
        let resources = LayercakeResourceRegistry::new(app.clone());
        let prompts = LayercakePromptRegistry::new();
        let auth = LayercakeAuth::new(db.clone());

        let state = LayercakeServerState {
            db,
            app,
            tools,
            resources,
            prompts,
            auth,
        };

        Self { state }
    }

    pub async fn list_tools(&self, context: &SecurityContext) -> McpResult<Vec<Tool>> {
        self.state.tool_registry().list_tools(context).await
    }

    pub async fn execute_tool(
        &self,
        name: &str,
        context: &SecurityContext,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolsCallResult> {
        let exec_context = ToolExecutionContext::new(context.clone());
        let exec_context = if let Some(args) = arguments {
            exec_context.with_arguments(args)
        } else {
            exec_context
        };

        self.state
            .tool_registry()
            .execute_tool(name, exec_context)
            .await
    }

    pub fn summarize_tool_result(result: &ToolsCallResult) -> String {
        let mut parts = Vec::new();
        for content in &result.content {
            match content {
                ToolContent::Text { text } => parts.push(text.clone()),
                ToolContent::Image { mime_type, .. } => {
                    parts.push(format!("(image {})", mime_type))
                }
                ToolContent::Resource { resource, text, .. } => {
                    if let Some(text) = text {
                        parts.push(format!("Resource {}: {}", resource.uri, text));
                    } else {
                        parts.push(format!("Resource {} (binary payload)", resource.uri));
                    }
                }
            }
        }

        if parts.is_empty() {
            "(tool produced no textual output)".to_string()
        } else {
            parts.join("\n")
        }
    }

    pub fn serialize_tool_result(result: &ToolsCallResult) -> serde_json::Value {
        let content: Vec<serde_json::Value> = result
            .content
            .iter()
            .map(|item| match item {
                ToolContent::Text { text } => json!({"type": "text", "text": text}),
                ToolContent::Image { data, mime_type } => {
                    json!({"type": "image", "data": data, "mimeType": mime_type})
                }
                ToolContent::Resource {
                    resource,
                    text,
                    blob,
                } => json!({
                    "type": "resource",
                    "resource": {"uri": resource.uri.clone()},
                    "text": text,
                    "blob": blob,
                }),
            })
            .collect();

        json!({
            "content": content,
            "is_error": result.is_error,
            "metadata": result.metadata,
        })
    }
}
