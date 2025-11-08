use crate::graph::Graph;
use crate::plan::RenderConfig;
use csv::Writer;
use std::error::Error;

pub fn render(graph: &Graph, _render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

    // Write the header
    wtr.write_record([
        "id",
        "label",
        "layer",
        "is_partition",
        "belongs_to",
        "comment",
    ])?;

    // Collect references and sort by ID
    let mut node_refs: Vec<_> = graph.nodes.iter().collect();
    node_refs.sort_by_key(|n| &n.id);

    for node in node_refs {
        wtr.write_record(&[
            &node.id.to_string(),
            &node.label,
            &node.layer,
            &node.is_partition.to_string(),
            node.belongs_to.as_deref().unwrap_or(""),
            node.comment.as_deref().unwrap_or(""),
        ])?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
