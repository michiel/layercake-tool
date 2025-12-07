use layercake_code_analysis::analyzer::AnalysisResult;
use serde_json::json;

use crate::graph::{Edge, Graph, Layer, Node};

pub fn analysis_to_graph(
    result: &AnalysisResult,
    annotation: Option<String>,
    coalesce_functions: bool,
) -> Graph {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut layers: Vec<Layer> = Vec::new();

    let mut function_ids: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut functions_by_name: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut functions_by_canonical: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut library_ids: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let _data_ids: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut entry_ids = Vec::new();
    let mut id_counts = std::collections::HashMap::<String, usize>::new();
    let mut function_imports: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

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

    fn function_key(name: &str, file: Option<&str>) -> String {
        match file {
            Some(path) => format!("{}::{}", path, name),
            None => name.to_string(),
        }
    }

    fn canonical_name(name: &str) -> String {
        let trimmed = name.trim();
        let base = trimmed
            .rsplit(['.', ':', ' '])
            .next()
            .unwrap_or(trimmed)
            .trim();
        base.trim_matches(|c| c == '(' || c == ')').to_string()
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
        let key = function_key(name, file);
        if let Some(id) = function_ids.get(&key) {
            return id.clone();
        }
        let id = unique_id(&format!("func_{}", name));
        function_ids.insert(key, id.clone());
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

    fn ensure_library_node(
        library_ids: &mut std::collections::HashMap<String, String>,
        nodes: &mut Vec<Node>,
        unique_id: &mut impl FnMut(&str) -> String,
        scope_id: &str,
        module: &str,
        file: Option<&str>,
    ) -> String {
        if let Some(id) = library_ids.get(module) {
            // verify it exists
            if nodes.iter().any(|n| &n.id == id) {
                return id.clone();
            }
        }
        let id = unique_id(&format!("lib_{}", module));
        library_ids.insert(module.to_string(), id.clone());
        nodes.push(Node {
            id: id.clone(),
            label: module.to_string(),
            layer: "library".to_string(),
            is_partition: false,
            belongs_to: Some(scope_id.to_string()),
            weight: 1,
            comment: file.map(|f| f.to_string()),
            dataset: None,
            attributes: None,
        });
        id
    }

    for function in &result.functions {
        functions_by_name
            .entry(function.name.clone())
            .or_default()
            .push(function.file_path.clone());
        functions_by_canonical
            .entry(canonical_name(&function.name))
            .or_default()
            .push(function.file_path.clone());
        let attrs = json!({
            "complexity": function.complexity,
            "return_type": function.return_type,
            "file": function.file_path,
            "line": function.line_number,
            "args": function.args,
        });
        let parent = file_nodes
            .get(&function.file_path)
            .cloned()
            .or_else(|| Some(scope_id.clone()));
        add_function_node(
            &mut function_ids,
            &mut nodes,
            &function.name,
            Some(&function.file_path),
            parent,
            attrs,
            &mut unique_id,
        );
    }

    for import in &result.imports {
        ensure_library_node(
            &mut library_ids,
            &mut nodes,
            &mut unique_id,
            &scope_id,
            &import.module,
            Some(&import.file_path),
        );
        function_imports
            .entry(import.file_path.clone())
            .or_default()
            .push(import.module.clone());
    }

    // Helper to normalize absolute file paths back to known relative paths
    let known_files: Vec<String> = file_nodes.keys().cloned().collect();
    let normalize_file_ref = |path: &str| -> String {
        if known_files.contains(&path.to_string()) {
            return path.to_string();
        }
        if let Some(found) = known_files
            .iter()
            .find(|f| path.ends_with(f.as_str()))
            .cloned()
        {
            return found;
        }
        path.to_string()
    };

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
        let flow_file = normalize_file_ref(&flow.file_path);
        let src_key = function_key(&flow.source, Some(&flow_file));
        let src_id = function_ids.get(&src_key).cloned().unwrap_or_else(|| {
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &flow.source,
                Some(&flow_file),
                file_nodes.get(&flow_file).cloned(),
                json!({"generated": true}),
                &mut unique_id,
            )
        });
        let sink_key = function_key(&flow.sink, Some(&flow_file));
        let sink_id = function_ids.get(&sink_key).cloned().unwrap_or_else(|| {
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &flow.sink,
                Some(&flow_file),
                file_nodes.get(&flow_file).cloned(),
                json!({"generated": true}),
                &mut unique_id,
            )
        });

        edges.push(Edge {
            id: next_edge_id(),
            source: src_id.clone(),
            target: sink_id.clone(),
            label: flow.variable.clone().unwrap_or_default(),
            layer: "dataflow".to_string(),
            weight: 1,
            comment: None,
            dataset: None,
            attributes: None,
        });
    }

    for call in &result.call_edges {
        let call_file = normalize_file_ref(&call.file_path);
        let caller_key = function_key(&call.caller, Some(&call_file));
        let caller_id = function_ids.get(&caller_key).cloned().unwrap_or_else(|| {
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &call.caller,
                Some(&call_file),
                file_nodes
                    .get(&call_file)
                    .cloned()
                    .or_else(|| Some(scope_id.clone())),
                json!({"generated": true}),
                &mut unique_id,
            )
        });
        let callee_key = function_key(&call.callee, Some(&call_file));
        let callee_id = function_ids.get(&callee_key).cloned().unwrap_or_else(|| {
            let callee_canon = canonical_name(&call.callee);
            let chosen_file = functions_by_canonical
                .get(&callee_canon)
                .or_else(|| functions_by_name.get(&call.callee))
                .and_then(|list| {
                    if list.len() == 1 {
                        Some(list[0].clone())
                    } else {
                        let caller_dir = std::path::Path::new(&call.file_path)
                            .parent()
                            .and_then(|p| p.to_str().map(|s| s.to_string()));
                        caller_dir
                            .and_then(|dir| {
                                list.iter()
                                    .find(|p| {
                                        std::path::Path::new(p).parent().and_then(|pp| pp.to_str())
                                            == Some(dir.as_str())
                                    })
                                    .cloned()
                            })
                            .or_else(|| list.first().cloned())
                    }
                })
                .map(|p| normalize_file_ref(&p));
            let target_file = chosen_file.as_deref().unwrap_or(&call_file);
            add_function_node(
                &mut function_ids,
                &mut nodes,
                &call.callee,
                Some(target_file),
                file_nodes
                    .get(target_file)
                    .cloned()
                    .or_else(|| Some(scope_id.clone())),
                json!({"generated": true}),
                &mut unique_id,
            )
        });

        edges.push(Edge {
            id: next_edge_id(),
            source: caller_id,
            target: callee_id,
            label: call.callee.clone(),
            layer: "controlflow".to_string(),
            weight: 1,
            comment: None,
            dataset: None,
            attributes: None,
        });
    }
    for (file_path, entry_id) in &entry_ids {
        for function in &result.functions {
            if &function.file_path == file_path {
                let func_key = function_key(&function.name, Some(&function.file_path));
                let func_id = function_ids.get(&func_key).cloned().unwrap_or_else(|| {
                    add_function_node(
                        &mut function_ids,
                        &mut nodes,
                        &function.name,
                        Some(&function.file_path),
                        file_nodes
                            .get(&function.file_path)
                            .cloned()
                            .or_else(|| Some(scope_id.clone())),
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

    // Import edges: library -> function using it
    for function in &result.functions {
        if let Some(libs) = function_imports.get(&function.file_path) {
            let func_key = function_key(&function.name, Some(&function.file_path));
            let func_id = function_ids.get(&func_key).cloned().unwrap_or_else(|| {
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

            for lib in libs {
                let lib_id = ensure_library_node(
                    &mut library_ids,
                    &mut nodes,
                    &mut unique_id,
                    &scope_id,
                    lib,
                    None,
                );
                edges.push(Edge {
                    id: next_edge_id(),
                    source: lib_id,
                    target: func_id.clone(),
                    label: lib.clone(),
                    layer: "import".to_string(),
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

    let mut graph = Graph {
        name: "code-analysis".to_string(),
        nodes,
        edges,
        layers,
        annotations: annotation,
    };
    graph.sanitize_labels();
    if coalesce_functions {
        graph.coalesce_functions_to_files();
    }
    graph
}
