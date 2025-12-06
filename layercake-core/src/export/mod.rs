mod csv_common;
pub mod sequence_renderer;
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
pub mod to_mermaid_sequence;
pub mod to_mermaid_treemap;
pub mod to_plantuml;
pub mod to_plantuml_mindmap;
pub mod to_plantuml_sequence;
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
            strip_alias_metadata(layer);
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

        let flow_edges = with_relative_weight(&data.flow_edges);
        let hierarchy_edges = with_relative_weight(&data.hierarchy_edges);

        json!({
            "graph_name": &graph.name,
            "config": render_config,
            "hierarchy_nodes": data.hierarchy_nodes,
            "hierarchy_edges": hierarchy_edges,
            "hierarchy_tree": data.hierarchy_tree,
            "hierarchy_tree_edges": data.hierarchy_tree_edges,
            "flow_nodes": data.flow_nodes,
            "flow_edges": flow_edges,
            "layers": data.layers,
            "layer_map": data.layer_map,
        })
    }

    fn with_relative_weight(edges: &[Edge]) -> Vec<Value> {
        if edges.is_empty() {
            return Vec::new();
        }

        let (min_w, max_w) = edges.iter().fold((i32::MAX, i32::MIN), |(min_w, max_w), e| {
            let w = std::cmp::max(1, e.weight);
            (min_w.min(w), max_w.max(w))
        });

        let range = (max_w - min_w).max(1) as f64;

        edges
            .iter()
            .map(|edge| {
                let weight = std::cmp::max(1, edge.weight) as f64;
                let ratio = ((weight - min_w as f64) / range).clamp(0.0, 1.0);
                let rel = (1.0 + (ratio * 5.0)).round() as i32; // 1-6 inclusive

                let mut value = serde_json::to_value(edge).unwrap_or(Value::Null);
                if let Some(map) = value.as_object_mut() {
                    map.insert("relative_weight".to_string(), Value::from(rel));
                }
                value
            })
            .collect()
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

    fn strip_alias_metadata(layer: &mut Layer) {
        layer.alias = None;
    }
}

#[cfg(test)]
mod tests {
    use super::renderer::prepare_graph_data;
    use crate::graph::{Graph, Layer, Node};
    use crate::plan::{
        NotePosition, RenderConfig, RenderConfigBuiltInStyle, RenderConfigOrientation,
        RenderTargetOptions,
    };

    fn create_node(id: &str, label: &str, layer: &str) -> Node {
        Node {
            id: id.to_string(),
            label: label.to_string(),
            layer: layer.to_string(),
            is_partition: false,
            belongs_to: None,
            weight: 1,
            comment: None,
            dataset: None,
            attributes: None,
        }
    }

    fn create_layer(id: &str) -> Layer {
        Layer::new(id, id, "aabbcc", "112233", "445566")
    }

    fn create_test_config() -> RenderConfig {
        RenderConfig {
            contain_nodes: false,
            orientation: RenderConfigOrientation::TB,
            apply_layers: true,
            built_in_styles: RenderConfigBuiltInStyle::Light,
            target_options: RenderTargetOptions::default(),
            add_node_comments_as_notes: false,
            note_position: NotePosition::Left,
            use_node_weight: true,
            use_edge_weight: true,
            layer_source_styles: vec![],
        }
    }

    #[test]
    fn test_missing_layers_are_created_with_defaults() {
        // Graph with nodes referencing layers that don't exist
        let graph = Graph {
            name: "Test".to_string(),
            nodes: vec![
                create_node("n1", "Node 1", "defined_layer"),
                create_node("n2", "Node 2", "missing_layer"),
                create_node("n3", "Node 3", "another_missing"),
            ],
            edges: vec![],
            layers: vec![create_layer("defined_layer")],
            annotations: None,
        };

        let config = create_test_config();
        let prepared = prepare_graph_data(&graph, &config);

        // Should have 3 layers: 1 defined + 2 missing (created with defaults)
        assert_eq!(prepared.layers.len(), 3);
        assert_eq!(prepared.layer_map.len(), 3);

        // Check that defined layer retains its colours
        let defined = prepared.layer_map.get("defined_layer").unwrap();
        assert_eq!(defined.background_color, "aabbcc");

        // Check that missing layers have default colours
        let missing1 = prepared.layer_map.get("missing_layer").unwrap();
        assert_eq!(missing1.background_color, "222222");
        assert_eq!(missing1.text_color, "ffffff");
        assert_eq!(missing1.border_color, "dddddd");

        let missing2 = prepared.layer_map.get("another_missing").unwrap();
        assert_eq!(missing2.background_color, "222222");
    }

    #[test]
    fn test_empty_layer_id_is_not_added() {
        let graph = Graph {
            name: "Test".to_string(),
            nodes: vec![
                create_node("n1", "Node 1", "layer1"),
                create_node("n2", "Node 2", ""), // empty layer
            ],
            edges: vec![],
            layers: vec![create_layer("layer1")],
            annotations: None,
        };

        let config = create_test_config();
        let prepared = prepare_graph_data(&graph, &config);

        // Should only have 1 layer (empty string layer should not be added)
        assert_eq!(prepared.layers.len(), 1);
        assert!(prepared.layer_map.contains_key("layer1"));
        assert!(!prepared.layer_map.contains_key(""));
    }

    #[test]
    fn test_dot_render_includes_nodes_with_missing_layers() {
        use crate::export::to_dot;

        let graph = Graph {
            name: "Test".to_string(),
            nodes: vec![
                create_node("n1", "Node 1", "defined_layer"),
                create_node("n2", "Node 2", "missing_layer"),
            ],
            edges: vec![],
            layers: vec![create_layer("defined_layer")],
            annotations: None,
        };

        let mut config = create_test_config();
        config.apply_layers = true;

        let result = to_dot::render(&graph, &config).unwrap();

        // Both nodes should be defined in the output
        assert!(result.contains("n1[label="), "Node n1 should be defined");
        assert!(result.contains("n2[label="), "Node n2 should be defined");

        // Missing layer should have default styling applied
        assert!(
            result.contains("222222"),
            "Default background colour should be present"
        );
    }

    #[test]
    fn test_mermaid_render_includes_nodes_with_missing_layers() {
        use crate::export::to_mermaid;

        let graph = Graph {
            name: "Test".to_string(),
            nodes: vec![
                create_node("n1", "Node 1", "defined_layer"),
                create_node("n2", "Node 2", "missing_layer"),
            ],
            edges: vec![],
            layers: vec![create_layer("defined_layer")],
            annotations: None,
        };

        let mut config = create_test_config();
        config.apply_layers = true;

        let result = to_mermaid::render(&graph, &config).unwrap();

        // Both nodes should be defined
        assert!(
            result.contains("n1[\"Node 1\"]"),
            "Node n1 should be defined"
        );
        assert!(
            result.contains("n2[\"Node 2\"]"),
            "Node n2 should be defined"
        );

        // Both class definitions should exist
        assert!(
            result.contains("classDef defined_layer"),
            "defined_layer class should be present"
        );
        assert!(
            result.contains("classDef missing_layer"),
            "missing_layer class should be present"
        );
    }
}
