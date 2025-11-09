use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

use super::csv_common::export_to_csv_sorted;

/// Export graph edges to CSV format
///
/// Edges are sorted by ID for consistent output.
pub fn render(graph: &Graph, _render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    export_to_csv_sorted(
        &graph.edges,
        &["id", "source", "target", "label", "layer", "comment"],
        |edge| edge.id.clone(), // Clone for sorting (small cost for consistency)
        |edge| {
            vec![
                edge.id.to_string(),
                edge.source.clone(),
                edge.target.clone(),
                edge.label.clone(),
                edge.layer.clone(),
                edge.comment.as_deref().unwrap_or("").to_string(),
            ]
        },
    )
}
