use crate::graph::Graph;
use csv::Writer;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

    // Write the header
    wtr.write_record(&["id", "source", "target", "label", "layer", "comment"])?;

    for edge in graph.edges {
        wtr.write_record(&[
            edge.id.to_string(),
            edge.source,
            edge.target,
            edge.label,
            edge.layer,
            edge.comment.unwrap_or_default(),
        ])?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
