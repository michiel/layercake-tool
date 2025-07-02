//! Graph analysis prompts for MCP

use crate::mcp::protocol::{Prompt, PromptArgument};
use crate::services::GraphService;
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};

/// Get graph analysis prompts
pub fn get_graph_analysis_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "analyze_graph_structure".to_string(),
            description: Some("Analyze the structure and connectivity of a graph".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "project_id".to_string(),
                    description: Some("ID of the project to analyze".to_string()),
                    required: Some(true),
                },
            ]),
        },
        Prompt {
            name: "analyze_node_relationships".to_string(),
            description: Some("Analyze relationships and connections for a specific node".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "project_id".to_string(),
                    description: Some("ID of the project".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "node_id".to_string(),
                    description: Some("ID of the node to analyze".to_string()),
                    required: Some(true),
                },
            ]),
        },
        Prompt {
            name: "analyze_layer_distribution".to_string(),
            description: Some("Analyze the distribution of nodes across layers".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "project_id".to_string(),
                    description: Some("ID of the project to analyze".to_string()),
                    required: Some(true),
                },
            ]),
        },
    ]
}

/// Get graph analysis prompt content
pub async fn get_graph_analysis_prompt_content(
    prompt_name: &str,
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    match prompt_name {
        "analyze_graph_structure" => analyze_graph_structure_prompt(arguments, db).await,
        "analyze_node_relationships" => analyze_node_relationships_prompt(arguments, db).await,
        "analyze_layer_distribution" => analyze_layer_distribution_prompt(arguments, db).await,
        _ => Err(format!("Unknown graph analysis prompt: {}", prompt_name)),
    }
}

/// Generate graph structure analysis prompt
async fn analyze_graph_structure_prompt(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = arguments
        .as_ref()
        .and_then(|args| args.get("project_id"))
        .and_then(|v| v.as_i64())
        .ok_or("Missing required argument: project_id")? as i32;

    let graph_service = GraphService::new(db.clone());
    
    let nodes = graph_service.get_nodes_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get nodes: {}", e))?;
        
    let edges = graph_service.get_edges_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get edges: {}", e))?;
        
    let layers = graph_service.get_layers_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get layers: {}", e))?;

    let prompt_text = format!(
        r#"Please analyze the structure of this graph and provide insights:

## Graph Overview
- **Nodes**: {} total nodes
- **Edges**: {} total edges  
- **Layers**: {} total layers

## Graph Data
**Nodes**: {}

**Edges**: {}

**Layers**: {}

## Analysis Questions
Please provide a comprehensive analysis covering:

1. **Connectivity**: How well-connected is this graph? Are there isolated components?
2. **Structure**: What patterns do you observe in the graph structure?
3. **Layers**: How are nodes distributed across layers? What might this indicate?
4. **Key Nodes**: Which nodes appear to be most important (high degree, central position)?
5. **Flow Patterns**: What patterns do you see in the edge connections?
6. **Potential Issues**: Are there any structural issues or anomalies?

Please format your response with clear sections and actionable insights."#,
        nodes.len(),
        edges.len(),
        layers.len(),
        serde_json::to_string_pretty(&nodes).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&edges).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string_pretty(&layers).unwrap_or_else(|_| "[]".to_string())
    );

    Ok(json!({
        "prompt": prompt_text,
        "metadata": {
            "project_id": project_id,
            "node_count": nodes.len(),
            "edge_count": edges.len(),
            "layer_count": layers.len(),
            "generated_at": chrono::Utc::now()
        }
    }))
}

/// Generate node relationships analysis prompt
async fn analyze_node_relationships_prompt(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = arguments
        .as_ref()
        .and_then(|args| args.get("project_id"))
        .and_then(|v| v.as_i64())
        .ok_or("Missing required argument: project_id")? as i32;

    let node_id = arguments
        .as_ref()
        .and_then(|args| args.get("node_id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing required argument: node_id")?;

    let graph_service = GraphService::new(db.clone());
    
    let nodes = graph_service.get_nodes_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get nodes: {}", e))?;
        
    let edges = graph_service.get_edges_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get edges: {}", e))?;

    // Find the target node
    let target_node = nodes.iter()
        .find(|n| n.node_id == node_id)
        .ok_or("Node not found")?;

    // Find incoming and outgoing edges
    let incoming_edges: Vec<_> = edges.iter()
        .filter(|e| e.target_node_id == node_id)
        .collect();
    
    let outgoing_edges: Vec<_> = edges.iter()
        .filter(|e| e.source_node_id == node_id)
        .collect();

    let prompt_text = format!(
        r#"Please analyze the relationships and connections for this specific node:

## Target Node
{}

## Relationships
**Incoming Connections** ({}): {}

**Outgoing Connections** ({}): {}

## Analysis Questions
Please provide a detailed analysis covering:

1. **Role Analysis**: What role does this node play in the graph? (hub, leaf, bridge, etc.)
2. **Connectivity Pattern**: What patterns do you observe in its connections?
3. **Importance**: How important is this node to the overall graph structure?
4. **Dependencies**: What other nodes does this node depend on? What depends on it?
5. **Layer Context**: How does its layer placement relate to its connections?
6. **Recommendations**: Are there any recommendations for this node's connections?

Please provide specific insights based on the connection patterns."#,
        serde_json::to_string_pretty(&target_node).unwrap_or_else(|_| "{}".to_string()),
        incoming_edges.len(),
        serde_json::to_string_pretty(&incoming_edges).unwrap_or_else(|_| "[]".to_string()),
        outgoing_edges.len(),
        serde_json::to_string_pretty(&outgoing_edges).unwrap_or_else(|_| "[]".to_string())
    );

    Ok(json!({
        "prompt": prompt_text,
        "metadata": {
            "project_id": project_id,
            "node_id": node_id,
            "incoming_count": incoming_edges.len(),
            "outgoing_count": outgoing_edges.len(),
            "generated_at": chrono::Utc::now()
        }
    }))
}

/// Generate layer distribution analysis prompt
async fn analyze_layer_distribution_prompt(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = arguments
        .as_ref()
        .and_then(|args| args.get("project_id"))
        .and_then(|v| v.as_i64())
        .ok_or("Missing required argument: project_id")? as i32;

    let graph_service = GraphService::new(db.clone());
    
    let nodes = graph_service.get_nodes_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get nodes: {}", e))?;
        
    let layers = graph_service.get_layers_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get layers: {}", e))?;

    // Calculate layer distribution
    let mut layer_distribution = std::collections::HashMap::new();
    for node in &nodes {
        if let Some(layer_id) = &node.layer_id {
            *layer_distribution.entry(layer_id.clone()).or_insert(0) += 1;
        } else {
            *layer_distribution.entry("unlayered".to_string()).or_insert(0) += 1;
        }
    }

    let prompt_text = format!(
        r#"Please analyze the distribution of nodes across layers in this graph:

## Layer Information
**Total Layers**: {}
**Layer Definitions**: {}

## Node Distribution
**Total Nodes**: {}
**Distribution by Layer**: {}

## Analysis Questions
Please provide insights covering:

1. **Balance**: How balanced is the distribution of nodes across layers?
2. **Layer Utilization**: Are all layers being effectively used?
3. **Unlayered Nodes**: How many nodes lack layer assignments? Is this intentional?
4. **Layer Purpose**: Based on the distribution, what might each layer represent?
5. **Optimization**: Are there opportunities to redistribute nodes for better organization?
6. **Patterns**: What patterns emerge from the layer distribution?

Please provide specific recommendations for layer organization and management."#,
        layers.len(),
        serde_json::to_string_pretty(&layers).unwrap_or_else(|_| "[]".to_string()),
        nodes.len(),
        serde_json::to_string_pretty(&layer_distribution).unwrap_or_else(|_| "{}".to_string())
    );

    Ok(json!({
        "prompt": prompt_text,
        "metadata": {
            "project_id": project_id,
            "total_nodes": nodes.len(),
            "total_layers": layers.len(),
            "layer_distribution": layer_distribution,
            "generated_at": chrono::Utc::now()
        }
    }))
}