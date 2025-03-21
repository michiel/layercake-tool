use crate::graph::Graph;
use crate::plan::RenderConfig;
use csv::WriterBuilder;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use tracing::warn;

pub fn render(graph: Graph, _render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    warn!("Rendering to CSV matrix is an experimental feature, may not work as expected and will change.");

    fn create_dynamic_2d_array<T: Clone>(rows: usize, cols: usize, default: T) -> Vec<Vec<T>> {
        let mut matrix = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for _ in 0..cols {
                row.push(default.clone());
            }
            matrix.push(row);
        }
        matrix
    }

    fn check_unique_ids<T, F>(items: &[T], id_extractor: F) -> bool
    where
        F: Fn(&T) -> &String,
    {
        let mut seen_ids = HashSet::new();
        for item in items {
            let id = id_extractor(item);
            if !seen_ids.insert(id.clone()) {
                return false; // Duplicate found
            }
        }
        true // All IDs are unique
    }

    let offset = 2;

    let mut nodes = graph.get_non_partition_nodes();
    let mut edges = graph.get_non_partition_edges();

    if !check_unique_ids(&nodes, |node| &node.id) {
        return Err("Duplicate node IDs found.".into());
    }

    if !check_unique_ids(&edges, |edge| &edge.id) {
        return Err("Duplicate edge IDs found.".into());
    }

    let mut matrix =
        create_dynamic_2d_array(nodes.len() + offset, nodes.len() + offset, JsonValue::Null);

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let positions: HashMap<String, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.clone(), i))
        .collect();

    matrix[0][1] = json!("Source");
    matrix[1][0] = json!("Target");

    for node in nodes.clone() {
        let i = positions[&node.id];
        // row
        matrix[i + offset][1] = json!(node.label);
        // column
        matrix[1][i + offset] = json!(node.label);
    }

    edges.sort_by(|a, b| a.id.cmp(&b.id));
    for edge in edges.clone() {
        let i = positions[&edge.source];
        let j = positions[&edge.target];
        matrix[i + offset][j + offset] = json!(edge.weight);
    }

    let mut wtr = WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Never)
        .from_writer(vec![]);

    for row in matrix {
        wtr.write_record(
            row.iter()
                .map(|cell| match cell {
                    JsonValue::Null => String::new(),
                    _ => cell.to_string(),
                })
                .collect::<Vec<String>>(),
        )?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
