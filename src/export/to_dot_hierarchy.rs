use crate::graph::Graph;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let handlebars = crate::common::get_handlebars();

    let res = handlebars.render_template(
        &get_template(),
        &json!({
            "hierarchy_nodes": graph.nodes,
            "hierarchy_edges": graph.get_hierarchy_edges(),
            // "hierarchy_tree": tree,
            // "flow_nodes": graph.get_non_partition_nodes(),
            // "flow_edges": graph.get_non_partition_edges(),
            "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    include_str!("to_dot_hierarchy.hbs").to_string()
}
