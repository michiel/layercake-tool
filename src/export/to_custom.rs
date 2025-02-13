use crate::plan::RenderConfig;
use crate::{graph::Graph, plan::CustomExportProfile};
use std::error::Error;
use std::fs;
use tracing::error;

pub fn render(
    graph: Graph,
    render_config: RenderConfig,
    params: CustomExportProfile,
) -> Result<String, Box<dyn Error>> {
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
            "config": render_config,
            "hierarchy_nodes": graph.nodes,
            "hierarchy_tree": tree,
            "flow_nodes": graph.get_non_partition_nodes(),
            "flow_edges": graph.get_non_partition_edges(),
            "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}
