//! Tests for FilterNode query evaluation.
//!
//! The filter used to build raw SQL against the legacy `graph_nodes` /
//! `graph_edges` / `graph_layers` tables (dropped by m20251215), so any plan
//! with a filter node failed with "no such table: graph_nodes". It now
//! evaluates rules against the in-memory graph. These tests exercise that
//! evaluation directly, independent of the DAG executor.

use layercake as layercake_core;
use layercake_core::graph::{Edge, Graph, Layer, Node};
use layercake_core::plan_dag::{FilterEvaluationContext, FilterNodeConfig};
use sea_orm::{Database, DatabaseConnection};
use serde_json::json;

async fn any_db() -> DatabaseConnection {
    // The filter no longer touches the DB, but the context still carries a
    // connection for API stability.
    Database::connect("sqlite::memory:").await.unwrap()
}

fn node(id: &str, label: &str, layer: &str, weight: i32) -> Node {
    Node {
        id: id.to_string(),
        label: label.to_string(),
        layer: layer.to_string(),
        is_partition: false,
        belongs_to: None,
        weight,
        comment: None,
        dataset: None,
        attributes: None,
    }
}

fn edge(id: &str, source: &str, target: &str) -> Edge {
    Edge {
        id: id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        label: String::new(),
        layer: "L1".to_string(),
        weight: 1,
        comment: None,
        dataset: None,
        attributes: None,
    }
}

fn sample_graph() -> Graph {
    Graph {
        name: "g".into(),
        nodes: vec![
            node("a", "Alpha", "L1", 5),
            node("b", "Beta", "L1", 1),
            node("c", "Gamma", "L2", 9),
        ],
        edges: vec![edge("e1", "a", "b"), edge("e2", "b", "c")],
        layers: vec![
            Layer {
                id: "L1".into(),
                label: "Layer 1".into(),
                background_color: "#fff".into(),
                text_color: "#000".into(),
                border_color: "#000".into(),
                alias: None,
                dataset: None,
                attributes: None,
            },
            Layer {
                id: "L2".into(),
                label: "Layer 2".into(),
                background_color: "#fff".into(),
                text_color: "#000".into(),
                border_color: "#000".into(),
                alias: None,
                dataset: None,
                attributes: None,
            },
        ],
        annotations: None,
    }
}

async fn apply(graph: &mut Graph, config_json: serde_json::Value) {
    let config: FilterNodeConfig = serde_json::from_str(&config_json.to_string()).unwrap();
    let db = any_db().await;
    config
        .apply_filters(
            graph,
            &FilterEvaluationContext {
                db: &db,
                graph_id: 1,
            },
        )
        .await
        .expect("filter should apply without touching dropped tables");
}

#[tokio::test]
async fn empty_rule_group_keeps_all_nodes() {
    let mut graph = sample_graph();
    apply(
        &mut graph,
        json!({"query": {
            "targets": ["nodes"], "mode": "include",
            "linkPruningMode": "retainEdges",
            "ruleGroup": {"combinator": "and", "rules": []},
            "fieldMetadataVersion": "v1"
        }}),
    )
    .await;
    assert_eq!(graph.nodes.len(), 3, "no rules => keep everything");
}

#[tokio::test]
async fn include_filters_nodes_by_layer_equals() {
    let mut graph = sample_graph();
    apply(
        &mut graph,
        json!({"query": {
            "targets": ["nodes"], "mode": "include",
            "linkPruningMode": "autoDropDanglingEdges",
            "ruleGroup": {"combinator": "and", "rules": [
                {"type": "rule", "field": "layer", "operator": "equals", "value": "L1"}
            ]},
            "fieldMetadataVersion": "v1"
        }}),
    )
    .await;

    let ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"], "only L1 nodes kept");
    // Edge c->... referencing the dropped node c is auto-pruned.
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].id, "e1");
}

#[tokio::test]
async fn exclude_mode_removes_matching_nodes() {
    let mut graph = sample_graph();
    apply(
        &mut graph,
        json!({"query": {
            "targets": ["nodes"], "mode": "exclude",
            "linkPruningMode": "retainEdges",
            "ruleGroup": {"combinator": "and", "rules": [
                {"type": "rule", "field": "layer", "operator": "equals", "value": "L2"}
            ]},
            "fieldMetadataVersion": "v1"
        }}),
    )
    .await;
    let ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"], "L2 node excluded");
}

#[tokio::test]
async fn numeric_greater_than_on_weight() {
    let mut graph = sample_graph();
    apply(
        &mut graph,
        json!({"query": {
            "targets": ["nodes"], "mode": "include",
            "linkPruningMode": "retainEdges",
            "ruleGroup": {"combinator": "and", "rules": [
                {"type": "rule", "field": "weight", "operator": "greaterThan", "value": "3"}
            ]},
            "fieldMetadataVersion": "v1"
        }}),
    )
    .await;
    let mut ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["a", "c"], "weight > 3 keeps a(5) and c(9)");
}

#[tokio::test]
async fn or_combinator_and_contains() {
    let mut graph = sample_graph();
    apply(
        &mut graph,
        json!({"query": {
            "targets": ["nodes"], "mode": "include",
            "linkPruningMode": "retainEdges",
            "ruleGroup": {"combinator": "or", "rules": [
                {"type": "rule", "field": "label", "operator": "contains", "value": "Alph"},
                {"type": "rule", "field": "label", "operator": "equals", "value": "Gamma"}
            ]},
            "fieldMetadataVersion": "v1"
        }}),
    )
    .await;
    let mut ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["a", "c"]);
}

#[tokio::test]
async fn filter_by_custom_attribute() {
    let mut graph = sample_graph();
    graph.nodes[0].attributes = Some(json!({"severity": "high"}));
    graph.nodes[2].attributes = Some(json!({"severity": "low"}));
    apply(
        &mut graph,
        json!({"query": {
            "targets": ["nodes"], "mode": "include",
            "linkPruningMode": "retainEdges",
            "ruleGroup": {"combinator": "and", "rules": [
                {"type": "rule", "field": "severity", "operator": "equals", "value": "high"}
            ]},
            "fieldMetadataVersion": "v1"
        }}),
    )
    .await;
    let ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["a"],
        "attribute-based filtering works via the JSON bag"
    );
}
