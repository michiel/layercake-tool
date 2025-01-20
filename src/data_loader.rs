use polars::prelude::*;
use std::path::Path;

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

fn is_valid_id(id: &str) -> bool {
    let trimmed = id.trim();
    !trimmed.is_empty()
        && trimmed != "null"
        && trimmed != "None"
        && trimmed != "NaN"
        && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn verify_nodes_df(df: &DataFrame) -> anyhow::Result<()> {
    let columns: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let required_columns = ["id", "label", "layer", "is_partition", "belongs_to"];

    // Check if columns are in the correct order and case-sensitive
    for (i, &col) in required_columns.iter().enumerate() {
        if columns.get(i) != Some(&col.to_string()) {
            return Err(anyhow::anyhow!(
                "Expected column '{}' at position {}, found '{}'",
                col,
                i,
                columns.get(i).unwrap_or(&"".to_string())
            ));
        }
    }

    // Ensure IDs are unique and not missing
    let id_series = df.column("id")?;
    let id_values: Vec<&str> = id_series.str()?.into_iter().flatten().collect();
    let mut id_set = std::collections::HashSet::new();
    let mut duplicates = Vec::new();
    let mut missing_ids = Vec::new();

    for id in &id_values {
        if !is_valid_id(id) {
            missing_ids.push(*id);
        } else if !id_set.insert(id) {
            duplicates.push(*id);
        }
    }

    if !missing_ids.is_empty() {
        return Err(anyhow::anyhow!(
            "Missing or invalid IDs found in 'id' column: {:?}",
            missing_ids
        ));
    }

    if !duplicates.is_empty() {
        return Err(anyhow::anyhow!(
            "Duplicate IDs found in 'id' column: {:?}",
            duplicates
        ));
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
