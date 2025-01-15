mod plan;
mod data_loader;

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use serde_yaml;
use std::fs;
use std::process;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    plan: Option<String>,
}

fn load_and_process_data(filename: &str) -> Result<DataFrame> {
    let df = data_loader::load_tsv(filename)?;
    println!("Loaded DataFrame:\n{}", df);
    Ok(df)
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
    let plan_path: plan::Plan = serde_yaml::from_str(&path_content)?;

    // Use the deserialized configuration
    let filename = &plan_path.import.profiles[0].filename;
    let mut df = load_and_process_data(filename)?;

    let sql_query = "SELECT repo1, repo2, repo1 || '-' || repo2 AS repo_id FROM df";
    df = data_loader::add_column_with_sql(&mut df, sql_query, "repo_id")?;
    println!("Updated DataFrame:\n{}", df);
    Ok(())
}
