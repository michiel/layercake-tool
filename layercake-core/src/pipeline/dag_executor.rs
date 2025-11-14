use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::info;

use crate::database::entities::graphs::ActiveModel as GraphActiveModel;
use crate::database::entities::graphs::{Column as GraphColumn, Entity as GraphEntity};
use crate::database::entities::{graph_edges, graph_nodes, graphs, plan_dag_nodes, ExecutionState};
use crate::graphql::types::plan_dag::{
    FilterEvaluationContext, FilterNodeConfig, TransformNodeConfig,
};
use crate::pipeline::dag_context::DagExecutionContext;
use crate::pipeline::layer_operations::insert_layers_to_db;
use crate::pipeline::types::LayerData;
use crate::pipeline::{DatasourceImporter, GraphBuilder, MergeBuilder};
use crate::services::graph_service::GraphService;

/// DAG executor that processes nodes in topological order
pub struct DagExecutor {
    db: DatabaseConnection,
    dataset_importer: DatasourceImporter,
    graph_builder: GraphBuilder,
    merge_builder: MergeBuilder,
    prefer_in_memory: bool,
}

impl DagExecutor {
    pub fn new(db: DatabaseConnection) -> Self {
        let dataset_importer = DatasourceImporter::new(db.clone());
        let graph_builder = GraphBuilder::new(db.clone());
        let merge_builder = MergeBuilder::new(db.clone());
        let prefer_in_memory = std::env::var("PIPELINE_IN_MEMORY")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Self {
            db,
            dataset_importer,
            graph_builder,
            merge_builder,
            prefer_in_memory,
        }
    }

    fn maybe_context(&self) -> Option<DagExecutionContext> {
        if self.prefer_in_memory {
            Some(DagExecutionContext::new())
        } else {
            None
        }
    }

    /// Execute a single node in the DAG
    /// This is called when a node is created, updated, or dependencies change
    pub async fn execute_node(
        &self,
        project_id: i32,
        plan_id: i32,
        node_id: &str,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)], // (source, target) pairs
        mut context: Option<&mut DagExecutionContext>,
    ) -> Result<()> {
        // Find the node
        let node = nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| anyhow!("Node not found: {}", node_id))?;

        // Parse node config to get details
        let config: serde_json::Value = serde_json::from_str(&node.config_json)?;
        let metadata: serde_json::Value = serde_json::from_str(&node.metadata_json)?;

        let node_name = metadata["label"].as_str().unwrap_or("Unnamed").to_string();

        match node.node_type.as_str() {
            "DataSetNode" => {
                // Check if this is a reference to an existing data_set (has dataSetId)
                // or a file import (has filePath)
                if config["dataSetId"].is_number() {
                    // DataSet references existing data_sets entry
                    // Create a graph entry from the data_set's graph_json
                    self.execute_dataset_reference_node(
                        project_id,
                        plan_id,
                        node_id,
                        &node_name,
                        &config,
                        context.as_deref_mut(),
                    )
                    .await?;
                } else if let Some(file_path) = config["filePath"].as_str() {
                    // Legacy path: import from file
                    self.dataset_importer
                        .import_dataset(
                            project_id,
                            node_id.to_string(),
                            node_name,
                            file_path.to_string(),
                        )
                        .await?;
                } else {
                    return Err(anyhow!(
                        "DataSet node must have either dataSetId or filePath in config"
                    ));
                }
            }
            "MergeNode" => {
                // Get upstream node IDs (can be DataSet, Graph, or Merge nodes)
                let upstream_ids = self.get_upstream_nodes(node_id, edges);

                // Merge data from upstream sources
                let graph_record = self
                    .merge_builder
                    .merge_sources(
                        project_id,
                        plan_id,
                        node_id.to_string(),
                        node_name,
                        upstream_ids,
                    )
                    .await?;

                let graph_service = GraphService::new(self.db.clone());
                let graph = graph_service
                    .build_graph_from_dag_graph(graph_record.id)
                    .await
                    .with_context(|| {
                        format!("Failed to materialize merged graph for node {}", node_id)
                    })?;

                info!(
                    "MergeNode {} produced nodes:{}, edges:{}, layers:{}",
                    node_id,
                    graph.nodes.len(),
                    graph.edges.len(),
                    graph.layers.len()
                );

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_graph(node_id.to_string(), graph.clone());
                }
            }
            "GraphNode" => {
                // Get upstream node IDs
                let upstream_ids = self.get_upstream_nodes(node_id, edges);

                // Build graph from upstream datasets (reads from data_sets table)
                let graph_record = self
                    .graph_builder
                    .build_graph(
                        project_id,
                        plan_id,
                        node_id.to_string(),
                        node_name,
                        upstream_ids,
                    )
                    .await?;

                if context.is_some() {
                    let graph_service = GraphService::new(self.db.clone());
                    if let Ok(graph) = graph_service
                        .build_graph_from_dag_graph(graph_record.id)
                        .await
                    {
                        if let Some(ctx) = context.as_deref_mut() {
                            ctx.set_graph(node_id.to_string(), graph);
                        }
                    }
                }
            }
            "GraphArtefactNode" | "TreeArtefactNode" => {
                // Output nodes deliver exports on demand; no proactive execution required
                return Ok(());
            }
            "TransformNode" => {
                self.execute_transform_node(
                    project_id,
                    node_id,
                    &node_name,
                    node,
                    nodes,
                    edges,
                    context.as_deref_mut(),
                )
                .await?;
            }
            "FilterNode" => {
                self.execute_filter_node(
                    project_id,
                    node_id,
                    &node_name,
                    node,
                    nodes,
                    edges,
                    context.as_deref_mut(),
                )
                .await?;
            }
            _ => {
                return Err(anyhow!("Unknown node type: {}", node.node_type));
            }
        }

        Ok(())
    }

    async fn execute_transform_node(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        node: &plan_dag_nodes::Model,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
        mut context: Option<&mut DagExecutionContext>,
    ) -> Result<()> {
        let config: TransformNodeConfig = serde_json::from_str(&node.config_json)
            .with_context(|| format!("Failed to parse transform config for node {}", node_id))?;

        let upstream_ids = self.get_upstream_nodes(node_id, edges);
        if upstream_ids.len() != 1 {
            return Err(anyhow!(
                "TransformNode {} expects exactly one upstream graph, found {}",
                node_id,
                upstream_ids.len()
            ));
        }
        let upstream_node_id = &upstream_ids[0];

        let upstream_node = nodes
            .iter()
            .find(|n| &n.id == upstream_node_id)
            .ok_or_else(|| anyhow!("Upstream node {} not found", upstream_node_id))?;

        match upstream_node.node_type.as_str() {
            "GraphNode" | "MergeNode" | "TransformNode" | "DataSetNode" => {}
            other => {
                return Err(anyhow!(
                    "TransformNode {} cannot consume from node type {} (node {})",
                    node_id,
                    other,
                    upstream_node_id
                ));
            }
        }

        let upstream_graph = GraphEntity::find()
            .filter(GraphColumn::ProjectId.eq(project_id))
            .filter(GraphColumn::NodeId.eq(upstream_node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "No graph output found for upstream node {}",
                    upstream_node_id
                )
            })?;

        let cached_graph = context
            .as_deref_mut()
            .and_then(|ctx| ctx.graph(upstream_node_id));

        let graph = match cached_graph {
            Some(graph) => graph,
            None => {
                let graph_service = GraphService::new(self.db.clone());
                let graph = graph_service
                    .build_graph_from_dag_graph(upstream_graph.id)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to materialize graph for upstream node {}",
                            upstream_node_id
                        )
                    })?;

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_graph(upstream_node_id.clone(), graph.clone());
                }
                graph
            }
        };

        config
            .apply_transforms(&mut graph)
            .with_context(|| format!("Failed to execute transforms for node {}", node_id))?;

        graph.name = node_name.to_string();
        info!(
            "TransformNode {} produced nodes:{}, edges:{}, layers:{}",
            node_id,
            graph.nodes.len(),
            graph.edges.len(),
            graph.layers.len(),
        );

        self.persist_transformed_graph(
            project_id,
            node_id,
            node_name,
            &config,
            &upstream_graph,
            &graph,
        )
        .await?;

        if let Some(ctx) = context.as_deref_mut() {
            ctx.set_graph(node_id.to_string(), graph);
        }

        Ok(())
    }

    async fn execute_filter_node(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        node: &plan_dag_nodes::Model,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
        mut context: Option<&mut DagExecutionContext>,
    ) -> Result<()> {
        let config: FilterNodeConfig = serde_json::from_str(&node.config_json)
            .with_context(|| format!("Failed to parse filter config for node {}", node_id))?;

        let upstream_ids = self.get_upstream_nodes(node_id, edges);
        if upstream_ids.len() != 1 {
            return Err(anyhow!(
                "FilterNode {} expects exactly one upstream graph, found {}",
                node_id,
                upstream_ids.len()
            ));
        }
        let upstream_node_id = &upstream_ids[0];

        let upstream_node = nodes
            .iter()
            .find(|n| &n.id == upstream_node_id)
            .ok_or_else(|| anyhow!("Upstream node {} not found", upstream_node_id))?;

        match upstream_node.node_type.as_str() {
            "GraphNode" | "MergeNode" | "TransformNode" | "FilterNode" | "DataSetNode" => {}
            other => {
                return Err(anyhow!(
                    "FilterNode {} cannot consume from node type {} (node {})",
                    node_id,
                    other,
                    upstream_node_id
                ));
            }
        }

        let upstream_graph = GraphEntity::find()
            .filter(GraphColumn::ProjectId.eq(project_id))
            .filter(GraphColumn::NodeId.eq(upstream_node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "No graph output found for upstream node {}",
                    upstream_node_id
                )
            })?;

        let cached_graph = context
            .as_deref_mut()
            .and_then(|ctx| ctx.graph(upstream_node_id));

        let mut graph = match cached_graph {
            Some(graph) => graph,
            None => {
                let graph_service = GraphService::new(self.db.clone());
                let graph = graph_service
                    .build_graph_from_dag_graph(upstream_graph.id)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to materialize graph for upstream node {}",
                            upstream_node_id
                        )
                    })?;

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_graph(upstream_node_id.clone(), graph.clone());
                }
                graph
            }
        };

        config
            .apply_filters(
                &mut graph,
                &FilterEvaluationContext {
                    db: &self.db,
                    graph_id: upstream_graph.id,
                },
            )
            .await
            .with_context(|| format!("Failed to execute filters for node {}", node_id))?;

        graph.name = node_name.to_string();
        info!(
            "FilterNode {} produced nodes:{}, edges:{}, layers:{}",
            node_id,
            graph.nodes.len(),
            graph.edges.len(),
            graph.layers.len(),
        );

        self.persist_filtered_graph(
            project_id,
            node_id,
            node_name,
            &config,
            &upstream_graph,
            &graph,
        )
        .await?;

        if let Some(ctx) = context.as_deref_mut() {
            ctx.set_graph(node_id.to_string(), graph);
        }

        Ok(())
    }

    async fn execute_dataset_reference_node(
        &self,
        project_id: i32,
        _plan_id: i32,
        node_id: &str,
        node_name: &str,
        config: &JsonValue,
        mut context: Option<&mut DagExecutionContext>,
    ) -> Result<()> {
        use crate::database::entities::data_sets::{
            Column as DataSetColumn, Entity as DataSetEntity,
        };

        // Get the referenced data_set
        let data_set_id = config["dataSetId"]
            .as_i64()
            .ok_or_else(|| anyhow!("dataSetId not found in config"))?
            as i32;

        let data_set = DataSetEntity::find()
            .filter(DataSetColumn::Id.eq(data_set_id))
            .filter(DataSetColumn::ProjectId.eq(project_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Data set {} not found", data_set_id))?;

        let cached_graph = context
            .as_deref_mut()
            .and_then(|ctx| ctx.dataset_graph(data_set_id));

        let mut graph = match cached_graph {
            Some(graph) => graph,
            None => {
                // Parse the graph_json from the data_set
                let graph_data: JsonValue = serde_json::from_str(&data_set.graph_json)
                    .with_context(|| {
                        format!("Failed to parse graph_json for data_set {}", data_set_id)
                    })?;

                // Convert to Graph structure
                let mut graph = crate::graph::Graph {
                    name: data_set.name.clone(),
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    layers: Vec::new(),
                };

                // Extract nodes
                if let Some(nodes_array) = graph_data.get("nodes").and_then(|v| v.as_array()) {
                    for node_val in nodes_array {
                        let node = crate::graph::Node {
                            id: node_val["id"].as_str().unwrap_or("").to_string(),
                            label: node_val["label"].as_str().unwrap_or("").to_string(),
                            layer: node_val["layer"].as_str().unwrap_or("").to_string(),
                            weight: node_val["weight"].as_i64().unwrap_or(1) as i32,
                            is_partition: node_val["is_partition"].as_bool().unwrap_or(false),
                            belongs_to: node_val["belongs_to"]
                                .as_str()
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                            comment: node_val["comment"].as_str().map(|s| s.to_string()),
                            dataset: None,
                        };
                        graph.nodes.push(node);
                    }
                }

                // Extract edges
                if let Some(edges_array) = graph_data.get("edges").and_then(|v| v.as_array()) {
                    for edge_val in edges_array {
                        let edge = crate::graph::Edge {
                            id: edge_val["id"].as_str().unwrap_or("").to_string(),
                            source: edge_val["source"].as_str().unwrap_or("").to_string(),
                            target: edge_val["target"].as_str().unwrap_or("").to_string(),
                            label: edge_val["label"].as_str().unwrap_or("").to_string(),
                            layer: edge_val["layer"].as_str().unwrap_or("").to_string(),
                            weight: edge_val["weight"].as_i64().unwrap_or(1) as i32,
                            comment: edge_val["comment"].as_str().map(|s| s.to_string()),
                            dataset: None,
                        };
                        graph.edges.push(edge);
                    }
                }

                // Extract layers (note: it's "graph_layers" in JSON format from data_sets)
                // Try both "graph_layers" and "layers" for compatibility
                let layers_array = graph_data
                    .get("graph_layers")
                    .or_else(|| graph_data.get("layers"))
                    .and_then(|v| v.as_array());

                if let Some(layers_array) = layers_array {
                    for layer_val in layers_array {
                        let layer = crate::graph::Layer {
                            id: layer_val["id"].as_str().unwrap_or("").to_string(),
                            label: layer_val["label"].as_str().unwrap_or("").to_string(),
                            background_color: layer_val["background_color"]
                                .as_str()
                                .unwrap_or("#FFFFFF")
                                .to_string(),
                            text_color: layer_val["text_color"]
                                .as_str()
                                .unwrap_or("#000000")
                                .to_string(),
                            border_color: layer_val["border_color"]
                                .as_str()
                                .unwrap_or("#CCCCCC")
                                .to_string(),
                            dataset: None,
                        };
                        graph.layers.push(layer);
                    }
                }

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_dataset_graph(data_set_id, graph.clone());
                }

                graph
            }
        };

        // Create or get graph record
        let metadata = Some(json!({
            "dataSetId": data_set_id,
            "dataSetName": data_set.name,
        }));

        let mut graph_record = self
            .get_or_create_graph_record(project_id, node_id, node_name, metadata.clone())
            .await?;

        // Compute hash based on data_set content
        let dataset_hash = format!("{:x}", Sha256::digest(data_set.graph_json.as_bytes()));

        // Check if we need to recompute
        if graph_record.source_hash.as_deref() == Some(&dataset_hash) {
            // Already up to date
            if let Some(ctx) = context.as_deref_mut() {
                ctx.set_graph(node_id.to_string(), graph);
            }
            return Ok(());
        }

        // Set to processing state
        let mut active: GraphActiveModel = graph_record.clone().into();
        active = active.set_state(ExecutionState::Processing);
        graph_record = active.update(&self.db).await?;

        // Publish execution status change
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db,
            project_id,
            node_id,
            &graph_record,
        )
        .await;

        // Persist graph contents to graph_nodes, graph_edges, and layers tables
        self.persist_graph_contents(graph_record.id, &graph).await?;

        // Update to completed state
        let mut active: GraphActiveModel = graph_record.into();
        active = active.set_completed(
            dataset_hash,
            graph.nodes.len() as i32,
            graph.edges.len() as i32,
        );
        let graph_record = active.update(&self.db).await?;

        // Publish completion status
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db,
            project_id,
            node_id,
            &graph_record,
        )
        .await;

        info!(
            "DataSetNode {} materialized from data_set {} with nodes:{}, edges:{}, layers:{}",
            node_id,
            data_set_id,
            graph.nodes.len(),
            graph.edges.len(),
            graph.layers.len(),
        );

        if let Some(ctx) = context.as_deref_mut() {
            ctx.set_graph(node_id.to_string(), graph);
        }

        Ok(())
    }

    async fn persist_transformed_graph(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        config: &TransformNodeConfig,
        upstream_graph: &graphs::Model,
        graph: &crate::graph::Graph,
    ) -> Result<()> {
        let metadata = Some(json!({
            "transforms": config.transforms,
            "upstreamGraphId": upstream_graph.id,
        }));

        let mut graph_record = self
            .get_or_create_graph_record(project_id, node_id, node_name, metadata.clone())
            .await?;

        let transform_hash = self.compute_transform_hash(node_id, upstream_graph, config)?;

        if graph_record.source_hash.as_deref() == Some(&transform_hash) {
            return Ok(());
        }

        let mut active: GraphActiveModel = graph_record.clone().into();
        active = active.set_state(ExecutionState::Processing);
        graph_record = active.update(&self.db).await?;

        // Publish execution status change
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db,
            project_id,
            node_id,
            &graph_record,
        )
        .await;

        self.persist_graph_contents(graph_record.id, graph).await?;

        let mut active: GraphActiveModel = graph_record.into();
        active.metadata = Set(metadata);
        active = active.set_completed(
            transform_hash,
            graph.nodes.len() as i32,
            graph.edges.len() as i32,
        );
        let updated = active.update(&self.db).await?;

        // Publish execution status change
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db, project_id, node_id, &updated,
        )
        .await;

        Ok(())
    }

    async fn persist_filtered_graph(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        config: &FilterNodeConfig,
        upstream_graph: &graphs::Model,
        graph: &crate::graph::Graph,
    ) -> Result<()> {
        let metadata = Some(json!({
            "query": config.query,
            "upstreamGraphId": upstream_graph.id,
        }));

        let mut graph_record = self
            .get_or_create_graph_record(project_id, node_id, node_name, metadata.clone())
            .await?;

        let filter_hash = self.compute_filter_hash(node_id, upstream_graph, config)?;

        if graph_record.source_hash.as_deref() == Some(&filter_hash) {
            return Ok(());
        }

        let mut active: GraphActiveModel = graph_record.clone().into();
        active = active.set_state(ExecutionState::Processing);
        graph_record = active.update(&self.db).await?;

        // Publish execution status change
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db,
            project_id,
            node_id,
            &graph_record,
        )
        .await;

        self.persist_graph_contents(graph_record.id, graph).await?;

        let mut active: GraphActiveModel = graph_record.into();
        active.metadata = Set(metadata);
        active = active.set_completed(
            filter_hash,
            graph.nodes.len() as i32,
            graph.edges.len() as i32,
        );
        let updated = active.update(&self.db).await?;

        // Publish execution status change
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db, project_id, node_id, &updated,
        )
        .await;

        Ok(())
    }

    async fn get_or_create_graph_record(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        metadata: Option<JsonValue>,
    ) -> Result<graphs::Model> {
        if let Some(mut graph) = GraphEntity::find()
            .filter(GraphColumn::ProjectId.eq(project_id))
            .filter(GraphColumn::NodeId.eq(node_id))
            .one(&self.db)
            .await?
        {
            let mut needs_update = false;
            let mut active: GraphActiveModel = graph.clone().into();

            if graph.name != node_name {
                active.name = Set(node_name.to_string());
                needs_update = true;
            }

            if graph.metadata != metadata {
                active.metadata = Set(metadata.clone());
                needs_update = true;
            }

            if needs_update {
                active = active.set_updated_at();
                graph = active.update(&self.db).await?;
            }

            Ok(graph)
        } else {
            let graph = GraphActiveModel {
                project_id: Set(project_id),
                node_id: Set(node_id.to_string()),
                name: Set(node_name.to_string()),
                metadata: Set(metadata.clone()),
                ..GraphActiveModel::new()
            }
            .insert(&self.db)
            .await?;
            Ok(graph)
        }
    }

    fn compute_transform_hash(
        &self,
        node_id: &str,
        upstream_graph: &graphs::Model,
        config: &TransformNodeConfig,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        hasher.update(upstream_graph.id.to_le_bytes());
        hasher.update(upstream_graph.updated_at.timestamp_micros().to_le_bytes());
        if let Some(hash) = &upstream_graph.source_hash {
            hasher.update(hash.as_bytes());
        }
        let serialized = serde_json::to_vec(config)?;
        hasher.update(&serialized);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn compute_filter_hash(
        &self,
        node_id: &str,
        upstream_graph: &graphs::Model,
        config: &FilterNodeConfig,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        hasher.update(upstream_graph.id.to_le_bytes());
        hasher.update(upstream_graph.updated_at.timestamp_micros().to_le_bytes());
        if let Some(hash) = &upstream_graph.source_hash {
            hasher.update(hash.as_bytes());
        }
        let serialized = serde_json::to_vec(config)?;
        hasher.update(&serialized);
        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn persist_graph_contents(
        &self,
        graph_id: i32,
        graph: &crate::graph::Graph,
    ) -> Result<()> {
        self.clear_graph_data(graph_id).await?;

        for node in &graph.nodes {
            let attrs = node
                .comment
                .as_ref()
                .map(|comment| json!({ "comment": comment }));

            let model = graph_nodes::ActiveModel {
                id: Set(node.id.clone()),
                graph_id: Set(graph_id),
                label: Set(Some(node.label.clone())),
                layer: Set(Some(node.layer.clone())),
                weight: Set(Some(node.weight as f64)),
                is_partition: Set(node.is_partition),
                belongs_to: Set(node.belongs_to.clone()),
                dataset_id: Set(node.dataset),
                attrs: Set(attrs),
                comment: Set(node.comment.clone()),
                created_at: Set(Utc::now()),
            };

            model.insert(&self.db).await?;
        }

        for edge in &graph.edges {
            let attrs = edge
                .comment
                .as_ref()
                .map(|comment| json!({ "comment": comment }));

            let model = graph_edges::ActiveModel {
                id: Set(edge.id.clone()),
                graph_id: Set(graph_id),
                source: Set(edge.source.clone()),
                target: Set(edge.target.clone()),
                label: Set(Some(edge.label.clone())),
                layer: Set(Some(edge.layer.clone())),
                weight: Set(Some(edge.weight as f64)),
                dataset_id: Set(edge.dataset),
                attrs: Set(attrs),
                comment: Set(edge.comment.clone()),
                created_at: Set(Utc::now()),
            };

            model.insert(&self.db).await?;
        }

        let mut layer_map = HashMap::new();
        for layer in &graph.layers {
            let properties = JsonMap::new();
            // Only include other properties, not the color fields which are now first-class

            let properties_json = if properties.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&JsonValue::Object(properties))?)
            };

            let background_color = if layer.background_color.is_empty() {
                None
            } else {
                Some(layer.background_color.clone())
            };

            let text_color = if layer.text_color.is_empty() {
                None
            } else {
                Some(layer.text_color.clone())
            };

            let border_color = if layer.border_color.is_empty() {
                None
            } else {
                Some(layer.border_color.clone())
            };

            layer_map.insert(
                layer.id.clone(),
                LayerData {
                    name: layer.label.clone(),
                    background_color,
                    text_color,
                    border_color,
                    comment: None, // Layer struct doesn't have comment field
                    properties: properties_json,
                    dataset_id: layer.dataset,
                },
            );
        }

        insert_layers_to_db(&self.db, graph_id, layer_map).await?;

        Ok(())
    }

    async fn clear_graph_data(&self, graph_id: i32) -> Result<()> {
        use crate::database::entities::graph_edges::{Column as EdgeColumn, Entity as EdgeEntity};
        use crate::database::entities::graph_layers::{
            Column as LayerColumn, Entity as LayerEntity,
        };
        use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as NodeEntity};

        EdgeEntity::delete_many()
            .filter(EdgeColumn::GraphId.eq(graph_id))
            .exec(&self.db)
            .await?;

        NodeEntity::delete_many()
            .filter(NodeColumn::GraphId.eq(graph_id))
            .exec(&self.db)
            .await?;

        LayerEntity::delete_many()
            .filter(LayerColumn::GraphId.eq(graph_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Execute nodes in topological order
    /// This processes the entire DAG or a subgraph
    pub async fn execute_dag(
        &self,
        project_id: i32,
        plan_id: i32,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<()> {
        // Perform topological sort
        let sorted_nodes = self.topological_sort(nodes, edges)?;
        let mut context = self.maybe_context();

        // Execute nodes in order
        for node_id in sorted_nodes {
            self.execute_node(
                project_id,
                plan_id,
                &node_id,
                nodes,
                edges,
                context.as_mut().map(|ctx| ctx),
            )
            .await?;
        }

        Ok(())
    }

    /// Execute nodes affected by a change
    /// This identifies downstream nodes and executes them in order
    pub async fn execute_affected_nodes(
        &self,
        project_id: i32,
        plan_id: i32,
        changed_node_id: &str,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<()> {
        // Find all downstream nodes
        let affected = self.find_downstream_nodes(changed_node_id, nodes, edges);

        // Include the changed node itself
        let mut all_affected = vec![changed_node_id.to_string()];
        all_affected.extend(affected);

        // Filter nodes to only affected ones that require automatic execution.
        // Skip downstream ArtefactNodes since they are executed on-demand for previews/exports.
        let affected_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| {
                all_affected.contains(&n.id)
                    && (n.id == changed_node_id
                        || (n.node_type != "GraphArtefactNode"
                            && n.node_type != "TreeArtefactNode"))
            })
            .cloned()
            .collect();

        // Execute in topological order
        let sorted = self.topological_sort(&affected_nodes, edges)?;
        let mut context = self.maybe_context();

        for node_id in sorted {
            self.execute_node(
                project_id,
                plan_id,
                &node_id,
                nodes,
                edges,
                context.as_mut().map(|ctx| ctx),
            )
            .await?;
        }

        Ok(())
    }

    /// Perform topological sort on DAG nodes
    /// Returns node IDs in execution order
    fn topological_sort(
        &self,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node in nodes {
            in_degree.insert(node.id.clone(), 0);
            adj_list.insert(node.id.clone(), Vec::new());
        }

        // Build adjacency list and in-degree count
        for (source, target) in edges {
            // Only consider edges between nodes in our set
            if in_degree.contains_key(source) && in_degree.contains_key(target) {
                adj_list
                    .get_mut(source)
                    .expect("Source node must exist in adjacency list")
                    .push(target.clone());
                *in_degree
                    .get_mut(target)
                    .expect("Target node must exist in in-degree map") += 1;
            }
        }

        // Find nodes with no incoming edges
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id.clone());

            // Reduce in-degree for neighbors
            if let Some(neighbors) = adj_list.get(&node_id) {
                for neighbor in neighbors {
                    let deg = in_degree
                        .get_mut(neighbor)
                        .expect("Neighbor node must exist in in-degree map");
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Check for cycles
        if sorted.len() != nodes.len() {
            return Err(anyhow!("DAG contains a cycle"));
        }

        Ok(sorted)
    }

    /// Find all downstream nodes from a given node
    fn find_downstream_nodes(
        &self,
        start_node: &str,
        _nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node.to_string());

        while let Some(node_id) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            // Find outgoing edges
            for (source, target) in edges {
                if source == &node_id && !visited.contains(target) {
                    queue.push_back(target.clone());
                }
            }
        }

        // Remove start node from results
        visited.remove(start_node);
        visited.into_iter().collect()
    }

    /// Execute a node and all its upstream dependencies
    /// This ensures upstream nodes (like Merge) are executed before the target node
    pub async fn execute_with_dependencies(
        &self,
        project_id: i32,
        plan_id: i32,
        target_node_id: &str,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<()> {
        // Find all upstream nodes (ancestors)
        let upstream = self.find_upstream_nodes(target_node_id, edges);

        // Include the target node itself
        let mut all_nodes_to_execute = upstream;
        all_nodes_to_execute.push(target_node_id.to_string());

        // Filter nodes to only those we need to execute
        let nodes_to_execute: Vec<_> = nodes
            .iter()
            .filter(|n| all_nodes_to_execute.contains(&n.id))
            .cloned()
            .collect();

        // Execute in topological order
        let sorted = self.topological_sort(&nodes_to_execute, edges)?;
        let mut context = self.maybe_context();

        for node_id in sorted {
            self.execute_node(
                project_id,
                plan_id,
                &node_id,
                nodes,
                edges,
                context.as_mut().map(|ctx| ctx),
            )
            .await?;
        }

        Ok(())
    }

    /// Find all upstream nodes (ancestors) from a given node
    fn find_upstream_nodes(&self, start_node: &str, edges: &[(String, String)]) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node.to_string());

        while let Some(node_id) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            // Find incoming edges (upstream nodes)
            for (source, target) in edges {
                if target == &node_id && !visited.contains(source) {
                    queue.push_back(source.clone());
                }
            }
        }

        // Remove start node from results
        visited.remove(start_node);
        visited.into_iter().collect()
    }

    /// Get upstream node IDs for a given node
    fn get_upstream_nodes(&self, node_id: &str, edges: &[(String, String)]) -> Vec<String> {
        edges
            .iter()
            .filter(|(_, target)| target == node_id)
            .map(|(source, _)| source.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        let executor = DagExecutor::new(DatabaseConnection::default());

        // Create test nodes
        let nodes = vec![
            plan_dag_nodes::Model {
                id: "A".to_string(),
                plan_id: 1,
                node_type: "DataSet".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "B".to_string(),
                plan_id: 1,
                node_type: "DataSet".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "C".to_string(),
                plan_id: 1,
                node_type: "Graph".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        // Edges: A -> C, B -> C
        let edges = vec![
            ("A".to_string(), "C".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let sorted = executor
            .topological_sort(&nodes, &edges)
            .expect("Topological sort should succeed in test");

        // C should come after both A and B
        let c_pos = sorted
            .iter()
            .position(|id| id == "C")
            .expect("Node C should be in sorted result");
        let a_pos = sorted
            .iter()
            .position(|id| id == "A")
            .expect("Node A should be in sorted result");
        let b_pos = sorted
            .iter()
            .position(|id| id == "B")
            .expect("Node B should be in sorted result");

        assert!(c_pos > a_pos);
        assert!(c_pos > b_pos);
    }

    #[test]
    fn test_find_downstream_nodes() {
        let executor = DagExecutor::new(DatabaseConnection::default());

        let nodes = vec![
            plan_dag_nodes::Model {
                id: "A".to_string(),
                plan_id: 1,
                node_type: "DataSet".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "B".to_string(),
                plan_id: 1,
                node_type: "Graph".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "C".to_string(),
                plan_id: 1,
                node_type: "Graph".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        // Edges: A -> B -> C
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let downstream = executor.find_downstream_nodes("A", &nodes, &edges);

        assert!(downstream.contains(&"B".to_string()));
        assert!(downstream.contains(&"C".to_string()));
        assert_eq!(downstream.len(), 2);
    }
}
