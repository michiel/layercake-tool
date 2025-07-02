//! MCP prompts implementation
//! 
//! Prompts provide templates for AI assistants to analyze and work with graph data.

pub mod graph_analysis;

use crate::mcp::protocol::Prompt;
use serde_json::Value;

/// Get all available MCP prompts
pub fn get_prompts() -> Vec<Prompt> {
    let mut prompts = Vec::new();
    
    // Add graph analysis prompts
    prompts.extend(graph_analysis::get_graph_analysis_prompts());
    
    prompts
}

/// Get prompt content by name
pub async fn get_prompt_content(
    prompt_name: &str,
    arguments: Option<Value>,
    db: &sea_orm::DatabaseConnection,
) -> Result<Value, String> {
    match prompt_name {
        name if name.starts_with("analyze_") => {
            graph_analysis::get_graph_analysis_prompt_content(name, arguments, db).await
        }
        _ => Err(format!("Unknown prompt: {}", prompt_name)),
    }
}