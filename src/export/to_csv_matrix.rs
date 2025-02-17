use crate::graph::Graph;
use crate::plan::RenderConfig;
use csv::Writer;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;

pub fn render(graph: Graph, _render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

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

    let offset = 2;

    let nodes = graph.get_non_partition_nodes();
    let edges = graph.get_non_partition_edges();

    let mut matrix =
        create_dynamic_2d_array(nodes.len() + offset, nodes.len() + offset, JsonValue::Null);
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
        matrix[i + offset][0] = json!(node.label);
        // column
        matrix[0][i + offset] = json!(node.label);
    }

    for edge in edges.clone() {
        let i = positions[&edge.source];
        let j = positions[&edge.target];
        matrix[i + offset][j + offset] = json!(edge.weight);
    }

    for row in matrix {
        wtr.write_record(
            &row.iter()
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
