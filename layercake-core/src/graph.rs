use csv::StringRecord;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info, warn};

use crate::data_loader::{DfEdgeLoadProfile, DfNodeLoadProfile};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Graph {
    #[serde(default)]
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
    pub annotations: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PartitionWidthAggregation {
    pub parent_id: String,
    pub parent_label: String,
    pub aggregate_node_id: String,
    pub aggregate_node_label: String,
    pub aggregated_nodes: Vec<(String, String)>,
    pub retained_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LayerAggregationSummary {
    pub layer_id: String,
    pub belongs_to: Option<String>,
    pub anchor_node_id: String,
    pub anchor_node_label: Option<String>,
    pub aggregate_node_id: String,
    pub aggregate_node_label: String,
    pub aggregated_nodes: Vec<(String, String)>,
}

impl Graph {
    fn sanitize_label_value(label: &str) -> String {
        let cleaned: String = label
            .chars()
            .filter_map(|c| {
                if matches!(c, '\n' | '\r' | '\t') {
                    Some(' ')
                } else if matches!(c, '"' | '\'' | '`' | '\\') {
                    Some(' ')
                } else if c.is_control() {
                    None
                } else {
                    Some(c)
                }
            })
            .collect();

        cleaned
            .split_whitespace()
            .filter(|chunk| !chunk.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Remove control/quote characters from node and edge labels, recording an annotation when changes occur.
    pub fn sanitize_labels(&mut self) -> (usize, usize) {
        let mut sanitized_nodes = 0;
        let mut sanitized_edges = 0;

        for node in &mut self.nodes {
            let cleaned = Self::sanitize_label_value(&node.label);
            if cleaned != node.label {
                node.label = cleaned;
                sanitized_nodes += 1;
            }
        }

        for edge in &mut self.edges {
            let cleaned = Self::sanitize_label_value(&edge.label);
            if cleaned != edge.label {
                edge.label = cleaned;
                sanitized_edges += 1;
            }
        }

        if sanitized_nodes > 0 || sanitized_edges > 0 {
            self.append_annotation(format!(
                "Sanitized labels: removed quotes/newlines/control characters from {} nodes and {} edges.",
                sanitized_nodes, sanitized_edges
            ));
        }

        (sanitized_nodes, sanitized_edges)
    }

    /// Coalesce function nodes into their owning file nodes and aggregate duplicate edges.
    /// File nodes become flow nodes (`is_partition = false`). Function nodes are removed
    /// once edges are rewired, and duplicate edges between the same source/target/layer
    /// are merged with summed weights and merged labels. Only functions with a resolvable
    /// file match are coalesced; unmatched functions remain.
    pub fn coalesce_functions_to_files(&mut self) -> Option<String> {
        let mut file_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut function_to_file: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut scope_root: Option<String> = None;

        for node in &self.nodes {
            if node.layer == "scope" && node.belongs_to.is_none() {
                scope_root = Some(node.id.clone());
            }
        }

        // Potential file nodes (scope nodes that look like files)
        let file_candidates: Vec<&Node> = self
            .nodes
            .iter()
            .filter(|n| {
                n.layer == "scope"
                    && (n.label.contains('.')
                        || n.comment
                            .as_deref()
                            .map(|c| c.contains('.'))
                            .unwrap_or(false))
            })
            .collect();

        let mut resolve_file = |hints: &[String]| -> Option<String> {
            for candidate in &file_candidates {
                for hint in hints {
                    if candidate.id == *hint
                        || candidate.label.ends_with(hint)
                        || candidate
                            .comment
                            .as_deref()
                            .map(|c| c.ends_with(hint))
                            .unwrap_or(false)
                        || std::path::Path::new(&candidate.label)
                            .file_name()
                            .map(|f| f.to_string_lossy() == hint.as_str())
                            .unwrap_or(false)
                    {
                        return Some(candidate.id.clone());
                    }
                }
            }
            None
        };

        let mut unmatched_functions = 0usize;

        for node in &self.nodes {
            if node.layer != "function" {
                continue;
            }

            let mut hints = Vec::new();
            if let Some(belongs) = node.belongs_to.clone() {
                hints.push(belongs);
            }
            if let Some(comment) = node.comment.clone() {
                hints.push(comment);
            }
            if let Some(attrs) = &node.attributes {
                if let Some(file) = attrs.get("file").and_then(|v| v.as_str()) {
                    hints.push(file.to_string());
                }
                if let Some(file) = attrs.get("file_path").and_then(|v| v.as_str()) {
                    hints.push(file.to_string());
                }
            }

            let file_id = resolve_file(&hints).or_else(|| scope_root.clone());
            if let Some(file_id) = file_id {
                function_to_file.insert(node.id.clone(), file_id);
            } else {
                unmatched_functions += 1;
            }
        }

        if function_to_file.is_empty() {
            return None;
        }

        for node in &mut self.nodes {
            if function_to_file.values().any(|f| f == &node.id) {
                file_ids.insert(node.id.clone());
                node.is_partition = false;
            }
        }

        let mut aggregated: indexmap::IndexMap<(String, String, String), (Edge, i32, IndexSet<String>)> =
            indexmap::IndexMap::new();

        for edge in self.edges.iter() {
            let mut new_edge = edge.clone();
            if let Some(file) = function_to_file.get(&edge.source) {
                new_edge.source = file.clone();
            }
            if let Some(file) = function_to_file.get(&edge.target) {
                new_edge.target = file.clone();
            }

            let weight = std::cmp::max(1, new_edge.weight);
            let key = (new_edge.source.clone(), new_edge.target.clone(), new_edge.layer.clone());
            let labels = new_edge
                .label
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            aggregated
                .entry(key)
                .and_modify(|(edge_ref, total, seen_labels)| {
                    *total += weight;
                    for lbl in &labels {
                        seen_labels.insert(lbl.clone());
                    }
                    if edge_ref.comment.is_none() {
                        edge_ref.comment = new_edge.comment.clone();
                    }
                    if edge_ref.dataset.is_none() {
                        edge_ref.dataset = new_edge.dataset;
                    }
                })
                .or_insert_with(|| {
                    let mut label_set = indexmap::IndexSet::new();
                    for lbl in labels {
                        label_set.insert(lbl);
                    }
                    (new_edge.clone(), weight, label_set)
                });
        }

        let mut next_id: usize = 1;
        self.edges = aggregated
            .into_iter()
            .map(|((_s, _t, _l), (mut edge, total_weight, labels))| {
                edge.id = format!("edge_coalesced_{next_id}");
                next_id += 1;
                edge.weight = total_weight;
                if !labels.is_empty() {
                    edge.label = labels.into_iter().collect::<Vec<_>>().join(", ");
                } else {
                    edge.label = edge.label.clone();
                }
                edge
            })
            .collect();

        let before_nodes = self.nodes.len();
        self.nodes.retain(|n| {
            if n.layer == "function" && function_to_file.contains_key(&n.id) {
                return false;
            }
            true
        });
        let removed_nodes = before_nodes.saturating_sub(self.nodes.len());

        let annotation = format!(
            "Coalesced functions into files: {} function nodes removed ({} unmatched kept); {} edges aggregated.",
            removed_nodes,
            unmatched_functions,
            next_id.saturating_sub(1)
        );
        self.append_annotation(annotation.clone());
        Some(annotation)
    }

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

    pub fn append_annotation(&mut self, annotation: impl AsRef<str>) {
        let text = annotation.as_ref().trim();
        if text.is_empty() {
            return;
        }

        match &mut self.annotations {
            Some(existing) if !existing.is_empty() => {
                existing.push_str("\n\n");
                existing.push_str(text);
            }
            Some(existing) => {
                existing.push_str(text);
            }
            None => {
                self.annotations = Some(text.to_string());
            }
        }
    }

    fn generate_aggregate_node_id(&self, parent_id: &str) -> String {
        let mut counter = 1;
        loop {
            let candidate = format!("agg_{}_{}", parent_id, counter);
            if self.get_node_by_id(&candidate).is_none() {
                return candidate;
            }
            counter += 1;
        }
    }

    pub fn get_root_nodes(&self) -> Vec<&Node> {
        let mut nodes: Vec<&Node> = self
            .nodes
            .iter()
            .filter(|n| {
                let belongs_to = n.belongs_to.as_deref();
                n.is_partition
                    && (belongs_to.is_none() || belongs_to == Some("synthetic_partition_root"))
            })
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

    #[allow(dead_code)] // Reserved for future hierarchy analysis
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

        // For compatibility with test expectations, ensure comments are "null" strings if empty
        for node in &mut nodes {
            if node.comment.is_none() || node.comment.as_ref().is_none_or(|s| s.is_empty()) {
                node.comment = Some("null".to_string());
            }
        }

        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        nodes
    }

    pub fn get_hierarchy_edges(&self) -> Vec<Edge> {
        let mut edges = Vec::new();
        self.nodes.iter().for_each(|node| {
            if let Some(parent_id) = &node.belongs_to {
                if let Some(parent) = self.get_node_by_id(parent_id) {
                    edges.push(Edge {
                        id: format!("{}_{}", parent.id, node.id),
                        source: parent.id.clone(),
                        target: node.id.clone(),
                        label: "".to_string(), // format!("{} -> {}", parent.label, node.label),
                        layer: parent.layer.clone(),
                        weight: 1,
                        comment: None,
                        dataset: None,
                        attributes: None,
                    });
                }
                // If parent node not found, skip this edge - could log warning if needed
            }
        });

        edges
    }

    pub fn get_non_partition_edges(&self) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .edges
            .iter()
            .filter(|e| {
                if let (Some(source), Some(target)) = (
                    self.get_node_by_id(&e.source),
                    self.get_node_by_id(&e.target),
                ) {
                    !(source.is_partition || target.is_partition)
                } else {
                    false // Skip edges with missing nodes
                }
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

            // For compatibility with test expectations, ensure comments are "null" strings if empty
            if tree_node.comment.is_none()
                || tree_node.comment.as_ref().is_none_or(|s| s.is_empty())
            {
                tree_node.comment = Some("null".to_string());
            }

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

    /// Build a hierarchy tree based on edges rather than `belongs_to` metadata.
    /// This is useful after GenerateHierarchy rewires structure into edges.
    pub fn build_tree_from_edges(&self) -> Vec<TreeNode> {
        use std::collections::{HashMap, HashSet};

        if self.edges.is_empty() {
            return Vec::new();
        }

        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut has_parent: HashSet<String> = HashSet::new();
        let mut referenced: HashSet<String> = HashSet::new();

        for edge in &self.edges {
            if self.get_node_by_id(&edge.source).is_none()
                || self.get_node_by_id(&edge.target).is_none()
            {
                continue;
            }
            children_map
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            has_parent.insert(edge.target.clone());
            referenced.insert(edge.source.clone());
            referenced.insert(edge.target.clone());
        }

        if children_map.is_empty() {
            return Vec::new();
        }

        for child_ids in children_map.values_mut() {
            child_ids.sort();
            child_ids.dedup();
        }

        let hierarchy_root_ids: HashSet<String> = self
            .nodes
            .iter()
            .filter(|node| {
                node.is_partition
                    && node.label == "Hierarchy"
                    && node.belongs_to.as_deref() == Some("")
            })
            .map(|node| node.id.clone())
            .collect();

        let mut root_ids: Vec<String> = referenced
            .into_iter()
            .filter(|node_id| {
                !hierarchy_root_ids.contains(node_id) && !has_parent.contains(node_id)
            })
            .collect();

        if root_ids.is_empty() {
            root_ids = children_map
                .keys()
                .filter(|node_id| !hierarchy_root_ids.contains(*node_id))
                .cloned()
                .collect();
        }

        root_ids.sort();
        root_ids.dedup();

        let mut visited = HashSet::new();
        let mut result = Vec::new();
        for root_id in root_ids {
            if let Some(node) =
                self.build_subtree_from_edges(&root_id, 0, &children_map, &mut visited)
            {
                result.push(node);
            }
        }
        result
    }

    fn build_subtree_from_edges(
        &self,
        node_id: &str,
        depth: i32,
        children_map: &std::collections::HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Option<TreeNode> {
        if !visited.insert(node_id.to_string()) {
            return None;
        }

        let node = self.get_node_by_id(node_id)?;
        let mut tree_node = TreeNode::from_node(node);
        tree_node.depth = depth;

        if tree_node.comment.is_none() || tree_node.comment.as_ref().is_none_or(|s| s.is_empty()) {
            tree_node.comment = Some("null".to_string());
        }

        if let Some(child_ids) = children_map.get(node_id) {
            let mut children = Vec::new();
            for child_id in child_ids {
                if let Some(child_node) =
                    self.build_subtree_from_edges(child_id, depth + 1, children_map, visited)
                {
                    children.push(child_node);
                }
            }
            tree_node.children = children;
        }

        visited.remove(node_id);
        Some(tree_node)
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

    fn replace_with_aggregate_node(
        &mut self,
        aggregated_ids: &[String],
        label: String,
        layer: String,
        belongs_to: Option<String>,
        comment: Option<String>,
    ) -> Option<(Node, Vec<(String, String)>)> {
        if aggregated_ids.is_empty() {
            return None;
        }

        let aggregated_children: Vec<Node> = aggregated_ids
            .iter()
            .filter_map(|id| self.get_node_by_id(id).cloned())
            .collect();

        if aggregated_children.is_empty() {
            return None;
        }

        let id_seed = belongs_to
            .as_deref()
            .map(|value| value.to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("layer_{}", layer));

        let aggregate_node_id = self.generate_aggregate_node_id(&id_seed);
        let aggregate_node = Node {
            id: aggregate_node_id.clone(),
            label,
            layer: layer.clone(),
            is_partition: false,
            belongs_to: belongs_to.clone(),
            weight: aggregated_children.iter().map(|child| child.weight).sum(),
            comment,
            dataset: None,
            attributes: None,
        };

        let aggregated_child_ids: HashSet<String> = aggregated_ids.iter().cloned().collect();
        let mut untouched_edges = Vec::new();
        let mut aggregated_edge_map: HashMap<(String, String, String, Option<i32>), Edge> =
            HashMap::new();

        for edge in self.edges.iter() {
            let mut new_edge = edge.clone();
            let source_replaced = aggregated_child_ids.contains(&edge.source);
            let target_replaced = aggregated_child_ids.contains(&edge.target);

            if source_replaced {
                new_edge.source = aggregate_node.id.clone();
            }
            if target_replaced {
                new_edge.target = aggregate_node.id.clone();
            }

            if new_edge.source == new_edge.target {
                continue;
            }

            if source_replaced || target_replaced {
                let key = (
                    new_edge.source.clone(),
                    new_edge.target.clone(),
                    new_edge.layer.clone(),
                    new_edge.dataset,
                );

                aggregated_edge_map
                    .entry(key)
                    .and_modify(|existing| existing.weight += new_edge.weight)
                    .or_insert(new_edge);
            } else {
                untouched_edges.push(new_edge);
            }
        }

        untouched_edges.extend(aggregated_edge_map.into_values());
        self.edges = untouched_edges;

        self.set_node(aggregate_node.clone());

        for node_id in aggregated_ids {
            self.remove_node(node_id.to_string());
        }

        let aggregated_pairs = aggregated_children
            .into_iter()
            .map(|child| (child.id, child.label))
            .collect();

        Some((aggregate_node, aggregated_pairs))
    }

    // This function is now an alias to get_node_by_id, defined above

    pub fn stats(&self) -> String {
        format!(
            "Nodes: {}, Edges: {}, Layers: {}",
            self.nodes.len(),
            self.edges.len(),
            self.layers.len()
        )
    }

    // Helper function to truncate text to a maximum length
    fn truncate_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }
        text[..max_length].to_string()
    }

    pub fn truncate_node_labels(&mut self, max_length: usize) {
        self.nodes.iter_mut().for_each(|n| {
            n.label = Self::truncate_text(&n.label, max_length);
        });
    }

    /// Remove nodes that are not referenced by any edge.
    /// When `exclude_partition_nodes` is true, partition nodes are retained even if unconnected.
    /// Returns the number of nodes removed.
    pub fn drop_unconnected_nodes(&mut self, exclude_partition_nodes: bool) -> usize {
        let connected_ids: HashSet<String> = self
            .edges
            .iter()
            .flat_map(|e| [e.source.clone(), e.target.clone()])
            .collect();

        let mut protected_ids: HashSet<String> = HashSet::new();
        if exclude_partition_nodes {
            for node in &self.nodes {
                if node.is_partition {
                    protected_ids.insert(node.id.clone());
                }
            }
            for node in &self.nodes {
                if let Some(parent) = &node.belongs_to {
                    protected_ids.insert(parent.clone());
                }
            }
        }

        let original_len = self.nodes.len();
        self.nodes.retain(|n| {
            connected_ids.contains(&n.id)
                || (exclude_partition_nodes && protected_ids.contains(&n.id))
        });

        let valid_ids: HashSet<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        self.edges
            .retain(|e| valid_ids.contains(&e.source) && valid_ids.contains(&e.target));

        original_len.saturating_sub(self.nodes.len())
    }

    // Helper function to insert newlines in a string at appropriate word boundaries
    fn insert_newlines_in_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }

        let mut new_text = String::new();
        let mut current_length = 0;
        for word in text.split_whitespace() {
            if current_length + word.len() > max_length {
                new_text.push('\n');
                current_length = 0;
            }
            new_text.push_str(word);
            new_text.push(' ');
            current_length += word.len() + 1;
        }
        new_text.trim().to_string()
    }

    pub fn insert_newlines_in_node_labels(&mut self, max_length: usize) {
        self.nodes.iter_mut().for_each(|n| {
            n.label = Self::insert_newlines_in_text(&n.label, max_length);
        });
    }

    pub fn truncate_edge_labels(&mut self, max_length: usize) {
        self.edges.iter_mut().for_each(|n| {
            n.label = Self::truncate_text(&n.label, max_length);
        });
    }

    pub fn insert_newlines_in_edge_labels(&mut self, max_length: usize) {
        self.edges.iter_mut().for_each(|n| {
            n.label = Self::insert_newlines_in_text(&n.label, max_length);
        });
    }

    pub fn modify_graph_limit_partition_depth(&mut self, depth: i32) -> Result<(), String> {
        let mut synthesized = false;
        fn trim_node(
            node_id: &String,
            graph: &mut Graph,
            current_depth: i32,
            max_depth: i32,
        ) -> Result<(), String> {
            let node = graph
                .get_node_by_id(node_id)
                .ok_or_else(|| format!("Node with id '{}' not found", node_id))?;
            let children = graph.get_children(node);

            let all_child_node_ids: Vec<String> = children.iter().map(|n| n.id.clone()).collect();

            debug!(
                "Trimming partition depth for node {} : current_depth: {} max_depth: {}",
                node_id, current_depth, max_depth
            );

            // Recursively process children first
            for child_id in &all_child_node_ids {
                trim_node(child_id, graph, current_depth + 1, max_depth)?;
            }

            if current_depth >= max_depth {
                let mut agg_node = {
                    let node = graph
                        .get_node_by_id(node_id)
                        .ok_or_else(|| format!("Node with id '{}' not found", node_id))?;
                    let mut cloned_node = node.clone();
                    cloned_node.is_partition = false; // Ensure the aggregated node is non-partition
                    cloned_node
                };

                let mut new_edges = Vec::new();

                for edge in &graph.edges {
                    let source_exists = graph.get_node_by_id(&edge.source).is_some();
                    let target_exists = graph.get_node_by_id(&edge.target).is_some();

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
                    if let Some(child) = graph.get_node_by_id(child_id) {
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
            Ok(())
        }

        let root_node_ids: Vec<String> = {
            let mut roots: Vec<String> = self
                .get_root_nodes()
                .iter()
                .map(|node| node.id.clone())
                .collect();
            if roots.is_empty() {
                synthesized = self.ensure_partition_hierarchy();
                roots = self
                    .get_root_nodes()
                    .iter()
                    .map(|node| node.id.clone())
                    .collect();
            }
            roots
        };

        for node_id in &root_node_ids {
            trim_node(node_id, self, 0, depth)?;
        }

        if synthesized {
            info!("Synthetic partition hierarchy used for depth limiting");
        }
        Ok(())
    }

    pub fn modify_graph_limit_partition_width(
        &mut self,
        max_width: i32,
    ) -> Result<Vec<PartitionWidthAggregation>, String> {
        fn trim_node(
            node_id: &String,
            graph: &mut Graph,
            max_width: i32,
            summaries: &mut Vec<PartitionWidthAggregation>,
        ) -> Result<(), String> {
            let node = {
                let node = graph
                    .get_node_by_id(node_id)
                    .ok_or_else(|| format!("Node with id '{}' not found", node_id))?;
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

            let width_threshold = if max_width <= 1 { 2 } else { max_width };

            // Recursively process partition children first
            for child_id in &partition_child_node_ids {
                debug!("Processing partition child: {} / {}", node.id, child_id);
                trim_node(child_id, graph, max_width, summaries)?;
            }

            if non_partition_child_node_ids.len() as i32 > width_threshold {
                debug!("\tChopping time in node: {}", node.id);

                let retain_count = if max_width > 1 {
                    (max_width - 1) as usize
                } else {
                    0
                }
                .min(non_partition_child_node_ids.len());
                let aggregate_ids: Vec<String> = non_partition_child_node_ids
                    .iter()
                    .skip(retain_count)
                    .cloned()
                    .collect();

                if aggregate_ids.is_empty() {
                    return Ok(());
                }

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

                let parent_label = if !node.label.is_empty() {
                    node.label.clone()
                } else {
                    node.id.clone()
                };
                let aggregate_label = format!(
                    "{} - {} nodes (aggregated)",
                    parent_label,
                    aggregate_ids.len()
                );

                let (agg_node, aggregated_pairs) = match graph.replace_with_aggregate_node(
                    &aggregate_ids,
                    aggregate_label,
                    "aggregated".to_string(),
                    Some(node.id.clone()),
                    node.comment.clone(),
                ) {
                    Some(value) => value,
                    None => return Ok(()),
                };

                summaries.push(PartitionWidthAggregation {
                    parent_id: node.id.clone(),
                    parent_label: node.label.clone(),
                    aggregate_node_id: agg_node.id.clone(),
                    aggregate_node_label: agg_node.label.clone(),
                    aggregated_nodes: aggregated_pairs,
                    retained_count: retain_count,
                });
            }
            debug!("Updated graph stats: {}", graph.stats());
            Ok(())
        }

        let mut synthesized = false;
        let root_node_ids: Vec<String> = {
            let mut roots: Vec<String> = self
                .get_root_nodes()
                .iter()
                .map(|node| node.id.clone())
                .collect();
            if roots.is_empty() {
                synthesized = self.ensure_partition_hierarchy();
                roots = self
                    .get_root_nodes()
                    .iter()
                    .map(|node| node.id.clone())
                    .collect();
            }
            roots
        };

        let mut summaries = Vec::new();
        for node_id in &root_node_ids {
            trim_node(node_id, self, max_width, &mut summaries)?;
        }

        if synthesized {
            info!("Synthetic partition hierarchy used for width limiting");
        }
        Ok(summaries)
    }

    pub fn aggregate_nodes_by_layer(
        &mut self,
        min_shared_neighbors: usize,
    ) -> Result<Vec<LayerAggregationSummary>, String> {
        if min_shared_neighbors == 0 {
            return Err("layer aggregation requires at least one shared connection".to_string());
        }

        let mut summaries = Vec::new();
        loop {
            if let Some(summary) = self.aggregate_nodes_by_layer_once(min_shared_neighbors)? {
                summaries.push(summary);
            } else {
                break;
            }
        }
        Ok(summaries)
    }

    fn aggregate_nodes_by_layer_once(
        &mut self,
        min_shared_neighbors: usize,
    ) -> Result<Option<LayerAggregationSummary>, String> {
        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();
        for edge in &self.edges {
            adjacency
                .entry(edge.source.clone())
                .or_default()
                .insert(edge.target.clone());
            adjacency
                .entry(edge.target.clone())
                .or_default()
                .insert(edge.source.clone());
        }

        let node_lookup: HashMap<String, Node> = self
            .nodes
            .iter()
            .cloned()
            .map(|node| (node.id.clone(), node))
            .collect();

        let mut groups: HashMap<(Option<String>, String), Vec<String>> = HashMap::new();
        for node in &self.nodes {
            if node.is_partition {
                continue;
            }
            if node.layer.trim().is_empty() {
                continue;
            }
            let key = (node.belongs_to.clone(), node.layer.clone());
            groups.entry(key).or_default().push(node.id.clone());
        }

        for ((belongs_to, layer_id), node_ids) in groups.into_iter() {
            if node_ids.len() < min_shared_neighbors {
                continue;
            }

            let group_set: HashSet<String> = node_ids.iter().cloned().collect();
            let mut neighbor_map: HashMap<String, HashSet<String>> = HashMap::new();

            for node_id in &node_ids {
                if let Some(neighbors) = adjacency.get(node_id) {
                    for neighbor_id in neighbors {
                        if group_set.contains(neighbor_id) {
                            continue;
                        }
                        if let Some(neighbor_node) = node_lookup.get(neighbor_id) {
                            if neighbor_node.layer == layer_id {
                                continue;
                            }
                            neighbor_map
                                .entry(neighbor_id.clone())
                                .or_default()
                                .insert(node_id.clone());
                        }
                    }
                }
            }

            if neighbor_map.is_empty() {
                continue;
            }

            let mut neighbor_entries: Vec<(String, HashSet<String>)> =
                neighbor_map.into_iter().collect();
            neighbor_entries.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

            for (anchor_id, members) in neighbor_entries {
                if members.len() < min_shared_neighbors {
                    continue;
                }

                let aggregate_ids: Vec<String> = members.into_iter().collect();
                let aggregate_label = format!("{} agg({})", layer_id, aggregate_ids.len());

                let (aggregate_node, aggregated_pairs) = match self.replace_with_aggregate_node(
                    &aggregate_ids,
                    aggregate_label,
                    layer_id.clone(),
                    belongs_to.clone(),
                    None,
                ) {
                    Some(value) => value,
                    None => continue,
                };

                let anchor_label = node_lookup.get(&anchor_id).map(|node| node.label.clone());

                return Ok(Some(LayerAggregationSummary {
                    layer_id,
                    belongs_to,
                    anchor_node_id: anchor_id,
                    anchor_node_label: anchor_label,
                    aggregate_node_id: aggregate_node.id,
                    aggregate_node_label: aggregate_node.label,
                    aggregated_nodes: aggregated_pairs,
                }));
            }
        }

        Ok(None)
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

    /// Convert the implicit `belongs_to` hierarchy into explicit edges and
    /// attach every node to a newly-created hierarchy root node.
    pub fn generate_hierarchy(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        let snapshot = self.nodes.clone();
        let snapshot_map: HashMap<String, Node> = snapshot
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect();

        let existing_ids: HashSet<String> = snapshot.iter().map(|node| node.id.clone()).collect();
        let hierarchy_node_id = if !existing_ids.contains("hierarchy") {
            "hierarchy".to_string()
        } else {
            let mut counter = 1usize;
            loop {
                let candidate = format!("hierarchy_{}", counter);
                if !existing_ids.contains(&candidate) {
                    break candidate;
                }
                counter += 1;
            }
        };

        let hierarchy_layer_id = "hierarchy".to_string();
        if !self.layer_exists(&hierarchy_layer_id) {
            self.add_layer(Layer::new(
                &hierarchy_layer_id,
                "Hierarchy",
                "1f2933",
                "f8fafc",
                "94a3b8",
            ));
        }

        self.edges.clear();
        let mut edge_counter = 0usize;

        for node in &snapshot {
            if let Some(parent_id) = node.belongs_to.as_ref().filter(|parent| !parent.is_empty()) {
                if let Some(parent_node) = snapshot_map.get(parent_id) {
                    edge_counter += 1;
                    let edge_id = format!("hierarchy_edge_{}_{}", edge_counter, node.id);
                    let edge_layer = parent_node.layer.clone();
                    self.edges.push(Edge {
                        id: edge_id,
                        source: parent_id.clone(),
                        target: node.id.clone(),
                        label: String::new(),
                        layer: edge_layer,
                        weight: 1,
                        comment: None,
                        dataset: None,
                        attributes: None,
                    });
                }
            }
        }

        let hierarchy_node = Node {
            id: hierarchy_node_id.clone(),
            label: "Hierarchy".to_string(),
            layer: hierarchy_layer_id,
            is_partition: true,
            belongs_to: Some(String::new()),
            weight: 0,
            comment: None,
            dataset: None,
            attributes: None,
        };

        // Update existing nodes to belong to the new hierarchy node and drop partition flags
        for node in self.nodes.iter_mut() {
            node.belongs_to = Some(hierarchy_node_id.clone());
            node.is_partition = false;
        }

        self.nodes.push(hierarchy_node);
    }

    /// Ensure the graph has partition metadata. When none exists, synthesize a
    /// shallow hierarchy rooted at a synthetic partition node so downstream
    /// transforms (depth/width limits) can operate.
    pub fn ensure_partition_hierarchy(&mut self) -> bool {
        if self.nodes.iter().any(|n| n.is_partition) {
            return false;
        }

        let root_id = "synthetic_partition_root".to_string();
        let mut child_counts: HashMap<String, usize> = HashMap::new();
        let mut parents_by_child: HashMap<String, String> = HashMap::new();
        for edge in &self.edges {
            *child_counts.entry(edge.source.clone()).or_insert(0) += 1;
            parents_by_child
                .entry(edge.target.clone())
                .or_insert_with(|| edge.source.clone());
        }

        for node in &mut self.nodes {
            if child_counts.contains_key(&node.id) {
                node.is_partition = true;
            }
        }

        if !self.layer_exists("aggregated") {
            self.add_layer(Layer::new(
                "aggregated",
                "Aggregated",
                "222222",
                "ffffff",
                "dddddd",
            ));
        }

        if !self.nodes.iter().any(|n| n.id == root_id) {
            self.nodes.push(Node {
                id: root_id.clone(),
                label: "Synthetic Root".to_string(),
                layer: "aggregated".to_string(),
                is_partition: true,
                belongs_to: None,
                weight: 0,
                comment: Some("auto-generated".to_string()),
                dataset: None,
                attributes: None,
            });
        }

        for node in &mut self.nodes {
            if node.id == root_id || node.belongs_to.is_some() {
                continue;
            }

            if let Some(parent) = parents_by_child.get(&node.id) {
                node.belongs_to = Some(parent.clone());
            } else {
                node.belongs_to = Some(root_id.clone());
            }
        }

        true
    }

    /// Remove nodes that have no edges connected to them (no incoming or outgoing edges)
    pub fn remove_unconnected_nodes(&mut self) {
        use std::collections::HashSet;

        // Collect all node IDs that are referenced in edges
        let mut connected_nodes = HashSet::new();
        for edge in &self.edges {
            connected_nodes.insert(edge.source.clone());
            connected_nodes.insert(edge.target.clone());
        }

        let original_count = self.nodes.len();
        self.nodes.retain(|node| connected_nodes.contains(&node.id));
        let removed_count = original_count - self.nodes.len();

        if removed_count > 0 {
            debug!("Removed {} unconnected nodes", removed_count);
        }
    }

    /// Remove edges where either the source or target node doesn't exist
    pub fn remove_dangling_edges(&mut self) {
        use std::collections::HashSet;

        // Collect all valid node IDs
        let valid_node_ids: HashSet<String> =
            self.nodes.iter().map(|node| node.id.clone()).collect();

        let original_count = self.edges.len();
        self.edges.retain(|edge| {
            valid_node_ids.contains(&edge.source) && valid_node_ids.contains(&edge.target)
        });
        let removed_count = original_count - self.edges.len();

        if removed_count > 0 {
            debug!("Removed {} dangling edges", removed_count);
        }
    }

    pub fn invert_graph(&mut self) -> Result<Graph, String> {
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
            annotations: self.annotations.clone(),
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
            dataset: None,
            attributes: None,
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
                label: edge_label(edge),
                layer: edge.layer.clone(),
                belongs_to: Some("inverted_root".to_string()),
                weight: edge.weight,
                comment: edge.comment.clone(),
                dataset: None,
                attributes: edge.attributes.clone(),
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
                        .ok_or_else(|| {
                            format!(
                                "Failed to find node in edge mapping for edge {} -> {}",
                                incident_edges[i].source, incident_edges[i].target
                            )
                        })?;
                    let node2 = edge_to_node_map
                        .get(&(
                            incident_edges[j].source.clone(),
                            incident_edges[j].target.clone(),
                        ))
                        .ok_or_else(|| {
                            format!(
                                "Failed to find node in edge mapping for edge {} -> {}",
                                incident_edges[j].source, incident_edges[j].target
                            )
                        })?;
                    inverted_graph.edges.push(Edge {
                        id: format!("{}_{}_{}", node1.id, node2.id, edge_counter),
                        source: node1.id.clone(),
                        target: node2.id.clone(),
                        label: "".to_string(),
                        layer: node.layer.clone(),
                        weight: 1,
                        comment: None,
                        dataset: None,
                        attributes: None,
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

        Ok(inverted_graph)
    }

    #[allow(dead_code)]
    pub fn build_json_tree(&self) -> serde_json::Value {
        let tree = self.build_tree();
        serde_json::json!(tree)
    }

    pub fn verify_graph_integrity(&self) -> Result<(), Vec<String>> {
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
            .filter(|n| !n.is_partition)
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
            if let Some(belongs_to) = &n.belongs_to {
                if !node_ids.contains(belongs_to) {
                    let err = format!(
                        "Node id:[{}] belongs_to {:?} not found in nodes",
                        n.id, belongs_to
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

        // verify that all nodes are assigned to a layer
        self.nodes.iter().for_each(|n| {
            if !self.layers.iter().any(|l| l.id == n.layer) {
                let err = format!("Node id:[{}] layer {:?} not found in layers", n.id, n.layer);
                errors.push(err);
            }
        });

        // verify that all edges are assigned to a layer (info level - not critical)
        self.edges.iter().for_each(|e| {
            if !self.layers.iter().any(|l| l.id == e.layer) {
                tracing::info!("Edge id:[{}] layer {:?} not found in layers - this is not critical but may affect styling", e.id, e.layer);
            }
        });

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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub is_partition: bool,
    pub belongs_to: Option<String>,
    pub weight: i32,
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
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
            dataset: node.dataset,
            attributes: node.attributes.clone(),
            children: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub weight: i32,
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Layer {
    pub id: String,
    pub label: String,
    #[serde(default = "Layer::default_background")]
    pub background_color: String,
    #[serde(default = "Layer::default_text_color")]
    pub text_color: String,
    #[serde(default = "Layer::default_border")]
    pub border_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
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
            alias: None,
            dataset: None,
            attributes: None,
        }
    }

    fn default_background() -> String {
        "#ffffff".to_string()
    }

    fn default_text_color() -> String {
        "#000000".to_string()
    }

    fn default_border() -> String {
        "#000000".to_string()
    }
}

fn is_truthy(s: &str) -> bool {
    let trimmed_lowercase = s.trim().to_lowercase();
    matches!(trimmed_lowercase.as_str(), "true" | "y" | "yes")
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
    record: &StringRecord,
    idx: usize,
    label: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let value = record.get(idx).ok_or(format!("Missing {}", label))?;
    Ok(strip_quotes_and_whitespace(value).to_string())
}

impl Node {
    pub fn from_row(
        record: &StringRecord,
        node_profile: &DfNodeLoadProfile,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Node {
            id: get_stripped_value(record, node_profile.id_column, "id")
                .unwrap_or("noId".to_string()),
            label: get_stripped_value(record, node_profile.label_column, "label")?,
            layer: get_stripped_value(record, node_profile.layer_column, "layer")?,
            is_partition: is_truthy(&get_stripped_value(
                record,
                node_profile.is_partition_column,
                "is_partition",
            )?),
            belongs_to: {
                let belongs_to =
                    get_stripped_value(record, node_profile.belongs_to_column, "belongs_to")?;
                if belongs_to.is_empty() {
                    None
                } else if belongs_to.to_lowercase() == "null" {
                    None
                } else {
                    Some(belongs_to)
                }
            },
            weight: get_stripped_value(record, node_profile.weight_column, "weight")
                .and_then(|c| {
                    c.parse::<i32>()
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                })
                .unwrap_or(1),
            comment: {
                // For compatibility with test expectations, empty comments should be "null"
                let comment = record
                    .get(node_profile.comment_column)
                    .map(|c| c.to_string());

                if comment.is_none() || comment.as_ref().is_none_or(|s| s.is_empty()) {
                    Some("null".to_string())
                } else {
                    comment
                }
            },
            dataset: None,
            attributes: None,
        })
    }
}

impl Edge {
    pub fn from_row(
        record: &StringRecord,
        edge_profile: &DfEdgeLoadProfile,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Edge {
            id: get_stripped_value(record, edge_profile.id_column, "id")?, // default to noId
            source: get_stripped_value(record, edge_profile.source_column, "source")?,
            target: get_stripped_value(record, edge_profile.target_column, "target")?,
            label: get_stripped_value(record, edge_profile.label_column, "label")?,
            layer: get_stripped_value(record, edge_profile.layer_column, "layer")?,
            weight: get_stripped_value(record, edge_profile.weight_column, "weight")
                .and_then(|c| {
                    c.parse::<i32>()
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                })
                .unwrap_or(1),
            comment: {
                // For compatibility with test expectations, need proper quoting for edge comments
                let comment = record
                    .get(edge_profile.comment_column)
                    .map(|c| c.to_string());

                if comment.is_none() {
                    Some("null".to_string())
                } else if comment.as_ref().is_none_or(|s| s.is_empty()) {
                    Some("null".to_string())
                } else {
                    comment.map(|c| format!("\"{}\"", c))
                }
            },
            dataset: None,
            attributes: None,
        })
    }
}

impl Layer {
    pub fn from_row(record: &StringRecord) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            id: get_stripped_value(record, 0, "layer")?,
            label: get_stripped_value(record, 1, "label")?,
            background_color: get_stripped_value(record, 2, "background")?,
            border_color: get_stripped_value(record, 3, "border_color")?,
            text_color: get_stripped_value(record, 4, "text_color")?,
            alias: None,
            dataset: None,
            attributes: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // is_truthy test moved to utils module

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
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "2".to_string(),
                    label: "Child1".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "3".to_string(),
                    label: "Child2".to_string(),
                    layer: "Layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                // Only non-partition to non-partition edges
                Edge {
                    id: "e2".to_string(),
                    source: "2".to_string(),
                    target: "3".to_string(),
                    label: "Edge2".to_string(),
                    layer: "Layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![Layer {
                id: "Layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        }
    }

    fn create_complex_test_graph() -> Graph {
        Graph {
            name: "Complex Test Graph".to_string(),
            nodes: vec![
                // Root partitions
                Node {
                    id: "root1".to_string(),
                    label: "Root1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Children under root1
                Node {
                    id: "child1".to_string(),
                    label: "Child1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("root1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child2".to_string(),
                    label: "Child2".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("root1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child3".to_string(),
                    label: "Child3".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("root1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child4".to_string(),
                    label: "Child4".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("root1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Another root
                Node {
                    id: "root2".to_string(),
                    label: "Root2".to_string(),
                    layer: "layer2".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Children under root2
                Node {
                    id: "child5".to_string(),
                    label: "Child5".to_string(),
                    layer: "layer2".to_string(),
                    is_partition: false,
                    belongs_to: Some("root2".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child6".to_string(),
                    label: "Child6".to_string(),
                    layer: "layer2".to_string(),
                    is_partition: false,
                    belongs_to: Some("root2".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "child1".to_string(),
                    target: "child2".to_string(),
                    label: "Edge1".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "child2".to_string(),
                    target: "child3".to_string(),
                    label: "Edge2".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e3".to_string(),
                    source: "child1".to_string(),
                    target: "child3".to_string(),
                    label: "Edge3".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e4".to_string(),
                    source: "child3".to_string(),
                    target: "child5".to_string(),
                    label: "Edge4".to_string(),
                    layer: "layer2".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e5".to_string(),
                    source: "child5".to_string(),
                    target: "child6".to_string(),
                    label: "Edge5".to_string(),
                    layer: "layer2".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![
                Layer {
                    id: "layer1".to_string(),
                    label: "Layer 1".to_string(),
                    background_color: "FFFFFF".to_string(),
                    text_color: "000000".to_string(),
                    border_color: "000000".to_string(),
                    alias: None,
                    dataset: None,
                    attributes: None,
                },
                Layer {
                    id: "layer2".to_string(),
                    label: "Layer 2".to_string(),
                    background_color: "EEEEEE".to_string(),
                    text_color: "000000".to_string(),
                    border_color: "000000".to_string(),
                    alias: None,
                    dataset: None,
                    attributes: None,
                },
                Layer {
                    id: "aggregated".to_string(),
                    label: "Aggregated".to_string(),
                    background_color: "222222".to_string(),
                    text_color: "FFFFFF".to_string(),
                    border_color: "DDDDDD".to_string(),
                    alias: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            annotations: None,
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
            "comment": "null",
            "children": [
                {
                    "id": "2",
                    "label": "Child1",
                    "layer": "Layer1",
                    "is_partition": false,
                    "belongs_to": "1",
                    "depth": 1,
                    "comment": "null",
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
                    "comment": "null",
                    "weight": 1,
                    "children": []
                }
            ]
        }]);
        assert_eq!(json_tree, expected_json);
    }

    #[test]
    fn test_invert_graph() {
        let mut graph = create_test_graph();
        let inverted = graph
            .invert_graph()
            .expect("Failed to invert graph in test");

        // Basic structure checks
        assert_eq!(inverted.name, "Inverted Test Graph");

        // Check inverted nodes
        assert_eq!(inverted.nodes.len(), 2); // Root node + 1 edge nodes

        // Find the root node
        let root_node = inverted
            .nodes
            .iter()
            .find(|n| n.id == "inverted_root")
            .unwrap();
        assert!(root_node.is_partition);

        // Check that we have nodes created for each original edge
        let edge_nodes: Vec<&Node> = inverted
            .nodes
            .iter()
            .filter(|n| n.id != "inverted_root")
            .collect();
        assert_eq!(edge_nodes.len(), 1);

        // All edge nodes should belong to the root
        for node in edge_nodes {
            assert_eq!(node.belongs_to, Some("inverted_root".to_string()));
        }

        // Original graph should remain unchanged
        assert_eq!(graph.name, "Test Graph");
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_modify_graph_limit_partition_width() {
        let mut graph = create_complex_test_graph();

        // Verify initial state
        let root1 = graph.get_node_by_id("root1").unwrap();
        let root1_children = graph.get_children(root1);
        assert_eq!(root1_children.len(), 4); // Initially has 4 children

        // Apply partition width limit of 2
        let summary = graph.modify_graph_limit_partition_width(2).unwrap();

        // Verify after transformation
        let root1_after = graph.get_node_by_id("root1").unwrap();
        let root1_children_after = graph.get_children(root1_after);

        // Final width should match the limit (1 retained + 1 aggregate)
        assert_eq!(root1_children_after.len(), 2);

        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].aggregated_nodes.len(), 3);
        assert_eq!(summary[0].retained_count, 1);

        // First child should remain visible
        assert!(root1_children_after.iter().any(|n| n.id == "child1"));

        // Check if we have an aggregated node
        let agg_node = root1_children_after
            .iter()
            .find(|n| n.id.starts_with("agg_root1"))
            .expect("aggregate node missing");
        assert_eq!(agg_node.layer, "aggregated");
        assert_eq!(agg_node.weight, 3); // child2 + child3 + child4

        // The second root should still have its normal children
        let root2_after = graph.get_node_by_id("root2").unwrap();
        let root2_children_after = graph.get_children(root2_after);
        assert_eq!(root2_children_after.len(), 2); // Should still have 2 children

        // Aggregated edges should collapse duplicates
        let mut edge_lookup: HashMap<(&str, &str), (&str, i32)> = HashMap::new();
        for edge in &graph.edges {
            edge_lookup.insert(
                (edge.source.as_str(), edge.target.as_str()),
                (edge.id.as_str(), edge.weight),
            );
        }

        let agg_edge = edge_lookup
            .get(&("child1", agg_node.id.as_str()))
            .expect("expected edge from retained node to aggregate");
        assert_eq!(agg_edge.1, 2); // e1 + e3

        let agg_to_child5 = edge_lookup
            .get(&(agg_node.id.as_str(), "child5"))
            .expect("expected edge from aggregate to downstream node");
        assert_eq!(agg_to_child5.1, 1);

        // Ensure no self-loop edges remain
        assert!(!graph.edges.iter().any(|e| e.source == e.target));
    }

    #[test]
    fn test_partition_width_single_slot() {
        let mut graph = create_complex_test_graph();
        let summary = graph.modify_graph_limit_partition_width(1).unwrap();

        let root1 = graph.get_node_by_id("root1").unwrap();
        let children = graph.get_children(root1);
        assert_eq!(children.len(), 1);
        let agg = &children[0];
        assert!(agg.id.starts_with("agg_root1"));
        assert_eq!(agg.weight, 4);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].aggregated_nodes.len(), 4);

        // Every original root1 child should now be gone
        for id in ["child1", "child2", "child3", "child4"] {
            assert!(graph.get_node_by_id(id).is_none());
        }
    }

    #[test]
    fn test_aggregate_nodes_by_layer() {
        let mut graph = Graph {
            name: "Layer aggregation".to_string(),
            nodes: vec![
                Node {
                    id: "parent".to_string(),
                    label: "Parent".to_string(),
                    layer: "partition".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "parent2".to_string(),
                    label: "Parent2".to_string(),
                    layer: "partition".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "a1".to_string(),
                    label: "App1".to_string(),
                    layer: "app".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "a2".to_string(),
                    label: "App2".to_string(),
                    layer: "app".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent".to_string()),
                    weight: 2,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "a3".to_string(),
                    label: "App3".to_string(),
                    layer: "app".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "a4".to_string(),
                    label: "App4".to_string(),
                    layer: "app".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "b1".to_string(),
                    label: "Report1".to_string(),
                    layer: "report".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent2".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "b2".to_string(),
                    label: "Report2".to_string(),
                    layer: "report".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent2".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "b3".to_string(),
                    label: "Report3".to_string(),
                    layer: "report".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent2".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "hub".to_string(),
                    label: "Hub".to_string(),
                    layer: "infra".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "side".to_string(),
                    label: "Side".to_string(),
                    layer: "infra".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "db".to_string(),
                    label: "Database".to_string(),
                    layer: "infra".to_string(),
                    is_partition: false,
                    belongs_to: Some("parent2".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "a1".to_string(),
                    target: "hub".to_string(),
                    label: "depends".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "a2".to_string(),
                    target: "hub".to_string(),
                    label: "depends".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e3".to_string(),
                    source: "a3".to_string(),
                    target: "hub".to_string(),
                    label: "depends".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e4".to_string(),
                    source: "a4".to_string(),
                    target: "side".to_string(),
                    label: "depends".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e5".to_string(),
                    source: "b1".to_string(),
                    target: "db".to_string(),
                    label: "writes".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e6".to_string(),
                    source: "b2".to_string(),
                    target: "db".to_string(),
                    label: "writes".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e7".to_string(),
                    source: "b3".to_string(),
                    target: "db".to_string(),
                    label: "writes".to_string(),
                    layer: "default".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![
                Layer::new("partition", "Partition", "111111", "ffffff", "cccccc"),
                Layer::new("app", "App", "eeeeee", "000000", "aaaaaa"),
                Layer::new("report", "Report", "dddddd", "000000", "bbbbbb"),
                Layer::new("infra", "Infra", "cccccc", "000000", "999999"),
            ],
            annotations: None,
        };

        let summary = graph.aggregate_nodes_by_layer(3).unwrap();
        assert_eq!(summary.len(), 2);
        assert!(summary
            .iter()
            .any(|entry| entry.layer_id == "app" && entry.anchor_node_id == "hub"));
        assert!(summary
            .iter()
            .any(|entry| entry.layer_id == "report" && entry.anchor_node_id == "db"));

        let app_aggregate = graph
            .nodes
            .iter()
            .find(|node| node.label == "app agg(3)")
            .expect("App aggregate node missing");
        assert_eq!(app_aggregate.layer, "app");
        assert_eq!(app_aggregate.belongs_to.as_deref(), Some("parent"));
        assert_eq!(app_aggregate.weight, 4);

        let hub_edge = graph
            .edges
            .iter()
            .find(|edge| {
                (edge.source == app_aggregate.id && edge.target == "hub")
                    || (edge.target == app_aggregate.id && edge.source == "hub")
            })
            .expect("Aggregated edge to hub missing");
        assert_eq!(hub_edge.weight, 3);

        assert!(graph.nodes.iter().any(|node| node.id == "a4"));

        let report_aggregate = graph
            .nodes
            .iter()
            .find(|node| node.label == "report agg(3)")
            .expect("Report aggregate node missing");
        assert_eq!(report_aggregate.layer, "report");
        assert_eq!(report_aggregate.belongs_to.as_deref(), Some("parent2"));
    }

    #[test]
    fn test_modify_graph_limit_partition_depth() {
        // Create a graph with hierarchical structure
        let mut graph = Graph {
            name: "Hierarchical Test Graph".to_string(),
            nodes: vec![
                // Level 0
                Node {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Level 1
                Node {
                    id: "level1_1".to_string(),
                    label: "Level 1 Node 1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: Some("root".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Level 2
                Node {
                    id: "level2_1".to_string(),
                    label: "Level 2 Node 1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: Some("level1_1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Level 3
                Node {
                    id: "level3_1".to_string(),
                    label: "Level 3 Node 1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("level2_1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "level3_2".to_string(),
                    label: "Level 3 Node 2".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("level2_1".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![Edge {
                id: "e1".to_string(),
                source: "level3_1".to_string(),
                target: "level3_2".to_string(),
                label: "Edge1".to_string(),
                layer: "layer1".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            }],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        // Verify initial depth
        assert_eq!(graph.get_max_hierarchy_depth(), 3);
        assert_eq!(graph.nodes.len(), 5);

        // Limit depth to 1
        graph.modify_graph_limit_partition_depth(1).unwrap();

        // After limiting, the max depth should be 1
        assert_eq!(graph.get_max_hierarchy_depth(), 1);

        // Verify nodes are merged/aggregated
        assert!(graph.nodes.len() < 5);

        // Level 3 nodes should be gone
        assert!(graph.get_node_by_id("level3_1").is_none());
        assert!(graph.get_node_by_id("level3_2").is_none());

        // Level 2 node should now be non-partition
        let level2_1 = graph.get_node_by_id("level2_1");
        if let Some(node) = level2_1 {
            assert!(!node.is_partition);
        }
    }

    #[test]
    fn test_ensure_partition_hierarchy_adds_root() {
        let mut graph = Graph {
            name: "No Partition Graph".to_string(),
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    layer: "layer".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    layer: "layer".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![Edge {
                id: "edge_ab".to_string(),
                source: "a".to_string(),
                target: "b".to_string(),
                label: "".to_string(),
                layer: "layer".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            }],
            layers: vec![Layer {
                id: "layer".to_string(),
                label: "Layer".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        assert!(graph.ensure_partition_hierarchy());
        let synthetic = graph
            .nodes
            .iter()
            .find(|n| n.id == "synthetic_partition_root")
            .expect("Synthetic root should be created");
        assert!(synthetic.is_partition);
        let node_a = graph.get_node_by_id("a").unwrap();
        assert_eq!(
            node_a.belongs_to.as_deref(),
            Some("synthetic_partition_root")
        );
        assert!(node_a.is_partition);
    }

    #[test]
    fn test_limit_partition_depth_without_metadata() {
        let mut graph = Graph {
            name: "Depth No Metadata".to_string(),
            nodes: vec![
                Node {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child".to_string(),
                    label: "Child".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "grandchild".to_string(),
                    label: "Grandchild".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "root".to_string(),
                    target: "child".to_string(),
                    label: "".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "child".to_string(),
                    target: "grandchild".to_string(),
                    label: "".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        let original_node_count = graph.nodes.len();
        graph
            .modify_graph_limit_partition_depth(1)
            .expect("Depth limit should succeed");
        assert!(graph.nodes.len() < original_node_count);
    }

    #[test]
    fn test_limit_partition_width_without_metadata() {
        let mut graph = Graph {
            name: "Width No Metadata".to_string(),
            nodes: vec![
                Node {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child1".to_string(),
                    label: "Child1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child2".to_string(),
                    label: "Child2".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "child3".to_string(),
                    label: "Child3".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "root".to_string(),
                    target: "child1".to_string(),
                    label: "".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "root".to_string(),
                    target: "child2".to_string(),
                    label: "".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e3".to_string(),
                    source: "root".to_string(),
                    target: "child3".to_string(),
                    label: "".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        graph
            .modify_graph_limit_partition_width(2)
            .expect("Width limit should succeed");
        let root = graph.get_node_by_id("root").unwrap();
        let children = graph.get_children(root);
        assert!(children.len() <= 3);
        assert!(
            children.iter().any(|n| n.id.starts_with("agg_")),
            "Aggregated node should be created when trimming width"
        );
    }

    #[test]
    fn test_aggregate_edges() {
        // Create a graph with duplicate edges
        let mut graph = Graph {
            name: "Test Graph".to_string(),
            nodes: vec![
                Node {
                    id: "1".to_string(),
                    label: "Node 1".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Node {
                    id: "2".to_string(),
                    label: "Node 2".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                // Three edges between the same nodes
                Edge {
                    id: "e1".to_string(),
                    source: "1".to_string(),
                    target: "2".to_string(),
                    label: "Edge 1".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "1".to_string(),
                    target: "2".to_string(),
                    label: "Edge 2".to_string(),
                    layer: "layer1".to_string(),
                    weight: 2,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                Edge {
                    id: "e3".to_string(),
                    source: "1".to_string(),
                    target: "2".to_string(),
                    label: "Edge 3".to_string(),
                    layer: "layer1".to_string(),
                    weight: 3,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        // Verify initial state
        assert_eq!(graph.edges.len(), 3);

        // Aggregate edges
        graph.aggregate_edges();

        // After aggregation, should have one edge with combined weight
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].weight, 6); // Sum of all weights (1+2+3)
    }

    #[test]
    fn test_verify_graph_integrity() {
        // Create a valid graph
        let mut graph = create_test_graph();
        // Make sure we have a layer that matches all node/edge layers
        graph.add_layer(Layer::new(
            "Layer1", "Layer 1", "FFFFFF", "000000", "000000",
        ));

        // Valid graph should verify successfully
        match graph.verify_graph_integrity() {
            Ok(_) => {}
            Err(errors) => {
                println!("Graph verification failed with errors:");
                for error in errors {
                    println!("  - {}", error);
                }
                panic!("Graph should have verified successfully");
            }
        };

        // Create a graph with an invalid connection
        let invalid_graph = Graph {
            name: "Invalid Graph".to_string(),
            nodes: vec![
                // Partition node
                Node {
                    id: "1".to_string(),
                    label: "Partition".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
                // Non-partition node
                Node {
                    id: "2".to_string(),
                    label: "Non-Partition".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("1".to_string()), // Belongs to partition node 1
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            edges: vec![
                // Invalid edge: connects partition to non-partition
                Edge {
                    id: "e1".to_string(),
                    source: "1".to_string(), // Partition node
                    target: "2".to_string(), // Non-partition node
                    label: "Invalid Edge".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        // This should fail verification due to partition to non-partition edge
        let result = invalid_graph.verify_graph_integrity();
        assert!(result.is_err());

        // The error should contain information about the invalid edge
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("partition node")));
        }

        // Create a graph with missing node reference
        let missing_node_graph = Graph {
            name: "Missing Node Graph".to_string(),
            nodes: vec![Node {
                id: "1".to_string(),
                label: "Node 1".to_string(),
                layer: "layer1".to_string(),
                is_partition: false,
                belongs_to: None,
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            }],
            edges: vec![
                // Edge with missing target
                Edge {
                    id: "e1".to_string(),
                    source: "1".to_string(),
                    target: "non_existent".to_string(), // This node doesn't exist
                    label: "Invalid Edge".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
                attributes: None,
            }],
            annotations: None,
        };

        // This should fail verification due to missing node
        let result = missing_node_graph.verify_graph_integrity();
        assert!(result.is_err());

        // The error should contain information about the missing node
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("not found in nodes")));
        }
    }

    #[test]
    fn sanitize_labels_strips_quotes_and_controls() {
        let mut graph = Graph {
            name: "".into(),
            nodes: vec![Node {
                id: "n1".into(),
                label: "\"Tricky`\nlabel\twith\\quotes'".into(),
                layer: "l".into(),
                weight: 1,
                is_partition: false,
                belongs_to: None,
                comment: None,
                dataset: None,
                attributes: None,
            }],
            edges: vec![Edge {
                id: "e1".into(),
                source: "n1".into(),
                target: "n1".into(),
                label: "Edge\n\"label\"\\with\tjunk'`".into(),
                layer: "l".into(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            }],
            layers: vec![],
            annotations: None,
        };

        let (nodes, edges) = graph.sanitize_labels();
        assert_eq!(nodes, 1);
        assert_eq!(edges, 1);
        assert_eq!(graph.nodes[0].label, "Tricky label with quotes");
        assert_eq!(graph.edges[0].label, "Edge label with junk");
        assert!(graph
            .annotations
            .unwrap_or_default()
            .contains("Sanitized labels"));
    }
}
