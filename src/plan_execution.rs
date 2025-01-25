use crate::data_loader;
use crate::graph::{Edge, Graph, Layer, Node};
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

pub fn execute_plan(plan: String) -> Result<()> {
    info!("Executing plan");

    let plan_file_path = std::path::Path::new(&plan);
    let path_content = std::fs::read_to_string(plan_file_path)?;
    let plan: Plan = serde_yaml::from_str(&path_content)?;

    debug!("Executing plan: {:?}", plan);

    let mut graph = Graph::default();

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
                    data_loader::verify_nodes_df(&df)?;
                    data_loader::verify_id_column_df(&df)?;

                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let node = Node::from_row(&row)?;
                        if let Some(ref belongs_to) = node.belongs_to {
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
                        graph.nodes.push(node);
                    }
                }
                ImportFileType::Edges => {
                    // TODO Add verification for edges
                    // data_loader::verify_id_column_df(&df)?;
                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let edge = Edge::from_row(&row)?;
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

    plan.export.profiles.iter().for_each(|profile| {
        info!(
            "Exporting file: {} using exporter {:?}",
            profile.filename, profile.exporter
        );
        let output = match profile.exporter.clone() {
            ExportFileType::GML => super::export::to_gml::render(graph.clone()),
            ExportFileType::DOT => super::export::to_dot::render(graph.clone()),
            ExportFileType::CSVNodes => "".to_string(),
            ExportFileType::CSVEdges => "".to_string(),
            ExportFileType::PlantUML => super::export::to_plantuml::render(graph.clone()),
            ExportFileType::Mermaid => super::export::to_mermaid::render(graph.clone()),
            ExportFileType::Custom(template) => {
                super::export::to_custom::render(graph.clone(), template)
            }
        };

        super::common::write_string_to_file(&profile.filename, &output).unwrap();
    });

    debug!("Graph: {:?}", graph);

    Ok(())
}
