use crate::plan::RenderConfig;
use crate::{graph::Graph, plan::CustomExportProfile};
use std::error::Error;
use std::fs;
use tracing::error;

pub fn render(
    graph: &Graph,
    render_config: &RenderConfig,
    params: &CustomExportProfile,
) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let mut handlebars = crate::common::get_handlebars();

    if let Some(partials) = &params.partials {
        for (name, partial) in partials {
            match fs::read_to_string(partial) {
                Ok(partial_content) => {
                    if let Err(err) = handlebars.register_partial(name, partial_content) {
                        error!("Failed to register partial '{}': {}", name, err);
                    }
                }
                Err(err) => {
                    error!("Failed to read partial file '{}': {}", partial, err);
                    return Err(
                        format!("Failed to read partial file '{}': {}", partial, err).into(),
                    );
                }
            }
        }
    }

    let template_content = fs::read_to_string(&params.template).map_err(|err| {
        format!(
            "Failed to read template file '{}': {}",
            params.template, err
        )
    })?;

    let prepared = crate::export::renderer::prepare_graph_data(graph, render_config);
    let layer_map_clone = prepared.layer_map.clone();
    let res = handlebars.render_template(
        &template_content,
        &json!({
            "config": render_config,
            "hierarchy_nodes": prepared.hierarchy_nodes,
            "hierarchy_edges": prepared.hierarchy_edges,
            "hierarchy_tree": prepared.hierarchy_tree,
            "hierarchy_tree_edges": prepared.hierarchy_tree_edges,
            "flow_nodes": prepared.flow_nodes,
            "flow_edges": prepared.flow_edges,
            "layers": layer_map_clone,
            "layer_map": prepared.layer_map,
            "layers_array": prepared.layers,
        }),
    )?;
    Ok(res)
}
