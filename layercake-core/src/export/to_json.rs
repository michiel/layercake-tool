use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

pub fn render(graph: &Graph, render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let prepared = crate::export::renderer::prepare_graph_data(graph, render_config);

    let res = json!({
        "hierarchy_nodes": prepared.hierarchy_nodes,
        "hierarchy_edges": prepared.hierarchy_edges,
        "flow_nodes": prepared.flow_nodes,
        "flow_edges": prepared.flow_edges,
        "tree": prepared.hierarchy_tree,
        "layers": prepared.layer_map,
        "hierarchy_tree_edges": prepared.hierarchy_tree_edges,
    });
    Ok(serde_json::to_string_pretty(&res)?)
}
