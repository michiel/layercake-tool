//! DOT format import/export functionality for Graphviz

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::debug;

use crate::graph::{Graph, Node, Edge, Layer};
use super::{ImportOptions, ExportOptions, ImportResult, ExportResult};

/// Export graph to DOT format (Graphviz)
pub fn export_dot(graph: &Graph, file_path: &Path, options: &ExportOptions) -> Result<ExportResult> {
    debug!("Exporting DOT to: {}", file_path.display());
    
    let mut dot_content = String::new();
    
    // Graph header
    dot_content.push_str(&format!("digraph \"{}\" {{\n", escape_dot_string(&graph.name)));
    dot_content.push_str("  rankdir=TB;\n");
    dot_content.push_str("  node [shape=box, style=filled];\n");
    dot_content.push_str("  edge [fontsize=10];\n\n");
    
    // Layer-based subgraphs (if requested)
    if options.include_layers && !graph.layers.is_empty() {
        write_layer_subgraphs(&mut dot_content, graph, options)?;
    } else {
        write_nodes_direct(&mut dot_content, graph, options)?;
    }
    
    // Write edges
    write_edges(&mut dot_content, graph, options)?;
    
    dot_content.push_str("}\n");
    
    fs::write(file_path, dot_content)?;
    
    Ok(ExportResult {
        success: true,
        output_path: file_path.display().to_string(),
        nodes_exported: graph.nodes.len(),
        edges_exported: graph.edges.len(),
        layers_exported: if options.include_layers { graph.layers.len() } else { 0 },
        warnings: vec![],
        error: None,
    })
}

/// Import graph from DOT format (basic implementation)
pub fn import_dot(file_path: &Path, options: &ImportOptions) -> Result<(Graph, ImportResult)> {
    debug!("Importing DOT from: {}", file_path.display());
    
    let content = fs::read_to_string(file_path)?;
    let mut graph = Graph::default();
    let mut warnings = Vec::new();
    
    // Set graph name from file
    if let Some(file_stem) = file_path.file_stem() {
        graph.name = file_stem.to_string_lossy().to_string();
    }
    
    // Basic DOT parsing (simplified)
    let (nodes, edges, parse_warnings) = parse_dot_content(&content)?;
    warnings.extend(parse_warnings);
    
    graph.nodes = nodes;
    graph.edges = edges;
    
    // Auto-generate layers if requested
    if options.auto_generate_layers {
        graph.layers = auto_generate_layers_from_nodes(&graph.nodes);
    }
    
    let result = ImportResult {
        success: true,
        nodes_imported: graph.nodes.len(),
        edges_imported: graph.edges.len(),
        layers_imported: graph.layers.len(),
        warnings,
        error: None,
    };
    
    Ok((graph, result))
}

/// Write layer-based subgraphs
fn write_layer_subgraphs(
    dot_content: &mut String,
    graph: &Graph,
    options: &ExportOptions,
) -> Result<()> {
    // Group nodes by layer
    let mut layer_nodes: HashMap<String, Vec<&Node>> = HashMap::new();
    for node in &graph.nodes {
        layer_nodes.entry(node.layer.clone()).or_default().push(node);
    }
    
    // Create subgraph for each layer
    for (layer_id, nodes) in layer_nodes {
        if let Some(layer) = graph.layers.iter().find(|l| l.id == layer_id) {
            dot_content.push_str(&format!(
                "  subgraph \"cluster_{}\" {{\n",
                escape_dot_string(&layer.id)
            ));
            dot_content.push_str(&format!(
                "    label=\"{}\";\n",
                escape_dot_string(&layer.label)
            ));
            dot_content.push_str(&format!(
                "    style=filled;\n    fillcolor=\"{}\";\n",
                convert_color_to_dot(&layer.background_color)
            ));
            dot_content.push_str(&format!(
                "    fontcolor=\"{}\";\n\n",
                convert_color_to_dot(&layer.text_color)
            ));
            
            // Write nodes in this layer
            for node in nodes {
                write_node(dot_content, node, Some(layer))?;
            }
            
            dot_content.push_str("  }\n\n");
        } else {
            // Layer not found, write nodes without subgraph
            for node in nodes {
                write_node(dot_content, node, None)?;
            }
        }
    }
    
    Ok(())
}

/// Write nodes directly without layer subgraphs
fn write_nodes_direct(
    dot_content: &mut String,
    graph: &Graph,
    options: &ExportOptions,
) -> Result<()> {
    for node in &graph.nodes {
        let layer = graph.layers.iter().find(|l| l.id == node.layer);
        write_node(dot_content, node, layer)?;
    }
    Ok(())
}

/// Write a single node
fn write_node(dot_content: &mut String, node: &Node, layer: Option<&Layer>) -> Result<()> {
    let node_id = escape_dot_id(&node.id);
    let label = escape_dot_string(&node.label);
    
    let mut attributes = vec![
        format!("label=\"{}\"", label),
    ];
    
    // Add layer-based styling
    if let Some(layer) = layer {
        attributes.push(format!("fillcolor=\"{}\"", convert_color_to_dot(&layer.background_color)));
        attributes.push(format!("fontcolor=\"{}\"", convert_color_to_dot(&layer.text_color)));
        attributes.push(format!("color=\"{}\"", convert_color_to_dot(&layer.border_color)));
    }
    
    // Partition nodes get different shape
    if node.is_partition {
        attributes.push("shape=folder".to_string());
        attributes.push("style=\"filled,bold\"".to_string());
    }
    
    // Add weight as tooltip
    if node.weight != 1 {
        attributes.push(format!("tooltip=\"Weight: {}\"", node.weight));
    }
    
    // Add comment if available
    if let Some(comment) = &node.comment {
        if !comment.is_empty() && comment != "null" {
            attributes.push(format!("xlabel=\"{}\"", escape_dot_string(comment)));
        }
    }
    
    dot_content.push_str(&format!(
        "  {} [{}];\n",
        node_id,
        attributes.join(", ")
    ));
    
    Ok(())
}

/// Write edges
fn write_edges(dot_content: &mut String, graph: &Graph, options: &ExportOptions) -> Result<()> {
    dot_content.push_str("\n  // Edges\n");
    
    for edge in &graph.edges {
        let source_id = escape_dot_id(&edge.source);
        let target_id = escape_dot_id(&edge.target);
        
        let mut attributes = Vec::new();
        
        // Add label if not empty
        if !edge.label.is_empty() {
            attributes.push(format!("label=\"{}\"", escape_dot_string(&edge.label)));
        }
        
        // Add weight styling
        if edge.weight != 1 {
            let penwidth = (edge.weight as f32 / 2.0).max(0.5).min(5.0);
            attributes.push(format!("penwidth={:.1}", penwidth));
            attributes.push(format!("tooltip=\"Weight: {}\"", edge.weight));
        }
        
        // Add comment if available
        if let Some(comment) = &edge.comment {
            if !comment.is_empty() && comment != "null" {
                attributes.push(format!("xlabel=\"{}\"", escape_dot_string(comment)));
            }
        }
        
        let attr_string = if attributes.is_empty() {
            String::new()
        } else {
            format!(" [{}]", attributes.join(", "))
        };
        
        dot_content.push_str(&format!(
            "  {} -> {}{};\n",
            source_id, target_id, attr_string
        ));
    }
    
    Ok(())
}

/// Parse DOT content (simplified implementation)
fn parse_dot_content(content: &str) -> Result<(Vec<Node>, Vec<Edge>, Vec<String>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut warnings = Vec::new();
    let mut node_counter = 0;
    let mut edge_counter = 0;
    
    warnings.push("DOT import is simplified and may not preserve all formatting".to_string());
    
    // Very basic parsing - this is a simplified implementation
    // A full DOT parser would be much more complex
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
            continue;
        }
        
        // Skip graph declarations and braces
        if line.starts_with("digraph") || line.starts_with("graph") || 
           line == "{" || line == "}" || line.starts_with("rankdir") ||
           line.starts_with("node") || line.starts_with("edge") {
            continue;
        }
        
        // Parse node definitions
        if line.contains("[") && !line.contains("->") && !line.contains("--") {
            if let Some(node) = parse_dot_node(line, &mut node_counter) {
                nodes.push(node);
            }
        }
        
        // Parse edge definitions
        if line.contains("->") || line.contains("--") {
            if let Some(edge) = parse_dot_edge(line, &mut edge_counter) {
                edges.push(edge);
            }
        }
    }
    
    Ok((nodes, edges, warnings))
}

/// Parse a DOT node definition
fn parse_dot_node(line: &str, counter: &mut usize) -> Option<Node> {
    // Very simplified node parsing
    let parts: Vec<&str> = line.split('[').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let node_id = parts[0].trim().trim_matches('"').to_string();
    let attributes = parts[1].trim_end_matches("];").trim_end_matches(";");
    
    let mut label = node_id.clone();
    
    // Extract label from attributes (simplified)
    if let Some(label_start) = attributes.find("label=\"") {
        let start = label_start + 7;
        if let Some(label_end) = attributes[start..].find('"') {
            label = attributes[start..start + label_end].to_string();
        }
    }
    
    *counter += 1;
    
    Some(Node {
        id: node_id,
        label,
        layer: "default".to_string(),
        is_partition: false,
        belongs_to: None,
        weight: 1,
        comment: None,
    })
}

/// Parse a DOT edge definition
fn parse_dot_edge(line: &str, counter: &mut usize) -> Option<Edge> {
    // Very simplified edge parsing
    let arrow = if line.contains("->") { "->" } else { "--" };
    let parts: Vec<&str> = line.split(arrow).collect();
    if parts.len() < 2 {
        return None;
    }
    
    let source = parts[0].trim().trim_matches('"').to_string();
    let target_part = parts[1].trim();
    
    // Extract target (before any attributes)
    let target = if let Some(bracket_pos) = target_part.find('[') {
        target_part[..bracket_pos].trim().trim_matches('"').to_string()
    } else {
        target_part.trim_end_matches(';').trim_matches('"').to_string()
    };
    
    *counter += 1;
    
    Some(Edge {
        id: format!("edge_{}", counter),
        source,
        target,
        label: String::new(),
        layer: "default".to_string(),
        weight: 1,
        comment: None,
    })
}

/// Auto-generate layers from nodes
fn auto_generate_layers_from_nodes(nodes: &[Node]) -> Vec<Layer> {
    let mut layer_ids: std::collections::HashSet<String> = nodes
        .iter()
        .map(|n| n.layer.clone())
        .collect();
    
    layer_ids.into_iter()
        .map(|id| Layer {
            id: id.clone(),
            label: id.clone(),
            background_color: "#f0f0f0".to_string(),
            text_color: "#000000".to_string(),
            border_color: "#cccccc".to_string(),
        })
        .collect()
}

/// Escape string for DOT format
fn escape_dot_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape identifier for DOT format
fn escape_dot_id(s: &str) -> String {
    // If the ID contains special characters, quote it
    if s.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        format!("\"{}\"", escape_dot_string(s))
    } else {
        s.to_string()
    }
}

/// Convert color format to DOT-compatible format
fn convert_color_to_dot(color: &str) -> String {
    // DOT supports hex colors, named colors, and RGB
    if color.starts_with('#') {
        // Already in hex format
        color.to_string()
    } else if color.starts_with("rgb(") {
        // Convert RGB to hex (simplified)
        color.to_string()
    } else {
        // Assume it's a named color or pass through
        color.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    fn create_test_graph() -> Graph {
        Graph {
            name: "Test Graph".to_string(),
            nodes: vec![
                Node {
                    id: "A".to_string(),
                    label: "Node A".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: Some("Comment A".to_string()),
                },
                Node {
                    id: "B".to_string(),
                    label: "Node B".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 2,
                    comment: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    label: "connects".to_string(),
                    layer: "layer1".to_string(),
                    weight: 3,
                    comment: Some("Edge comment".to_string()),
                },
            ],
            layers: vec![
                Layer {
                    id: "layer1".to_string(),
                    label: "Layer 1".to_string(),
                    background_color: "#ffffff".to_string(),
                    text_color: "#000000".to_string(),
                    border_color: "#cccccc".to_string(),
                },
            ],
        }
    }
    
    #[test]
    fn test_dot_export() {
        let graph = create_test_graph();
        let temp_file = NamedTempFile::new().unwrap();
        
        let export_options = ExportOptions {
            format: super::super::GraphFormat::DOT,
            include_layers: true,
            ..Default::default()
        };
        
        let result = export_dot(&graph, temp_file.path(), &export_options).unwrap();
        assert!(result.success);
        assert_eq!(result.nodes_exported, 2);
        assert_eq!(result.edges_exported, 1);
        assert_eq!(result.layers_exported, 1);
        
        // Check that file was created and has content
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("digraph"));
        assert!(content.contains("Node A"));
        assert!(content.contains("A -> B"));
    }
    
    #[test]
    fn test_escape_functions() {
        assert_eq!(escape_dot_string("hello"), "hello");
        assert_eq!(escape_dot_string("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_dot_string("line1\nline2"), "line1\\nline2");
        
        assert_eq!(escape_dot_id("simple"), "simple");
        assert_eq!(escape_dot_id("with-dash"), "\"with-dash\"");
        assert_eq!(escape_dot_id("with space"), "\"with space\"");
    }
}