//! MCP tools implementation
//! 
//! Tools are functions that can be called by AI assistants to perform operations.

pub mod projects;
pub mod graph_data;
pub mod plans;
pub mod analysis;

use crate::mcp::protocol::Tool;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get all available MCP tools
pub fn get_tools() -> Vec<Tool> {
    let mut tools = Vec::new();
    
    // Add project management tools
    tools.extend(projects::get_project_tools());
    
    // Add graph data tools
    tools.extend(graph_data::get_graph_data_tools());
    
    // Add plan tools
    tools.extend(plans::get_plan_tools());
    
    // Add analysis tools
    tools.extend(analysis::get_analysis_tools());
    
    tools
}

/// Execute a tool by name with given arguments
pub async fn execute_tool(
    tool_name: &str,
    arguments: Option<Value>,
    db: &sea_orm::DatabaseConnection,
) -> Result<Value, String> {
    match tool_name {
        // Project tools
        "list_projects" => projects::list_projects(db).await,
        "create_project" => projects::create_project(arguments, db).await,
        "get_project" => projects::get_project(arguments, db).await,
        "delete_project" => projects::delete_project(arguments, db).await,
        
        // Graph data tools
        "import_csv" => graph_data::import_csv(arguments, db).await,
        "export_graph" => graph_data::export_graph(arguments, db).await,
        "get_graph_data" => graph_data::get_graph_data(arguments, db).await,
        
        // Plan tools
        "create_plan" => plans::create_plan(arguments, db).await,
        "execute_plan" => plans::execute_plan(arguments, db).await,
        "get_plan_status" => plans::get_plan_status(arguments, db).await,
        
        // Analysis tools
        "analyze_connectivity" => analysis::analyze_connectivity(arguments, db).await,
        "find_paths" => analysis::find_paths(arguments, db).await,
        
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

/// Helper function to extract required parameter
pub fn get_required_param(args: &Option<Value>, param_name: &str) -> Result<Value, String> {
    match args {
        Some(Value::Object(map)) => map
            .get(param_name)
            .cloned()
            .ok_or_else(|| format!("Missing required parameter: {}", param_name)),
        _ => Err(format!("Invalid arguments format")),
    }
}

/// Helper function to extract optional parameter
pub fn get_optional_param(args: &Option<Value>, param_name: &str) -> Option<Value> {
    match args {
        Some(Value::Object(map)) => map.get(param_name).cloned(),
        _ => None,
    }
}