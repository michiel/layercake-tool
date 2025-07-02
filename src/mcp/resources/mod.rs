//! MCP resources implementation
//! 
//! Resources provide read-only access to data and content.

pub mod projects;
pub mod graph_data;

use crate::mcp::protocol::Resource;
use serde_json::Value;

/// Get all available MCP resources
pub fn get_resources() -> Vec<Resource> {
    let mut resources = Vec::new();
    
    // Add project resources
    resources.extend(projects::get_project_resources());
    
    // Add graph data resources
    resources.extend(graph_data::get_graph_data_resources());
    
    resources
}

/// Get resource content by URI
pub async fn get_resource_content(
    uri: &str,
    db: &sea_orm::DatabaseConnection,
) -> Result<Value, String> {
    if uri.starts_with("layercake://project/") {
        projects::get_project_resource_content(uri, db).await
    } else if uri.starts_with("layercake://graph/") {
        graph_data::get_graph_data_resource_content(uri, db).await
    } else {
        Err(format!("Unsupported resource URI: {}", uri))
    }
}