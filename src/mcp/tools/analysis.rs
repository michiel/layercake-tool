//! Graph analysis tools for MCP

use crate::mcp::protocol::Tool;
use crate::mcp::tools::{get_required_param, get_optional_param};
use crate::services::GraphService;
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
        },
    ]
}

/// Analyze graph connectivity
pub async fn analyze_connectivity(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or("Project ID must be a number")? as i32;

    let graph_service = GraphService::new(db.clone());
    
    // Get nodes and edges
    let nodes = graph_service.get_nodes_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get nodes: {}", e))?;
        
    let edges = graph_service.get_edges_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get edges: {}", e))?;

    // Build adjacency list for analysis
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, i32> = HashMap::new();
    let mut out_degree: HashMap<String, i32> = HashMap::new();

    // Initialize all nodes
    for node in &nodes {
        adjacency.insert(node.node_id.clone(), Vec::new());
        in_degree.insert(node.node_id.clone(), 0);
        out_degree.insert(node.node_id.clone(), 0);
    }

    // Build adjacency list and degree counts
    for edge in &edges {
        adjacency.entry(edge.source_node_id.clone())
            .or_default()
            .push(edge.target_node_id.clone());
        
        *out_degree.entry(edge.source_node_id.clone()).or_insert(0) += 1;
        *in_degree.entry(edge.target_node_id.clone()).or_insert(0) += 1;
    }

    // Find connected components
    let components = find_connected_components(&adjacency);
    
    // Calculate statistics
    let total_nodes = nodes.len();
    let total_edges = edges.len();
    let num_components = components.len();
    let largest_component_size = components.iter().map(|c| c.len()).max().unwrap_or(0);
    
    // Find nodes with special properties
    let isolated_nodes: Vec<&String> = nodes.iter()
        .map(|n| &n.node_id)
        .filter(|node_id| {
            in_degree.get(*node_id).unwrap_or(&0) == &0 && 
            out_degree.get(*node_id).unwrap_or(&0) == &0
        })
        .collect();

    let source_nodes: Vec<&String> = nodes.iter()
        .map(|n| &n.node_id)
        .filter(|node_id| {
            in_degree.get(*node_id).unwrap_or(&0) == &0 && 
            out_degree.get(*node_id).unwrap_or(&0) > &0
        })
        .collect();

    let sink_nodes: Vec<&String> = nodes.iter()
        .map(|n| &n.node_id)
        .filter(|node_id| {
            in_degree.get(*node_id).unwrap_or(&0) > &0 && 
            out_degree.get(*node_id).unwrap_or(&0) == &0
        })
        .collect();

    // Calculate density
    let max_edges = if total_nodes > 1 { total_nodes * (total_nodes - 1) } else { 0 };
    let density = if max_edges > 0 { total_edges as f64 / max_edges as f64 } else { 0.0 };

    Ok(json!({
        "project_id": project_id,
        "connectivity_analysis": {
            "basic_stats": {
                "total_nodes": total_nodes,
                "total_edges": total_edges,
                "density": density,
                "average_degree": if total_nodes > 0 { (total_edges * 2) as f64 / total_nodes as f64 } else { 0.0 }
            },
            "components": {
                "count": num_components,
                "largest_size": largest_component_size,
                "is_connected": num_components <= 1
            },
            "special_nodes": {
                "isolated_count": isolated_nodes.len(),
                "source_count": source_nodes.len(),
                "sink_count": sink_nodes.len(),
                "isolated_nodes": isolated_nodes,
                "source_nodes": source_nodes,
                "sink_nodes": sink_nodes
            },
            "degree_distribution": {
                "max_in_degree": in_degree.values().max().unwrap_or(&0),
                "max_out_degree": out_degree.values().max().unwrap_or(&0),
                "avg_in_degree": if total_nodes > 0 { in_degree.values().sum::<i32>() as f64 / total_nodes as f64 } else { 0.0 },
                "avg_out_degree": if total_nodes > 0 { out_degree.values().sum::<i32>() as f64 / total_nodes as f64 } else { 0.0 }
            }
        }
    }))
}

/// Find paths between nodes
pub async fn find_paths(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or("Project ID must be a number")? as i32;

    let source_node = get_required_param(&arguments, "source_node")?
        .as_str()
        .ok_or("Source node must be a string")?
        .to_string();

    let target_node = get_required_param(&arguments, "target_node")?
        .as_str()
        .ok_or("Target node must be a string")?
        .to_string();

    let max_paths = get_optional_param(&arguments, "max_paths")
        .and_then(|v| v.as_i64())
        .unwrap_or(10) as usize;

    let graph_service = GraphService::new(db.clone());
    
    // Get edges to build adjacency list
    let edges = graph_service.get_edges_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get edges: {}", e))?;

    // Build adjacency list
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &edges {
        adjacency.entry(edge.source_node_id.clone())
            .or_default()
            .push(edge.target_node_id.clone());
    }

    // Find paths using BFS with path tracking
    let paths = find_all_paths(&adjacency, &source_node, &target_node, max_paths);

    // Calculate path statistics
    let path_lengths: Vec<usize> = paths.iter().map(|p| p.len()).collect();
    let shortest_path_length = path_lengths.iter().min().cloned().unwrap_or(0);
    let longest_path_length = path_lengths.iter().max().cloned().unwrap_or(0);
    let avg_path_length = if !path_lengths.is_empty() {
        path_lengths.iter().sum::<usize>() as f64 / path_lengths.len() as f64
    } else {
        0.0
    };

    Ok(json!({
        "project_id": project_id,
        "source_node": source_node,
        "target_node": target_node,
        "path_analysis": {
            "paths_found": paths.len(),
            "paths": paths,
            "statistics": {
                "shortest_path_length": if shortest_path_length > 0 { shortest_path_length - 1 } else { 0 }, // -1 because path includes both endpoints
                "longest_path_length": if longest_path_length > 0 { longest_path_length - 1 } else { 0 },
                "average_path_length": if avg_path_length > 0.0 { avg_path_length - 1.0 } else { 0.0 },
                "reachable": !paths.is_empty()
            }
        }
    }))
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
        if current_path.len() > 10 { // Prevent very long paths
            continue;
        }

        if let Some(neighbors) = adjacency.get(current_node) {
            for neighbor in neighbors {
                if !current_path.contains(neighbor) { // Avoid cycles
                    let mut new_path = current_path.clone();
                    new_path.push(neighbor.clone());
                    queue.push_back(new_path);
                }
            }
        }
    }

    paths
}