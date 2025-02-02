use crate::graph::Graph;
use csv::Writer;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);

    // Write the header
    wtr.write_record(&[
        "id",
        "label",
        "layer",
        "belongs_to",
        "is_partition",
        "comment",
    ])?;

    for node in graph.nodes {
        wtr.write_record(&[
            node.id.to_string(),
            node.label,
            node.layer,
            node.belongs_to.unwrap_or_default(),
            node.is_partition.to_string(),
            node.comment.unwrap_or_default(),
        ])?;
    }

    let data = wtr.into_inner()?;
    let csv_string = String::from_utf8(data)?;

    Ok(csv_string)
}
