use crate::data_loader;
use crate::graph::{Edge, Graph, Layer, Node};
use crate::plan::{ExportFileType, ExportProfileItem, ImportFileType, Plan};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use tracing::{debug, error, info, warn};

use anyhow::{anyhow, Result};
use csv::StringRecord;

/// Loads a data file from disk, supporting CSV and TSV formats
fn load_file(file_path: &str) -> Result<(Vec<String>, Vec<StringRecord>), anyhow::Error> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");

    let separator = match extension {
        "csv" => b',',
        "tsv" => b'\t',
        _ => {
            error!("Error: unsupported extension {}", extension);
            anyhow::bail!("Unsupported extension");
        }
    };

    let headers = data_loader::get_headers_from_file(file_path, separator)?;
    let records = match extension {
        "csv" => data_loader::load_csv(file_path),
        "tsv" => data_loader::load_tsv(file_path),
        _ => unreachable!(), // We already checked extension above
    }?;

    debug!(
        "Loaded {} records with headers: {:?}",
        records.len(),
        headers
    );
    Ok((headers, records))
}

/// Creates a new graph with metadata from the plan
fn create_graph_from_plan(plan: &Plan) -> Graph {
    let mut graph = Graph::default();
    graph.name = match &plan.meta {
        Some(meta) => match &meta.name {
            Some(name) => name.clone(),
            _ => "Unnamed Graph".to_string(),
        },
        _ => "Unnamed Graph".to_string(),
    };
    graph
}

/// Loads data from import profiles into the graph
fn load_data_into_graph(graph: &mut Graph, plan: &Plan, plan_file_path: &Path) -> Result<()> {
    for profile in &plan.import.profiles {
        let parent_dir = plan_file_path
            .parent()
            .ok_or_else(|| anyhow!("Plan file has no parent directory"))?;
        let import_file_path = parent_dir.join(&profile.filename);
        info!(
            "Importing file: {} as {:?}",
            import_file_path.display(),
            profile.filetype
        );

        let file_path_str = import_file_path.to_str().ok_or_else(|| {
            anyhow!(
                "Import file path contains invalid UTF-8: {}",
                import_file_path.display()
            )
        })?;
        let (headers, records) = load_file(file_path_str)?;

        match profile.filetype {
            ImportFileType::Nodes => {
                let node_profile = data_loader::create_df_node_load_profile(&headers);
                info!("{}", node_profile);
                data_loader::verify_nodes_headers(&headers)?;
                data_loader::verify_id_column(&records, node_profile.id_column)?;

                for record in &records {
                    match Node::from_row(record, &node_profile) {
                        Ok(node) => graph.nodes.push(node),
                        Err(e) => return Err(anyhow::anyhow!("Error creating node: {}", e)),
                    };
                }
            }
            ImportFileType::Edges => {
                // TODO Add verification for edges
                let edge_profile = data_loader::create_df_edge_load_profile(&headers);
                info!("{}", edge_profile);
                for record in &records {
                    match Edge::from_row(record, &edge_profile) {
                        Ok(edge) => graph.edges.push(edge),
                        Err(e) => return Err(anyhow::anyhow!("Error creating edge: {}", e)),
                    };
                }
            }
            ImportFileType::Layers => {
                // TODO Add verification for layers
                for record in &records {
                    match Layer::from_row(record) {
                        Ok(layer) => graph.layers.push(layer.clone()),
                        Err(e) => return Err(anyhow::anyhow!("Error creating layer: {}", e)),
                    };
                }
            }
        }
    }

    info!(
        "Graph loaded with {} nodes, {} edges and {} layers",
        graph.nodes.len(),
        graph.edges.len(),
        graph.layers.len()
    );

    Ok(())
}

/// Applies transformations to the graph based on the profile configuration
fn apply_graph_transformations(
    graph: &mut Graph,
    graph_config: &crate::plan::GraphConfig,
) -> Result<()> {
    if graph_config.invert_graph {
        info!("Inverting graph (flipping nodes and edges)");
        warn!("Inverting graph is an experimental feature");
        *graph = graph
            .invert_graph()
            .map_err(|e| anyhow::anyhow!("Failed to invert graph: {}", e))?;
    }

    if graph_config.max_partition_depth > 0 {
        let max_partition_depth = graph_config.max_partition_depth;
        info!("Reducing graph partition depth to {}", max_partition_depth);
        debug!("Graph stats {}", graph.stats());
        let synthesized = graph.ensure_partition_hierarchy();
        match graph.modify_graph_limit_partition_depth(max_partition_depth) {
            Ok(_) => {
                debug!("Graph partition depth limited to {}", max_partition_depth);
                debug!("Graph stats {}", graph.stats());
                if synthesized {
                    info!("Synthetic partition hierarchy used because original graph lacked partition metadata");
                }
            }
            Err(e) => {
                error!("Failed to limit graph partition depth: {}", e);
            }
        }
    }

    if graph_config.max_partition_width > 0 {
        let max_partition_width = graph_config.max_partition_width;
        info!("Reducing graph partition width to {}", max_partition_width);
        debug!("Graph stats {}", graph.stats());
        let synthesized = graph.ensure_partition_hierarchy();
        match graph.modify_graph_limit_partition_width(max_partition_width) {
            Ok(_) => {
                debug!("Graph partition width limited to {}", max_partition_width);
                debug!("Graph stats {}", graph.stats());
                if synthesized {
                    info!("Synthetic partition hierarchy used because original graph lacked partition metadata");
                }
            }
            Err(e) => {
                error!("Failed to limit graph partition width: {}", e);
            }
        }
    }

    // Apply label transformations
    if graph_config.node_label_max_length > 0 {
        let node_label_max_length = graph_config.node_label_max_length;
        info!("Truncating node labels to {}", node_label_max_length);
        graph.truncate_node_labels(node_label_max_length);
    }

    if graph_config.node_label_insert_newlines_at > 0 {
        let node_label_insert_newlines_at = graph_config.node_label_insert_newlines_at;
        info!(
            "Inserting newlines in node labels at {}",
            node_label_insert_newlines_at
        );
        graph.insert_newlines_in_node_labels(node_label_insert_newlines_at);
    }

    if graph_config.edge_label_max_length > 0 {
        let edge_label_max_length = graph_config.edge_label_max_length;
        info!("Truncating edge labels to {}", edge_label_max_length);
        graph.truncate_edge_labels(edge_label_max_length);
    }

    if graph_config.edge_label_insert_newlines_at > 0 {
        let edge_label_insert_newlines_at = graph_config.edge_label_insert_newlines_at;
        info!(
            "Inserting newlines in edge labels at {}",
            edge_label_insert_newlines_at
        );
        graph.insert_newlines_in_edge_labels(edge_label_insert_newlines_at);
    }

    // Aggregate any duplicate edges if configured
    if graph_config.aggregate_edges {
        graph.aggregate_edges();
    }

    Ok(())
}

/// Exports the graph to the specified file using the appropriate renderer
fn export_graph(graph: &Graph, profile: &ExportProfileItem) -> Result<()> {
    info!(
        "Starting export to file: {} using exporter {:?}",
        profile.filename, profile.exporter
    );

    let render_config = profile.get_render_config();

    let result = match &profile.exporter {
        ExportFileType::GML => crate::export::to_gml::render(graph, &render_config),
        ExportFileType::DOT => crate::export::to_dot::render(graph, &render_config),
        ExportFileType::DOTHierarchy => {
            crate::export::to_dot_hierarchy::render(graph, &render_config)
        }
        ExportFileType::JSON => crate::export::to_json::render(graph, &render_config),
        ExportFileType::CSVNodes => crate::export::to_csv_nodes::render(graph, &render_config),
        ExportFileType::CSVEdges => crate::export::to_csv_edges::render(graph, &render_config),
        ExportFileType::CSVMatrix => crate::export::to_csv_matrix::render(graph, &render_config),
        ExportFileType::PlantUML => crate::export::to_plantuml::render(graph, &render_config),
        ExportFileType::PlantUmlMindmap => {
            crate::export::to_plantuml_mindmap::render(graph, &render_config)
        }
        ExportFileType::PlantUmlWbs => {
            crate::export::to_plantuml_wbs::render(graph, &render_config)
        }
        ExportFileType::Mermaid => crate::export::to_mermaid::render(graph, &render_config),
        ExportFileType::MermaidMindmap => {
            crate::export::to_mermaid_mindmap::render(graph, &render_config)
        }
        ExportFileType::MermaidTreemap => {
            crate::export::to_mermaid_treemap::render(graph, &render_config)
        }
        ExportFileType::JSGraph => crate::export::to_jsgraph::render(graph, &render_config),
        ExportFileType::Custom(template_config) => {
            crate::export::to_custom::render(graph, &render_config, template_config)
        }
    };

    match result {
        Ok(output) => {
            if let Err(e) = crate::common::write_string_to_file(&profile.filename, &output) {
                error!("Failed to write to file {}: {}", profile.filename, e);
            }
        }
        Err(e) => {
            error!("Failed to export file {}: {}", profile.filename, e);
        }
    }

    Ok(())
}

/// Executes a single export plan
fn run_plan(plan: Plan, plan_file_path: &Path) -> Result<()> {
    // Create the graph from the plan
    let mut graph = create_graph_from_plan(&plan);

    // Load data into the graph
    load_data_into_graph(&mut graph, &plan, plan_file_path)?;

    // Verify graph integrity before proceeding
    match graph.verify_graph_integrity() {
        Ok(_) => {
            info!("Graph integrity verified : ok - rendering exports");

            // Process each export profile
            for profile in &plan.export.profiles {
                let mut graph_copy = graph.clone();
                let graph_config = profile.get_graph_config();

                // Apply transformations to the graph
                if let Err(e) = apply_graph_transformations(&mut graph_copy, &graph_config) {
                    error!("Failed to apply transformations: {}", e);
                    continue;
                }

                // Verify graph integrity after transformations
                if let Err(errors) = graph_copy.verify_graph_integrity() {
                    warn!("Identified {} graph integrity error(s)", errors.len());
                    errors.iter().for_each(|e| warn!("{}", e));
                    error!("Failed to export file {}", profile.filename);
                    continue;
                }

                // Export the graph
                if let Err(e) = export_graph(&graph_copy, profile) {
                    error!("Failed to export graph: {}", e);
                }
            }
        }
        Err(errors) => {
            warn!("Identified {} graph integrity error(s)", errors.len());
            errors.iter().for_each(|e| warn!("{}", e));
            warn!("Not rendering exports");
        }
    }

    Ok(())
}

/// Main function to execute a plan, with optional file watching
pub fn execute_plan(plan: String, watch: bool) -> Result<()> {
    info!("Executing plan {}", plan);

    let plan_file_path = std::path::Path::new(&plan);
    let path_content = std::fs::read_to_string(plan_file_path)?;
    let plan: Plan = serde_yaml::from_str(&path_content)?;

    debug!("Executing plan: {:?}", plan);
    run_plan(plan.clone(), plan_file_path)?;

    if watch {
        watch_for_changes(plan, plan_file_path)?;
    }

    Ok(())
}

/// Sets up file watching for input files to re-run the plan on changes
fn watch_for_changes(plan: Plan, plan_file_path: &Path) -> Result<()> {
    info!("Watching for changes");
    let files: Vec<String> = plan
        .import
        .profiles
        .iter()
        .map(|profile| profile.filename.clone())
        .collect();

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    for file in &files {
        let parent_dir = plan_file_path
            .parent()
            .ok_or_else(|| anyhow!("Plan file has no parent directory"))?;
        let path = parent_dir.join(file);
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
    }

    loop {
        match rx.recv() {
            Ok(event) => {
                // debug!("Event: {:?}", event);
                if let Ok(event) = event {
                    if let EventKind::Modify(_) = event.kind {
                        debug!("File modified {:?}", event.paths);
                        info!("Change detected, re-executing plan");
                        run_plan(plan.clone(), plan_file_path)?;
                    }
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
}
