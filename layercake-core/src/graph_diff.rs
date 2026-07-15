//! Structural diff between two graphs (datasets or computed graphs).
//!
//! Answers "what did the merge/transform do?" — the added/removed/changed nodes
//! and edges between a `from` and a `to` graph, keyed by id. Change detection
//! compares the full serialised item, so any field difference (label, layer,
//! weight, attrs…) counts as a change.

use crate::graph::{Edge, Graph, Node};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GraphDiff {
    pub nodes: ItemDiff,
    pub edges: ItemDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ItemDiff {
    /// Ids present in `to` but not `from`.
    pub added: Vec<String>,
    /// Ids present in `from` but not `to`.
    pub removed: Vec<String>,
    /// Ids present in both but with differing content.
    pub changed: Vec<String>,
    /// Count present in both and identical.
    pub unchanged: usize,
}

impl GraphDiff {
    pub fn is_empty(&self) -> bool {
        self.nodes.added.is_empty()
            && self.nodes.removed.is_empty()
            && self.nodes.changed.is_empty()
            && self.edges.added.is_empty()
            && self.edges.removed.is_empty()
            && self.edges.changed.is_empty()
    }
}

/// Diff two graphs parsed from their JSON representations.
pub fn diff_graph_json(from_json: &str, to_json: &str) -> Result<GraphDiff, serde_json::Error> {
    let from: Graph = serde_json::from_str(from_json)?;
    let to: Graph = serde_json::from_str(to_json)?;
    Ok(diff_graphs(&from, &to))
}

pub fn diff_graphs(from: &Graph, to: &Graph) -> GraphDiff {
    GraphDiff {
        nodes: diff_items(&from.nodes, &to.nodes, node_id),
        edges: diff_items(&from.edges, &to.edges, edge_id),
    }
}

fn node_id(n: &Node) -> String {
    n.id.clone()
}

fn edge_id(e: &Edge) -> String {
    if e.id.is_empty() {
        format!("{}:{}", e.source, e.target)
    } else {
        e.id.clone()
    }
}

fn diff_items<T: Serialize, F: Fn(&T) -> String>(from: &[T], to: &[T], id_of: F) -> ItemDiff {
    let from_map: HashMap<String, serde_json::Value> = from
        .iter()
        .map(|item| (id_of(item), serde_json::to_value(item).unwrap_or_default()))
        .collect();
    let to_map: HashMap<String, serde_json::Value> = to
        .iter()
        .map(|item| (id_of(item), serde_json::to_value(item).unwrap_or_default()))
        .collect();

    let mut diff = ItemDiff::default();
    for (id, to_val) in &to_map {
        match from_map.get(id) {
            None => diff.added.push(id.clone()),
            Some(from_val) if from_val != to_val => diff.changed.push(id.clone()),
            Some(_) => diff.unchanged += 1,
        }
    }
    for id in from_map.keys() {
        if !to_map.contains_key(id) {
            diff.removed.push(id.clone());
        }
    }
    diff.added.sort();
    diff.removed.sort();
    diff.changed.sort();
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(json: &str) -> String {
        json.to_string()
    }

    #[test]
    fn detects_added_removed_changed() {
        let from = g(r#"{"nodes":[
            {"id":"a","label":"A","layer":"l","weight":1},
            {"id":"b","label":"B","layer":"l","weight":1}
        ],"edges":[
            {"id":"e1","source":"a","target":"b","label":"x","layer":"l","weight":1}
        ],"layers":[]}"#);
        let to = g(r#"{"nodes":[
            {"id":"a","label":"A2","layer":"l","weight":1},
            {"id":"c","label":"C","layer":"l","weight":1}
        ],"edges":[
            {"id":"e1","source":"a","target":"b","label":"x","layer":"l","weight":1}
        ],"layers":[]}"#);

        let d = diff_graph_json(&from, &to).unwrap();
        assert_eq!(d.nodes.added, vec!["c"]);       // c is new
        assert_eq!(d.nodes.removed, vec!["b"]);     // b is gone
        assert_eq!(d.nodes.changed, vec!["a"]);     // a's label changed
        assert_eq!(d.edges.unchanged, 1);           // e1 identical
        assert!(d.edges.added.is_empty() && d.edges.removed.is_empty());
        assert!(!d.is_empty());
    }

    #[test]
    fn identical_graphs_diff_empty() {
        let j = g(r#"{"nodes":[{"id":"a","label":"A","layer":"l","weight":1}],"edges":[],"layers":[]}"#);
        let d = diff_graph_json(&j, &j).unwrap();
        assert!(d.is_empty());
        assert_eq!(d.nodes.unchanged, 1);
    }
}
