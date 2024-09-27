mod data_loader;
mod config;

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use std::fs;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "config.yaml")]
    config: String,
}

fn load_and_process_data(filename: &str) -> Result<DataFrame> {
    let df = data_loader::load_tsv(filename)?;
    println!("Loaded DataFrame:\n{}", df);
    Ok(df)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Read and deserialize the configuration file
    let config_content = fs::read_to_string(&args.config)?;
    let plan: config::PlanConfig = serde_yaml::from_str(&config_content)?;

    // Use the deserialized configuration
    let filename = &plan.import.profiles[0].filename;
    let mut df = load_and_process_data(filename)?;

    let sql_query = "SELECT repo1, repo2, repo1 || '-' || repo2 AS repo_id FROM df";
    df = data_loader::add_column_with_sql(&mut df, sql_query, "repo_id")?;
    println!("Updated DataFrame:\n{}", df);
    Ok(())
}