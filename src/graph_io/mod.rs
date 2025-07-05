//! Graph data import/export functionality
//!
//! This module provides comprehensive support for importing and exporting
//! graph data in various formats including CSV, JSON, GraphML, GEXF, and DOT.

pub mod csv_io;
pub mod json_io;
pub mod graphml_io;
pub mod gexf_io;
pub mod dot_io;
pub mod formats;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info};

use crate::graph::Graph;

/// Supported import/export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphFormat {
    /// Comma-separated values (nodes.csv, edges.csv, layers.csv)
    CSV,
    /// JavaScript Object Notation
    JSON,
    /// Graph Markup Language (XML-based)
    GraphML,
    /// Graph Exchange XML Format
    GEXF,
    /// DOT graph description language (Graphviz)
    DOT,
    /// Layercake native format (includes hierarchy and layers)
    Layercake,
}

/// Import options for customizing the import process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    /// Format to import from
    pub format: GraphFormat,
    /// Whether to validate the graph after import
    pub validate: bool,
    /// Whether to merge with existing graph or replace
    pub merge_mode: MergeMode,
    /// Custom field mappings for CSV import
    pub field_mappings: Option<FieldMappings>,
    /// Whether to auto-generate missing layers
    pub auto_generate_layers: bool,
    /// Whether to preserve original IDs or generate new ones
    pub preserve_ids: bool,
}

/// Export options for customizing the export process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Format to export to
    pub format: GraphFormat,
    /// Whether to include metadata in export
    pub include_metadata: bool,
    /// Whether to export hierarchy information
    pub include_hierarchy: bool,
    /// Whether to export layer information
    pub include_layers: bool,
    /// Custom field mappings for export
    pub field_mappings: Option<FieldMappings>,
    /// Whether to prettify output (for JSON/XML formats)
    pub prettify: bool,
}

/// Merge modes for importing data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeMode {
    /// Replace existing graph entirely
    Replace,
    /// Merge nodes and edges, keeping existing data
    Merge,
    /// Only add new nodes/edges, skip existing ones
    AddOnly,
    /// Update existing nodes/edges, add new ones
    Upsert,
}

/// Field mappings for customizing import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappings {
    /// Node field mappings (source_field -> target_field)
    pub node_fields: std::collections::HashMap<String, String>,
    /// Edge field mappings (source_field -> target_field)
    pub edge_fields: std::collections::HashMap<String, String>,
    /// Layer field mappings (source_field -> target_field)
    pub layer_fields: std::collections::HashMap<String, String>,
}

/// Result of import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Whether import was successful
    pub success: bool,
    /// Number of nodes imported
    pub nodes_imported: usize,
    /// Number of edges imported
    pub edges_imported: usize,
    /// Number of layers imported
    pub layers_imported: usize,
    /// Any warnings generated during import
    pub warnings: Vec<String>,
    /// Error message if import failed
    pub error: Option<String>,
}

/// Result of export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Whether export was successful
    pub success: bool,
    /// Path where data was exported
    pub output_path: String,
    /// Number of nodes exported
    pub nodes_exported: usize,
    /// Number of edges exported
    pub edges_exported: usize,
    /// Number of layers exported
    pub layers_exported: usize,
    /// Any warnings generated during export
    pub warnings: Vec<String>,
    /// Error message if export failed
    pub error: Option<String>,
}

/// Main graph I/O handler
pub struct GraphIO;

impl GraphIO {
    /// Import a graph from a file
    pub fn import_from_file<P: AsRef<Path>>(
        file_path: P,
        options: ImportOptions,
    ) -> Result<(Graph, ImportResult)> {
        let path = file_path.as_ref();
        info!("Importing graph from: {}", path.display());
        debug!("Import options: {:?}", options);

        let result = match options.format {
            GraphFormat::CSV => csv_io::import_csv(path, &options),
            GraphFormat::JSON => json_io::import_json(path, &options),
            GraphFormat::GraphML => graphml_io::import_graphml(path, &options),
            GraphFormat::GEXF => gexf_io::import_gexf(path, &options),
            GraphFormat::DOT => dot_io::import_dot(path, &options),
            GraphFormat::Layercake => json_io::import_layercake(path, &options),
        };

        match result {
            Ok((mut graph, mut import_result)) => {
                // Validate graph if requested
                if options.validate {
                    if let Err(errors) = graph.verify_graph_integrity() {
                        import_result.warnings.extend(
                            errors.into_iter().map(|e| format!("Validation warning: {}", e))
                        );
                    }
                }

                info!(
                    "Import completed: {} nodes, {} edges, {} layers",
                    import_result.nodes_imported,
                    import_result.edges_imported,
                    import_result.layers_imported
                );

                Ok((graph, import_result))
            }
            Err(e) => {
                let import_result = ImportResult {
                    success: false,
                    nodes_imported: 0,
                    edges_imported: 0,
                    layers_imported: 0,
                    warnings: vec![],
                    error: Some(e.to_string()),
                };
                Ok((Graph::default(), import_result))
            }
        }
    }

    /// Export a graph to a file
    pub fn export_to_file<P: AsRef<Path>>(
        graph: &Graph,
        file_path: P,
        options: ExportOptions,
    ) -> Result<ExportResult> {
        let path = file_path.as_ref();
        info!("Exporting graph to: {}", path.display());
        debug!("Export options: {:?}", options);

        let result = match options.format {
            GraphFormat::CSV => csv_io::export_csv(graph, path, &options),
            GraphFormat::JSON => json_io::export_json(graph, path, &options),
            GraphFormat::GraphML => graphml_io::export_graphml(graph, path, &options),
            GraphFormat::GEXF => gexf_io::export_gexf(graph, path, &options),
            GraphFormat::DOT => dot_io::export_dot(graph, path, &options),
            GraphFormat::Layercake => json_io::export_layercake(graph, path, &options),
        };

        match result {
            Ok(export_result) => {
                info!(
                    "Export completed: {} nodes, {} edges, {} layers to {}",
                    export_result.nodes_exported,
                    export_result.edges_exported,
                    export_result.layers_exported,
                    export_result.output_path
                );
                Ok(export_result)
            }
            Err(e) => Ok(ExportResult {
                success: false,
                output_path: path.display().to_string(),
                nodes_exported: 0,
                edges_exported: 0,
                layers_exported: 0,
                warnings: vec![],
                error: Some(e.to_string()),
            }),
        }
    }

    /// Auto-detect format from file extension
    pub fn detect_format<P: AsRef<Path>>(file_path: P) -> Option<GraphFormat> {
        let path = file_path.as_ref();
        let extension = path.extension()?.to_str()?.to_lowercase();

        match extension.as_str() {
            "csv" => Some(GraphFormat::CSV),
            "json" => Some(GraphFormat::JSON),
            "graphml" | "xml" => Some(GraphFormat::GraphML),
            "gexf" => Some(GraphFormat::GEXF),
            "dot" | "gv" => Some(GraphFormat::DOT),
            "layercake" => Some(GraphFormat::Layercake),
            _ => None,
        }
    }

    /// Get supported file extensions for a format
    pub fn get_extensions(format: GraphFormat) -> Vec<&'static str> {
        match format {
            GraphFormat::CSV => vec!["csv"],
            GraphFormat::JSON => vec!["json"],
            GraphFormat::GraphML => vec!["graphml", "xml"],
            GraphFormat::GEXF => vec!["gexf"],
            GraphFormat::DOT => vec!["dot", "gv"],
            GraphFormat::Layercake => vec!["layercake", "json"],
        }
    }

    /// Create default import options for a format
    pub fn default_import_options(format: GraphFormat) -> ImportOptions {
        ImportOptions {
            format,
            validate: true,
            merge_mode: MergeMode::Replace,
            field_mappings: None,
            auto_generate_layers: true,
            preserve_ids: true,
        }
    }

    /// Create default export options for a format
    pub fn default_export_options(format: GraphFormat) -> ExportOptions {
        ExportOptions {
            format,
            include_metadata: true,
            include_hierarchy: true,
            include_layers: true,
            field_mappings: None,
            prettify: true,
        }
    }
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            format: GraphFormat::JSON,
            validate: true,
            merge_mode: MergeMode::Replace,
            field_mappings: None,
            auto_generate_layers: true,
            preserve_ids: true,
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: GraphFormat::JSON,
            include_metadata: true,
            include_hierarchy: true,
            include_layers: true,
            field_mappings: None,
            prettify: true,
        }
    }
}

impl Default for FieldMappings {
    fn default() -> Self {
        Self {
            node_fields: std::collections::HashMap::new(),
            edge_fields: std::collections::HashMap::new(),
            layer_fields: std::collections::HashMap::new(),
        }
    }
}