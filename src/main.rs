mod data_loader;
use anyhow::Result;
use polars::prelude::*;

fn load_and_process_data(filename: &str) -> Result<DataFrame> {
    let df = data_loader::load_tsv(filename)?;
    println!("Loaded DataFrame:\n{}", df);
    Ok(df)
}

fn main() -> Result<()> {
    let filename = "path/to/your/file.tsv";
    let mut df = load_and_process_data(filename)?;

    let sql_query = "SELECT repo1, repo2, repo1 || '-' || repo2 AS repo_id FROM df";
    df = data_loader::add_column_with_sql(&mut df, sql_query, "repo_id")?;
    println!("Updated DataFrame:\n{}", df);
    Ok(())
}
