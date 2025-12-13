use crate::services::graph_data_service::{GraphDataEdgeInput, GraphDataNodeInput};
use chrono::Utc;
use serde_json::{json, Value};

/// Helper to build a GraphDataNodeInput from a Graph node.
/// Converts from the in-memory Graph representation to the graph_data schema.
pub fn node_to_graph_data_input(node: &crate::graph::Node) -> GraphDataNodeInput {
    // Build attributes JSON with layer information
    let mut attrs = serde_json::Map::new();

    // Store layer information in attributes
    attrs.insert("layer".to_string(), json!(node.layer.clone()));

    // Add comment to attributes if present
    if let Some(comment) = &node.comment {
        attrs.insert("comment".to_string(), json!(comment));
    }

    let attributes = if attrs.is_empty() {
        None
    } else {
        Some(Value::Object(attrs))
    };

    GraphDataNodeInput {
        external_id: node.id.clone(),
        label: Some(node.label.clone()),
        layer: Some(node.layer.clone()),
        weight: Some(node.weight as f64),
        is_partition: Some(node.is_partition),
        belongs_to: node.belongs_to.clone(),
        comment: node.comment.clone(),
        source_dataset_id: node.dataset,
        attributes,
        created_at: Some(Utc::now()),
    }
}

/// Helper to build a GraphDataEdgeInput from a Graph edge.
/// Converts from the in-memory Graph representation to the graph_data schema.
pub fn edge_to_graph_data_input(edge: &crate::graph::Edge) -> GraphDataEdgeInput {
    // Build attributes JSON with layer information
    let mut attrs = serde_json::Map::new();

    // Store layer information in attributes
    attrs.insert("layer".to_string(), json!(edge.layer.clone()));

    // Add comment to attributes if present
    if let Some(comment) = &edge.comment {
        attrs.insert("comment".to_string(), json!(comment));
    }

    let attributes = if attrs.is_empty() {
        None
    } else {
        Some(Value::Object(attrs))
    };

    GraphDataEdgeInput {
        external_id: edge.id.clone(),
        source: edge.source.clone(),
        target: edge.target.clone(),
        label: Some(edge.label.clone()),
        layer: Some(edge.layer.clone()),
        weight: Some(edge.weight as f64),
        comment: edge.comment.clone(),
        source_dataset_id: edge.dataset,
        attributes,
        created_at: Some(Utc::now()),
    }
}

/// Convert a collection of Graph nodes to GraphDataNodeInput
pub fn nodes_to_graph_data_inputs(nodes: &[crate::graph::Node]) -> Vec<GraphDataNodeInput> {
    nodes.iter().map(node_to_graph_data_input).collect()
}

/// Convert a collection of Graph edges to GraphDataEdgeInput
pub fn edges_to_graph_data_inputs(edges: &[crate::graph::Edge]) -> Vec<GraphDataEdgeInput> {
    edges.iter().map(edge_to_graph_data_input).collect()
}
