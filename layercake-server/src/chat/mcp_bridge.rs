use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

use layercake_core::app_context::AppContext;
use sea_orm::DatabaseConnection;

#[derive(Clone, Debug)]
pub struct ToolContext {
    pub user_id: i32,
    pub user_name: String,
    pub project_id: i32,
}

#[derive(Clone, Debug)]
pub struct ToolDefinition {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct ToolExecutionResult {
    pub content: Vec<ToolContent>,
    pub is_error: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub enum ToolContent {
    Text { text: String },
}

/// Convenience wrapper for tool invocation; MCP transport has been removed.
pub struct McpBridge {
    _app: Arc<AppContext>,
}

impl McpBridge {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            _app: Arc::new(AppContext::new(db)),
        }
    }

    pub async fn list_tools(&self, _context: &ToolContext) -> Result<Vec<ToolDefinition>> {
        Ok(Vec::new())
    }

    pub async fn execute_tool(
        &self,
        _name: &str,
        context: &ToolContext,
        _arguments: Option<serde_json::Value>,
    ) -> Result<ToolExecutionResult> {
        Ok(ToolExecutionResult {
            content: vec![ToolContent::Text {
                text: "MCP tools are disabled for this server.".to_string(),
            }],
            is_error: true,
            metadata: Some(json!({
                "userId": context.user_id,
                "userName": context.user_name,
                "projectId": context.project_id,
            })),
        })
    }

    pub fn summarize_tool_result(result: &ToolExecutionResult) -> String {
        let mut parts = Vec::new();
        for content in &result.content {
            match content {
                ToolContent::Text { text } => parts.push(text.clone()),
            }
        }

        if parts.is_empty() {
            "(tool produced no textual output)".to_string()
        } else {
            parts.join("\n")
        }
    }

    pub fn serialize_tool_result(result: &ToolExecutionResult) -> serde_json::Value {
        let content: Vec<serde_json::Value> = result
            .content
            .iter()
            .map(|item| match item {
                ToolContent::Text { text } => json!({"type": "text", "text": text}),
            })
            .collect();

        json!({
            "content": content,
            "is_error": result.is_error,
            "metadata": result.metadata,
        })
    }
}
