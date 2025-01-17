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
use tracing::Level;
use tracing::{debug, error, info};
use tracing_subscriber;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    plan: Option<String>,
    #[clap(short, long)]
    log_level: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let log_level = match args
        .log_level
        .unwrap_or("info".to_string())
        .to_lowercase()
        .as_str()
    {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Exit if args.path is not provided
    let plan_file_path = match args.plan {
        Some(path) => path,
        None => {
            error!("Error: configuration file must be provided with the -c option.");
            process::exit(1);
        }
    };

    // Read and deserialize the configuration file
    let path_content = fs::read_to_string(&plan_file_path)?;
    let plan: plan::Plan = serde_yaml::from_str(&path_content)?;

    plan_execution::execute_plan(plan)?;

    Ok(())
}
