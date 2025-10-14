//! Graph analysis tools for MCP

use axum_mcp::prelude::*;
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};

/// Get graph analysis tools
pub fn get_analysis_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_connectivity".to_string(),
            description: "Analyze graph connectivity and structure".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to analyze"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "find_paths".to_string(),
            description: "Find paths between nodes in the graph".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project"
                    },
                    "source_node": {
                        "type": "string",
                        "description": "ID of the source node"
                    },
                    "target_node": {
                        "type": "string",
                        "description": "ID of the target node"
                    },
                    "max_paths": {
                        "type": "integer",
                        "description": "Maximum number of paths to find (default: 10)"
                    }
                },
                "required": ["project_id", "source_node", "target_node"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

/// Analyze graph connectivity
pub async fn analyze_connectivity(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    // TODO: Fix this function after data model refactoring
    Err(McpError::Internal {
        message: "analyze_connectivity is not implemented yet".to_string(),
    })
}

/// Find paths between nodes
pub async fn find_paths(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    // TODO: Fix this function after data model refactoring
    Err(McpError::Internal {
        message: "find_paths is not implemented yet".to_string(),
    })
}

/// Find connected components using DFS
fn find_connected_components(adjacency: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut components: Vec<Vec<String>> = Vec::new();

    for node in adjacency.keys() {
        if !visited.contains(node) {
            let mut component = Vec::new();
            dfs_component(node, adjacency, &mut visited, &mut component);
            if !component.is_empty() {
                components.push(component);
            }
        }
    }

    components
}

/// DFS to find a connected component
fn dfs_component(
    node: &str,
    adjacency: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    component: &mut Vec<String>,
) {
    visited.insert(node.to_string());
    component.push(node.to_string());

    if let Some(neighbors) = adjacency.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_component(neighbor, adjacency, visited, component);
            }
        }
    }
}

/// Find all paths between two nodes using modified BFS
fn find_all_paths(
    adjacency: &HashMap<String, Vec<String>>,
    source: &str,
    target: &str,
    max_paths: usize,
) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    let mut queue = VecDeque::new();

    // Start with a path containing only the source
    queue.push_back(vec![source.to_string()]);

    while let Some(current_path) = queue.pop_front() {
        if paths.len() >= max_paths {
            break;
        }

        let current_node = current_path.last().unwrap();

        if current_node == target {
            paths.push(current_path);
            continue;
        }

        // Avoid cycles by checking if we've already visited this node in this path
        if current_path.len() > 10 {
            // Prevent very long paths
            continue;
        }

        if let Some(neighbors) = adjacency.get(current_node) {
            for neighbor in neighbors {
                if !current_path.contains(neighbor) {
                    // Avoid cycles
                    let mut new_path = current_path.clone();
                    new_path.push(neighbor.clone());
                    queue.push_back(new_path);
                }
            }
        }
    }

    paths
}
