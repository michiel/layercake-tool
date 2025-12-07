use crate::analyzer::{analyze_path, AnalysisRun};
use crate::infra::{self, analyze_infra};
use crate::report::{markdown::MarkdownReporter, ReportMetadata};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(
    name = "code-analysis",
    about = "Analyze code and emit Layercake datasets"
)]
pub struct CodeAnalysisArgs {
    #[command(subcommand)]
    pub command: CodeAnalysisCommand,
}

#[derive(Subcommand, Debug)]
pub enum CodeAnalysisCommand {
    #[command(name = "report")]
    Report {
        /// Path to the project directory to analyze
        path: PathBuf,
        /// Optional path to write the markdown report; defaults to STDOUT
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Disable infrastructure scanning and correlation
        #[arg(long)]
        no_infra: bool,
        /// Emit CSV node/edge inventories alongside the report
        #[arg(long)]
        csv: bool,
        /// Optional directory to write CSVs (defaults to report output dir or current dir)
        #[arg(long, value_name = "DIR")]
        csv_dir: Option<PathBuf>,
    },
}

fn export_csv(result: &crate::analyzer::AnalysisResult, dir: &PathBuf) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    std::fs::create_dir_all(dir)?;

    let mut node_ids = std::collections::HashSet::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    nodes.push(("root_scope".to_string(), "Codebase Root".to_string(), "SCOPE".to_string(), true, "".to_string(), "Logical root".to_string()));
    node_ids.insert("root_scope".to_string());

    for func in &result.functions {
        let id = format!("func_{}", func.name.replace(|c: char| !c.is_alphanumeric(), "_").to_lowercase());
        if node_ids.insert(id.clone()) {
            nodes.push((id.clone(), func.name.clone(), "COMPUTE".to_string(), false, "root_scope".to_string(), format!("Function in {}", func.file_path)));
        }
    }

    for flow in &result.data_flows {
        let src = format!("func_{}", flow.source.replace(|c: char| !c.is_alphanumeric(), "_").to_lowercase());
        let dst = format!("func_{}", flow.sink.replace(|c: char| !c.is_alphanumeric(), "_").to_lowercase());
        let id = format!("edge_{}", edges.len() + 1);
        edges.push((id, src, dst, "DATA".to_string(), flow.variable.clone().unwrap_or_default(), format!("Data flow in {}", flow.file_path)));
    }

    let mut nodes_file = File::create(dir.join("nodes.csv"))?;
    writeln!(nodes_file, "id,label,layer,is_partition,belongs_to,comment")?;
    for (id, label, layer, is_partition, belongs_to, comment) in nodes {
        writeln!(
            nodes_file,
            "{},{},{},{},{},\"{}\"",
            id,
            label,
            layer,
            if is_partition { "true" } else { "false" },
            belongs_to,
            comment.replace('"', "'")
        )?;
    }

    let mut edges_file = File::create(dir.join("edges.csv"))?;
    writeln!(edges_file, "id,source,target,layer,label,relative_weight,comment")?;
    for (id, src, dst, layer, label, comment) in edges {
        writeln!(
            edges_file,
            "{},{},{},{},{},1,\"{}\"",
            id,
            src,
            dst,
            layer,
            label,
            comment.replace('"', "'")
        )?;
    }

    Ok(())
}

pub fn run(args: CodeAnalysisArgs) -> Result<()> {
    match args.command {
        CodeAnalysisCommand::Report {
            path,
            output,
            no_infra,
            csv,
            csv_dir,
        } => {
            let AnalysisRun {
                result,
                files_scanned,
            } = analyze_path(&path)?;
            let (infra_graph, correlation) = if no_infra {
                (None, None)
            } else {
                let infra = analyze_infra(&path)?;
                let corr = infra::correlate_code_infra(&result, &infra);
                (Some(infra), Some(corr))
            };

            let metadata = ReportMetadata::new(path, files_scanned);
            let reporter = MarkdownReporter::default();
            let rendered = reporter.render_with_infra(
                &result,
                &metadata,
                infra_graph.as_ref(),
                correlation.as_ref(),
            )?;

            if let Some(ref output_path) = output {
                fs::write(&output_path, rendered)?;
            } else {
                println!("{rendered}");
            }
            if csv {
                let dir = csv_dir
                    .or_else(|| output.as_ref().and_then(|p| p.parent().map(|p| p.to_path_buf())))
                    .unwrap_or(std::env::current_dir()?);
                export_csv(&result, &dir)?;
            }
            Ok(())
        }
    }
}
