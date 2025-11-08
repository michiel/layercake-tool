use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

pub fn render(graph: &Graph, _render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let tree = graph.build_json_tree();

    let res = json!({
        "hierarchy_nodes": graph.get_hierarchy_nodes(),
        "hierarchy_edges": graph.get_hierarchy_edges(),
        "flow_nodes": graph.get_non_partition_nodes(),
        "flow_edges": graph.get_non_partition_edges(),
        "tree": tree,
        "layers": graph.get_layer_map(),
    });
    Ok(serde_json::to_string_pretty(&res)?)
}
