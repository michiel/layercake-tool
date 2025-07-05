//! CSV import/export functionality

use anyhow::{Result, anyhow};
use csv::{Reader, Writer};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use crate::graph::{Graph, Node, Edge, Layer};
use super::{ImportOptions, ExportOptions, ImportResult, ExportResult, MergeMode, FieldMappings};

/// Import graph from CSV files (nodes.csv, edges.csv, layers.csv)
pub fn import_csv(base_path: &Path, options: &ImportOptions) -> Result<(Graph, ImportResult)> {
    debug!("Importing CSV from: {}", base_path.display());
    
    let mut graph = Graph::default();
    let mut warnings = Vec::new();
    let mut nodes_imported = 0;
    let mut edges_imported = 0;
    let mut layers_imported = 0;
    
    // Determine file paths
    let (nodes_path, edges_path, layers_path) = resolve_csv_paths(base_path)?;
    
    // Import layers first (if exists)
    if layers_path.exists() {
        debug!("Importing layers from: {}", layers_path.display());
        let (layers, layer_warnings) = import_layers_csv(&layers_path, options)?;
        graph.layers = layers;
        layers_imported = graph.layers.len();
        warnings.extend(layer_warnings);
    } else if options.auto_generate_layers {
        debug!("No layers file found, will auto-generate layers");
    }
    
    // Import nodes
    if nodes_path.exists() {
        debug!("Importing nodes from: {}", nodes_path.display());
        let (nodes, node_warnings) = import_nodes_csv(&nodes_path, options)?;
        graph.nodes = nodes;
        nodes_imported = graph.nodes.len();
        warnings.extend(node_warnings);
        
        // Auto-generate layers if needed
        if graph.layers.is_empty() && options.auto_generate_layers {
            let auto_layers = auto_generate_layers(&graph.nodes);
            graph.layers = auto_layers;
            layers_imported = graph.layers.len();
            warnings.push("Auto-generated layers from node data".to_string());
        }
    } else {
        return Err(anyhow!("Nodes file not found: {}", nodes_path.display()));
    }
    
    // Import edges (if exists)
    if edges_path.exists() {
        debug!("Importing edges from: {}", edges_path.display());
        let (edges, edge_warnings) = import_edges_csv(&edges_path, options)?;
        graph.edges = edges;
        edges_imported = graph.edges.len();
        warnings.extend(edge_warnings);
    } else {
        debug!("No edges file found, graph will have no edges");
    }
    
    // Set graph name based on file path
    if let Some(file_stem) = base_path.file_stem() {
        graph.name = file_stem.to_string_lossy().to_string();
    }
    
    let result = ImportResult {
        success: true,
        nodes_imported,
        edges_imported,
        layers_imported,
        warnings,
        error: None,
    };
    
    Ok((graph, result))
}

/// Export graph to CSV files
pub fn export_csv(graph: &Graph, base_path: &Path, options: &ExportOptions) -> Result<ExportResult> {
    debug!("Exporting CSV to: {}", base_path.display());
    
    let mut warnings = Vec::new();
    
    // Create output directory if it doesn't exist
    if let Some(parent) = base_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Determine output file paths
    let base_name = base_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("graph");
    let parent_dir = base_path.parent().unwrap_or(Path::new("."));
    
    let nodes_path = parent_dir.join(format!("{}_nodes.csv", base_name));
    let edges_path = parent_dir.join(format!("{}_edges.csv", base_name));
    let layers_path = parent_dir.join(format!("{}_layers.csv", base_name));
    
    // Export nodes
    export_nodes_csv(&graph.nodes, &nodes_path, options)?;
    
    // Export edges
    export_edges_csv(&graph.edges, &edges_path, options)?;
    
    // Export layers (if requested and available)
    if options.include_layers && !graph.layers.is_empty() {
        export_layers_csv(&graph.layers, &layers_path, options)?;
    }
    
    Ok(ExportResult {
        success: true,
        output_path: base_path.display().to_string(),
        nodes_exported: graph.nodes.len(),
        edges_exported: graph.edges.len(),
        layers_exported: if options.include_layers { graph.layers.len() } else { 0 },
        warnings,
        error: None,
    })
}

/// Resolve CSV file paths from base path
fn resolve_csv_paths(base_path: &Path) -> Result<(PathBuf, PathBuf, PathBuf)> {
    if base_path.is_file() {
        // Single file provided, derive other paths
        let parent = base_path.parent().ok_or_else(|| anyhow!("Invalid file path"))?;
        let file_stem = base_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;
        
        let base_name = if file_stem.ends_with("_nodes") {
            &file_stem[..file_stem.len() - 6]
        } else if file_stem.ends_with("_edges") {
            &file_stem[..file_stem.len() - 6]
        } else if file_stem.ends_with("_layers") {
            &file_stem[..file_stem.len() - 7]
        } else {
            file_stem
        };
        
        Ok((
            parent.join(format!("{}_nodes.csv", base_name)),
            parent.join(format!("{}_edges.csv", base_name)),
            parent.join(format!("{}_layers.csv", base_name)),
        ))
    } else if base_path.is_dir() {
        // Directory provided, look for standard files
        Ok((
            base_path.join("nodes.csv"),
            base_path.join("edges.csv"),
            base_path.join("layers.csv"),
        ))
    } else {
        // Path doesn't exist, assume it's a base name
        let parent = base_path.parent().unwrap_or(Path::new("."));
        let file_stem = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("graph");
        
        Ok((
            parent.join(format!("{}_nodes.csv", file_stem)),
            parent.join(format!("{}_edges.csv", file_stem)),
            parent.join(format!("{}_layers.csv", file_stem)),
        ))
    }
}

/// Import nodes from CSV file
fn import_nodes_csv(file_path: &Path, options: &ImportOptions) -> Result<(Vec<Node>, Vec<String>)> {
    let mut reader = Reader::from_path(file_path)?;
    let mut nodes = Vec::new();
    let mut warnings = Vec::new();
    
    // Get headers for field mapping
    let headers = reader.headers()?.clone();
    let field_map = build_node_field_mapping(&headers, options);
    
    for (line_num, result) in reader.records().enumerate() {
        let record = result?;
        
        match parse_node_record(&record, &field_map, &headers) {
            Ok(node) => nodes.push(node),
            Err(e) => {
                warnings.push(format!("Line {}: {}", line_num + 2, e));
            }
        }
    }
    
    Ok((nodes, warnings))
}

/// Import edges from CSV file
fn import_edges_csv(file_path: &Path, options: &ImportOptions) -> Result<(Vec<Edge>, Vec<String>)> {
    let mut reader = Reader::from_path(file_path)?;
    let mut edges = Vec::new();
    let mut warnings = Vec::new();
    
    // Get headers for field mapping
    let headers = reader.headers()?.clone();
    let field_map = build_edge_field_mapping(&headers, options);
    
    for (line_num, result) in reader.records().enumerate() {
        let record = result?;
        
        match parse_edge_record(&record, &field_map, &headers) {
            Ok(edge) => edges.push(edge),
            Err(e) => {
                warnings.push(format!("Line {}: {}", line_num + 2, e));
            }
        }
    }
    
    Ok((edges, warnings))
}

/// Import layers from CSV file
fn import_layers_csv(file_path: &Path, options: &ImportOptions) -> Result<(Vec<Layer>, Vec<String>)> {
    let mut reader = Reader::from_path(file_path)?;
    let mut layers = Vec::new();
    let mut warnings = Vec::new();
    
    // Get headers for field mapping
    let headers = reader.headers()?.clone();
    let field_map = build_layer_field_mapping(&headers, options);
    
    for (line_num, result) in reader.records().enumerate() {
        let record = result?;
        
        match parse_layer_record(&record, &field_map, &headers) {
            Ok(layer) => layers.push(layer),
            Err(e) => {
                warnings.push(format!("Line {}: {}", line_num + 2, e));
            }
        }
    }
    
    Ok((layers, warnings))
}

/// Export nodes to CSV file
fn export_nodes_csv(nodes: &[Node], file_path: &Path, options: &ExportOptions) -> Result<()> {
    let mut writer = Writer::from_path(file_path)?;
    
    // Write headers
    writer.write_record(&[
        "id", "label", "layer", "is_partition", "belongs_to", "weight", "comment"
    ])?;
    
    // Write node data
    for node in nodes {
        writer.write_record(&[
            &node.id,
            &node.label,
            &node.layer,
            &node.is_partition.to_string(),
            node.belongs_to.as_deref().unwrap_or(""),
            &node.weight.to_string(),
            node.comment.as_deref().unwrap_or(""),
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

/// Export edges to CSV file
fn export_edges_csv(edges: &[Edge], file_path: &Path, options: &ExportOptions) -> Result<()> {
    let mut writer = Writer::from_path(file_path)?;
    
    // Write headers
    writer.write_record(&[
        "id", "source", "target", "label", "layer", "weight", "comment"
    ])?;
    
    // Write edge data
    for edge in edges {
        writer.write_record(&[
            &edge.id,
            &edge.source,
            &edge.target,
            &edge.label,
            &edge.layer,
            &edge.weight.to_string(),
            edge.comment.as_deref().unwrap_or(""),
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

/// Export layers to CSV file
fn export_layers_csv(layers: &[Layer], file_path: &Path, options: &ExportOptions) -> Result<()> {
    let mut writer = Writer::from_path(file_path)?;
    
    // Write headers
    writer.write_record(&[
        "id", "label", "background_color", "text_color", "border_color"
    ])?;
    
    // Write layer data
    for layer in layers {
        writer.write_record(&[
            &layer.id,
            &layer.label,
            &layer.background_color,
            &layer.text_color,
            &layer.border_color,
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

/// Build field mapping for nodes
fn build_node_field_mapping(headers: &csv::StringRecord, options: &ImportOptions) -> HashMap<String, usize> {
    let mut field_map = HashMap::new();
    
    // Default field mappings
    let default_mappings = [
        ("id", vec!["id", "node_id", "name"]),
        ("label", vec!["label", "name", "title", "node_name"]),
        ("layer", vec!["layer", "layer_id", "type", "category"]),
        ("is_partition", vec!["is_partition", "partition", "is_group"]),
        ("belongs_to", vec!["belongs_to", "parent", "parent_id"]),
        ("weight", vec!["weight", "value", "size"]),
        ("comment", vec!["comment", "description", "notes"]),
    ];
    
    // Find field positions
    for (field, candidates) in default_mappings {
        for (i, header) in headers.iter().enumerate() {
            let header_lower = header.to_lowercase();
            if candidates.contains(&header_lower.as_str()) {
                field_map.insert(field.to_string(), i);
                break;
            }
        }
    }
    
    // Apply custom field mappings if provided
    if let Some(custom_mappings) = &options.field_mappings {
        for (custom_field, target_field) in &custom_mappings.node_fields {
            for (i, header) in headers.iter().enumerate() {
                if header.to_lowercase() == custom_field.to_lowercase() {
                    field_map.insert(target_field.clone(), i);
                    break;
                }
            }
        }
    }
    
    field_map
}

/// Build field mapping for edges
fn build_edge_field_mapping(headers: &csv::StringRecord, options: &ImportOptions) -> HashMap<String, usize> {
    let mut field_map = HashMap::new();
    
    // Default field mappings
    let default_mappings = [
        ("id", vec!["id", "edge_id"]),
        ("source", vec!["source", "from", "source_id", "src"]),
        ("target", vec!["target", "to", "target_id", "dst"]),
        ("label", vec!["label", "name", "type", "relation"]),
        ("layer", vec!["layer", "layer_id", "type", "category"]),
        ("weight", vec!["weight", "value", "strength"]),
        ("comment", vec!["comment", "description", "notes"]),
    ];
    
    // Find field positions
    for (field, candidates) in default_mappings {
        for (i, header) in headers.iter().enumerate() {
            let header_lower = header.to_lowercase();
            if candidates.contains(&header_lower.as_str()) {
                field_map.insert(field.to_string(), i);
                break;
            }
        }
    }
    
    // Apply custom field mappings if provided
    if let Some(custom_mappings) = &options.field_mappings {
        for (custom_field, target_field) in &custom_mappings.edge_fields {
            for (i, header) in headers.iter().enumerate() {
                if header.to_lowercase() == custom_field.to_lowercase() {
                    field_map.insert(target_field.clone(), i);
                    break;
                }
            }
        }
    }
    
    field_map
}

/// Build field mapping for layers
fn build_layer_field_mapping(headers: &csv::StringRecord, options: &ImportOptions) -> HashMap<String, usize> {
    let mut field_map = HashMap::new();
    
    // Default field mappings
    let default_mappings = [
        ("id", vec!["id", "layer_id", "name"]),
        ("label", vec!["label", "name", "title"]),
        ("background_color", vec!["background_color", "bg_color", "color"]),
        ("text_color", vec!["text_color", "font_color"]),
        ("border_color", vec!["border_color", "stroke_color"]),
    ];
    
    // Find field positions
    for (field, candidates) in default_mappings {
        for (i, header) in headers.iter().enumerate() {
            let header_lower = header.to_lowercase();
            if candidates.contains(&header_lower.as_str()) {
                field_map.insert(field.to_string(), i);
                break;
            }
        }
    }
    
    // Apply custom field mappings if provided
    if let Some(custom_mappings) = &options.field_mappings {
        for (custom_field, target_field) in &custom_mappings.layer_fields {
            for (i, header) in headers.iter().enumerate() {
                if header.to_lowercase() == custom_field.to_lowercase() {
                    field_map.insert(target_field.clone(), i);
                    break;
                }
            }
        }
    }
    
    field_map
}

/// Parse a CSV record into a Node
fn parse_node_record(
    record: &csv::StringRecord,
    field_map: &HashMap<String, usize>,
    headers: &csv::StringRecord,
) -> Result<Node> {
    let get_field = |field: &str| -> Option<&str> {
        field_map.get(field)
            .and_then(|&i| record.get(i))
            .filter(|s| !s.is_empty())
    };
    
    let id = get_field("id")
        .ok_or_else(|| anyhow!("Missing required field: id"))?
        .to_string();
    
    let label = get_field("label").unwrap_or(&id).to_string();
    let layer = get_field("layer").unwrap_or("default").to_string();
    
    let is_partition = get_field("is_partition")
        .map(|s| s.to_lowercase() == "true" || s == "1")
        .unwrap_or(false);
    
    let belongs_to = get_field("belongs_to").map(|s| s.to_string());
    
    let weight = get_field("weight")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);
    
    let comment = get_field("comment").map(|s| s.to_string());
    
    Ok(Node {
        id,
        label,
        layer,
        is_partition,
        belongs_to,
        weight,
        comment,
    })
}

/// Parse a CSV record into an Edge
fn parse_edge_record(
    record: &csv::StringRecord,
    field_map: &HashMap<String, usize>,
    headers: &csv::StringRecord,
) -> Result<Edge> {
    let get_field = |field: &str| -> Option<&str> {
        field_map.get(field)
            .and_then(|&i| record.get(i))
            .filter(|s| !s.is_empty())
    };
    
    let source = get_field("source")
        .ok_or_else(|| anyhow!("Missing required field: source"))?
        .to_string();
    
    let target = get_field("target")
        .ok_or_else(|| anyhow!("Missing required field: target"))?
        .to_string();
    
    let id = get_field("id")
        .unwrap_or(&format!("{}_{}", source, target))
        .to_string();
    
    let label = get_field("label").unwrap_or("").to_string();
    let layer = get_field("layer").unwrap_or("default").to_string();
    
    let weight = get_field("weight")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);
    
    let comment = get_field("comment").map(|s| s.to_string());
    
    Ok(Edge {
        id,
        source,
        target,
        label,
        layer,
        weight,
        comment,
    })
}

/// Parse a CSV record into a Layer
fn parse_layer_record(
    record: &csv::StringRecord,
    field_map: &HashMap<String, usize>,
    headers: &csv::StringRecord,
) -> Result<Layer> {
    let get_field = |field: &str| -> Option<&str> {
        field_map.get(field)
            .and_then(|&i| record.get(i))
            .filter(|s| !s.is_empty())
    };
    
    let id = get_field("id")
        .ok_or_else(|| anyhow!("Missing required field: id"))?
        .to_string();
    
    let label = get_field("label").unwrap_or(&id).to_string();
    let background_color = get_field("background_color").unwrap_or("#ffffff").to_string();
    let text_color = get_field("text_color").unwrap_or("#000000").to_string();
    let border_color = get_field("border_color").unwrap_or("#cccccc").to_string();
    
    Ok(Layer {
        id,
        label,
        background_color,
        text_color,
        border_color,
    })
}

/// Auto-generate layers from node data
fn auto_generate_layers(nodes: &[Node]) -> Vec<Layer> {
    let mut layer_ids: std::collections::HashSet<String> = nodes
        .iter()
        .map(|n| n.layer.clone())
        .collect();
    
    layer_ids.into_iter()
        .enumerate()
        .map(|(i, id)| {
            // Generate colors using a simple palette
            let colors = [
                ("#e3f2fd", "#000000", "#2196f3"), // Blue
                ("#f3e5f5", "#000000", "#9c27b0"), // Purple
                ("#e8f5e8", "#000000", "#4caf50"), // Green
                ("#fff3e0", "#000000", "#ff9800"), // Orange
                ("#ffebee", "#000000", "#f44336"), // Red
                ("#f1f8e9", "#000000", "#8bc34a"), // Light Green
                ("#fce4ec", "#000000", "#e91e63"), // Pink
                ("#e0f2f1", "#000000", "#009688"), // Teal
            ];
            
            let (bg, text, border) = colors[i % colors.len()];
            
            Layer {
                id: id.clone(),
                label: id.clone(),
                background_color: bg.to_string(),
                text_color: text.to_string(),
                border_color: border.to_string(),
            }
        })
        .collect()
}