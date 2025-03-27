use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::Result;
use csv::StringRecord;

pub struct DfNodeLoadProfile {
    pub id_column: usize,
    pub label_column: usize,
    pub layer_column: usize,
    pub is_partition_column: usize,
    pub belongs_to_column: usize,
    pub weight_column: usize,
    pub comment_column: usize,
}

impl Default for DfNodeLoadProfile {
    fn default() -> Self {
        Self {
            id_column: 0,
            label_column: 1,
            layer_column: 2,
            is_partition_column: 3,
            belongs_to_column: 4,
            weight_column: 5,
            comment_column: 6,
        }
    }
}

impl Display for DfNodeLoadProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node column offsets: id:{}, label:{}, layer:{}, is_partition:{}, belongs_to:{}, weight:{}, comment:{}",
            self.id_column,
            self.label_column,
            self.layer_column,
            self.is_partition_column,
            self.belongs_to_column,
            self.weight_column,
            self.comment_column,
        )
    }
}

pub struct DfEdgeLoadProfile {
    pub id_column: usize,
    pub source_column: usize,
    pub target_column: usize,
    pub label_column: usize,
    pub layer_column: usize,
    pub weight_column: usize,
    pub comment_column: usize,
}

impl Default for DfEdgeLoadProfile {
    fn default() -> Self {
        Self {
            id_column: 0,
            source_column: 1,
            target_column: 2,
            label_column: 3,
            layer_column: 4,
            weight_column: 5,
            comment_column: 6,
        }
    }
}

impl Display for DfEdgeLoadProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Edge column offsets: id:{}, source:{}, target:{}, label:{}, layer:{}, weight:{}, comment:{}",
            self.id_column,
            self.source_column,
            self.target_column,
            self.label_column,
            self.layer_column,
            self.weight_column,
            self.comment_column,
        )
    }
}

pub fn create_df_node_load_profile(headers: &[String]) -> DfNodeLoadProfile {
    let mut profile = DfNodeLoadProfile::default();
    for (i, field) in headers.iter().enumerate() {
        match field.as_str() {
            "id" => profile.id_column = i,
            "label" => profile.label_column = i,
            "layer" => profile.layer_column = i,
            "is_partition" => profile.is_partition_column = i,
            "belongs_to" => profile.belongs_to_column = i,
            "weight" => profile.weight_column = i,
            "comment" => profile.comment_column = i,
            _ => {}
        }
    }
    profile
}

pub fn create_df_edge_load_profile(headers: &[String]) -> DfEdgeLoadProfile {
    let mut profile = DfEdgeLoadProfile::default();
    for (i, field) in headers.iter().enumerate() {
        match field.as_str() {
            "id" => profile.id_column = i,
            "source" => profile.source_column = i,
            "target" => profile.target_column = i,
            "label" => profile.label_column = i,
            "layer" => profile.layer_column = i,
            "weight" => profile.weight_column = i,
            "comment" => profile.comment_column = i,
            _ => {}
        }
    }
    profile
}

pub fn get_headers_from_file(filename: &str, separator: u8) -> anyhow::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    if let Some(Ok(header)) = lines.next() {
        let headers: Vec<String> = header
            .split(separator as char)
            .map(|col_name| col_name.trim().to_string())
            .collect();

        Ok(headers)
    } else {
        Err(anyhow::anyhow!("Failed to read header from file"))
    }
}

pub fn load_tsv(filename: &str) -> anyhow::Result<Vec<StringRecord>> {
    let path = Path::new(filename);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_path(path)?;
    
    let records: Vec<StringRecord> = reader.records()
        .collect::<Result<_, _>>()?;
    
    Ok(records)
}

pub fn load_csv(filename: &str) -> anyhow::Result<Vec<StringRecord>> {
    let path = Path::new(filename);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_path(path)?;
    
    let records: Vec<StringRecord> = reader.records()
        .collect::<Result<_, _>>()?;
    
    Ok(records)
}

fn is_valid_id(id: &str) -> bool {
    let trimmed = id.trim();
    !trimmed.is_empty()
        && trimmed != "null"
        && trimmed != "None"
        && trimmed != "NaN"
        && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn verify_nodes_headers(headers: &[String]) -> anyhow::Result<()> {
    let columns: HashSet<&String> = headers.iter().collect();
    let required_columns: HashSet<&str> = ["id", "label", "layer", "is_partition", "belongs_to"]
        .iter()
        .cloned()
        .collect();

    for &col in &required_columns {
        if !columns.contains(&col.to_string()) {
            return Err(anyhow::anyhow!("Missing required column '{}'", col));
        }
    }
    Ok(())
}

pub fn verify_id_column(
    records: &[StringRecord],
    id_column_index: usize,
) -> anyhow::Result<()> {
    // Ensure IDs are unique and not missing
    let mut id_set = std::collections::HashSet::new();
    let mut duplicates = Vec::new();
    let mut missing_ids = Vec::new();

    for record in records {
        if let Some(id) = record.get(id_column_index) {
            if !is_valid_id(id) {
                missing_ids.push(id.to_string());
            } else if !id_set.insert(id) {
                duplicates.push(id.to_string());
            }
        } else {
            missing_ids.push("<missing>".to_string());
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
