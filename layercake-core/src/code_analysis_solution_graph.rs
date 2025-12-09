use layercake_code_analysis::analyzer::AnalysisResult;

use crate::graph::{Edge, Graph, Layer, Node};

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else {
            slug.push('_');
        }
    }
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    slug.trim_matches('_').to_string()
}

/// Build a solution-level graph without function nodes, focusing on entry/exit, files, and external calls.
pub fn analysis_to_solution_graph(result: &AnalysisResult, annotation: Option<String>) -> Graph {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut layers: Vec<Layer> = Vec::new();
    let mut id_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut ensure_layer = |id: &str, label: &str, bg: &str, text: &str, border: &str| {
        if !layers.iter().any(|l| l.id == id) {
            layers.push(Layer::new(id, label, bg, text, border));
        }
    };

    let mut unique_id = |base: String| {
        if id_set.insert(base.clone()) {
            base
        } else {
            let mut counter = 2;
            loop {
                let cand = format!("{base}_{counter}");
                if id_set.insert(cand.clone()) {
                    break cand;
                }
                counter += 1;
            }
        }
    };

    ensure_layer("scope", "Scope", "#eef2ff", "#111827", "#6366f1");
    ensure_layer("entry", "Entry", "#ecfccb", "#1a2e05", "#84cc16");
    ensure_layer("exit", "Exit", "#fee2e2", "#450a0a", "#ef4444");
    ensure_layer(
        "external_call",
        "External Call",
        "#e0f2fe",
        "#0f172a",
        "#0ea5e9",
    );
    ensure_layer("env", "Env Var", "#fef9c3", "#713f12", "#f59e0b");

    let root_id = "solution_root".to_string();
    let root_id = unique_id(root_id);
    nodes.push(Node {
        id: root_id.clone(),
        label: "Solution".to_string(),
        layer: "scope".to_string(),
        is_partition: true,
        belongs_to: None,
        weight: 1,
        comment: None,
        dataset: None,
        attributes: None,
    });

    // Directories as partitions
    let mut dir_nodes = std::collections::HashMap::new();
    for dir in &result.directories {
        let id = unique_id(format!("dir_{}", slugify(dir)));
        let parent = dir
            .rsplit_once('/')
            .map(|(p, _)| p.to_string())
            .and_then(|p| dir_nodes.get(&p).cloned())
            .unwrap_or_else(|| root_id.clone());
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

    // Files as flow nodes
    let mut file_nodes = std::collections::HashMap::new();
    for file in &result.files {
        let id = unique_id(format!("file_{}", slugify(file)));
        let parent_dir = file
            .rsplit_once('/')
            .map(|(p, _)| p.to_string())
            .and_then(|p| dir_nodes.get(&p).cloned())
            .unwrap_or_else(|| root_id.clone());
        file_nodes.insert(file.clone(), id.clone());
        nodes.push(Node {
            id,
            label: file.clone(),
            layer: "scope".to_string(),
            is_partition: false,
            belongs_to: Some(parent_dir),
            weight: 1,
            comment: Some(file.clone()),
            dataset: None,
            attributes: None,
        });
    }

    let mut edge_counter = 0;
    let mut next_edge_id = || {
        edge_counter += 1;
        format!("edge_{edge_counter}")
    };

    // Entry nodes and edges to files in the same file_path
    for entry in &result.entry_points {
        let id = unique_id(format!(
            "entry_{}_{}",
            slugify(&entry.file_path),
            entry.line_number
        ));
        nodes.push(Node {
            id: id.clone(),
            label: entry.condition.clone(),
            layer: "entry".to_string(),
            is_partition: false,
            belongs_to: Some(root_id.clone()),
            weight: 1,
            comment: Some(entry.file_path.clone()),
            dataset: None,
            attributes: Some(serde_json::json!({
                "line": entry.line_number,
                "file": entry.file_path,
            })),
        });
        if let Some(file_id) = file_nodes.get(&entry.file_path) {
            edges.push(Edge {
                id: next_edge_id(),
                source: id.clone(),
                target: file_id.clone(),
                label: "entry".to_string(),
                layer: "entry".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            });
        }
    }

    // Exit nodes: attach to files
    for exit in &result.exits {
        let id = unique_id(format!(
            "exit_{}_{}",
            slugify(&exit.file_path),
            exit.line_number
        ));
        nodes.push(Node {
            id: id.clone(),
            label: exit.condition.clone(),
            layer: "exit".to_string(),
            is_partition: false,
            belongs_to: Some(root_id.clone()),
            weight: 1,
            comment: Some(exit.file_path.clone()),
            dataset: None,
            attributes: Some(serde_json::json!({
                "line": exit.line_number,
                "file": exit.file_path,
            })),
        });
        if let Some(file_id) = file_nodes.get(&exit.file_path) {
            edges.push(Edge {
                id: next_edge_id(),
                source: file_id.clone(),
                target: id.clone(),
                label: "exit".to_string(),
                layer: "exit".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            });
        }
    }

    // External calls: create nodes and edges from owning file
    for call in &result.external_calls {
        let node_id = unique_id(format!(
            "extcall_{}_{}",
            slugify(&call.target),
            call.line_number
        ));
        nodes.push(Node {
            id: node_id.clone(),
            label: call.target.clone(),
            layer: "external_call".to_string(),
            is_partition: false,
            belongs_to: Some(root_id.clone()),
            weight: 1,
            comment: Some(call.file_path.clone()),
            dataset: None,
            attributes: Some(serde_json::json!({
                "method": call.method,
                "path": call.path,
                "line": call.line_number
            })),
        });
        if let Some(file_id) = file_nodes.get(&call.file_path) {
            edges.push(Edge {
                id: next_edge_id(),
                source: file_id.clone(),
                target: node_id.clone(),
                label: call.method.clone().unwrap_or_else(|| "call".to_string()),
                layer: "external_call".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            });
        }
    }

    // Env vars: summarize as nodes per file
    for env in &result.env_vars {
        let node_id = unique_id(format!("env_{}_{}", slugify(&env.name), env.line_number));
        nodes.push(Node {
            id: node_id.clone(),
            label: env.name.clone(),
            layer: "env".to_string(),
            is_partition: false,
            belongs_to: Some(root_id.clone()),
            weight: 1,
            comment: Some(env.file_path.clone()),
            dataset: None,
            attributes: Some(serde_json::json!({
                "kind": env.kind,
                "line": env.line_number,
                "file": env.file_path,
            })),
        });
        if let Some(file_id) = file_nodes.get(&env.file_path) {
            edges.push(Edge {
                id: next_edge_id(),
                source: file_id.clone(),
                target: node_id.clone(),
                label: env.kind.clone(),
                layer: "env".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            });
        }
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    layers.sort_by(|a, b| a.id.cmp(&b.id));

    Graph {
        name: "solution-analysis".to_string(),
        nodes,
        edges,
        layers,
        annotations: annotation,
    }
}
