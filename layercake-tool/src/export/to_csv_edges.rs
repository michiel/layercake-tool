use crate::graph::Graph;
use crate::plan::RenderConfig;
use csv::Writer;
use std::error::Error;

pub fn render(graph: Graph, _render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

    // Write the header
    wtr.write_record(["id", "source", "target", "label", "layer", "comment"])?;

    let mut edges = graph.edges.clone();
    edges.sort_by(|a, b| a.id.cmp(&b.id));

    for edge in edges {
        wtr.write_record(&[
            edge.id.to_string(),
            edge.source,
            edge.target,
            edge.label,
            edge.layer,
            edge.comment.unwrap_or("".to_string()),
        ])?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
