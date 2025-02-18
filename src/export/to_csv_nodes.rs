use crate::graph::Graph;
use crate::plan::RenderConfig;
use csv::Writer;
use std::error::Error;

pub fn render(graph: Graph, _render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
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

    let mut nodes = graph.nodes.clone();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    for node in nodes {
        wtr.write_record(&[
            node.id.to_string(),
            node.label,
            node.layer,
            node.is_partition.to_string(),
            node.belongs_to.unwrap_or("".to_string()),
            node.comment.unwrap_or("".to_string()),
        ])?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
