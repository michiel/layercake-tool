use polars::prelude::*;
use std::fmt::{Display, Formatter};
use std::path::Path;
use tracing::info;

pub struct DfNodeLoadProfile {
    pub id_column: usize,
    pub label_column: usize,
    pub layer_columns: usize,
    pub is_partition_column: usize,
    pub belongs_to_column: usize,
    pub comment: usize,
}

impl Default for DfNodeLoadProfile {
    fn default() -> Self {
        Self {
            id_column: 0,
            label_column: 1,
            layer_columns: 2,
            is_partition_column: 3,
            belongs_to_column: 4,
            comment: 5,
        }
    }
}

impl Display for DfNodeLoadProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node column offsets: id:{}, label:{}, layer:{}, is_partition:{}, belongs_to:{}, comment:{}",
            self.id_column,
            self.label_column,
            self.layer_columns,
            self.is_partition_column,
            self.belongs_to_column,
            self.comment
        )
    }
}

pub fn create_df_node_load_profile(df: &DataFrame) -> DfNodeLoadProfile {
    let mut profile = DfNodeLoadProfile::default();
    for (i, field) in df.schema().iter_fields().enumerate() {
        match field.name().as_str() {
            "id" => profile.id_column = i as usize,
            "label" => profile.label_column = i as usize,
            "layer" => profile.layer_columns = i as usize,
            "is_partition" => profile.is_partition_column = i as usize,
            "belongs_to" => profile.belongs_to_column = i as usize,
            "comment" => profile.comment = i as usize,
            _ => {}
        }
    }
    profile
}

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
    Ok(())
}

pub fn verify_id_column_df(
    df: &DataFrame,
    _node_profile: &DfNodeLoadProfile,
) -> anyhow::Result<()> {
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

    Ok(())
}

#[cfg(test)]
mod tests {}
