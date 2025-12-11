use layercake_code_analysis::analyzer::AnalysisResult;
use layercake_code_analysis::infra::enhanced_correlate;
use serde_json::json;

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

/// Enhanced solution graph that includes infrastructure-code data flows
pub fn analysis_to_enhanced_solution_graph(
    result: &AnalysisResult,
    annotation: Option<String>,
) -> Graph {
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

    // Layers
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
    ensure_layer("infra", "Infrastructure", "#ddd6fe", "#3b0764", "#a78bfa");
    ensure_layer(
        "code-to-infra",
        "Code → Infra",
        "#d1fae5",
        "#064e3b",
        "#34d399",
    );
    ensure_layer(
        "infra-to-code",
        "Infra → Code",
        "#fce7f3",
        "#831843",
        "#f472b6",
    );

    let root_id = unique_id("solution_root".to_string());
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

    // Directories as partitions (but non-partition nodes in solution view)
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
            is_partition: false,
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

    // Entry points
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
            attributes: Some(json!({
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
                attributes: Some(json!({"edge_type": "entry_point"})),
            });
        }
    }

    // Exit points
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
            attributes: Some(json!({
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
                attributes: Some(json!({"edge_type": "exit_point"})),
            });
        }
    }

    // External calls - store with detailed IDs
    let mut external_call_nodes = std::collections::HashMap::new();
    for call in &result.external_calls {
        let node_id = unique_id(format!(
            "extcall_{}_{}",
            slugify(&call.target),
            call.line_number
        ));
        let call_key = format!("{}:{}", call.file_path, call.line_number);
        external_call_nodes.insert(call_key, node_id.clone());

        nodes.push(Node {
            id: node_id.clone(),
            label: call.target.clone(),
            layer: "external_call".to_string(),
            is_partition: false,
            belongs_to: Some(root_id.clone()),
            weight: 1,
            comment: Some(call.file_path.clone()),
            dataset: None,
            attributes: Some(json!({
                "method": call.method,
                "path": call.path,
                "line": call.line_number,
                "target": call.target,
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
                comment: Some("Code invokes external service".to_string()),
                dataset: None,
                attributes: Some(json!({
                    "edge_type": "external_invocation"
                })),
            });
        }
    }

    // Environment variables
    let mut env_var_nodes = std::collections::HashMap::new();
    for env in &result.env_vars {
        let node_id = unique_id(format!("env_{}_{}", slugify(&env.name), env.line_number));
        env_var_nodes.insert(env.name.clone(), node_id.clone());

        nodes.push(Node {
            id: node_id.clone(),
            label: env.name.clone(),
            layer: "env".to_string(),
            is_partition: false,
            belongs_to: Some(root_id.clone()),
            weight: 1,
            comment: Some(env.file_path.clone()),
            dataset: None,
            attributes: Some(json!({
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
                comment: Some("Code reads environment variable".to_string()),
                dataset: None,
                attributes: Some(json!({
                    "edge_type": "env_read"
                })),
            });
        }
    }

    // Add infrastructure nodes and perform enhanced correlation
    let mut infra_node_map = std::collections::HashMap::new();
    if let Some(infra) = &result.infra {
        // Add infrastructure nodes
        for (resource_id, resource) in &infra.resources {
            let infra_node_id = unique_id(format!("infra_{}", slugify(resource_id)));
            infra_node_map.insert(resource_id.clone(), infra_node_id.clone());

            nodes.push(Node {
                id: infra_node_id.clone(),
                label: resource.name.clone(),
                layer: "infra".to_string(),
                is_partition: false,
                belongs_to: Some(root_id.clone()),
                weight: 1,
                comment: Some(format!("{:?}", resource.resource_type)),
                dataset: None,
                attributes: Some(json!({
                    "resource_type": format!("{:?}", resource.resource_type),
                    "source_file": resource.source_file,
                    "properties": resource.properties,
                })),
            });
        }

        // Add infrastructure-to-infrastructure edges
        for edge in &infra.edges {
            if let (Some(from_id), Some(to_id)) =
                (infra_node_map.get(&edge.from), infra_node_map.get(&edge.to))
            {
                edges.push(Edge {
                    id: next_edge_id(),
                    source: from_id.clone(),
                    target: to_id.clone(),
                    label: edge.label.clone().unwrap_or_else(|| "depends".to_string()),
                    layer: "infra".to_string(),
                    weight: 1,
                    comment: Some(format!("{:?}", edge.edge_type)),
                    dataset: None,
                    attributes: Some(json!({
                        "edge_type": format!("{:?}", edge.edge_type)
                    })),
                });
            }
        }

        // Perform enhanced correlation
        let enhanced_corr = enhanced_correlate(result, infra);

        // Link external calls to infrastructure
        for ext_match in &enhanced_corr.external_call_matches {
            if let (Some(call_node_id), Some(infra_node_id)) = (
                external_call_nodes.get(&ext_match.external_call_id),
                infra_node_map.get(&ext_match.infra_resource_id),
            ) {
                edges.push(Edge {
                    id: next_edge_id(),
                    source: call_node_id.clone(),
                    target: infra_node_id.clone(),
                    label: ext_match
                        .operation
                        .clone()
                        .unwrap_or_else(|| "accesses".to_string()),
                    layer: "code-to-infra".to_string(),
                    weight: ext_match.confidence.max(10) as i32,
                    comment: Some(format!(
                        "{} (confidence: {}%)",
                        ext_match.reason, ext_match.confidence
                    )),
                    dataset: None,
                    attributes: Some(json!({
                        "edge_type": "external_call_to_resource",
                        "confidence": ext_match.confidence,
                        "reason": ext_match.reason,
                        "operation": ext_match.operation,
                    })),
                });
            }
        }

        // Link environment variables to infrastructure
        for env_match in &enhanced_corr.env_var_matches {
            if let (Some(env_node_id), Some(infra_node_id)) = (
                env_var_nodes.get(&env_match.env_var_name),
                infra_node_map.get(&env_match.infra_resource_id),
            ) {
                edges.push(Edge {
                    id: next_edge_id(),
                    source: infra_node_id.clone(),
                    target: env_node_id.clone(),
                    label: "configures".to_string(),
                    layer: "infra-to-code".to_string(),
                    weight: env_match.confidence.max(10) as i32,
                    comment: Some(format!(
                        "{} (confidence: {}%)",
                        env_match.reason, env_match.confidence
                    )),
                    dataset: None,
                    attributes: Some(json!({
                        "edge_type": "env_var_from_resource",
                        "confidence": env_match.confidence,
                        "reason": env_match.reason,
                    })),
                });
            }
        }

        // Add data flow edges (infra → code and code → infra)
        for data_flow in &enhanced_corr.data_flow_matches {
            if let (Some(from_infra), Some(to_code)) = (&data_flow.from_infra, &data_flow.to_code) {
                // Infra → Code (e.g., reading from S3)
                if let (Some(infra_id), Some(code_file_id)) = (
                    infra_node_map.get(from_infra),
                    file_nodes.get(to_code.split("::").next().unwrap_or(to_code)),
                ) {
                    edges.push(Edge {
                        id: next_edge_id(),
                        source: infra_id.clone(),
                        target: code_file_id.clone(),
                        label: data_flow.flow_type.clone(),
                        layer: "infra-to-code".to_string(),
                        weight: data_flow.confidence.max(10) as i32,
                        comment: Some(format!(
                            "Data flows from infrastructure to code ({})",
                            data_flow.flow_type
                        )),
                        dataset: None,
                        attributes: Some(json!({
                            "edge_type": "data_flow_infra_to_code",
                            "flow_type": data_flow.flow_type,
                            "confidence": data_flow.confidence,
                        })),
                    });
                }
            }

            if let (Some(from_code), Some(to_infra)) = (&data_flow.from_code, &data_flow.to_infra) {
                // Code → Infra (e.g., writing to DynamoDB)
                if let (Some(code_file_id), Some(infra_id)) = (
                    file_nodes.get(from_code.split("::").next().unwrap_or(from_code)),
                    infra_node_map.get(to_infra),
                ) {
                    edges.push(Edge {
                        id: next_edge_id(),
                        source: code_file_id.clone(),
                        target: infra_id.clone(),
                        label: data_flow.flow_type.clone(),
                        layer: "code-to-infra".to_string(),
                        weight: data_flow.confidence.max(10) as i32,
                        comment: Some(format!(
                            "Data flows from code to infrastructure ({})",
                            data_flow.flow_type
                        )),
                        dataset: None,
                        attributes: Some(json!({
                            "edge_type": "data_flow_code_to_infra",
                            "flow_type": data_flow.flow_type,
                            "confidence": data_flow.confidence,
                        })),
                    });
                }
            }
        }
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    layers.sort_by(|a, b| a.id.cmp(&b.id));

    Graph {
        name: "enhanced-solution-analysis".to_string(),
        nodes,
        edges,
        layers,
        annotations: annotation,
    }
}
