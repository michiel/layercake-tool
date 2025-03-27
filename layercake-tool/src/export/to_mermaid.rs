use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

pub fn render(graph: Graph, render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
        "config": render_config,
        // "tree": data,
        "hierarchy_nodes": graph.get_hierarchy_nodes(),
        "hierarchy_edges": graph.get_hierarchy_edges(),
        "hierarchy_tree": graph.build_json_tree(),
        "flow_nodes": graph.get_non_partition_nodes(),
        "flow_edges": graph.get_non_partition_edges(),
        "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    include_str!("to_mermaid.hbs").to_string()
}
