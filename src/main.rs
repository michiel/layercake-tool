mod data_loader;
mod config;

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use std::fs;
use std::process;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    config: Option<String>,
}

fn load_and_process_data(filename: &str) -> Result<DataFrame> {
    let df = data_loader::load_tsv(filename)?;
    println!("Loaded DataFrame:\n{}", df);
    Ok(df)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Exit if args.config is not provided
    let config_path = match args.config {
        Some(path) => path,
        None => {
            eprintln!("Error: Configuration file must be provided with the -c option.");
            process::exit(1);
        }
    };

    // Read and deserialize the configuration file
    let config_content = fs::read_to_string(&config_path)?;
    let plan_config: config::PlanConfig = toml::from_str(&config_content)?;

    // Use the deserialized configuration
    let filename = &plan_config.import.profiles[0].filename;
    let mut df = load_and_process_data(filename)?;

    let sql_query = "SELECT repo1, repo2, repo1 || '-' || repo2 AS repo_id FROM df";
    df = data_loader::add_column_with_sql(&mut df, sql_query, "repo_id")?;
    println!("Updated DataFrame:\n{}", df);
    Ok(())
}