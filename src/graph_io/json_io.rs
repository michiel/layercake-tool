//! JSON import/export functionality

use anyhow::{Result, anyhow};
use serde_json;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use crate::graph::{Graph, Node, Edge, Layer};
use super::{ImportOptions, ExportOptions, ImportResult, ExportResult, MergeMode};

/// Import graph from JSON format
pub fn import_json(file_path: &Path, options: &ImportOptions) -> Result<(Graph, ImportResult)> {
    debug!("Importing JSON from: {}", file_path.display());
    
    let content = fs::read_to_string(file_path)?;
    let graph: Graph = serde_json::from_str(&content)?;
    
    let result = ImportResult {
        success: true,
        nodes_imported: graph.nodes.len(),
        edges_imported: graph.edges.len(),
        layers_imported: graph.layers.len(),
        warnings: vec![],
        error: None,
    };
    
    Ok((graph, result))
}

/// Import graph from Layercake native format (JSON with metadata)
pub fn import_layercake(file_path: &Path, options: &ImportOptions) -> Result<(Graph, ImportResult)> {
    debug!("Importing Layercake format from: {}", file_path.display());
    
    let content = fs::read_to_string(file_path)?;
    
    // Try to parse as LayercakeFormat first, fall back to plain Graph
    let graph = if let Ok(layercake_data) = serde_json::from_str::<LayercakeFormat>(&content) {
        debug!("Parsed as Layercake format with metadata");
        layercake_data.graph
    } else {
        debug!("Falling back to plain Graph format");
        serde_json::from_str::<Graph>(&content)?
    };
    
    let result = ImportResult {
        success: true,
        nodes_imported: graph.nodes.len(),
        edges_imported: graph.edges.len(),
        layers_imported: graph.layers.len(),
        warnings: vec![],
        error: None,
    };
    
    Ok((graph, result))
}

/// Export graph to JSON format
pub fn export_json(graph: &Graph, file_path: &Path, options: &ExportOptions) -> Result<ExportResult> {
    debug!("Exporting JSON to: {}", file_path.display());
    
    let json_string = if options.prettify {
        serde_json::to_string_pretty(graph)?
    } else {
        serde_json::to_string(graph)?
    };
    
    fs::write(file_path, json_string)?;
    
    Ok(ExportResult {
        success: true,
        output_path: file_path.display().to_string(),
        nodes_exported: graph.nodes.len(),
        edges_exported: graph.edges.len(),
        layers_exported: graph.layers.len(),
        warnings: vec![],
        error: None,
    })
}

/// Export graph to Layercake native format (JSON with metadata)
pub fn export_layercake(graph: &Graph, file_path: &Path, options: &ExportOptions) -> Result<ExportResult> {
    debug!("Exporting Layercake format to: {}", file_path.display());
    
    let layercake_data = LayercakeFormat {
        version: "1.0".to_string(),
        format: "layercake".to_string(),
        metadata: LayercakeMetadata {
            created_at: chrono::Utc::now(),
            created_by: "layercake-tool".to_string(),
            description: Some(format!("Exported graph: {}", graph.name)),
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            layer_count: graph.layers.len(),
        },
        graph: graph.clone(),
    };
    
    let json_string = if options.prettify {
        serde_json::to_string_pretty(&layercake_data)?
    } else {
        serde_json::to_string(&layercake_data)?
    };
    
    fs::write(file_path, json_string)?;
    
    Ok(ExportResult {
        success: true,
        output_path: file_path.display().to_string(),
        nodes_exported: graph.nodes.len(),
        edges_exported: graph.edges.len(),
        layers_exported: graph.layers.len(),
        warnings: vec![],
        error: None,
    })
}

/// Layercake native format with metadata
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct LayercakeFormat {
    version: String,
    format: String,
    metadata: LayercakeMetadata,
    graph: Graph,
}

/// Metadata for Layercake format
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct LayercakeMetadata {
    created_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    description: Option<String>,
    node_count: usize,
    edge_count: usize,
    layer_count: usize,
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
                    id: "1".to_string(),
                    label: "Node 1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                },
                Node {
                    id: "2".to_string(),
                    label: "Node 2".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "1".to_string(),
                    target: "2".to_string(),
                    label: "Edge 1".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
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
    fn test_json_export_import_roundtrip() {
        let original_graph = create_test_graph();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Export
        let export_options = ExportOptions::default();
        let export_result = export_json(&original_graph, temp_file.path(), &export_options).unwrap();
        assert!(export_result.success);
        assert_eq!(export_result.nodes_exported, 2);
        assert_eq!(export_result.edges_exported, 1);
        assert_eq!(export_result.layers_exported, 1);
        
        // Import
        let import_options = ImportOptions::default();
        let (imported_graph, import_result) = import_json(temp_file.path(), &import_options).unwrap();
        assert!(import_result.success);
        assert_eq!(import_result.nodes_imported, 2);
        assert_eq!(import_result.edges_imported, 1);
        assert_eq!(import_result.layers_imported, 1);
        
        // Verify data integrity
        assert_eq!(imported_graph.name, original_graph.name);
        assert_eq!(imported_graph.nodes.len(), original_graph.nodes.len());
        assert_eq!(imported_graph.edges.len(), original_graph.edges.len());
        assert_eq!(imported_graph.layers.len(), original_graph.layers.len());
        
        // Check specific node data
        assert_eq!(imported_graph.nodes[0].id, "1");
        assert_eq!(imported_graph.nodes[0].label, "Node 1");
        assert_eq!(imported_graph.edges[0].source, "1");
        assert_eq!(imported_graph.edges[0].target, "2");
    }
    
    #[test]
    fn test_layercake_format_export_import() {
        let original_graph = create_test_graph();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Export as Layercake format
        let export_options = ExportOptions {
            format: super::super::GraphFormat::Layercake,
            ..Default::default()
        };
        let export_result = export_layercake(&original_graph, temp_file.path(), &export_options).unwrap();
        assert!(export_result.success);
        
        // Import as Layercake format
        let import_options = ImportOptions {
            format: super::super::GraphFormat::Layercake,
            ..Default::default()
        };
        let (imported_graph, import_result) = import_layercake(temp_file.path(), &import_options).unwrap();
        assert!(import_result.success);
        
        // Verify data integrity
        assert_eq!(imported_graph.name, original_graph.name);
        assert_eq!(imported_graph.nodes.len(), original_graph.nodes.len());
        assert_eq!(imported_graph.edges.len(), original_graph.edges.len());
        assert_eq!(imported_graph.layers.len(), original_graph.layers.len());
    }
}