use crate::app_context::GraphValidationSummary;
use crate::database::entities::{
    data_sets, graph_data, graph_edges, graph_edges::Entity as GraphEdges, graph_layers,
    graph_layers::Entity as Layers, graph_nodes, graph_nodes::Entity as GraphNodes, layer_aliases,
    plan_dag_edges, plan_dag_nodes, project_layers,
};
use crate::errors::{GraphError, GraphResult};
use crate::graph::{Edge, Graph, Layer, Node};
use crate::services::GraphDataService;
use chrono::Utc;
use indexmap::IndexMap;
use sea_orm::prelude::Expr;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde_json::Value;
use std::collections::HashSet;

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn dataset_contains_layers(data_set: &data_sets::Model) -> bool {
        serde_json::from_str::<Value>(&data_set.graph_json)
            .ok()
            .and_then(|parsed| parsed.get("layers").and_then(|v| v.as_array()).cloned())
            .map(|layers| !layers.is_empty())
            .unwrap_or(false)
    }

    /// Normalize color value to ensure it has # prefix for CSS compatibility
    fn normalize_color(color: &str) -> String {
        let trimmed = color.trim();
        if trimmed.is_empty() {
            return String::from("#FFFFFF");
        }
        if trimmed.starts_with('#') {
            trimmed.to_string()
        } else {
            format!("#{}", trimmed)
        }
    }

    fn normalize_alias(alias: Option<String>) -> Option<String> {
        alias.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    async fn seed_layers_from_dataset(
        &self,
        project_id: i32,
        dataset_id: i32,
        enabled: bool,
    ) -> GraphResult<usize> {
        use crate::database::entities::data_sets;

        tracing::debug!(
            "seed_layers_from_dataset: project_id={}, dataset_id={}, enabled={}",
            project_id,
            dataset_id,
            enabled
        );

        let data_set = data_sets::Entity::find_by_id(dataset_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or_else(|| GraphError::Validation(format!("Dataset {} not found", dataset_id)))?;

        if data_set.project_id != project_id {
            tracing::warn!(
                "Dataset {} project_id mismatch: expected {}, got {}",
                dataset_id,
                project_id,
                data_set.project_id
            );
            return Ok(0);
        }

        let parsed: Value = serde_json::from_str(&data_set.graph_json).unwrap_or_default();
        let mut updated = 0usize;

        if let Some(arr) = parsed.get("layers").and_then(|v| v.as_array()) {
            tracing::debug!(
                "Found {} layers in dataset {} graph_json",
                arr.len(),
                dataset_id
            );
            for item in arr {
                if let Some(obj) = item.as_object() {
                    let layer_id = obj
                        .get("id")
                        .or_else(|| obj.get("layer_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if layer_id.is_empty() {
                        tracing::warn!("Skipping layer with empty id in dataset {}", dataset_id);
                        continue;
                    }
                    let name = obj
                        .get("label")
                        .or_else(|| obj.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(layer_id.as_str())
                        .to_string();
                    let background_color = Self::normalize_color(
                        obj.get("background_color")
                            .or_else(|| obj.get("backgroundColor"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("FFFFFF"),
                    );
                    let text_color = Self::normalize_color(
                        obj.get("text_color")
                            .or_else(|| obj.get("textColor"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("000000"),
                    );
                    let border_color = Self::normalize_color(
                        obj.get("border_color")
                            .or_else(|| obj.get("borderColor"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("000000"),
                    );
                    let alias = obj
                        .get("alias")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    tracing::debug!(
                        "Upserting layer '{}' from dataset {} (enabled={})",
                        layer_id,
                        dataset_id,
                        enabled
                    );

                    let _ = self
                        .upsert_project_layer(
                            project_id,
                            layer_id.clone(),
                            name,
                            background_color,
                            text_color,
                            border_color,
                            alias,
                            Some(dataset_id),
                            enabled,
                        )
                        .await?;
                    updated += 1;
                }
            }
        } else {
            tracing::warn!(
                "No 'layers' array found in dataset {} graph_json",
                dataset_id
            );
        }

        tracing::info!(
            "Seeded {} layers from dataset {} for project {}",
            updated,
            dataset_id,
            project_id
        );

        // Always ensure all layers from this dataset have the correct enabled state
        // This handles cases where layers existed but their enabled state needs updating
        let now = chrono::Utc::now();
        let _result = project_layers::Entity::update_many()
            .col_expr(project_layers::Column::Enabled, Expr::value(enabled))
            .col_expr(project_layers::Column::UpdatedAt, Expr::value(now))
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::SourceDatasetId.eq(Some(dataset_id)))
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        Ok(updated)
    }

    async fn seed_project_layers_if_empty(&self, project_id: i32) -> GraphResult<()> {
        use crate::database::entities::data_sets;

        tracing::debug!(
            "seed_project_layers_if_empty: checking project {}",
            project_id
        );

        let existing = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .count(&self.db)
            .await
            .map_err(GraphError::Database)?;

        if existing > 0 {
            tracing::debug!(
                "Project {} already has {} layers, skipping seed",
                project_id,
                existing
            );
            return Ok(());
        }

        let datasets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        let mut layer_datasets = Vec::new();
        for ds in datasets {
            if Self::dataset_contains_layers(&ds) {
                layer_datasets.push(ds);
            }
        }

        let dataset_count = layer_datasets.len();
        tracing::info!(
            "Found {} layer datasets for project {}, seeding...",
            dataset_count,
            project_id
        );

        for ds in layer_datasets {
            let _ = self
                .seed_layers_from_dataset(project_id, ds.id, true)
                .await?;
        }

        tracing::info!(
            "Completed seeding project {} layers from {} datasets",
            project_id,
            dataset_count
        );

        Ok(())
    }

    async fn get_project_layers_palette(&self, project_id: i32) -> GraphResult<Vec<Layer>> {
        // Ensure palette exists by seeding from layer datasets if empty
        self.seed_project_layers_if_empty(project_id).await?;

        // Query enabled layers ordered by priority:
        // - Manual layers (source_dataset_id = NULL) first (NULLs sort first in ascending order)
        // - Then dataset layers ordered by dataset ID and insertion time
        let db_layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::Enabled.eq(true))
            .order_by_asc(project_layers::Column::SourceDatasetId)
            .order_by_asc(project_layers::Column::Id)
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        // Deduplication: When multiple sources define the same layer_id, only the first
        // occurrence is included in the palette. Priority order is determined by the
        // SQL ORDER BY clause above:
        // 1. Manual layers (source_dataset_id = NULL) come before dataset layers
        // 2. Among dataset layers, ordered by source_dataset_id then by insertion order (id)
        // This means manual layer definitions override dataset-provided ones with the same ID.
        let mut seen = HashSet::new();
        let mut palette = Vec::new();

        for db_layer in db_layers {
            if seen.contains(&db_layer.layer_id) {
                continue; // Skip duplicate layer_id
            }
            seen.insert(db_layer.layer_id.clone());
            palette.push(self.hydrate_project_layer(project_id, db_layer).await?);
        }

        Ok(palette)
    }

    /// Get database graph_layers for a graph
    #[allow(dead_code)]
    pub async fn get_layers_for_graph(
        &self,
        graph_id: i32,
    ) -> GraphResult<Vec<graph_layers::Model>> {
        let db_layers = Layers::find()
            .filter(graph_layers::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;
        Ok(db_layers)
    }

    /// Build a Graph from a DAG-built graph; graph_data-first, fallback to legacy graphs table.
    pub async fn build_graph_from_dag_graph(&self, graph_id: i32) -> GraphResult<Graph> {
        // Try unified schema first
        if let Some(gd) = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
        {
            let graph_data_service = GraphDataService::new(self.db.clone());
            let (gd_model, nodes, edges) = graph_data_service
                .load_full(gd.id)
                .await
                .map_err(GraphError::Database)?;

            let graph_nodes: Vec<Node> = nodes
                .into_iter()
                .map(|n| Node {
                    id: n.external_id,
                    label: n.label.unwrap_or_default(),
                    layer: n.layer.unwrap_or_default(),
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
                    label: e.label.unwrap_or_default(),
                    layer: e.layer.unwrap_or_default(),
                    weight: e.weight.map(|w| w as i32).unwrap_or(1),
                    comment: e.comment,
                    dataset: e.source_dataset_id,
                    attributes: e.attributes,
                })
                .collect();

            // Derive layers from node attributes
            let mut layer_map: IndexMap<String, Layer> = IndexMap::new();
            for node in &graph_nodes {
                if node.layer.is_empty() || layer_map.contains_key(&node.layer) {
                    continue;
                }

                let (bg_color, text_color, border_color) = node
                    .attributes
                    .as_ref()
                    .and_then(|attrs| attrs.as_object())
                    .map(|obj| {
                        let bg = obj
                            .get("backgroundColor")
                            .or_else(|| obj.get("color"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let text = obj
                            .get("textColor")
                            .or_else(|| obj.get("labelColor"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let border = obj
                            .get("borderColor")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        (bg, text, border)
                    })
                    .unwrap_or((None, None, None));

                layer_map.insert(
                    node.layer.clone(),
                    Layer {
                        id: node.layer.clone(),
                        label: node.layer.clone(),
                        background_color: bg_color.unwrap_or_else(|| "#FFFFFF".to_string()),
                        text_color: text_color.unwrap_or_else(|| "#000000".to_string()),
                        border_color: border_color.unwrap_or_else(|| "#000000".to_string()),
                        alias: None,
                        dataset: node.dataset,
                        attributes: node.attributes.clone(),
                    },
                );
            }

            let layers: Vec<Layer> = layer_map.into_values().collect();

            let graph = Graph {
                name: gd_model.name.clone(),
                nodes: graph_nodes,
                edges: graph_edges,
                layers,
                annotations: gd_model
                    .annotations
                    .as_ref()
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
            };

            return Ok(graph);
        }

        // Fetch the graph metadata
        use crate::database::entities::graphs::Entity as Graphs;
        let graph_meta = Graphs::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or(GraphError::NotFound(graph_id))?;
        tracing::warn!(
            "GraphService falling back to legacy graphs table for graph_id {} (project {})",
            graph_id,
            graph_meta.project_id
        );

        // Fetch graph nodes
        let db_graph_nodes = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        // Fetch graph edges
        let db_graph_edges = GraphEdges::find()
            .filter(graph_edges::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        // Fetch project-wide layers; fall back to legacy graph-level layers
        let palette_layers = self
            .get_project_layers_palette(graph_meta.project_id)
            .await?;

        if palette_layers.is_empty() {
            tracing::warn!(
                "Project {} has no enabled palette layers; exports will fall back to neutral styling",
                graph_meta.project_id
            );
        }

        let mut layer_map: IndexMap<String, Layer> = palette_layers
            .into_iter()
            .map(|layer| {
                (
                    layer.id.clone(),
                    Layer {
                        id: layer.id,
                        label: layer.label,
                        background_color: sanitize_hex(&layer.background_color, "FFFFFF"),
                        text_color: sanitize_hex(&layer.text_color, "000000"),
                        border_color: sanitize_hex(&layer.border_color, "000000"),
                        alias: layer.alias,
                        dataset: layer.dataset,
                        attributes: None,
                    },
                )
            })
            .collect();

        // Track data quality issues for logging
        let mut nodes_missing_label = 0;
        let mut edges_missing_layer = 0;

        // Convert to Graph Node structs
        let graph_nodes: Vec<Node> = db_graph_nodes
            .into_iter()
            .map(|db_node| {
                // Use node ID as label fallback for visibility
                let label = if let Some(label) = db_node.label {
                    label
                } else {
                    nodes_missing_label += 1;
                    db_node.id.clone()
                };

                // Empty layer means inherit default styling
                let layer = db_node.layer.unwrap_or_default();
                let comment = db_node.comment.or_else(|| {
                    db_node
                        .attrs
                        .as_ref()
                        .and_then(|attrs| attrs.get("comment"))
                        .and_then(|value| value.as_str())
                        .map(|s| s.to_string())
                });

                Node {
                    id: db_node.id,
                    label,
                    layer,
                    is_partition: db_node.is_partition,
                    belongs_to: db_node.belongs_to,
                    weight: db_node.weight.unwrap_or(1.0) as i32,
                    comment,
                    dataset: db_node.dataset_id,
                    attributes: db_node.attrs,
                }
            })
            .collect();

        // Convert to Graph Edge structs
        let graph_edges: Vec<Edge> = db_graph_edges
            .into_iter()
            .map(|db_edge| {
                // Empty layer means inherit default styling
                let layer = if let Some(layer) = db_edge.layer {
                    layer
                } else {
                    edges_missing_layer += 1;
                    String::new()
                };

                Edge {
                    id: db_edge.id.clone(),
                    source: db_edge.source,
                    target: db_edge.target,
                    label: db_edge.label.unwrap_or_default(),
                    layer,
                    weight: db_edge.weight.unwrap_or(1.0) as i32,
                    comment: None,
                    dataset: db_edge.dataset_id,
                    attributes: db_edge.attrs,
                }
            })
            .collect();

        // Ensure every referenced layer has a placeholder entry so template exporters
        // always receive a complete layer map (important for DOT/Mermaid/PUML helpers).
        let mut ensure_layer = |layer_id: &str| {
            if layer_id.is_empty() || layer_map.contains_key(layer_id) {
                return;
            }
            layer_map.insert(
                layer_id.to_string(),
                Layer {
                    id: layer_id.to_string(),
                    label: layer_id.to_string(),
                    background_color: "f7f7f8".to_string(),
                    text_color: "0f172a".to_string(),
                    border_color: "1f2933".to_string(),
                    alias: None,
                    dataset: None,
                    attributes: None,
                },
            );
        };

        for node in &graph_nodes {
            ensure_layer(&node.layer);
        }
        for edge in &graph_edges {
            ensure_layer(&edge.layer);
        }

        // Log data quality warnings
        if nodes_missing_label > 0 {
            tracing::warn!(
                "Graph {}: {} nodes missing label, using node ID as fallback",
                graph_id,
                nodes_missing_label
            );
        }
        if edges_missing_layer > 0 {
            tracing::debug!(
                "Graph {}: {} edges have no layer (will inherit default styling)",
                graph_id,
                edges_missing_layer
            );
        }

        Ok(Graph {
            name: graph_meta.name,
            nodes: graph_nodes,
            edges: graph_edges,
            layers: layer_map.into_values().collect(),
            annotations: graph_meta.annotations,
        })
    }

    pub async fn validate_graph(&self, graph_id: i32) -> GraphResult<GraphValidationSummary> {
        // Prefer graph_data; fall back to legacy graphs table (logs warning)
        let (graph_meta_project, graph) = if let Some(gd) = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
        {
            let graph_meta_project = gd.project_id;
            let graph = self.build_graph_from_dag_graph(graph_id).await?;
            (graph_meta_project, graph)
        } else {
            use crate::database::entities::graphs::Entity as Graphs;
            let graph_meta = Graphs::find_by_id(graph_id)
                .one(&self.db)
                .await
                .map_err(GraphError::Database)?
                .ok_or(GraphError::NotFound(graph_id))?;
            let graph = self.build_graph_from_dag_graph(graph_id).await?;
            (graph_meta.project_id, graph)
        };

        let mut errors = Vec::new();
        let warnings = Vec::new();

        if let Err(mut validation_errors) = graph.verify_graph_integrity() {
            errors.append(&mut validation_errors);
        }

        Ok(GraphValidationSummary {
            graph_id,
            project_id: graph_meta_project,
            is_valid: errors.is_empty(),
            errors,
            warnings,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            layer_count: graph.layers.len(),
            checked_at: Utc::now(),
        })
    }

    pub async fn create_graph(
        &self,
        project_id: i32,
        name: String,
        node_id: Option<String>,
    ) -> GraphResult<crate::database::entities::graphs::Model> {
        use crate::database::entities::graphs;
        use sea_orm::{ActiveModelTrait, Set};

        let node_id =
            node_id.unwrap_or_else(|| format!("graphnode_{}", uuid::Uuid::new_v4().simple()));

        let mut graph = graphs::ActiveModel::new();
        graph.project_id = Set(project_id);
        graph.name = Set(name);
        graph.node_id = Set(node_id);

        let graph = graph.insert(&self.db).await.map_err(GraphError::Database)?;

        Ok(graph)
    }

    pub async fn update_graph(
        &self,
        id: i32,
        name: Option<String>,
    ) -> GraphResult<crate::database::entities::graphs::Model> {
        use crate::database::entities::graphs;
        use sea_orm::{ActiveModelTrait, Set};

        let graph = graphs::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or(GraphError::NotFound(id))?;

        let mut active_model: graphs::ActiveModel = graph.into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model
            .update(&self.db)
            .await
            .map_err(GraphError::Database)?;
        Ok(updated)
    }

    pub async fn delete_graph(&self, id: i32) -> GraphResult<()> {
        use crate::database::entities::graphs;

        let graph = graphs::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or(GraphError::NotFound(id))?;

        // Find and delete all plan_dag_nodes that reference this graph by node_id
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::Id.eq(&graph.node_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        for dag_node in dag_nodes {
            // Delete connected edges first
            plan_dag_edges::Entity::delete_many()
                .filter(plan_dag_edges::Column::SourceNodeId.eq(&dag_node.id))
                .exec(&self.db)
                .await
                .map_err(GraphError::Database)?;

            plan_dag_edges::Entity::delete_many()
                .filter(plan_dag_edges::Column::TargetNodeId.eq(&dag_node.id))
                .exec(&self.db)
                .await
                .map_err(GraphError::Database)?;

            // Delete the node
            plan_dag_nodes::Entity::delete_by_id(&dag_node.id)
                .exec(&self.db)
                .await
                .map_err(GraphError::Database)?;
        }

        // Delete the graph itself
        graphs::Entity::delete_by_id(graph.id)
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        Ok(())
    }

    pub async fn add_graph_node(
        &self,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        is_partition: bool,
        belongs_to: Option<String>,
        weight: Option<f64>,
        attrs: Option<serde_json::Value>,
    ) -> GraphResult<graph_nodes::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let now = chrono::Utc::now();
        let active_model = graph_nodes::ActiveModel {
            id: Set(node_id),
            graph_id: Set(graph_id),
            label: Set(label),
            layer: Set(layer),
            is_partition: Set(is_partition),
            belongs_to: Set(belongs_to),
            weight: Set(weight),
            attrs: Set(attrs),
            dataset_id: Set(None),
            comment: Set(None),
            created_at: Set(now),
        };

        let inserted = active_model
            .insert(&self.db)
            .await
            .map_err(GraphError::Database)?;
        Ok(inserted)
    }

    pub async fn delete_graph_node(
        &self,
        graph_id: i32,
        node_id: String,
    ) -> GraphResult<graph_nodes::Model> {
        use sea_orm::{EntityTrait, QueryFilter};

        let node = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .filter(graph_nodes::Column::Id.eq(&node_id))
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or_else(|| GraphError::InvalidNode(node_id.clone()))?;

        // Delete the node
        GraphNodes::delete_many()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .filter(graph_nodes::Column::Id.eq(&node_id))
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        Ok(node)
    }

    pub async fn update_graph_node(
        &self,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attrs: Option<serde_json::Value>,
        belongs_to: Option<Option<String>>,
    ) -> GraphResult<graph_nodes::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let node = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .filter(graph_nodes::Column::Id.eq(&node_id))
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or_else(|| GraphError::InvalidNode(node_id.clone()))?;

        let mut active_model: graph_nodes::ActiveModel = node.into();

        if let Some(label) = label {
            active_model.label = Set(Some(label));
        }

        if let Some(layer) = layer {
            active_model.layer = Set(if layer.is_empty() { None } else { Some(layer) });
        }

        if let Some(attrs) = attrs {
            active_model.attrs = Set(Some(attrs));
        }

        if let Some(belongs_to_value) = belongs_to {
            active_model.belongs_to = Set(belongs_to_value);
        }

        let updated = active_model
            .update(&self.db)
            .await
            .map_err(GraphError::Database)?;
        Ok(updated)
    }

    pub async fn update_layer_properties(
        &self,
        layer_id: i32,
        name: Option<String>,
        alias: Option<String>,
        properties: Option<serde_json::Value>,
    ) -> GraphResult<graph_layers::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let layer = Layers::find_by_id(layer_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or_else(|| GraphError::InvalidLayer(format!("Layer {} not found", layer_id)))?;

        let mut active_model: graph_layers::ActiveModel = layer.into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }

        if let Some(alias_value) = alias {
            active_model.alias = Set(Self::normalize_alias(Some(alias_value)));
        }

        if let Some(properties) = properties {
            let properties_string = serde_json::to_string(&properties).map_err(|e| {
                GraphError::Validation(format!("Invalid layer properties JSON: {}", e))
            })?;
            active_model.properties = Set(Some(properties_string));
        }

        let updated = active_model
            .update(&self.db)
            .await
            .map_err(GraphError::Database)?;
        Ok(updated)
    }

    pub async fn list_project_layers(
        &self,
        project_id: i32,
    ) -> GraphResult<Vec<project_layers::Model>> {
        // Ensure palette exists by seeding from layer datasets if empty
        self.seed_project_layers_if_empty(project_id).await?;

        let layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .order_by_asc(project_layers::Column::SourceDatasetId)
            .order_by_asc(project_layers::Column::LayerId)
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;
        Ok(layers)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_project_layer(
        &self,
        project_id: i32,
        layer_id: String,
        name: String,
        background_color: String,
        text_color: String,
        border_color: String,
        alias: Option<String>,
        source_dataset_id: Option<i32>,
        enabled: bool,
    ) -> GraphResult<project_layers::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let alias = Self::normalize_alias(alias);

        let mut existing_query = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id.clone()));
        existing_query = if let Some(dataset_id) = source_dataset_id {
            existing_query.filter(project_layers::Column::SourceDatasetId.eq(dataset_id))
        } else {
            existing_query.filter(project_layers::Column::SourceDatasetId.is_null())
        };
        let existing = existing_query
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?;

        let now = chrono::Utc::now();

        if let Some(model) = existing {
            let mut active: project_layers::ActiveModel = model.into();
            active.name = Set(name);
            active.background_color = Set(background_color);
            active.text_color = Set(text_color);
            active.border_color = Set(border_color);
            active.alias = Set(alias.clone());
            active.enabled = Set(enabled);
            active.updated_at = Set(now);

            active.update(&self.db).await.map_err(GraphError::Database)
        } else {
            let active = project_layers::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                project_id: Set(project_id),
                layer_id: Set(layer_id),
                name: Set(name),
                background_color: Set(background_color),
                text_color: Set(text_color),
                border_color: Set(border_color),
                alias: Set(alias),
                source_dataset_id: Set(source_dataset_id),
                enabled: Set(enabled),
                created_at: Set(now),
                updated_at: Set(now),
            };

            active.insert(&self.db).await.map_err(GraphError::Database)
        }
    }

    pub async fn delete_project_layer(
        &self,
        project_id: i32,
        layer_id: String,
        source_dataset_id: Option<i32>,
    ) -> GraphResult<u64> {
        use crate::database::entities::layer_aliases;

        // Find the layer first
        let mut query = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id.clone()));
        query = if let Some(dataset_id) = source_dataset_id {
            query.filter(project_layers::Column::SourceDatasetId.eq(dataset_id))
        } else {
            query.filter(project_layers::Column::SourceDatasetId.is_null())
        };
        let layer = query.one(&self.db).await.map_err(GraphError::Database)?;

        if let Some(layer_model) = layer {
            // Delete all aliases pointing to this layer
            layer_aliases::Entity::delete_many()
                .filter(layer_aliases::Column::TargetLayerId.eq(layer_model.id))
                .exec(&self.db)
                .await
                .map_err(GraphError::Database)?;

            // Now delete the layer itself
            let result = project_layers::Entity::delete_by_id(layer_model.id)
                .exec(&self.db)
                .await
                .map_err(GraphError::Database)?;

            Ok(result.rows_affected)
        } else {
            // Layer not found, return 0
            Ok(0)
        }
    }

    pub async fn set_layer_dataset_enabled(
        &self,
        project_id: i32,
        dataset_id: i32,
        enabled: bool,
    ) -> GraphResult<usize> {
        let updated = self
            .seed_layers_from_dataset(project_id, dataset_id, enabled)
            .await?;
        Ok(updated)
    }

    pub async fn reset_project_layers(&self, project_id: i32) -> GraphResult<()> {
        let now = chrono::Utc::now();

        layer_aliases::Entity::delete_many()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        project_layers::Entity::delete_many()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::SourceDatasetId.is_null())
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        project_layers::Entity::update_many()
            .col_expr(project_layers::Column::Enabled, Expr::value(false))
            .col_expr(project_layers::Column::UpdatedAt, Expr::value(now))
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::SourceDatasetId.is_not_null())
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        Ok(())
    }

    pub async fn missing_layers(&self, project_id: i32) -> GraphResult<Vec<String>> {
        use crate::database::entities::graphs::Entity as Graphs;

        // Get enabled project layers to build the known set. Disabled entries should be treated
        // as missing so that palette coverage reflects the active configuration.
        let enabled_layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;
        let mut enabled_layer_ids = HashSet::new();
        let mut known: HashSet<String> = HashSet::new();
        for layer in enabled_layers {
            enabled_layer_ids.insert(layer.id);
            known.insert(layer.layer_id);
        }

        // Also include aliased layer IDs in the known set
        // An aliased layer is not "missing" since it resolves to an existing layer
        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;
        for alias in aliases {
            if enabled_layer_ids.contains(&alias.target_layer_id) {
                known.insert(alias.alias_layer_id);
            }
        }

        let graph_ids: Vec<i32> = Graphs::find()
            .filter(crate::database::entities::graphs::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?
            .into_iter()
            .map(|g| g.id)
            .collect();

        if graph_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut missing = HashSet::new();

        let node_layers = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.is_in(graph_ids.clone()))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;
        for node in node_layers {
            if let Some(layer) = node.layer {
                let trimmed = layer.trim();
                if !trimmed.is_empty() && !known.contains(trimmed) {
                    missing.insert(trimmed.to_string());
                }
            }
        }

        let edge_layers = GraphEdges::find()
            .filter(graph_edges::Column::GraphId.is_in(graph_ids))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;
        for edge in edge_layers {
            if let Some(layer) = edge.layer {
                let trimmed = layer.trim();
                if !trimmed.is_empty() && !known.contains(trimmed) {
                    missing.insert(trimmed.to_string());
                }
            }
        }

        let mut missing_list: Vec<String> = missing.into_iter().collect();
        missing_list.sort();
        Ok(missing_list)
    }

    /// Resolve a layer by ID, including aliases
    /// Returns the layer with colors from the target if aliased, but using the requested layer_id
    #[allow(dead_code)]
    pub async fn resolve_layer(
        &self,
        project_id: i32,
        layer_id: &str,
    ) -> GraphResult<Option<Layer>> {
        // 1. Try to find direct match in project_layers
        let direct_layer = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id))
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?;

        if let Some(layer) = direct_layer {
            return Ok(Some(self.hydrate_project_layer(project_id, layer).await?));
        }

        // 2. Check if this layer_id is aliased
        let alias = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .filter(layer_aliases::Column::AliasLayerId.eq(layer_id))
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?;

        if let Some(alias_record) = alias {
            // Get the target layer
            let target_layer = project_layers::Entity::find_by_id(alias_record.target_layer_id)
                .one(&self.db)
                .await
                .map_err(GraphError::Database)?;

            if let Some(target) = target_layer {
                // Return layer using the alias ID but with target layer's colors
                let (background_color, text_color, border_color) =
                    self.resolve_layer_colors(project_id, &target).await?;
                return Ok(Some(Layer {
                    id: layer_id.to_string(), // Use the alias ID
                    label: target.name,
                    background_color,
                    text_color,
                    border_color,
                    alias: Some(target.layer_id),
                    dataset: target.source_dataset_id,
                    attributes: None,
                }));
            }
        }

        // 3. Layer not found and not aliased
        Ok(None)
    }

    /// Get all layers for a project, including aliases as separate layer entries
    /// Only returns enabled layers (or aliases pointing to enabled layers)
    pub async fn get_all_resolved_layers(&self, project_id: i32) -> GraphResult<Vec<Layer>> {
        let mut layers = Vec::new();

        // Get all direct layers (only enabled ones)
        let direct_layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        for layer in direct_layers {
            layers.push(self.hydrate_project_layer(project_id, layer).await?);
        }

        // Get all aliases and add them as separate layer entries
        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        for alias_record in aliases {
            // Get the target layer
            let target_layer = project_layers::Entity::find_by_id(alias_record.target_layer_id)
                .one(&self.db)
                .await
                .map_err(GraphError::Database)?;

            if let Some(target) = target_layer {
                // Only include if the target layer is enabled
                if target.enabled {
                    let (background_color, text_color, border_color) =
                        self.resolve_layer_colors(project_id, &target).await?;
                    layers.push(Layer {
                        id: alias_record.alias_layer_id,
                        label: target.name.clone(),
                        background_color,
                        text_color,
                        border_color,
                        alias: Some(target.layer_id.clone()),
                        dataset: target.source_dataset_id,
                        attributes: None,
                    });
                }
            }
        }

        Ok(layers)
    }

    async fn hydrate_project_layer(
        &self,
        project_id: i32,
        layer: project_layers::Model,
    ) -> GraphResult<Layer> {
        let (background_color, text_color, border_color) =
            self.resolve_layer_colors(project_id, &layer).await?;

        Ok(Layer {
            id: layer.layer_id,
            label: layer.name,
            background_color,
            text_color,
            border_color,
            alias: layer.alias,
            dataset: layer.source_dataset_id,
            attributes: None,
        })
    }

    async fn resolve_layer_colors(
        &self,
        project_id: i32,
        layer: &project_layers::Model,
    ) -> GraphResult<(String, String, String)> {
        let mut background_color = sanitize_hex(&layer.background_color, "FFFFFF");
        let mut text_color = sanitize_hex(&layer.text_color, "000000");
        let mut border_color = sanitize_hex(&layer.border_color, "000000");

        let mut current_alias = layer.alias.clone();
        let mut visited = HashSet::new();

        while let Some(alias_layer_id) = current_alias {
            if !visited.insert(alias_layer_id.clone()) {
                tracing::warn!(
                    "Detected alias cycle while resolving layer '{}' in project {}",
                    layer.layer_id,
                    project_id
                );
                break;
            }

            let Some(target_layer) = self
                .find_enabled_project_layer(project_id, &alias_layer_id)
                .await?
            else {
                tracing::warn!(
                    "Alias '{}' on layer '{}' has no enabled target in project {}",
                    alias_layer_id,
                    layer.layer_id,
                    project_id
                );
                break;
            };

            background_color = sanitize_hex(&target_layer.background_color, "FFFFFF");
            text_color = sanitize_hex(&target_layer.text_color, "000000");
            border_color = sanitize_hex(&target_layer.border_color, "000000");
            current_alias = target_layer.alias.clone();
        }

        Ok((background_color, text_color, border_color))
    }

    async fn find_enabled_project_layer(
        &self,
        project_id: i32,
        layer_id: &str,
    ) -> GraphResult<Option<project_layers::Model>> {
        let manual = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id))
            .filter(project_layers::Column::Enabled.eq(true))
            .filter(project_layers::Column::SourceDatasetId.is_null())
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?;

        if manual.is_some() {
            return Ok(manual);
        }

        let fallback = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id))
            .filter(project_layers::Column::Enabled.eq(true))
            .order_by_asc(project_layers::Column::SourceDatasetId)
            .order_by_asc(project_layers::Column::Id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?;

        Ok(fallback)
    }
}

fn sanitize_hex(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    let source = if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    };
    source.trim_start_matches('#').to_string()
}
