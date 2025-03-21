// src/graph_utils.rs
use crate::data_loader;
use crate::graph::{Edge, Graph, Layer, Node};
use crate::plan::{ImportFileType, Plan};
use anyhow::Result;
use csv::StringRecord;
use std::path::Path;
use tracing::{debug, error, info};

// Functions for creating and loading graphs
pub fn create_graph_from_plan(plan: &Plan) -> Graph {
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

pub fn load_file(file_path: &str) -> Result<(Vec<String>, Vec<StringRecord>)> {
    let extension = Path::new(file_path)
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

pub fn load_data_into_graph(graph: &mut Graph, plan: &Plan, plan_file_path: &Path) -> Result<()> {
    for profile in &plan.import.profiles {
        let import_file_path = plan_file_path.parent().unwrap().join(&profile.filename);
        info!(
            "Importing file: {} as {:?}",
            import_file_path.display(),
            profile.filetype
        );

        let (headers, records) = load_file(import_file_path.to_str().unwrap())?;

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
