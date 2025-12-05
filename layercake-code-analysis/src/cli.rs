use crate::analyzer::{analyze_path, AnalysisRun};
use crate::report::{markdown::MarkdownReporter, ReportMetadata};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(name = "code-analysis", about = "Analyze code and emit Layercake datasets")]
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
    },
}

pub fn run(args: CodeAnalysisArgs) -> Result<()> {
    match args.command {
        CodeAnalysisCommand::Report { path, output } => {
            let AnalysisRun {
                result,
                files_scanned,
            } = analyze_path(&path)?;

            let metadata = ReportMetadata::new(path, files_scanned);
            let reporter = MarkdownReporter::default();
            let rendered = reporter.render(&result, &metadata)?;

            if let Some(output_path) = output {
                fs::write(&output_path, rendered)?;
            } else {
                println!("{rendered}");
            }
            Ok(())
        }
    }
}
