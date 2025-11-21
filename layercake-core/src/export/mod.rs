mod csv_common;
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
pub mod to_mermaid_mindmap;
pub mod to_mermaid_treemap;
pub mod to_plantuml;
pub mod to_plantuml_mindmap;
pub mod to_plantuml_wbs;

/// Common rendering function used by all exporters
/// This helps eliminate duplication across export modules
pub mod renderer {
    use crate::graph::{Edge, Graph, Layer, Node, TreeNode};
    use crate::plan::{LayerSourceStyle, RenderConfig};
    use indexmap::IndexMap;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::error::Error;

    pub struct PreparedGraphData {
        pub flow_nodes: Vec<Node>,
        pub flow_edges: Vec<Edge>,
        pub hierarchy_nodes: Vec<Node>,
        pub hierarchy_edges: Vec<Edge>,
        pub hierarchy_tree: Value,
        pub hierarchy_tree_edges: Vec<TreeNode>,
        pub layer_map: IndexMap<String, Layer>,
        pub layers: Vec<Layer>,
    }

    pub fn prepare_graph_data(graph: &Graph, render_config: &RenderConfig) -> PreparedGraphData {
        let mut hierarchy_nodes = graph.get_hierarchy_nodes();
        let mut hierarchy_edges = graph.get_hierarchy_edges();
        let mut flow_nodes: Vec<Node> = graph
            .get_non_partition_nodes()
            .into_iter()
            .cloned()
            .collect();
        let mut flow_edges: Vec<Edge> = graph
            .get_non_partition_edges()
            .into_iter()
            .cloned()
            .collect();
        let mut hierarchy_tree_nodes = graph.build_tree();
        let mut hierarchy_tree_edges = graph.build_tree_from_edges();

        if !render_config.use_node_weight {
            reset_node_weights(&mut flow_nodes);
            reset_node_weights(&mut hierarchy_nodes);
            reset_tree_weights(&mut hierarchy_tree_nodes);
            reset_tree_weights(&mut hierarchy_tree_edges);
        }

        if !render_config.use_edge_weight {
            reset_edge_weights(&mut flow_edges);
            reset_edge_weights(&mut hierarchy_edges);
        }

        let hierarchy_tree = serde_json::to_value(&hierarchy_tree_nodes).unwrap_or(Value::Null);

        let mut layer_map = graph.get_layer_map();

        let mut ensure_layer = |layer_id: &str| {
            if layer_id.is_empty() || layer_map.contains_key(layer_id) {
                return;
            }
            layer_map.insert(
                layer_id.to_string(),
                Layer::new(layer_id, layer_id, "222222", "ffffff", "dddddd"),
            );
        };

        for node in flow_nodes.iter().chain(hierarchy_nodes.iter()) {
            ensure_layer(&node.layer);
        }

        for edge in flow_edges.iter().chain(hierarchy_edges.iter()) {
            ensure_layer(&edge.layer);
        }

        let overrides: HashMap<Option<i32>, LayerSourceStyle> = render_config
            .layer_source_styles
            .iter()
            .map(|override_entry| (override_entry.source_dataset_id, override_entry.mode))
            .collect();

        for layer in layer_map.values_mut() {
            if let Some(mode) = overrides.get(&layer.dataset) {
                apply_layer_style(layer, mode);
            }
        }

        let layers: Vec<_> = layer_map.values().cloned().collect();

        PreparedGraphData {
            flow_nodes,
            flow_edges,
            hierarchy_nodes,
            hierarchy_edges,
            hierarchy_tree,
            hierarchy_tree_edges,
            layer_map,
            layers,
        }
    }

    /// Standard rendering function for template-based exports
    pub fn render_template(
        graph: &Graph,
        render_config: &RenderConfig,
        template: &str,
    ) -> Result<String, Box<dyn Error>> {
        let handlebars = crate::common::get_handlebars();

        let context = create_standard_context(graph, render_config);

        let res = handlebars.render_template(template, &context)?;
        Ok(res)
    }

    /// Creates a standard context object used for most templates
    pub fn create_standard_context(graph: &Graph, render_config: &RenderConfig) -> Value {
        let data = prepare_graph_data(graph, render_config);

        json!({
            "graph_name": &graph.name,
            "config": render_config,
            "hierarchy_nodes": data.hierarchy_nodes,
            "hierarchy_edges": data.hierarchy_edges,
            "hierarchy_tree": data.hierarchy_tree,
            "hierarchy_tree_edges": data.hierarchy_tree_edges,
            "flow_nodes": data.flow_nodes,
            "flow_edges": data.flow_edges,
            "layers": data.layers,
            "layer_map": data.layer_map,
        })
    }

    fn reset_node_weights(nodes: &mut [Node]) {
        for node in nodes {
            node.weight = 1;
        }
    }

    fn reset_edge_weights(edges: &mut [Edge]) {
        for edge in edges {
            edge.weight = 1;
        }
    }

    fn reset_tree_weights(nodes: &mut [TreeNode]) {
        for node in nodes {
            node.weight = 1;
            reset_tree_weights(&mut node.children);
        }
    }

    fn apply_layer_style(layer: &mut Layer, mode: &LayerSourceStyle) {
        let palette = match mode {
            LayerSourceStyle::Default => ("222222", "ffffff", "dddddd"),
            LayerSourceStyle::Light => ("f7f7f8", "0f172a", "e2e8f0"),
            LayerSourceStyle::Dark => ("1f2933", "f8fafc", "94a3b8"),
        };

        layer.background_color = palette.0.to_string();
        layer.text_color = palette.1.to_string();
        layer.border_color = palette.2.to_string();
    }
}
