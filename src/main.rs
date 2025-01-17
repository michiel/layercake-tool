mod common;
mod data_loader;
mod export;
mod graph;
mod plan;
mod plan_execution;

use anyhow::Result;
use clap::Parser;
use serde_yaml;
use std::fs;
use std::process;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    plan: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Exit if args.path is not provided
    let plan_file_path = match args.plan {
        Some(path) => path,
        None => {
            eprintln!("Error: configuration file must be provided with the -c option.");
            process::exit(1);
        }
    };

    // Read and deserialize the configuration file
    let path_content = fs::read_to_string(&plan_file_path)?;
    let plan: plan::Plan = serde_yaml::from_str(&path_content)?;

    plan_execution::execute_plan(plan)?;

    // // Use the deserialized configuration
    // let filename = &plan.import.profiles[0].filename;
    // let _df = load_file(filename)?;

    Ok(())
}
