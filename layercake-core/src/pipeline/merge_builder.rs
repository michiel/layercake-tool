use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::database::entities::graphs::{Column as GraphColumn, Entity as GraphEntity};
use crate::graph::{Edge, Graph, Layer, Node};
use crate::services::graph_data_service::GraphDataService;
use crate::services::graph_service::GraphService;

/// Service for merging data from multiple upstream sources using graph_data-first resolution.
pub struct MergeBuilder {
    db: DatabaseConnection,
    graph_data_service: std::sync::Arc<GraphDataService>,
    graph_service: GraphService,
}

impl MergeBuilder {
    pub fn new(
        db: DatabaseConnection,
        graph_data_service: std::sync::Arc<GraphDataService>,
    ) -> Self {
        let graph_service = GraphService::new(db.clone());
        Self {
            db,
            graph_data_service,
            graph_service,
        }
    }

    /// Merge data from upstream nodes into a single Graph struct (in-memory).
    pub async fn merge_sources(
        &self,
        project_id: i32,
        _plan_id: i32,
        _node_id: String,
        name: String,
        upstream_node_ids: Vec<String>,
    ) -> Result<Graph> {
        // Load upstream graphs via graph_data first, falling back to legacy graphs table
        let mut upstream_graphs = Vec::new();
        for upstream_id in upstream_node_ids {
            let graph = self
                .load_graph_by_dag_node(project_id, &upstream_id)
                .await?;
            upstream_graphs.push(graph);
        }

        // Merge nodes/edges/layers
        let mut nodes_map: HashMap<String, NodeMerge> = HashMap::new();
        let mut edges_map: HashMap<String, EdgeMerge> = HashMap::new();
        let mut layers_map: HashMap<String, Layer> = HashMap::new();

        for graph in &upstream_graphs {
            for node in &graph.nodes {
                let entry = nodes_map.entry(node.id.clone()).or_default();
                entry.id = node.id.clone();
                entry.label = entry.label.clone().or_else(|| Some(node.label.clone()));
                entry.layer = entry.layer.clone().or_else(|| Some(node.layer.clone()));
                entry.is_partition |= node.is_partition;
                entry.belongs_to = entry
                    .belongs_to
                    .clone()
                    .or_else(|| node.belongs_to.clone());
                entry.weight += node.weight as i64;
                entry.comment = entry.comment.clone().or_else(|| node.comment.clone());
                entry.dataset = entry.dataset.or(node.dataset);
                entry.attributes = entry.attributes.clone().or_else(|| node.attributes.clone());
            }

            for edge in &graph.edges {
                let entry = edges_map.entry(edge.id.clone()).or_default();
                entry.id = edge.id.clone();
                entry.source = edge.source.clone();
                entry.target = edge.target.clone();
                entry.label = entry.label.clone().or_else(|| Some(edge.label.clone()));
                entry.layer = entry.layer.clone().or_else(|| Some(edge.layer.clone()));
                entry.weight += edge.weight as i64;
                entry.comment = entry.comment.clone().or_else(|| edge.comment.clone());
                entry.dataset = entry.dataset.or(edge.dataset);
                entry.attributes = entry.attributes.clone().or_else(|| edge.attributes.clone());
            }

            for layer in &graph.layers {
                layers_map.entry(layer.id.clone()).or_insert_with(|| layer.clone());
            }
        }

        // Validate edges reference nodes
        let node_ids: HashSet<_> = nodes_map.keys().cloned().collect();
        for (edge_id, edge) in &edges_map {
            if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
                return Err(anyhow!(
                    "Merge edge {} references missing node (source:{} target:{})",
                    edge_id,
                    edge.source,
                    edge.target
                ));
            }
        }

        // Build merged Graph struct
        let nodes: Vec<Node> = nodes_map
            .into_values()
            .map(|n| Node {
                id: n.id.clone(),
                label: n.label.unwrap_or_else(|| n.id.clone()),
                layer: n.layer.unwrap_or_else(|| "default".to_string()),
                is_partition: n.is_partition,
                belongs_to: n.belongs_to,
                weight: n.weight as i32,
                comment: n.comment,
                dataset: n.dataset,
                attributes: n.attributes,
            })
            .collect();

        let edges: Vec<Edge> = edges_map
            .into_values()
            .map(|e| Edge {
                id: e.id.clone(),
                source: e.source,
                target: e.target,
                label: e.label.unwrap_or_else(|| e.id.clone()),
                layer: e.layer.unwrap_or_else(|| "default".to_string()),
                weight: e.weight as i32,
                comment: e.comment,
                dataset: e.dataset,
                attributes: e.attributes,
            })
            .collect();

        let layers: Vec<Layer> = if layers_map.is_empty() {
            // Derive layers from node attributes if none were present
            derive_layers_from_nodes(&nodes)
        } else {
            layers_map.into_values().collect()
        };

        Ok(Graph {
            name,
            nodes,
            edges,
            layers,
            annotations: None,
        })
    }

    async fn load_graph_by_dag_node(
        &self,
        project_id: i32,
        dag_node_id: &str,
    ) -> Result<Graph> {
        // Try graph_data first
        if let Some(gd) = self
            .graph_data_service
            .get_by_dag_node(dag_node_id)
            .await?
        {
            let (gd, nodes, edges) = self
                .graph_data_service
                .load_full(gd.id)
                .await
                .map_err(|e| anyhow!("load_full graph_data {}: {}", gd.id, e))?;

            let graph_nodes: Vec<Node> = nodes
                .into_iter()
                .map(|n| Node {
                    id: n.external_id,
                    label: n.label.unwrap_or_else(|| "".into()),
                    layer: n.layer.unwrap_or_else(|| "default".into()),
                    is_partition: n.is_partition,
                    belongs_to: n.belongs_to,
                    weight: n.weight.map(|w| w as i32).unwrap_or(1),
                    comment: n.comment,
                    dataset: n.source_dataset_id,
                    attributes: n.attributes,
                })
                .collect();

            let graph_edges: Vec<Edge> = edges
                .into_iter()
                .map(|e| Edge {
                    id: e.external_id,
                    source: e.source,
                    target: e.target,
                    label: e.label.unwrap_or_else(|| "".into()),
                    layer: e.layer.unwrap_or_else(|| "default".into()),
                    weight: e.weight.map(|w| w as i32).unwrap_or(1),
                    comment: e.comment,
                    dataset: e.source_dataset_id,
                    attributes: e.attributes,
                })
                .collect();

            let layers = derive_layers_from_nodes(&graph_nodes);

            return Ok(Graph {
                name: gd.name,
                nodes: graph_nodes,
                edges: graph_edges,
                layers,
                annotations: gd.annotations.and_then(|v| v.as_str().map(|s| s.to_string())),
            });
        }

        // Fallback to legacy graphs table
        let legacy_graph = GraphEntity::find()
            .filter(GraphColumn::ProjectId.eq(project_id))
            .filter(GraphColumn::NodeId.eq(dag_node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "No graph found for dag node {} in graph_data or legacy graphs table",
                    dag_node_id
                )
            })?;

        let graph = self
            .graph_service
            .build_graph_from_dag_graph(legacy_graph.id)
            .await?;

        Ok(graph)
    }
}

/// Helper structures for merging
#[derive(Default)]
struct NodeMerge {
    id: String,
    label: Option<String>,
    layer: Option<String>,
    is_partition: bool,
    belongs_to: Option<String>,
    weight: i64,
    comment: Option<String>,
    dataset: Option<i32>,
    attributes: Option<Value>,
}

#[derive(Default)]
struct EdgeMerge {
    id: String,
    source: String,
    target: String,
    label: Option<String>,
    layer: Option<String>,
    weight: i64,
    comment: Option<String>,
    dataset: Option<i32>,
    attributes: Option<Value>,
}

fn derive_layers_from_nodes(nodes: &[Node]) -> Vec<Layer> {
    let mut layer_map: HashMap<String, Layer> = HashMap::new();
    for node in nodes {
        let entry = layer_map.entry(node.layer.clone()).or_insert_with(|| Layer {
            id: node.layer.clone(),
            label: node.layer.clone(),
            background_color: "#FFFFFF".into(),
            text_color: "#000000".into(),
            border_color: "#000000".into(),
            alias: None,
            dataset: node.dataset,
            attributes: None,
        });

        if let Some(attrs) = &node.attributes {
            if let Some(obj) = attrs.as_object() {
                if let Some(bg) = obj
                    .get("backgroundColor")
                    .or_else(|| obj.get("color"))
                    .and_then(|v| v.as_str())
                {
                    entry.background_color = bg.to_string();
                }
                if let Some(txt) = obj.get("textColor").and_then(|v| v.as_str()) {
                    entry.text_color = txt.to_string();
                }
                if let Some(border) = obj.get("borderColor").and_then(|v| v.as_str()) {
                    entry.border_color = border.to_string();
                }
                entry.attributes = Some(json!(obj));
            }
        }
    }

    layer_map.into_values().collect()
}
