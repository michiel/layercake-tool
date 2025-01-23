mod common;
mod data_loader;
mod export;
mod generate_commands;
mod graph;
mod plan;
mod plan_execution;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_yaml;
use std::fs;
use tracing::Level;
use tracing::{error, info};
use tracing_subscriber;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(short, long, global = true)]
    log_level: Option<String>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[clap(short, long)]
        plan: String,
    },
    Init {
        #[clap(short, long)]
        plan: String,
    },
    Generate {
        exporter: String,
    },
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

    match args.command {
        Commands::Run { plan } => {
            info!("Running plan: {}", plan);
            let plan_file_path = plan;
            let path_content = fs::read_to_string(&plan_file_path)?;
            let plan: plan::Plan = serde_yaml::from_str(&path_content)?;
            plan_execution::execute_plan(plan)?;
        }
        Commands::Init { plan } => {
            info!("Initializing plan: {}", plan);
            let plan_file_path = plan;
            let plan = plan::Plan::default();
            let serialized_plan = serde_yaml::to_string(&plan)?;
            common::write_string_to_file(&plan_file_path, &serialized_plan)?;
        }
        Commands::Generate { exporter } => {
            generate_commands::generate_template(exporter);
        }
    }

    Ok(())
}
