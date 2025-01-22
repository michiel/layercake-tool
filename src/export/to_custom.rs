use crate::{graph::Graph, plan::CustomExportProfile};
use std::fs;
use tracing::error;

pub fn render(graph: Graph, params: CustomExportProfile) -> String {
    use serde_json::json;

    let tree = graph.build_json_tree();
    let mut handlebars = crate::common::get_handlebars();

    if let Some(partials) = params.partials {
        for (name, partial) in partials {
            let partial_content = fs::read_to_string(&partial).unwrap();

            if let Err(err) = handlebars.register_partial(&name, partial_content) {
                error!("Failed to register partial: {}", err);
            }
        }
    }

    let res = handlebars.render_template(
        &fs::read_to_string(&params.template).unwrap(),
        &json!({
            "nodes": graph.get_non_partition_nodes(),
            "edges": graph.get_non_partition_edges(),
            "tree": tree,
            "layers": graph.get_layer_map(),
        }),
    );
    res.unwrap()
}
