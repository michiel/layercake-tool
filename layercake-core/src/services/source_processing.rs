use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::database::entities::common_types::{DataType, FileFormat};

/// Shared routines for processing dataset files into graph JSON payloads
pub async fn process_file(
    file_format: &FileFormat,
    data_type: &DataType,
    file_data: &[u8],
) -> Result<String> {
    match (file_format, data_type) {
        (FileFormat::Csv, DataType::Nodes) => process_delimited_nodes(file_data, b',').await,
        (FileFormat::Csv, DataType::Edges) => process_delimited_edges(file_data, b',').await,
        (FileFormat::Csv, DataType::Layers) => process_delimited_layers(file_data, b',').await,
        (FileFormat::Tsv, DataType::Nodes) => process_delimited_nodes(file_data, b'\t').await,
        (FileFormat::Tsv, DataType::Edges) => process_delimited_edges(file_data, b'\t').await,
        (FileFormat::Tsv, DataType::Layers) => process_delimited_layers(file_data, b'\t').await,
        (FileFormat::Json, DataType::Graph) => process_json_graph(file_data).await,
        _ => Err(anyhow!("Invalid format/type combination")),
    }
}

async fn process_delimited_nodes(file_data: &[u8], delimiter: u8) -> Result<String> {
    let content = String::from_utf8(file_data.to_vec())?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delimiter)
        .from_reader(content.as_bytes());

    let headers = reader.headers()?.clone();
    let mut nodes = Vec::new();

    if !headers.iter().any(|h| h == "id") || !headers.iter().any(|h| h == "label") {
        return Err(anyhow!("CSV must contain 'id' and 'label' columns"));
    }

    for result in reader.records() {
        let record = result?;
        let mut node = HashMap::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                match header {
                    "id" => {
                        node.insert("id".to_string(), json!(field));
                    }
                    "label" => {
                        node.insert("label".to_string(), json!(field));
                    }
                    "layer" => {
                        if !field.is_empty() {
                            node.insert("layer".to_string(), json!(field));
                        }
                    }
                    "x" => {
                        if let Ok(x) = field.parse::<f64>() {
                            node.insert("x".to_string(), json!(x));
                        }
                    }
                    "y" => {
                        if let Ok(y) = field.parse::<f64>() {
                            node.insert("y".to_string(), json!(y));
                        }
                    }
                    _ => {
                        if !field.is_empty() {
                            node.insert(header.to_string(), json!(field));
                        }
                    }
                };
            }
        }

        nodes.push(json!(node));
    }

    let graph_json = json!({
        "nodes": nodes,
        "edges": [],
        "layers": []
    });

    Ok(serde_json::to_string(&graph_json)?)
}

async fn process_delimited_edges(file_data: &[u8], delimiter: u8) -> Result<String> {
    let content = String::from_utf8(file_data.to_vec())?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delimiter)
        .from_reader(content.as_bytes());

    let headers = reader.headers()?.clone();
    let mut edges = Vec::new();

    let required_headers = ["id", "source", "target"];
    for required in &required_headers {
        if !headers.iter().any(|h| h == *required) {
            return Err(anyhow!("CSV must contain '{}' column", required));
        }
    }

    for result in reader.records() {
        let record = result?;
        let mut edge = HashMap::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                match header {
                    "id" | "source" | "target" => {
                        edge.insert(header.to_string(), json!(field));
                    }
                    "label" => {
                        if !field.is_empty() {
                            edge.insert(header.to_string(), json!(field));
                        }
                    }
                    "weight" => {
                        if let Ok(weight) = field.parse::<f64>() {
                            edge.insert("weight".to_string(), json!(weight));
                        }
                    }
                    _ => {
                        if !field.is_empty() {
                            edge.insert(header.to_string(), json!(field));
                        }
                    }
                };
            }
        }

        edges.push(json!(edge));
    }

    let graph_json = json!({
        "nodes": [],
        "edges": edges,
        "layers": []
    });

    Ok(serde_json::to_string(&graph_json)?)
}

async fn process_delimited_layers(file_data: &[u8], delimiter: u8) -> Result<String> {
    let content = String::from_utf8(file_data.to_vec())?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delimiter)
        .from_reader(content.as_bytes());

    let headers = reader.headers()?.clone();
    let mut layers = Vec::new();

    if !headers.iter().any(|h| h == "layer" || h == "id") {
        return Err(anyhow!("CSV must contain 'layer' column"));
    }
    if !headers.iter().any(|h| h == "label") {
        return Err(anyhow!("CSV must contain 'label' column"));
    }

    for result in reader.records() {
        let record = result?;
        let mut layer = HashMap::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                let key = if header == "layer" { "id" } else { header };
                match key {
                    "id" | "label" => {
                        layer.insert(key.to_string(), json!(field));
                    }
                    "description" => {
                        if !field.is_empty() {
                            layer.insert(key.to_string(), json!(field));
                        }
                    }
                    "color" | "background" => {
                        if !field.is_empty() {
                            layer.insert("background_color".to_string(), json!(field));
                        }
                    }
                    "border" => {
                        if !field.is_empty() {
                            layer.insert("border_color".to_string(), json!(field));
                        }
                    }
                    "text" => {
                        if !field.is_empty() {
                            layer.insert("text_color".to_string(), json!(field));
                        }
                    }
                    "z_index" => {
                        if let Ok(z) = field.parse::<i32>() {
                            layer.insert("z_index".to_string(), json!(z));
                        }
                    }
                    _ => {
                        if !field.is_empty() {
                            layer.insert(key.to_string(), json!(field));
                        }
                    }
                };
            }
        }

        layers.push(json!(layer));
    }

    let graph_json = json!({
        "nodes": [],
        "edges": [],
        "layers": layers
    });

    Ok(serde_json::to_string(&graph_json)?)
}

async fn process_json_graph(file_data: &[u8]) -> Result<String> {
    let content = String::from_utf8(file_data.to_vec())?;
    let graph_data: Value = serde_json::from_str(&content)?;

    if !graph_data.is_object() {
        return Err(anyhow!("JSON must be an object"));
    }

    let obj = graph_data
        .as_object()
        .ok_or_else(|| anyhow!("JSON data is not a valid object"))?;

    if !obj.contains_key("nodes") || !obj.contains_key("edges") || !obj.contains_key("layers") {
        return Err(anyhow!(
            "JSON must contain 'nodes', 'edges', and 'layers' arrays"
        ));
    }

    if !obj["nodes"].is_array() || !obj["edges"].is_array() || !obj["layers"].is_array() {
        return Err(anyhow!("'nodes', 'edges', and 'layers' must be arrays"));
    }

    Ok(serde_json::to_string(&graph_data)?)
}
