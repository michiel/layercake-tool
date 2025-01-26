use polars::frame::row::Row;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
}

impl Graph {
    pub fn get_layer_map(&self) -> HashMap<String, Layer> {
        self.layers
            .iter()
            .cloned()
            .map(|l| (l.id.clone(), l))
            .collect()
    }
    pub fn get_root_nodes(&self) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| n.belongs_to.is_none() && n.is_partition)
            .collect()
    }

    pub fn get_children(&self, parent: &Node) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| n.belongs_to.as_deref() == Some(&parent.id))
            .collect()
    }

    // pub fn get_edges(&self, node: &Node) -> Vec<&Edge> {
    //     self.edges
    //         .iter()
    //         .filter(|e| e.source == node.id || e.target == node.id)
    //         .collect()
    // }
    //
    // pub fn get_edges_from(&self, node: &Node) -> Vec<&Edge> {
    //     self.edges.iter().filter(|e| e.source == node.id).collect()
    // }
    //
    // pub fn get_edges_to(&self, node: &Node) -> Vec<&Edge> {
    //     self.edges.iter().filter(|e| e.target == node.id).collect()
    // }

    pub fn get_node_by_id(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    // pub fn get_partition_edges(&self) -> Vec<&Edge> {
    //     self.edges
    //         .iter()
    //         .filter(|e| {
    //             let source = self.get_node_by_id(&e.source).unwrap();
    //             let target = self.get_node_by_id(&e.target).unwrap();
    //             source.is_partition || target.is_partition
    //         })
    //         .collect()
    // }

    pub fn get_non_partition_edges(&self) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| {
                let source = self.get_node_by_id(&e.source).unwrap();
                let target = self.get_node_by_id(&e.target).unwrap();
                !(source.is_partition || target.is_partition)
            })
            .collect()
    }

    pub fn get_non_partition_nodes(&self) -> Vec<&Node> {
        self.nodes.iter().filter(|n| !n.is_partition).collect()
    }

    pub fn build_json_tree_recursive(&self, parent: &Node) -> serde_json::Value {
        let children = self.get_children(parent);
        let mut tree = Vec::new();
        for child in children {
            let node = serde_json::json!({
                "id": child.id,
                "label": child.label,
                "layer": child.layer,
                "is_partition": child.is_partition,
                "belongs_to": child.belongs_to,
                "comment": child.comment,
                "children": self.build_json_tree_recursive(child),
            });
            tree.push(node);
        }
        serde_json::json!(tree)
    }

    pub fn build_json_tree(&self) -> serde_json::Value {
        let root_nodes = self.get_root_nodes();
        let mut tree = Vec::new();
        for root_node in root_nodes {
            let node = serde_json::json!({
                "id": root_node.id,
                "label": root_node.label,
                "layer": root_node.layer,
                "is_partition": root_node.is_partition,
                "belongs_to": root_node.belongs_to,
                "comment": root_node.comment,
                "children": self.build_json_tree_recursive(root_node),
            });
            tree.push(node);
        }
        serde_json::json!(tree)
    }

    pub fn verify_graph_integrity(&self) -> Result<(), Vec<String>> {
        // TODO verify graph integrity
        // TODO verify that all nodes have unique ids

        let node_ids: HashSet<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        let mut errors = Vec::new();

        let mut all_edges_have_nodes = true;
        for edge in &self.edges {
            if !node_ids.contains(&edge.source) {
                all_edges_have_nodes = false;
                let err = format!(
                    "Edge id:[{}] source {:?} not found in nodes",
                    edge.id, edge.source
                );
                errors.push(err);
            }
            if !node_ids.contains(&edge.target) {
                all_edges_have_nodes = false;
                let err = format!(
                    "Edge id:[{}] target {:?} not found in nodes",
                    edge.id, edge.target
                );
                errors.push(err);
            }
        }

        if all_edges_have_nodes {
            info!("All edges have valid source and target nodes");
        } else {
            warn!("Some edges have missing source and/or target nodes");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub is_partition: bool,
    pub belongs_to: Option<String>,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Layer {
    pub id: String,
    pub label: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
}

fn is_truthy(s: &str) -> bool {
    let trimmed_lowercase = s.trim().to_lowercase();
    let re = Regex::new(r"(true|y|yes)").unwrap();
    re.is_match(&trimmed_lowercase)
}

fn strip_quotes_and_whitespace(s: &str) -> &str {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len() - 1].trim()
    } else {
        trimmed
    }
}

fn get_stripped_value(
    row: &Row,
    idx: usize,
    label: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let value = row
        .0
        .get(idx)
        .ok_or(format!("Missing {}", label))?
        .to_string();
    Ok(strip_quotes_and_whitespace(&value).to_string())
}

impl Node {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Node {
            id: get_stripped_value(row, 0, "id").unwrap_or("noId".to_string()),
            label: get_stripped_value(row, 1, "label")?,
            layer: get_stripped_value(row, 2, "layer")?,
            is_partition: is_truthy(&get_stripped_value(row, 3, "is_partition")?),
            belongs_to: {
                let belongs_to = get_stripped_value(row, 4, "belongs_to")?;
                if belongs_to.is_empty() {
                    None
                } else if belongs_to.to_lowercase() == "null" {
                    None
                } else {
                    Some(belongs_to)
                }
            },
            comment: row.0.get(5).map(|c| c.to_string()),
        })
    }
}

impl Edge {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Edge {
            id: get_stripped_value(row, 0, "id")?, // default to noId
            source: get_stripped_value(row, 1, "source")?,
            target: get_stripped_value(row, 2, "target")?,
            label: get_stripped_value(row, 3, "label")?,
            layer: get_stripped_value(row, 4, "layer")?,
            comment: row.0.get(5).map(|c| c.to_string()),
        })
    }
}

impl Layer {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            id: get_stripped_value(row, 0, "layer")?,
            label: get_stripped_value(row, 1, "label")?,
            background_color: get_stripped_value(row, 2, "background")?,
            border_color: get_stripped_value(row, 3, "border_color")?,
            text_color: get_stripped_value(row, 4, "text_color")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy("true"));
        assert!(is_truthy("True"));
        assert!(is_truthy("TRUE"));
        assert!(is_truthy("y"));
        assert!(is_truthy("Y"));
        assert!(is_truthy("yes"));
        assert!(is_truthy("Yes"));
        assert!(is_truthy("YES"));
        assert!(is_truthy(" true "));
        assert!(is_truthy("\ntrue\n"));
        assert!(is_truthy("  YES  "));

        assert!(!is_truthy("false"));
        assert!(!is_truthy("False"));
        assert!(!is_truthy("FALSE"));
        assert!(!is_truthy("n"));
        assert!(!is_truthy("N"));
        assert!(!is_truthy("no"));
        assert!(!is_truthy("No"));
        assert!(!is_truthy("NO"));
        assert!(!is_truthy("  false  "));
        assert!(!is_truthy("\nfalse\n"));
        assert!(!is_truthy("  NO  "));
    }

    fn create_test_graph() -> Graph {
        Graph {
            nodes: vec![
                Node {
                    id: "1".to_string(),
                    label: "Root".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    comment: None,
                },
                Node {
                    id: "2".to_string(),
                    label: "Child1".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()),
                    comment: None,
                },
                Node {
                    id: "3".to_string(),
                    label: "Child2".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()),
                    comment: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "1".to_string(),
                    target: "2".to_string(),
                    label: "Edge1".to_string(),
                    layer: "Layer1".to_string(),
                    comment: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "2".to_string(),
                    target: "3".to_string(),
                    label: "Edge2".to_string(),
                    layer: "Layer1".to_string(),
                    comment: None,
                },
            ],
            layers: Vec::new(),
        }
    }

    #[test]
    fn test_get_root_nodes() {
        let graph = create_test_graph();
        let root_nodes = graph.get_root_nodes();
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].id, "1");
    }

    #[test]
    fn test_get_children() {
        let graph = create_test_graph();
        let root_node = graph.get_node_by_id("1").unwrap();
        let children = graph.get_children(root_node);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].id, "2");
        assert_eq!(children[1].id, "3");
    }

    // #[test]
    // fn test_get_edges() {
    //     let graph = create_test_graph();
    //     let node = graph.get_node_by_id("2").unwrap();
    //     let edges = graph.get_edges(node);
    //     assert_eq!(edges.len(), 2);
    //     assert_eq!(edges[0].id, "e1");
    //     assert_eq!(edges[1].id, "e2");
    // }
    //
    // #[test]
    // fn test_get_edges_from() {
    //     let graph = create_test_graph();
    //     let node = graph.get_node_by_id("1").unwrap();
    //     let edges = graph.get_edges_from(node);
    //     assert_eq!(edges.len(), 1);
    //     assert_eq!(edges[0].id, "e1");
    // }
    //
    // #[test]
    // fn test_get_edges_to() {
    //     let graph = create_test_graph();
    //     let node = graph.get_node_by_id("3").unwrap();
    //     let edges = graph.get_edges_to(node);
    //     assert_eq!(edges.len(), 1);
    //     assert_eq!(edges[0].id, "e2");
    // }

    #[test]
    fn test_get_node_by_id() {
        let graph = create_test_graph();
        let node = graph.get_node_by_id("2").unwrap();
        assert_eq!(node.id, "2");
        assert_eq!(node.label, "Child1");
    }

    #[test]
    fn test_build_json_tree() {
        let graph = create_test_graph();
        let json_tree = graph.build_json_tree();
        let expected_json = serde_json::json!([{
            "id": "1",
            "label": "Root",
            "layer": "Layer1",
            "is_partition": true,
            "belongs_to": null,
            "comment": null,
            "children": [
                {
                    "id": "2",
                    "label": "Child1",
                    "layer": "Layer1",
                    "is_partition": false,
                    "belongs_to": "1",
                    "comment": null,
                    "children": []
                },
                {
                    "id": "3",
                    "label": "Child2",
                    "layer": "Layer1",
                    "is_partition": false,
                    "belongs_to": "1",
                    "comment": null,
                    "children": []
                }
            ]
        }]);
        assert_eq!(json_tree, expected_json);
    }
}
