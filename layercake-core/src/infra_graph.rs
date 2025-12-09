use layercake_code_analysis::infra::{EdgeType, InfrastructureGraph};
use serde_json::json;

use crate::graph::{Edge, Graph, Layer, Node};

pub fn infra_to_graph(graph: &InfrastructureGraph, annotation: Option<String>) -> Graph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut layers = Vec::new();

    let mut ensure_layer = |id: &str, label: &str, bg: &str, text: &str, border: &str| {
        if !layers.iter().any(|l: &Layer| l.id == id) {
            layers.push(Layer::new(id, label, bg, text, border));
        }
    };

    ensure_layer("infra", "Infra Resource", "#eef2ff", "#0f172a", "#6366f1");
    ensure_layer(
        "infra-partition",
        "Infra Partition",
        "#f1f5f9",
        "#0f172a",
        "#94a3b8",
    );
    ensure_layer(
        "infra-depends",
        "Infra Depends",
        "#fff7ed",
        "#7c2d12",
        "#fdba74",
    );
    ensure_layer(
        "infra-ref",
        "Infra Reference",
        "#ecfccb",
        "#365314",
        "#bef264",
    );
    ensure_layer(
        "infra-code-link",
        "Code ↔ Infra",
        "#e0f2fe",
        "#0ea5e9",
        "#0ea5e9",
    );

    for partition in graph.partitions.values() {
        nodes.push(Node {
            id: partition.id.clone(),
            label: partition.label.clone(),
            layer: "infra-partition".to_string(),
            is_partition: true,
            belongs_to: partition.parent.clone(),
            weight: 1,
            comment: partition.comment.clone(),
            dataset: None,
            attributes: None,
        });
    }

    for resource in graph.resources.values() {
        let belongs_to = resource
            .belongs_to
            .clone()
            .or_else(|| {
                // fall back to partition matching the directory of the source file
                std::path::Path::new(&resource.source_file)
                    .parent()
                    .and_then(|p| p.to_str())
                    .and_then(|dir| graph.partitions.get(dir).map(|p| p.id.clone()))
            })
            .or(Some(graph.root_id.clone()));
        nodes.push(Node {
            id: resource.id.clone(),
            label: resource.name.clone(),
            layer: "infra".to_string(),
            is_partition: false,
            belongs_to,
            weight: 1,
            comment: Some(resource.source_file.clone()),
            dataset: None,
            attributes: Some(json!({
                "resource_type": format!("{:?}", resource.resource_type),
                "properties": resource.properties,
            })),
        });
    }

    let mut edge_counter = 0;
    let mut next_edge_id = || {
        edge_counter += 1;
        format!("edge_{edge_counter}")
    };

    for edge in &graph.edges {
        let layer = match edge.edge_type {
            EdgeType::DependsOn => "infra-depends",
            EdgeType::References => "infra-ref",
            EdgeType::CodeLink => "infra-code-link",
        };
        edges.push(Edge {
            id: next_edge_id(),
            source: edge.from.clone(),
            target: edge.to.clone(),
            label: edge.label.clone().unwrap_or_default(),
            layer: layer.to_string(),
            weight: 1,
            comment: None,
            dataset: None,
            attributes: Some(json!({
                "edge_type": format!("{:?}", edge.edge_type),
            })),
        });
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    layers.sort_by(|a, b| a.id.cmp(&b.id));

    Graph {
        name: "infra-analysis".to_string(),
        nodes,
        edges,
        layers,
        annotations: annotation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layercake_code_analysis::infra::{EdgeType, ResourceNode, ResourceType};

    #[test]
    fn converts_infra_graph_to_layercake_graph() {
        let mut infra = InfrastructureGraph::new("root");
        let mut node = ResourceNode::new(
            "aws_s3_bucket.bucket",
            ResourceType::Aws("aws_s3_bucket".into()),
            "bucket",
            "main.tf",
        );
        node.properties.insert("region".into(), "us-east-1".into());
        let partition = infra.ensure_partition("modules/storage", None, None);
        node.belongs_to = Some(partition.clone());
        infra.add_resource(node);
        infra.add_edge(layercake_code_analysis::infra::GraphEdge {
            from: partition.clone(),
            to: "aws_s3_bucket_bucket".into(),
            edge_type: EdgeType::DependsOn,
            label: None,
        });

        let graph = infra_to_graph(&infra, Some("infra".into()));
        assert_eq!(graph.name, "infra-analysis");
        assert_eq!(graph.annotations.as_deref(), Some("infra"));
        assert!(graph.nodes.iter().any(|n| n.layer == "infra"));
        assert!(graph.layers.iter().any(|l| l.id == "infra"));
        assert_eq!(graph.edges.len(), 1);
    }
}
