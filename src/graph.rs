use polars::frame::row::Row;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info, warn};

use crate::data_loader::{DfEdgeLoadProfile, DfNodeLoadProfile};

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

    pub fn get_children_exluding_partition_nodes(&self, parent: &Node) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| n.belongs_to.as_deref() == Some(&parent.id) && !n.is_partition)
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

    pub fn build_tree(&self) -> Vec<TreeNode> {
        fn build_tree(
            node: &Node,
            depth: i32,
            graph: &Graph,
            tree: &mut Vec<TreeNode>,
        ) -> TreeNode {
            let mut tree_node = TreeNode::from_node(node);
            tree_node.depth = depth;
            let children = graph.get_children(node);
            for child in children {
                let child_node = build_tree(child, depth + 1, graph, tree);
                tree_node.children.push(child_node);
            }
            tree_node
        }

        let root_nodes = self.get_root_nodes();
        let mut tree = Vec::new();
        for root_node in root_nodes {
            let node = build_tree(root_node, 0, self, &mut tree);
            tree.push(node);
        }
        tree
    }

    pub fn remove_node(&mut self, id: String) {
        self.nodes.retain(|n| n.id != id);
    }

    pub fn set_node(&mut self, node: Node) {
        let idx = self.nodes.iter().position(|n| n.id == node.id);
        if let Some(idx) = idx {
            self.nodes[idx] = node;
        } else {
            self.nodes.push(node);
        }
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn stats(&self) -> String {
        format!(
            "Nodes: {}, Edges: {}, Layers: {}",
            self.nodes.len(),
            self.edges.len(),
            self.layers.len()
        )
    }

    pub fn modify_graph_limit_partition_depth(&mut self, depth: i32) -> Result<(), String> {
        fn trim_node(node_id: &String, graph: &mut Graph, current_depth: i32, max_depth: i32) {
            debug!(
                "Trimming node: {} current_depth: {} max_depth: {}",
                node_id, current_depth, max_depth
            );

            // Clone child node IDs before any mutation
            let child_node_ids: Vec<String> = {
                let node = graph.get_node(node_id).unwrap();
                graph
                    .get_children_exluding_partition_nodes(node)
                    .iter()
                    .map(|child| child.id.clone())
                    .collect()
            };

            // Recursively process children first
            for child_id in &child_node_ids {
                trim_node(child_id, graph, current_depth + 1, max_depth);
            }

            if current_depth >= max_depth {
                // Clone the node before mutating the graph
                let mut agg_node = {
                    let node = graph.get_node(node_id).unwrap();
                    node.clone()
                };

                // HashSet to track edges that need modification
                let mut nodes_to_remove = Vec::new();
                let mut new_edges = graph.edges.clone();

                for child_id in &child_node_ids {
                    if let Some(child) = graph.get_node(child_id) {
                        // Aggregate weights
                        agg_node.weight += child.weight;

                        // Process edges without duplicating them
                        for edge in &mut new_edges {
                            if edge.source == child.id {
                                edge.source = agg_node.id.clone();
                            }
                            if edge.target == child.id {
                                edge.target = agg_node.id.clone();
                            };
                        }

                        // Mark child for removal
                        nodes_to_remove.push(child.id.clone());
                    } else {
                        error!("Child node not found: {}", child_id);
                    }
                }

                graph.edges = new_edges;

                // Remove child nodes after edge updates
                for node_id in nodes_to_remove {
                    graph.remove_node(node_id);
                }

                // Update the parent node in the graph
                graph.set_node(agg_node);
            }
        }

        // Collect root nodes first to avoid borrowing issues
        let root_node_ids: Vec<String> = self
            .get_root_nodes()
            .iter()
            .map(|node| node.id.clone())
            .collect();

        for node_id in &root_node_ids {
            trim_node(node_id, self, 0, depth);
        }

        Ok(())
    }

    pub fn modify_graph_limit_partition_width(&mut self, max_width: i32) -> Result<(), String> {
        fn trim_node(node_id: &String, graph: &mut Graph, max_width: i32) {
            let mut node = {
                let node = graph.get_node(node_id).unwrap();
                node.clone()
            };

            // Clone child node IDs before any mutation
            let child_node_ids: Vec<String> = {
                graph
                    .get_children(&node)
                    .iter()
                    .map(|child| child.id.clone())
                    .collect()
            };

            debug!(
                "Trimming node: {} max_width: {}, children: {}",
                node_id,
                max_width,
                child_node_ids.len()
            );

            // Recursively process children first
            for child_id in &child_node_ids {
                trim_node(child_id, graph, max_width);
            }

            if child_node_ids.len() as i32 > max_width {
                info!("Chopping time");
                let children: Vec<Node> = child_node_ids
                    .iter()
                    .map(|id| graph.get_node(id).unwrap().clone())
                    .collect();

                // Remove children beyond max_width
                let mut nodes_to_remove = Vec::new();
                let mut new_edges = graph.edges.clone();

                for child in children.iter() {
                    // Aggregate weights
                    node.weight += child.weight;
                    // Process edges without duplicating them
                    for edge in &mut new_edges {
                        if edge.source == child.id {
                            edge.source = node.id.clone();
                        }
                        if edge.target == child.id {
                            edge.target = node.id.clone();
                        };
                    }

                    // Mark child for removal
                    nodes_to_remove.push(child.id.clone());
                }

                graph.edges = new_edges;

                // Remove child nodes after edge updates
                for node_id in nodes_to_remove {
                    graph.remove_node(node_id);
                }
                info!("Graph: {}", graph.stats());
            }
        }

        // Collect root nodes first to avoid borrowing issues
        let root_node_ids: Vec<String> = self
            .get_root_nodes()
            .iter()
            .map(|node| node.id.clone())
            .collect();

        for node_id in &root_node_ids {
            trim_node(node_id, self, max_width);
        }

        Ok(())
    }

    pub fn build_json_tree(&self) -> serde_json::Value {
        let tree = self.build_tree();
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

        self.nodes.iter().for_each(|n| {
            if n.belongs_to.is_some() {
                if !node_ids.contains(n.belongs_to.as_ref().unwrap()) {
                    let err = format!(
                        "Node id:[{}] belongs_to {:?} not found in nodes",
                        n.id,
                        n.belongs_to.as_ref().unwrap()
                    );
                    errors.push(err);
                }
            }
        });

        // verify that all nodes that are not partitions have a parent

        self.nodes.iter().for_each(|n| {
            if n.belongs_to.is_none() && !n.is_partition {
                let err = format!(
                    "Node id:[{}] is not a partition AND does not belong to a partition",
                    n.id,
                );
                errors.push(err);
            }
        });

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
    pub weight: i32,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TreeNode {
    pub id: String,
    pub depth: i32,
    pub label: String,
    pub layer: String,
    pub is_partition: bool,
    pub belongs_to: Option<String>,
    pub weight: i32,
    pub comment: Option<String>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn from_node(node: &Node) -> Self {
        Self {
            id: node.id.clone(),
            depth: 0,
            label: node.label.clone(),
            layer: node.layer.clone(),
            is_partition: node.is_partition,
            belongs_to: node.belongs_to.clone(),
            weight: node.weight,
            comment: node.comment.clone(),
            children: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub weight: i32,
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
    pub fn from_row(
        row: &Row,
        node_profile: &DfNodeLoadProfile,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Node {
            id: get_stripped_value(row, node_profile.id_column, "id").unwrap_or("noId".to_string()),
            label: get_stripped_value(row, node_profile.label_column, "label")?,
            layer: get_stripped_value(row, node_profile.layer_column, "layer")?,
            is_partition: is_truthy(&get_stripped_value(
                row,
                node_profile.is_partition_column,
                "is_partition",
            )?),
            belongs_to: {
                let belongs_to =
                    get_stripped_value(row, node_profile.belongs_to_column, "belongs_to")?;
                if belongs_to.is_empty() {
                    None
                } else if belongs_to.to_lowercase() == "null" {
                    None
                } else {
                    Some(belongs_to)
                }
            },
            weight: get_stripped_value(row, node_profile.weight_column, "weight")
                .and_then(|c| {
                    c.parse::<i32>()
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                })
                .unwrap_or(1),
            comment: row
                .0
                .get(node_profile.comment_column)
                .map(|c| c.to_string()),
        })
    }
}

impl Edge {
    pub fn from_row(
        row: &Row,
        edge_profile: &DfEdgeLoadProfile,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Edge {
            id: get_stripped_value(row, edge_profile.id_column, "id")?, // default to noId
            source: get_stripped_value(row, edge_profile.source_column, "source")?,
            target: get_stripped_value(row, edge_profile.target_column, "target")?,
            label: get_stripped_value(row, edge_profile.label_column, "label")?,
            layer: get_stripped_value(row, edge_profile.layer_column, "layer")?,
            weight: get_stripped_value(row, edge_profile.weight_column, "weight")
                .and_then(|c| {
                    c.parse::<i32>()
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                })
                .unwrap_or(1),
            comment: row
                .0
                .get(edge_profile.comment_column)
                .map(|c| c.to_string()),
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
                    weight: 1,
                    comment: None,
                },
                Node {
                    id: "2".to_string(),
                    label: "Child1".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()),
                    weight: 1,
                    comment: None,
                },
                Node {
                    id: "3".to_string(),
                    label: "Child2".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()),
                    weight: 1,
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
                    weight: 1,
                    comment: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "2".to_string(),
                    target: "3".to_string(),
                    label: "Edge2".to_string(),
                    layer: "Layer1".to_string(),
                    weight: 1,
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
            "weight": 1,
            "comment": null,
            "children": [
                {
                    "id": "2",
                    "label": "Child1",
                    "layer": "Layer1",
                    "is_partition": false,
                    "belongs_to": "1",
                    "comment": null,
                    "weight": 1,
                    "children": []
                },
                {
                    "id": "3",
                    "label": "Child2",
                    "layer": "Layer1",
                    "is_partition": false,
                    "belongs_to": "1",
                    "comment": null,
                    "weight": 1,
                    "children": []
                }
            ]
        }]);
        assert_eq!(json_tree, expected_json);
    }
}
