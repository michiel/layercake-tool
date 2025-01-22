use crate::graph::Graph;
use std::fs;

pub fn render(graph: Graph, file: String) -> String {
    use serde_json::json;

    let tree = graph.build_json_tree();
    let handlebars = crate::common::get_handlebars();

    let res = handlebars.render_template(
        &fs::read_to_string(&file).unwrap(),
        &json!({
            "nodes": graph.get_non_partition_nodes(),
            "edges": graph.get_non_partition_edges(),
            "tree": tree,
            "layers": graph.get_layer_map(),
        }),
    );
    res.unwrap()
}
