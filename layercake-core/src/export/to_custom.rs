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

    let tree = graph.build_json_tree();
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

    let res = handlebars.render_template(
        &template_content,
        &json!({
            "config": render_config,
            "hierarchy_nodes": graph.get_hierarchy_nodes(),
            "hierarchy_tree": tree,
            "flow_nodes": graph.get_non_partition_nodes(),
            "flow_edges": graph.get_non_partition_edges(),
            "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}
