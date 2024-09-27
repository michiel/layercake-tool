mod data_loader;
mod config;

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use std::fs;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(short, long, default_value = "config.yaml")]
    config: Option<String>,
}

fn load_and_process_data(filename: &str) -> Result<DataFrame> {
    let df = data_loader::load_tsv(filename)?;
    tracing::info!("Loaded DataFrame:\n{}", df);
    Ok(df)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("Info"))
        .add_directive("layercake::http=trace".parse()?);

    Registry::default()
        .with(env_filter)
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();

    let config_path = &args.config.unwrap_or_else(|| {
        tracing::info!("No argument provided, using default configuration file: config.yaml");
        "config.yaml".to_string()
    });
    tracing::info!("Using configuration file: {}", config_path);

    // Read and deserialize the configuration file
    let config_content = fs::read_to_string(config_path)?;

    let plan: config::PlanConfig = serde_yaml::from_str(&config_content)?;

    // Use the deserialized configuration
    let filename = &plan.import.profiles[0].filename;
    let mut df = load_and_process_data(filename)?;

    let sql_query = "SELECT repo1, repo2, repo1 || '-' || repo2 AS repo_id FROM df";
    df = data_loader::add_column_with_sql(&mut df, sql_query, "repo_id")?;
    tracing::info!("Updated DataFrame:\n{}", df);
    Ok(())
}