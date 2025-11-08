use crate::graph::Graph;
use crate::plan::RenderConfig;
use csv::Writer;
use std::error::Error;

pub fn render(graph: &Graph, _render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

    // Write the header
    wtr.write_record(["id", "source", "target", "label", "layer", "comment"])?;

    // Collect references and sort by ID
    let mut edge_refs: Vec<_> = graph.edges.iter().collect();
    edge_refs.sort_by_key(|e| &e.id);

    for edge in edge_refs {
        wtr.write_record(&[
            &edge.id.to_string(),
            &edge.source,
            &edge.target,
            &edge.label,
            &edge.layer,
            edge.comment.as_deref().unwrap_or(""),
        ])?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
