use crate::graph::Graph;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let tree = graph.build_json_tree();

    let res = json!({
        "hierarchy_nodes": graph.nodes,
        "hierarchy_edges": graph.get_hierarchy_edges(),
        "flow_nodes": graph.get_non_partition_nodes(),
        "flow_edges": graph.get_non_partition_edges(),
        "tree": tree,
        "layers": graph.get_layer_map(),
    });
    Ok(serde_json::to_string_pretty(&res)?)
}
