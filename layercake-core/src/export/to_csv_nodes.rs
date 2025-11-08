use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

use super::csv_common::export_to_csv_sorted;

/// Export graph nodes to CSV format
///
/// Nodes are sorted by ID for consistent output.
pub fn render(graph: &Graph, _render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    export_to_csv_sorted(
        &graph.nodes,
        &["id", "label", "layer", "is_partition", "belongs_to", "comment"],
        |node| node.id.clone(),  // Clone for sorting (small cost for consistency)
        |node| {
            vec![
                node.id.to_string(),
                node.label.clone(),
                node.layer.clone(),
                node.is_partition.to_string(),
                node.belongs_to.as_deref().unwrap_or("").to_string(),
                node.comment.as_deref().unwrap_or("").to_string(),
            ]
        },
    )
}
