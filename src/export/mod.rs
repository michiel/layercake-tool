pub mod to_csv_edges;
pub mod to_csv_matrix;
pub mod to_csv_nodes;
pub mod to_custom;
pub mod to_dot;
pub mod to_dot_hierarchy;
pub mod to_gml;
pub mod to_jsgraph;
pub mod to_json;
pub mod to_mermaid;
pub mod to_plantuml;

/// Common rendering function used by all exporters
/// This helps eliminate duplication across export modules
pub mod renderer {
    use crate::graph::Graph;
    use crate::plan::legacy_plan::RenderConfig;
    use serde_json::{json, Value};
    use std::error::Error;

    /// Standard rendering function for template-based exports
    pub fn render_template(
        graph: Graph,
        render_config: RenderConfig,
        template: &str,
    ) -> Result<String, Box<dyn Error>> {
        let handlebars = crate::common::get_handlebars();

        let context = create_standard_context(graph, render_config);

        let res = handlebars.render_template(template, &context)?;
        Ok(res)
    }

    /// Creates a standard context object used for most templates
    pub fn create_standard_context(graph: Graph, render_config: RenderConfig) -> Value {
        json!({
            "graph_name": graph.name,
            "config": render_config,
            "hierarchy_nodes": graph.get_hierarchy_nodes(),
            "hierarchy_edges": graph.get_hierarchy_edges(),
            "hierarchy_tree": graph.build_json_tree(),
            "flow_nodes": graph.get_non_partition_nodes(),
            "flow_edges": graph.get_non_partition_edges(),
            "layers": graph.get_layer_map(),
        })
    }
}

