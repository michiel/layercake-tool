use anyhow::Result;
use csv::ReaderBuilder;
use serde_json::Value;
use std::collections::HashSet;

use crate::database::entities::common_types::{DataType, FileFormat};

/// Detect data type from file content using heuristics
pub fn detect_data_type(file_format: &FileFormat, file_data: &[u8]) -> Result<DataType> {
    match file_format {
        FileFormat::Csv => detect_from_csv(file_data, b','),
        FileFormat::Tsv => detect_from_csv(file_data, b'\t'),
        FileFormat::Json => detect_from_json(file_data),
        FileFormat::Xlsx | FileFormat::Ods | FileFormat::Pdf | FileFormat::Xml => {
            anyhow::bail!("File format {:?} is not supported for data type detection", file_format)
        }
    }
}

/// Detect data type from CSV/TSV headers
fn detect_from_csv(file_data: &[u8], delimiter: u8) -> Result<DataType> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(file_data);

    let headers = reader.headers()?;
    let header_set: HashSet<String> = headers.iter().map(|h| h.trim().to_lowercase()).collect();

    // Check for edges: must have id, source, target
    if header_set.contains("id") && header_set.contains("source") && header_set.contains("target") {
        return Ok(DataType::Edges);
    }

    // Check for layers: must have id and (name or label), often has color
    if header_set.contains("id")
        && (header_set.contains("name") || header_set.contains("label"))
        && (header_set.contains("color") || header_set.contains("colour"))
    {
        return Ok(DataType::Layers);
    }

    // Check for nodes: must have id, typically has label/layer/position fields
    if header_set.contains("id") {
        // Look for common node attributes
        let node_indicators = [
            "label",
            "layer",
            "x",
            "y",
            "position",
            "is_partition",
            "belongs_to",
            "weight",
        ];

        for indicator in &node_indicators {
            if header_set.contains(*indicator) {
                return Ok(DataType::Nodes);
            }
        }

        // If has id but no specific indicators, default to nodes
        return Ok(DataType::Nodes);
    }

    Err(anyhow::anyhow!(
        "Cannot determine data type from CSV headers: {:?}",
        headers.iter().collect::<Vec<_>>()
    ))
}

/// Detect data type from JSON structure
fn detect_from_json(file_data: &[u8]) -> Result<DataType> {
    let json: Value = serde_json::from_slice(file_data)?;

    // Check for graph structure (has both nodes and edges)
    if let Some(obj) = json.as_object() {
        if obj.contains_key("nodes") && obj.contains_key("edges") {
            return Ok(DataType::Graph);
        }

        // Single collection - try to detect type
        if obj.contains_key("nodes") {
            return Ok(DataType::Nodes);
        }
        if obj.contains_key("edges") {
            return Ok(DataType::Edges);
        }
        if obj.contains_key("layers") {
            return Ok(DataType::Layers);
        }
    }

    // Check if it's an array - analyze first element
    if let Some(arr) = json.as_array() {
        if let Some(first) = arr.first() {
            if let Some(obj) = first.as_object() {
                let keys: HashSet<String> = obj.keys().map(|k| k.to_lowercase()).collect();

                // Check for edge indicators
                if keys.contains("source") && keys.contains("target") {
                    return Ok(DataType::Edges);
                }

                // Check for layer indicators
                if (keys.contains("name") || keys.contains("label"))
                    && (keys.contains("color") || keys.contains("colour"))
                {
                    return Ok(DataType::Layers);
                }

                // Check for node indicators
                if keys.contains("id") {
                    return Ok(DataType::Nodes);
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Cannot determine data type from JSON structure"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_csv_nodes() {
        let csv_data = b"id,label,layer,x,y\nnode1,Node 1,layer1,10,20\n";
        let result = detect_from_csv(csv_data, b',');
        assert!(result.is_ok());
        assert_eq!(result.expect("Should detect nodes"), DataType::Nodes);
    }

    #[test]
    fn test_detect_csv_edges() {
        let csv_data = b"id,source,target,label\nedge1,node1,node2,connects\n";
        let result = detect_from_csv(csv_data, b',');
        assert!(result.is_ok());
        assert_eq!(result.expect("Should detect edges"), DataType::Edges);
    }

    #[test]
    fn test_detect_csv_layers() {
        let csv_data = b"id,name,color\nlayer1,Layer 1,#FF0000\n";
        let result = detect_from_csv(csv_data, b',');
        assert!(result.is_ok());
        assert_eq!(result.expect("Should detect layers"), DataType::Layers);
    }

    #[test]
    fn test_detect_json_graph() {
        let json_data = br#"{"nodes": [], "edges": []}"#;
        let result = detect_from_json(json_data);
        assert!(result.is_ok());
        assert_eq!(result.expect("Should detect graph"), DataType::Graph);
    }

    #[test]
    fn test_detect_json_nodes_array() {
        let json_data = br#"[{"id": "1", "label": "Node 1"}]"#;
        let result = detect_from_json(json_data);
        assert!(result.is_ok());
        assert_eq!(result.expect("Should detect nodes array"), DataType::Nodes);
    }

    #[test]
    fn test_detect_json_edges_array() {
        let json_data = br#"[{"id": "e1", "source": "n1", "target": "n2"}]"#;
        let result = detect_from_json(json_data);
        assert!(result.is_ok());
        assert_eq!(result.expect("Should detect edges array"), DataType::Edges);
    }
}
