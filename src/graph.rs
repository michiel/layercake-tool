use indexmap::IndexMap;
use polars::frame::row::Row;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, warn};

use crate::data_loader::{DfEdgeLoadProfile, DfNodeLoadProfile};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Graph {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
}

impl Graph {
    pub fn get_layer_map(&self) -> IndexMap<String, Layer> {
        let layers: IndexMap<String, Layer> = self
            .layers
            .iter()
            .cloned()
            .map(|l| (l.id.clone(), l))
            .collect();

        layers.into_iter().collect()
    }

    // Check if a layer exists
    fn layer_exists(&self, layer_id: &str) -> bool {
        self.layers.iter().any(|l| l.id == layer_id)
    }

    // Add a new layer if it does not exist
    pub fn add_layer(&mut self, layer: Layer) {
        if !self.layers.iter().any(|l| l.id == layer.id) {
            self.layers.push(layer);
        }
    }

    pub fn get_root_nodes(&self) -> Vec<&Node> {
        let mut nodes: Vec<&Node> = self
            .nodes
            .iter()
            .filter(|n| n.belongs_to.is_none() && n.is_partition)
            .collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        nodes
    }

    pub fn get_children(&self, parent: &Node) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| n.belongs_to.as_deref() == Some(&parent.id))
            .collect()
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_max_hierarchy_depth(&self) -> i32 {
        fn max_child_depth(node: &TreeNode) -> i32 {
            let mut max_depth = node.depth;
            for child in &node.children {
                let child_depth = max_child_depth(child);
                if child_depth > max_depth {
                    max_depth = child_depth;
                }
            }
            max_depth
        }

        let mut max_depth = 0;
        let tree = self.build_tree();
        for node in &tree {
            let child_depth = max_child_depth(node);
            if child_depth > max_depth {
                max_depth = child_depth;
            }
        }
        max_depth
    }

    pub fn get_hierarchy_nodes(&self) -> Vec<Node> {
        let mut nodes = self.nodes.clone();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        nodes
    }

    pub fn get_hierarchy_edges(&self) -> Vec<Edge> {
        let mut edges = Vec::new();
        self.nodes.iter().for_each(|node| {
            if let Some(parent_id) = &node.belongs_to {
                let parent = self.get_node(parent_id).unwrap();
                edges.push(Edge {
                    id: format!("{}_{}", parent.id, node.id),
                    source: parent.id.clone(),
                    target: node.id.clone(),
                    label: "".to_string(), // format!("{} -> {}", parent.label, node.label),
                    layer: parent.layer.clone(),
                    weight: 1,
                    comment: None,
                });
            }
        });

        edges
    }

    pub fn get_non_partition_edges(&self) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .edges
            .iter()
            .filter(|e| {
                let source = self.get_node_by_id(&e.source).unwrap();
                let target = self.get_node_by_id(&e.target).unwrap();
                !(source.is_partition || target.is_partition)
            })
            .collect();

        // Sort edges by source and then by target
        edges.sort_by_key(|e| (&e.source, &e.target));
        edges
    }

    pub fn get_non_partition_nodes(&self) -> Vec<&Node> {
        let mut nodes: Vec<&Node> = self.nodes.iter().filter(|n| !n.is_partition).collect();

        // Sort nodes by id or another consistent attribute
        nodes.sort_by_key(|n| &n.id);
        nodes
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

    pub fn truncate_node_labels(&mut self, max_length: usize) {
        self.nodes.iter_mut().for_each(|n| {
            if n.label.len() > max_length {
                n.label = n.label[..max_length].to_string();
            }
        });
    }

    pub fn insert_newlines_in_node_labels(&mut self, max_length: usize) {
        self.nodes.iter_mut().for_each(|n| {
            if n.label.len() > max_length {
                let mut new_label = String::new();
                let mut current_length = 0;
                for word in n.label.split_whitespace() {
                    if current_length + word.len() > max_length {
                        new_label.push_str("\n");
                        current_length = 0;
                    }
                    new_label.push_str(word);
                    new_label.push_str(" ");
                    current_length += word.len() + 1;
                }
                n.label = new_label.trim().to_string();
            }
        });
    }

    pub fn truncate_edge_labels(&mut self, max_length: usize) {
        self.edges.iter_mut().for_each(|n| {
            if n.label.len() > max_length {
                n.label = n.label[..max_length].to_string();
            }
        });
    }

    pub fn insert_newlines_in_edge_labels(&mut self, max_length: usize) {
        self.edges.iter_mut().for_each(|n| {
            if n.label.len() > max_length {
                let mut new_label = String::new();
                let mut current_length = 0;
                for word in n.label.split_whitespace() {
                    if current_length + word.len() > max_length {
                        new_label.push_str("\n");
                        current_length = 0;
                    }
                    new_label.push_str(word);
                    new_label.push_str(" ");
                    current_length += word.len() + 1;
                }
                n.label = new_label.trim().to_string();
            }
        });
    }

    pub fn modify_graph_limit_partition_depth(&mut self, depth: i32) -> Result<(), String> {
        fn trim_node(node_id: &String, graph: &mut Graph, current_depth: i32, max_depth: i32) {
            let node = graph.get_node(node_id).unwrap();
            let children = graph.get_children(&node);

            let all_child_node_ids: Vec<String> = children.iter().map(|n| n.id.clone()).collect();

            debug!(
                "Trimming partition depth for node {} : current_depth: {} max_depth: {}",
                node_id, current_depth, max_depth
            );

            // Recursively process children first
            for child_id in &all_child_node_ids {
                trim_node(child_id, graph, current_depth + 1, max_depth);
            }

            if current_depth >= max_depth {
                let mut agg_node = {
                    let node = graph.get_node(node_id).unwrap();
                    let mut cloned_node = node.clone();
                    cloned_node.is_partition = false; // Ensure the aggregated node is non-partition
                    cloned_node
                };

                let mut new_edges = Vec::new();

                for edge in &graph.edges {
                    let source_exists = graph.get_node(&edge.source).is_some();
                    let target_exists = graph.get_node(&edge.target).is_some();

                    if source_exists && target_exists {
                        if all_child_node_ids.contains(&edge.source) {
                            new_edges.push(Edge {
                                source: agg_node.id.clone(),
                                target: edge.target.clone(),
                                ..edge.clone()
                            });
                        } else if all_child_node_ids.contains(&edge.target) {
                            new_edges.push(Edge {
                                source: edge.source.clone(),
                                target: agg_node.id.clone(),
                                ..edge.clone()
                            });
                        } else {
                            new_edges.push(edge.clone());
                        }
                    }
                }

                // Aggregate weights
                for child_id in &all_child_node_ids {
                    if let Some(child) = graph.get_node(child_id) {
                        agg_node.weight += child.weight;
                    } else {
                        error!("Child node not found: {}", child_id);
                    }
                }

                graph.edges = new_edges;

                // Remove child nodes after edge updates
                for node_id in all_child_node_ids {
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
            let node = {
                let node = graph.get_node(node_id).unwrap();
                node.clone()
            };

            let children = graph.get_children(&node);

            let non_partition_child_node_ids: Vec<String> = children
                .iter()
                .filter(|n| !n.is_partition)
                .map(|n| n.id.clone())
                .collect();

            let partition_child_node_ids: Vec<String> = children
                .iter()
                .filter(|n| n.is_partition)
                .map(|n| n.id.clone())
                .collect();

            let child_node_ids: Vec<String> = children.iter().map(|n| n.id.clone()).collect();

            debug!(
                "Trimming non-partition width for node: {} max_width: {}, children: {}, non_partition_children: {}, partition_children: {}",
                node_id,
                max_width,
                child_node_ids.len(),
                non_partition_child_node_ids.len(),
                partition_child_node_ids.len()
            );

            if partition_child_node_ids.len() as i32 > max_width {
                warn!(
                    "Partition children count for node {} is {} and exceeds max_width {} for non-partition nodes. This might not be the intended behavior",
                    node.id,
                    partition_child_node_ids.len(),
                    max_width
                );
                return;
            }

            // Recursively process partition children first
            for child_id in &partition_child_node_ids {
                debug!("Processing partition child: {} / {}", node.id, child_id);
                trim_node(child_id, graph, max_width);
            }

            if non_partition_child_node_ids.len() as i32 > max_width {
                debug!("\tChopping time in node: {}", node.id);

                let agg_node_id = format!("agg_{}", node.id.clone());

                // Make sure there is an aggregated layer
                if !graph.layer_exists("aggregated") {
                    warn!("Aggregating nodes, but a layer 'aggregated' does not exist. Creating one. Add this layer to your graph config if you want to style it");
                    graph.add_layer(Layer::new(
                        "aggregated",
                        "Aggregated",
                        "222222",
                        "ffffff",
                        "dddddd",
                    ));
                }

                let mut agg_node = {
                    Node {
                        id: agg_node_id.clone(),
                        label: format!("{} nodes (aggregated)", non_partition_child_node_ids.len()),
                        layer: "aggregated".to_string(),
                        is_partition: false,
                        belongs_to: Some(node.id),
                        weight: 0,
                        comment: node.comment.clone(),
                    }
                };

                let children: Vec<Node> = non_partition_child_node_ids
                    .iter()
                    .map(|id| graph.get_node(id).unwrap().clone())
                    .collect();

                // Remove children beyond max_width
                let mut new_edges = graph.edges.clone();

                for child in children.iter() {
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
                }

                graph.set_node(agg_node);
                graph.edges = new_edges;

                // Remove child nodes after edge updates
                for node_id in non_partition_child_node_ids {
                    debug!("\tRemoving node: {}", node_id);
                    graph.remove_node(node_id);
                }
            }
            debug!("Updated graph stats: {}", graph.stats());
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

    // Aggregate duplicate edges
    pub fn aggregate_edges(&mut self) {
        let mut edge_map: HashMap<String, Edge> = HashMap::new();
        for edge in &self.edges {
            let key = format!("{}_{}", edge.source, edge.target);
            if let Some(existing_edge) = edge_map.get_mut(&key) {
                existing_edge.weight += edge.weight;
            } else {
                edge_map.insert(key, edge.clone());
            }
        }
        debug!(
            "Aggregated {} edges to {}",
            self.edges.len(),
            edge_map.len()
        );
        self.edges = edge_map.values().cloned().collect();
    }

    pub fn invert_graph(&mut self) -> Graph {
        /*
         * Invert the graph
         * 1. Create a new graph
         * 2. For each edge (u,v) in the original graph G:
         * 3. Create a new node N(u,v) in G'
         * 4. For each original node x in G:
         * 5. Find all edges incident to x in G
         * 6. Create an edge in G' connecting the nodes that correspond to these original edges.
         */
        let mut inverted_graph = Graph {
            name: format!("Inverted {}", self.name),
            nodes: Vec::new(),
            edges: Vec::new(),
            layers: self.layers.clone(),
        };

        // Map to store new nodes created for each edge
        let mut edge_to_node_map = std::collections::HashMap::new();

        inverted_graph.nodes.push(Node {
            id: "inverted_root".to_string(),
            label: "Root".to_string(),
            layer: "inverted_root".to_string(),
            is_partition: true,
            belongs_to: None,
            weight: 0,
            comment: None,
        });

        fn edge_label(edge: &Edge) -> String {
            if edge.label.is_empty() {
                format!("{} -> {}", edge.source, edge.target)
            } else {
                edge.label.clone()
            }
        }

        // Step 2 & 3: Create a new node for each edge in the original graph
        let mut node_counter = 0; // Initialize a counter for unique IDs
        for edge in &self.edges {
            let new_node = Node {
                id: format!("n_{}_{}_{}", edge.source, edge.target, node_counter),
                is_partition: false,
                label: edge_label(&edge),
                layer: edge.layer.clone(),
                belongs_to: Some("inverted_root".to_string()),
                weight: edge.weight,
                comment: edge.comment.clone(),
            };
            inverted_graph.nodes.push(new_node.clone());
            edge_to_node_map.insert((edge.source.clone(), edge.target.clone()), new_node);
            node_counter += 1; // Increment the counter for the next node
        }

        // Step 4, 5 & 6: Create edges in the inverted graph
        let mut edge_counter = 0; // Initialize a counter for unique IDs
        for node in &self.nodes {
            // Find all edges incident to this node
            let incident_edges: Vec<&Edge> = self
                .edges
                .iter()
                .filter(|e| e.source == node.id || e.target == node.id)
                .collect();

            // Create edges in the inverted graph
            for i in 0..incident_edges.len() {
                for j in (i + 1)..incident_edges.len() {
                    let node1 = edge_to_node_map
                        .get(&(
                            incident_edges[i].source.clone(),
                            incident_edges[i].target.clone(),
                        ))
                        .unwrap();
                    let node2 = edge_to_node_map
                        .get(&(
                            incident_edges[j].source.clone(),
                            incident_edges[j].target.clone(),
                        ))
                        .unwrap();
                    inverted_graph.edges.push(Edge {
                        id: format!("{}_{}_{}", node1.id, node2.id, edge_counter),
                        source: node1.id.clone(),
                        target: node2.id.clone(),
                        label: "".to_string(),
                        layer: node.layer.clone(),
                        weight: 1,
                        comment: None,
                    });
                    edge_counter += 1; // Increment the counter for the next edge
                }
            }
        }

        let edge_layer_ids = inverted_graph
            .edges
            .iter()
            .map(|e| e.layer.clone())
            .collect::<HashSet<String>>();

        let node_layer_ids = inverted_graph
            .nodes
            .iter()
            .map(|e| e.layer.clone())
            .collect::<HashSet<String>>();

        let layer_ids = edge_layer_ids.union(&node_layer_ids);

        for layer_id in layer_ids {
            if !inverted_graph.layers.iter().any(|l| l.id == *layer_id) {
                warn!("Layer {} not found in inverted graph, adding a placeholder - please add one if you want to style it", layer_id);
                inverted_graph.add_layer(Layer::new(
                    layer_id.as_str(),
                    layer_id.as_str(),
                    "222222",
                    "ffffff",
                    "dddddd",
                ));
            }
        }

        inverted_graph
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
            debug!("All edges have valid source and target nodes");
        } else {
            warn!("Some edges have missing source and/or target nodes");
        }

        let partition_node_ids = self
            .nodes
            .iter()
            .filter(|n| n.is_partition)
            .map(|n| n.id.clone())
            .collect::<HashSet<String>>();

        let non_partition_node_ids = self
            .nodes
            .iter()
            .filter(|n| n.is_partition)
            .map(|n| n.id.clone())
            .collect::<HashSet<String>>();

        //
        // verify that partition nodes and non-partition nodes do not have edges between them

        self.edges.iter().for_each(|e| {
            if partition_node_ids.contains(&e.source) && non_partition_node_ids.contains(&e.target)
            {
                let err = format!(
                    "Edge id:[{}] source {:?} is a partition node and target {:?} is a non-partition node",
                    e.id, e.source, e.target
                );
                errors.push(err);
            }
            if partition_node_ids.contains(&e.target) && non_partition_node_ids.contains(&e.source)
            {
                let err = format!(
                    "Edge id:[{}] target {:?} is a partition node and source {:?} is a non-partition node",
                    e.id, e.target, e.source
                );
                errors.push(err);
            }
        });

        self.nodes.iter().for_each(|n| {
            if n.belongs_to.is_some() && !node_ids.contains(n.belongs_to.as_ref().unwrap()) {
                let err = format!(
                    "Node id:[{}] belongs_to {:?} not found in nodes",
                    n.id,
                    n.belongs_to.as_ref().unwrap()
                );
                errors.push(err);
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

        // verify that all nodes are assigned to a layer
        self.nodes.iter().for_each(|n| {
            if !self.layers.iter().any(|l| l.id == n.layer) {
                let err = format!("Node id:[{}] layer {:?} not found in layers", n.id, n.layer);
                errors.push(err);
            }
        });

        // TODO implement an info level log for the following
        // // verify that all edges are assigned to a layer
        // self.edges.iter().for_each(|e| {
        //     if !self.layers.iter().any(|l| l.id == e.layer) {
        //         let err = format!("Edge id:[{}] layer {:?} not found in layers", e.id, e.layer);
        //         errors.push(err);
        //     }
        // });

        // verify that all nodes have unique ids
        let mut node_id_set = HashSet::new();
        for node in &self.nodes {
            if node_id_set.contains(&node.id) {
                let err = format!("Duplicate node id: {}", node.id);
                errors.push(err);
            } else {
                node_id_set.insert(node.id.clone());
            }
        }

        // verify that all edges have unique ids
        let mut edge_id_set = HashSet::new();
        for edge in &self.edges {
            if edge_id_set.contains(&edge.id) {
                let err = format!("Duplicate edge id: {}", edge.id);
                errors.push(err);
            } else {
                edge_id_set.insert(edge.id.clone());
            }
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

impl Layer {
    pub fn new(
        id: &str,
        label: &str,
        background_color: &str,
        text_color: &str,
        border_color: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            background_color: background_color.to_string(),
            text_color: text_color.to_string(),
            border_color: border_color.to_string(),
        }
    }
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
            name: "Test Graph".to_string(),
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
            "depth": 0,
            "comment": null,
            "children": [
                {
                    "id": "2",
                    "label": "Child1",
                    "layer": "Layer1",
                    "is_partition": false,
                    "belongs_to": "1",
                    "depth": 1,
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
                    "depth": 1,
                    "comment": null,
                    "weight": 1,
                    "children": []
                }
            ]
        }]);
        assert_eq!(json_tree, expected_json);
    }
}
