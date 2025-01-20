use crate::data_loader;
use crate::graph::{Edge, Graph, Node};
use crate::plan::{ExportFileType, ImportFileType, Plan};
use tracing::{debug, error, info};

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

pub fn execute_plan(plan: Plan) -> Result<()> {
    info!("Executing plan");
    debug!("Executing plan: {:?}", plan);

    let mut graph = Graph::default();

    plan.import
        .profiles
        .iter()
        .try_for_each(|profile| -> Result<(), Box<dyn std::error::Error>> {
            info!(
                "Importing file: {} as {:?}",
                profile.filename, profile.filetype
            );
            let df = load_file(&profile.filename)?;
            match profile.filetype {
                ImportFileType::Nodes => {
                    data_loader::verify_nodes_df(&df)?;
                    data_loader::verify_id_column_df(&df)?;

                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let node = Node::from_row(&row)?;
                        match node.belongs_to {
                            Some(ref belongs_to) => {
                                let edge = Edge {
                                    id: format!("{}-{}", node.id, belongs_to),
                                    source: node.id.clone(),
                                    target: belongs_to.to_string(),
                                    label: "belongs to".to_string(),
                                    layer: "partition".to_string(),
                                    comment: None,
                                };
                                graph.edges.push(edge);
                            }
                            None => {}
                        }
                        graph.nodes.push(node);
                    }
                }
                ImportFileType::Edges => {
                    // TODO Add verification for edges
                    data_loader::verify_id_column_df(&df)?;
                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let edge = Edge::from_row(&row)?;
                        graph.edges.push(edge);
                    }
                }
            }
            Ok(())
        })
        .unwrap();

    // TODO verify that all nodes in edges are present in nodes
    // TODO verify graph integrity

    plan.export.profiles.iter().for_each(|profile| {
        info!(
            "Exporting file: {} using exporter {:?}",
            profile.filename, profile.exporter
        );
        let output = match profile.exporter {
            ExportFileType::GML => super::export::to_gml::render(graph.clone()),
            ExportFileType::DOT => super::export::to_dot::render(graph.clone()),
            ExportFileType::CSVNodes => "".to_string(),
            ExportFileType::CSVEdges => "".to_string(),
            ExportFileType::PlantUML => super::export::to_plantuml::render(graph.clone()),
            ExportFileType::Mermaid => super::export::to_mermaid::render(graph.clone()),
        };

        super::common::write_string_to_file(&profile.filename, &output).unwrap();
    });

    debug!("Graph: {:?}", graph);

    Ok(())
}
