use polars::prelude::*;
use std::path::Path;
use tracing::{debug, error, info};

pub fn load_tsv(filename: &str) -> anyhow::Result<DataFrame> {
    let path = Path::new(filename);
    LazyCsvReader::new(path)
        .with_has_header(true)
        .with_separator(b'\t')
        .finish()?
        .collect()
        .map_err(Into::into)
}

pub fn load_csv(filename: &str) -> anyhow::Result<DataFrame> {
    let path = Path::new(filename);
    LazyCsvReader::new(path)
        .with_has_header(true)
        .with_separator(b',')
        .finish()?
        .collect()
        .map_err(Into::into)
}

pub fn verify_nodes_df(df: &DataFrame) -> anyhow::Result<()> {
    let columns: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let required_columns = ["id", "label", "layer", "is_container", "belongs_to"];

    for &col in &required_columns {
        if !columns.contains(&col.to_string()) {
            return Err(anyhow::anyhow!("Missing required column: {}", col));
        }
    }

    if columns.len() < 5 {
        return Err(anyhow::anyhow!(
            "Expected a minimum of 5 columns, found {}",
            columns.len()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
