use layercake_code_analysis::analyzer::{AnalysisResult, GraphConvertible};
use serde_json::json;

use crate::graph::{Edge, Graph, Layer, Node};

impl GraphConvertible<Graph> for AnalysisResult {
    fn to_graph(&self, annotation: Option<String>) -> Graph {
        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut layers: Vec<Layer> = Vec::new();

        let mut function_ids = std::collections::HashMap::new();
        let mut library_ids = std::collections::HashMap::new();
        let mut data_ids = std::collections::HashMap::new();
        let mut entry_ids = Vec::new();

        let mut ensure_layer = |id: &str, label: &str, bg: &str, text: &str, border: &str| {
            if !layers.iter().any(|l| l.id == id) {
                layers.push(Layer::new(id, label, bg, text, border));
            }
        };

        ensure_layer("function", "Function", "#e0f2ff", "#0f172a", "#0ea5e9");
        ensure_layer("data", "Data", "#fff7ed", "#431407", "#f97316");
        ensure_layer("library", "Library", "#f1f5f9", "#0f172a", "#94a3b8");
        ensure_layer("entry", "Entry", "#ecfccb", "#1a2e05", "#84cc16");
        ensure_layer("exit", "Exit", "#fee2e2", "#450a0a", "#ef4444");

        fn add_function_node(
            function_ids: &mut std::collections::HashMap<String, String>,
            nodes: &mut Vec<Node>,
            name: &str,
            file: Option<&str>,
            attrs: serde_json::Value,
        ) -> String {
            if let Some(id) = function_ids.get(name) {
                return id.clone();
            }
            let id = format!("func:{name}");
            function_ids.insert(name.to_string(), id.clone());
            nodes.push(Node {
                id: id.clone(),
                label: name.to_string(),
                layer: "function".to_string(),
                is_partition: false,
                belongs_to: None,
                weight: 1,
                comment: file.map(|f| f.to_string()),
                dataset: None,
                attributes: Some(attrs),
            });
            id
        }

        for function in &self.functions {
            let attrs = json!({
                "complexity": function.complexity,
                "return_type": function.return_type,
                "file": function.file_path,
                "line": function.line_number,
                "args": function.args,
            });
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &function.name,
                Some(&function.file_path),
                attrs,
            );
        }

        for import in &self.imports {
            let id = format!("lib:{}", import.module);
            if library_ids
                .insert(import.module.clone(), id.clone())
                .is_none()
            {
                nodes.push(Node {
                    id: id.clone(),
                    label: import.module.clone(),
                    layer: "library".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: Some(import.file_path.clone()),
                    dataset: None,
                    attributes: None,
                });
            }
        }

        for entry in &self.entry_points {
            let id = format!("entry:{}:{}", entry.file_path, entry.line_number);
            entry_ids.push((entry.file_path.clone(), id.clone()));
            nodes.push(Node {
                id: id.clone(),
                label: entry.condition.clone(),
                layer: "entry".to_string(),
                is_partition: false,
                belongs_to: None,
                weight: 1,
                comment: Some(entry.file_path.clone()),
                dataset: None,
                attributes: None,
            });
        }

        let mut edge_counter = 0;
        let mut next_edge_id = || {
            edge_counter += 1;
            format!("edge-{edge_counter}")
        };

        for flow in &self.data_flows {
            let src_id = function_ids.get(&flow.source).cloned().unwrap_or_else(|| {
                add_function_node(
                    &mut function_ids,
                    &mut nodes,
                    &flow.source,
                    Some(&flow.file_path),
                    json!({"generated": true}),
                )
            });
            let sink_id = function_ids.get(&flow.sink).cloned().unwrap_or_else(|| {
                add_function_node(
                    &mut function_ids,
                    &mut nodes,
                    &flow.sink,
                    Some(&flow.file_path),
                    json!({"generated": true}),
                )
            });

            if let Some(var) = flow.variable.as_ref().filter(|v| !v.is_empty()) {
                let data_id = data_ids
                    .entry(var.clone())
                    .or_insert_with(|| {
                        let id = format!("data:{var}");
                        nodes.push(Node {
                            id: id.clone(),
                            label: var.clone(),
                            layer: "data".to_string(),
                            is_partition: false,
                            belongs_to: None,
                            weight: 1,
                            comment: Some(flow.file_path.clone()),
                            dataset: None,
                            attributes: None,
                        });
                        id
                    })
                    .clone();

                edges.push(Edge {
                    id: next_edge_id(),
                    source: src_id.clone(),
                    target: data_id.clone(),
                    label: var.clone(),
                    layer: "data".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                });

                edges.push(Edge {
                    id: next_edge_id(),
                    source: data_id,
                    target: sink_id,
                    label: var.clone(),
                    layer: "data".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                });
            } else {
                edges.push(Edge {
                    id: next_edge_id(),
                    source: src_id,
                    target: sink_id,
                    label: flow.variable.clone().unwrap_or_default(),
                    layer: "function".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                });
            }
        }

        for (file_path, entry_id) in &entry_ids {
            for function in &self.functions {
                if &function.file_path == file_path {
                    let func_id = function_ids
                        .get(&function.name)
                        .cloned()
                        .unwrap_or_else(|| {
                            add_function_node(
                                &mut function_ids,
                                &mut nodes,
                                &function.name,
                                Some(&function.file_path),
                                json!({}),
                            )
                        });
                    edges.push(Edge {
                        id: next_edge_id(),
                        source: entry_id.clone(),
                        target: func_id,
                        label: function.name.clone(),
                        layer: "entry".to_string(),
                        weight: 1,
                        comment: None,
                        dataset: None,
                        attributes: None,
                    });
                }
            }
        }

        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        edges.sort_by(|a, b| a.id.cmp(&b.id));
        layers.sort_by(|a, b| a.id.cmp(&b.id));

        Graph {
            name: "code-analysis".to_string(),
            nodes,
            edges,
            layers,
            annotations: annotation,
        }
    }
}
