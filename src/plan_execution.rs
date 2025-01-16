use crate::data_loader;
use crate::graph::{Edge, Graph, Node};
use crate::plan::{ImportFileType, Plan};

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
            eprintln!("Error: unsupported extension {}", extension);
            anyhow::bail!("Unsupported extension");
        }
    }?;

    println!("Loaded DataFrame:\n{}", df);
    Ok(df)
}

pub fn execute_plan(plan: Plan) -> Result<()> {
    println!("Executing plan: {:?}", plan);

    let mut graph = Graph::default();

    plan.import
        .profiles
        .iter()
        .try_for_each(|profile| -> Result<(), Box<dyn std::error::Error>> {
            println!("Importing file: {}", profile.filename);
            let df = load_file(&profile.filename)?;
            match profile.filetype {
                ImportFileType::Nodes => {
                    data_loader::verify_nodes_df(&df)?;
                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let node = Node::from_row(&row);
                        graph.nodes.push(node);
                    }
                }
                ImportFileType::Edges => {
                    for idx in 0..df.height() {
                        let row = df.get_row(idx)?;
                        let edge = Edge::from_row(&row);
                        graph.edges.push(edge);
                    }
                }
            }
            Ok(())
        })
        .unwrap();

    plan.export.profiles.iter().for_each(|profile| {
        println!("Exporting file: {}", profile.filename);
    });

    Ok(())
}
