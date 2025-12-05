use layercake_code_analysis::analyzer::AnalysisResult;
use serde_json::json;

use crate::graph::{Edge, Graph, Layer, Node};

pub fn analysis_to_graph(result: &AnalysisResult, annotation: Option<String>) -> Graph {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut layers: Vec<Layer> = Vec::new();

    let mut function_ids = std::collections::HashMap::new();
    let mut library_ids = std::collections::HashMap::new();
    let mut data_ids = std::collections::HashMap::new();
    let mut entry_ids = Vec::new();
    let mut id_counts = std::collections::HashMap::<String, usize>::new();

    let mut ensure_layer = |id: &str, label: &str, bg: &str, text: &str, border: &str| {
        if !layers.iter().any(|l| l.id == id) {
            layers.push(Layer::new(id, label, bg, text, border));
        }
    };

    let mut unique_id = |base: &str| -> String {
        let slug: String = base
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect();
        let count = id_counts.entry(slug.clone()).or_insert(0);
        if *count == 0 {
            *count += 1;
            slug
        } else {
            *count += 1;
            format!("{}_{}", slug, *count)
        }
    };

    ensure_layer("function", "Function", "#e0f2ff", "#0f172a", "#0ea5e9");
    ensure_layer("data", "Data", "#fff7ed", "#431407", "#f97316");
    ensure_layer("library", "Library", "#f1f5f9", "#0f172a", "#94a3b8");
    ensure_layer("entry", "Entry", "#ecfccb", "#1a2e05", "#84cc16");
    ensure_layer("exit", "Exit", "#fee2e2", "#450a0a", "#ef4444");
    ensure_layer("scope", "Scope", "#eef2ff", "#111827", "#6366f1");

    let scope_id = unique_id("scope");
    nodes.push(Node {
        id: scope_id.clone(),
        label: "Codebase".to_string(),
        layer: "scope".to_string(),
        is_partition: true,
        belongs_to: None,
        weight: 1,
        comment: None,
        dataset: None,
        attributes: None,
    });

    let mut dir_nodes = std::collections::HashMap::new();
    for dir in &result.directories {
        let id = unique_id(&format!("dir_{}", dir));
        let parent = dir
            .rsplit_once('/')
            .map(|(p, _)| p.to_string())
            .and_then(|p| dir_nodes.get(&p).cloned())
            .unwrap_or_else(|| scope_id.clone());
        dir_nodes.insert(dir.clone(), id.clone());
        nodes.push(Node {
            id,
            label: dir.clone(),
            layer: "scope".to_string(),
            is_partition: true,
            belongs_to: Some(parent),
            weight: 1,
            comment: Some(dir.clone()),
            dataset: None,
            attributes: None,
        });
    }

    let mut file_nodes = std::collections::HashMap::new();
    for file in &result.files {
        let id = unique_id(&format!("file_{}", file));
        let parent_dir = file
            .rsplit_once('/')
            .map(|(p, _)| p.to_string())
            .and_then(|p| dir_nodes.get(&p).cloned())
            .unwrap_or_else(|| scope_id.clone());
        file_nodes.insert(file.clone(), id.clone());
        nodes.push(Node {
            id,
            label: file.clone(),
            layer: "scope".to_string(),
            is_partition: true,
            belongs_to: Some(parent_dir),
            weight: 1,
            comment: Some(file.clone()),
            dataset: None,
            attributes: None,
        });
    }

    fn add_function_node(
        function_ids: &mut std::collections::HashMap<String, String>,
        nodes: &mut Vec<Node>,
        name: &str,
        file: Option<&str>,
        belongs_to: Option<String>,
        attrs: serde_json::Value,
        mut unique_id: impl FnMut(&str) -> String,
    ) -> String {
        if let Some(id) = function_ids.get(name) {
            return id.clone();
        }
        let id = unique_id(&format!("func_{}", name));
        function_ids.insert(name.to_string(), id.clone());
        nodes.push(Node {
            id: id.clone(),
            label: name.to_string(),
            layer: "function".to_string(),
            is_partition: false,
            belongs_to,
            weight: 1,
            comment: file.map(|f| f.to_string()),
            dataset: None,
            attributes: Some(attrs),
        });
        id
    }

    for function in &result.functions {
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
            file_nodes.get(&function.file_path).cloned(),
            attrs,
            &mut unique_id,
        );
    }

    for import in &result.imports {
        let id = unique_id(&format!("lib_{}", import.module));
        if library_ids
            .insert(import.module.clone(), id.clone())
            .is_none()
        {
            nodes.push(Node {
                id: id.clone(),
                label: import.module.clone(),
                layer: "library".to_string(),
                is_partition: false,
                belongs_to: Some(scope_id.clone()),
                weight: 1,
                comment: Some(import.file_path.clone()),
                dataset: None,
                attributes: None,
            });
        }
    }

    for entry in &result.entry_points {
        let id = unique_id(&format!("entry_{}_{}", entry.file_path, entry.line_number));
        entry_ids.push((entry.file_path.clone(), id.clone()));
        nodes.push(Node {
            id: id.clone(),
            label: entry.condition.clone(),
            layer: "entry".to_string(),
            is_partition: false,
            belongs_to: Some(scope_id.clone()),
            weight: 1,
            comment: Some(entry.file_path.clone()),
            dataset: None,
            attributes: None,
        });
    }

    let mut edge_counter = 0;
    let mut next_edge_id = || {
        edge_counter += 1;
        format!("edge_{edge_counter}")
    };

    for flow in &result.data_flows {
        let src_id = function_ids.get(&flow.source).cloned().unwrap_or_else(|| {
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &flow.source,
                Some(&flow.file_path),
                file_nodes.get(&flow.file_path).cloned(),
                json!({"generated": true}),
                &mut unique_id,
            )
        });
        let sink_id = function_ids.get(&flow.sink).cloned().unwrap_or_else(|| {
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &flow.sink,
                Some(&flow.file_path),
                file_nodes.get(&flow.file_path).cloned(),
                json!({"generated": true}),
                &mut unique_id,
            )
        });

        if let Some(var) = flow.variable.as_ref().filter(|v| !v.is_empty()) {
            let data_id = data_ids
                .entry(var.clone())
                .or_insert_with(|| {
                    let id = unique_id(&format!("data_{var}"));
                    nodes.push(Node {
                        id: id.clone(),
                        label: var.clone(),
                        layer: "data".to_string(),
                        is_partition: false,
                        belongs_to: Some(scope_id.clone()),
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
        for function in &result.functions {
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
                            file_nodes.get(&function.file_path).cloned(),
                            json!({}),
                            &mut unique_id,
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
