use crate::data_loader;
use crate::graph::{Edge, Graph, Layer, Node};
use crate::plan::{ExportFileType, ImportFileType, Plan};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use tracing::{debug, error, info, warn};

use anyhow::Result;
use polars::prelude::*;

fn load_file(file_path: &str) -> Result<DataFrame, anyhow::Error> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");

    let df = match extension {
        "csv" => data_loader::load_csv(file_path),
        "tsv" => data_loader::load_tsv(file_path),
        _ => {
            error!("Error: unsupported extension {}", extension);
            anyhow::bail!("Unsupported extension");
        }
    }?;

    debug!("Loaded DataFrame:\n{}", df);
    Ok(df)
}

fn run_plan(plan: Plan, plan_file_path: &std::path::Path) -> Result<()> {
    let mut graph = Graph::default();
    graph.name = match plan.meta {
        Some(meta) => match meta.name {
            Some(name) => name,
            _ => "Unnamed Graph".to_string(),
        },
        _ => "Unnamed Graph".to_string(),
    };

    plan.import
        .profiles
        .iter()
        .try_for_each(|profile| -> Result<(), Box<dyn std::error::Error>> {
            let import_file_path = plan_file_path.parent().unwrap().join(&profile.filename);
            info!(
                "Importing file: {} as {:?}",
                import_file_path.display(),
                profile.filetype
            );
            let df = load_file(import_file_path.to_str().unwrap())?;
            match profile.filetype {
                ImportFileType::Nodes => {
                    let node_profile = data_loader::create_df_node_load_profile(&df);
                    info!("{}", node_profile);
                    data_loader::verify_nodes_df(&df)?;
                    data_loader::verify_id_column_df(&df, &node_profile)?;

                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let node = Node::from_row(&row, &node_profile)?;
                        graph.nodes.push(node);
                    }
                }
                ImportFileType::Edges => {
                    // TODO Add verification for edges
                    // data_loader::verify_id_column_df(&df)?;
                    let edge_profile = data_loader::create_df_edge_load_profile(&df);
                    info!("{}", edge_profile);
                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let edge = Edge::from_row(&row, &edge_profile)?;
                        graph.edges.push(edge);
                    }
                }
                ImportFileType::Layers => {
                    // TODO Add verification for layers
                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let layer = Layer::from_row(&row)?;
                        graph.layers.push(layer.clone());
                    }
                }
            }
            Ok(())
        })
        .unwrap();

    // TODO verify that all nodes in edges are present in nodes
    // TODO verify graph integrity

    info!(
        "Graph loaded with {} nodes, {} edges and {} layers",
        graph.nodes.len(),
        graph.edges.len(),
        graph.layers.len()
    );

    // debug!("Graph: {:?}", graph);

    match graph.verify_graph_integrity() {
        Ok(_) => {
            info!("Graph integrity verified : ok - rendering exports");
            plan.export.profiles.iter().for_each(|profile| {
                info!(
                    "Starting export to file: {} using exporter {:?}",
                    profile.filename, profile.exporter
                );
                let mut graph = graph.clone();
                let graph_config = profile.get_graph_config();

                if graph_config.invert_graph {
                    info!("Inverting graph (flipping nodes and edges)");
                    warn!("Inverting graph is an experimental feature");
                    graph = graph.invert_graph();
                }

                if graph_config.max_partition_depth > 0 {
                    let max_partition_depth = graph_config.max_partition_depth;
                    info!("Reducing graph partition depth to {}", max_partition_depth);
                    debug!("Graph stats {}", graph.stats());
                    match graph.modify_graph_limit_partition_depth(max_partition_depth) {
                        Ok(_) => {
                            debug!("Graph partition depth limited to {}", max_partition_depth);
                            debug!("Graph stats {}", graph.stats());
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
                    match graph.modify_graph_limit_partition_width(max_partition_width) {
                        Ok(_) => {
                            debug!("Graph partition width limited to {}", max_partition_width);
                            debug!("Graph stats {}", graph.stats());
                        }
                        Err(e) => {
                            error!("Failed to limit graph partition width: {}", e);
                        }
                    }
                }

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

                if let Err(errors) = graph.verify_graph_integrity() {
                    warn!("Identified {} graph integrity error(s)", errors.len());
                    errors.iter().for_each(|e| warn!("{}", e));
                    // TODO exit early - or not if we're watching
                    error!("Failed to export file {}", profile.filename);
                } else {
                    debug!("All clear for export target {}", profile.filename);
                }

                graph.aggregate_edges();

                let render_config = profile.get_render_config();

                let result = match profile.exporter.clone() {
                    ExportFileType::GML => super::export::to_gml::render(graph, render_config),
                    ExportFileType::DOT => super::export::to_dot::render(graph, render_config),
                    ExportFileType::DOTHierarchy => {
                        super::export::to_dot_hierarchy::render(graph, render_config)
                    }
                    ExportFileType::JSON => super::export::to_json::render(graph, render_config),
                    ExportFileType::CSVNodes => {
                        super::export::to_csv_nodes::render(graph, render_config)
                    }
                    ExportFileType::CSVEdges => {
                        super::export::to_csv_edges::render(graph, render_config)
                    }
                    ExportFileType::CSVMatrix => {
                        super::export::to_csv_matrix::render(graph, render_config)
                    }
                    ExportFileType::PlantUML => {
                        super::export::to_plantuml::render(graph, render_config)
                    }
                    ExportFileType::Mermaid => {
                        super::export::to_mermaid::render(graph, render_config)
                    }
                    ExportFileType::JSGraph => {
                        super::export::to_jsgraph::render(graph, render_config)
                    }
                    ExportFileType::Custom(template) => {
                        super::export::to_custom::render(graph, render_config, template)
                    }
                };

                match result {
                    Ok(output) => {
                        if let Err(e) =
                            super::common::write_string_to_file(&profile.filename, &output)
                        {
                            error!("Failed to write to file {}: {}", profile.filename, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to export file {}: {}", profile.filename, e);
                    }
                }
            });
        }
        Err(errors) => {
            warn!("Identified {} graph integrity error(s)", errors.len());
            errors.iter().for_each(|e| warn!("{}", e));
            warn!("Not rendering exports");
        }
    }

    Ok(())
}

pub fn execute_plan(plan: String, watch: bool) -> Result<()> {
    info!("Executing plan {}", plan);

    let plan_file_path = std::path::Path::new(&plan);
    let path_content = std::fs::read_to_string(plan_file_path)?;
    let plan: Plan = serde_yaml::from_str(&path_content)?;

    debug!("Executing plan: {:?}", plan);
    run_plan(plan.clone(), plan_file_path)?;
    if watch {
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
            let path = plan_file_path.parent().unwrap().join(file);
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }

        loop {
            match rx.recv() {
                Ok(event) => {
                    // debug!("Event: {:?}", event);
                    if event.is_ok() {
                        let event = event.unwrap();
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

    Ok(())
}
