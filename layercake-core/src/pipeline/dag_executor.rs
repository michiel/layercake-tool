use anyhow::{anyhow, Context, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug_span, info, warn, Instrument};

use crate::database::entities::graph_data;
use crate::database::entities::graphs::{Column as GraphColumn, Entity as GraphEntity};
use crate::database::entities::{graphs, plan_dag_nodes, projections, sequence_contexts};
use crate::graphql::types::plan_dag::{
    config::StoryNodeConfig, FilterEvaluationContext, FilterNodeConfig, TransformNodeConfig,
};
use crate::pipeline::dag_context::DagExecutionContext;
use crate::pipeline::graph_data_persist_utils::{
    edges_to_graph_data_inputs, nodes_to_graph_data_inputs,
};
use crate::pipeline::{DatasourceImporter, GraphDataBuilder, MergeBuilder};
use crate::sequence_context::{build_story_context, SequenceStoryContext};
use crate::services::graph_service::GraphService;
use chrono::Utc;

/// DAG executor that processes nodes in topological order
pub struct DagExecutor {
    db: DatabaseConnection,
    dataset_importer: DatasourceImporter,
    graph_data_builder: GraphDataBuilder,
    merge_builder: MergeBuilder,
}

/// Options for creating or updating graph_data records originating from DAG nodes
struct GraphRecordOptions {
    metadata: Option<JsonValue>,
    source_type: String,
    file_format: Option<String>,
    origin: Option<String>,
    filename: Option<String>,
    file_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ProjectionNodeConfig {
    #[serde(rename = "projectionId")]
    projection_id: Option<i32>,
    name: Option<String>,
    #[serde(rename = "projectionType")]
    projection_type: Option<String>,
    #[serde(rename = "storyMode")]
    story_mode: Option<JsonValue>,
}

impl DagExecutor {
    pub fn new(db: DatabaseConnection) -> Self {
        let graph_data_service =
            std::sync::Arc::new(crate::services::GraphDataService::new(db.clone()));
        let dataset_importer = DatasourceImporter::new(db.clone(), graph_data_service.clone());
        let graph_data_builder = GraphDataBuilder::new(
            graph_data_service.clone(),
            std::sync::Arc::new(crate::services::LayerPaletteService::new(db.clone())),
        );
        let merge_builder = MergeBuilder::new(db.clone(), graph_data_service.clone());

        Self {
            db,
            dataset_importer,
            graph_data_builder,
            merge_builder,
        }
    }

    fn maybe_context(&self) -> Option<DagExecutionContext> {
        Some(DagExecutionContext::new())
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

                // Merge data from upstream sources using in-memory graph_data-first resolution
                let graph = self
                    .merge_builder
                    .merge_sources(
                        project_id,
                        plan_id,
                        node_id.to_string(),
                        node_name.clone(),
                        upstream_ids.clone(),
                    )
                    .await?;

                info!(
                    "MergeNode {} produced nodes:{}, edges:{}, layers:{}",
                    node_id,
                    graph.nodes.len(),
                    graph.edges.len(),
                    graph.layers.len()
                );

                // Mirror merge output into unified graph_data schema
                let merge_hash = self.compute_merge_hash(node_id, &upstream_ids, &graph)?;
                let metadata = Some(json!({
                    "upstreamNodes": upstream_ids,
                    "mergeSourceHash": merge_hash,
                }));

                // Create or update graph_data record for the merge node
                let graph_data_record = self
                    .get_or_create_graph_record(
                        project_id,
                        node_id,
                        &node_name,
                        GraphRecordOptions {
                            metadata: metadata.clone(),
                            source_type: "computed".to_string(),
                            file_format: None,
                            origin: None,
                            filename: None,
                            file_size: None,
                        },
                    )
                    .await?;

                // Mark processing
                self.graph_data_builder
                    .graph_data_service
                    .mark_processing(graph_data_record.id)
                    .await?;

                // Persist merge output to graph_data tables
                self.persist_graph_contents(graph_data_record.id, &graph)
                    .await?;

                // Update metadata, annotations, and completion state
                let graph_annotations_json = graph.annotations.as_ref().map(|s| {
                    serde_json::from_str::<JsonValue>(s).unwrap_or(JsonValue::String(s.clone()))
                });
                let mut active: crate::database::entities::graph_data::ActiveModel =
                    graph_data_record.into();
                active.metadata = Set(metadata);
                active.annotations = Set(graph_annotations_json);
                active.source_hash = Set(Some(merge_hash));
                active.computed_date = Set(Some(Utc::now()));
                active.node_count = Set(graph.nodes.len() as i32);
                active.edge_count = Set(graph.edges.len() as i32);
                active.status =
                    Set(crate::database::entities::graph_data::GraphDataStatus::Active.into());
                active.updated_at = Set(Utc::now());
                let _updated = active.update(&self.db).await?;

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_graph(node_id.to_string(), graph.clone());
                }
            }
            "GraphNode" => {
                // Stage 2: Route all new graphs through GraphDataBuilder
                let upstream_dag_node_ids = self.get_upstream_nodes(node_id, edges);

                // Resolve upstream DAG node IDs to graph_data IDs
                let mut upstream_graph_data_ids = Vec::new();
                for upstream_node_id in &upstream_dag_node_ids {
                    if let Some(graph_data) = self
                        .graph_data_builder
                        .graph_data_service
                        .get_by_dag_node(upstream_node_id)
                        .await?
                    {
                        upstream_graph_data_ids.push(graph_data.id);
                    } else {
                        // Upstream node doesn't have graph_data yet (legacy data)
                        // This is expected during transition period - log and skip
                        tracing::warn!(
                            "Upstream node {} has no graph_data entry, skipping for now. \
                             This is expected during schema migration.",
                            upstream_node_id
                        );
                    }
                }

                // Build graph via unified schema
                let _graph_data = self
                    .graph_data_builder
                    .build_graph(
                        project_id,
                        node_id.to_string(),
                        node_name,
                        upstream_graph_data_ids,
                    )
                    .await?;

                // TODO: Populate context with graph for downstream transforms
                // This requires reading back from graph_data, deferred for now
            }
            "ProjectionNode" => {
                self.execute_projection_node(
                    project_id,
                    node_id,
                    &node_name,
                    node,
                    edges,
                    context.as_deref_mut(),
                )
                .await?;
            }
            "GraphArtefactNode" | "TreeArtefactNode" | "OutputNode" | "Output" => {
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
                    project_id, node_id, &node_name, node, nodes, edges, context,
                )
                .await?;
            }
            "StoryNode" | "Story" | "story" => {
                let config: StoryNodeConfig = serde_json::from_str(&node.config_json)
                    .with_context(|| format!("Failed to parse story config for {}", node_id))?;
                let story_id = config
                    .story_id
                    .ok_or_else(|| anyhow!("Story node {} is not configured", node_id))?;
                let story_context = build_story_context(&self.db, project_id, story_id).await?;
                self.persist_story_context(project_id, node_id, story_id, &story_context)
                    .await?;
            }
            "SequenceArtefactNode" | "SequenceArtefact" | "sequence_artefact" => {
                // Rendering nodes run during export; nothing to execute eagerly
                return Ok(());
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
            "GraphNode" | "MergeNode" | "TransformNode" | "DataSetNode" | "FilterNode" => {}
            other => {
                return Err(anyhow!(
                    "TransformNode {} cannot consume from node type {} (node {})",
                    node_id,
                    other,
                    upstream_node_id
                ));
            }
        }

        // Check cache first
        let cached_graph = context
            .as_deref_mut()
            .and_then(|ctx| ctx.graph(upstream_node_id));

        let (mut graph, (_upstream_id, _is_from_graph_data)) = match cached_graph {
            Some(graph) => (graph, (0, false)), // Cached graph, metadata not needed for transforms
            None => {
                let (graph, metadata) = self
                    .load_graph_by_dag_node(project_id, upstream_node_id)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to load graph for upstream node {}",
                            upstream_node_id
                        )
                    })?;

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_graph(upstream_node_id.clone(), graph.clone());
                }
                (graph, metadata)
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
            upstream_node_id,
            &graph,
        )
        .await?;

        if let Some(ctx) = context {
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

        // Check cache first
        let cached_graph = context
            .as_deref_mut()
            .and_then(|ctx| ctx.graph(upstream_node_id));

        let (mut graph, (upstream_id, _is_from_graph_data)) = match cached_graph {
            Some(graph) => (graph, (0, false)), // Cached graph, metadata not needed for filters
            None => {
                let (graph, metadata) = self
                    .load_graph_by_dag_node(project_id, upstream_node_id)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to load graph for upstream node {}",
                            upstream_node_id
                        )
                    })?;

                if let Some(ctx) = context.as_deref_mut() {
                    ctx.set_graph(upstream_node_id.clone(), graph.clone());
                }
                (graph, metadata)
            }
        };

        config
            .apply_filters(
                &mut graph,
                &FilterEvaluationContext {
                    db: &self.db,
                    graph_id: upstream_id,
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
            upstream_node_id,
            &graph,
        )
        .await?;

        if let Some(ctx) = context {
            ctx.set_graph(node_id.to_string(), graph);
        }

        Ok(())
    }

    async fn execute_projection_node(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        node: &plan_dag_nodes::Model,
        edges: &[(String, String)],
        mut context: Option<&mut DagExecutionContext>,
    ) -> Result<()> {
        let config: ProjectionNodeConfig = serde_json::from_str(&node.config_json)
            .with_context(|| format!("Failed to parse projection config for node {}", node_id))?;

        let upstream_ids = self.get_upstream_nodes(node_id, edges);
        if upstream_ids.len() != 1 {
            return Err(anyhow!(
                "ProjectionNode {} expects exactly one upstream graph, found {}",
                node_id,
                upstream_ids.len()
            ));
        }
        let upstream_node_id = &upstream_ids[0];

        // Ensure upstream graph is materialized and determine its graph_data id
        let (graph, (graph_source_id, is_from_graph_data)) = self
            .load_graph_by_dag_node(project_id, upstream_node_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to load upstream graph for projection node {} (upstream {})",
                    node_id, upstream_node_id
                )
            })?;

        if !is_from_graph_data {
            tracing::warn!(
                "ProjectionNode {} upstream {} not yet migrated to graph_data; skipping projection materialization",
                node_id,
                upstream_node_id
            );
            if let Some(ctx) = context.as_deref_mut() {
                ctx.set_graph(node_id.to_string(), graph);
            }
            return Ok(());
        }

        // Use name from config if provided, otherwise fall back to node metadata label
        let projection_name = config
            .name
            .clone()
            .unwrap_or_else(|| node_name.to_string());

        let projection_type = config
            .projection_type
            .unwrap_or_else(|| "force3d".to_string());

        // Build settings_json from storyMode if present
        let settings_json = if let Some(story_mode) = config.story_mode {
            Some(json!({
                "storyMode": story_mode
            }))
        } else {
            None
        };

        let create_projection = |graph_id: i32,
                                 project_id: i32,
                                 name: &str,
                                 projection_type: String,
                                 settings_json: Option<JsonValue>|
         -> projections::ActiveModel {
            let now = Utc::now();
            projections::ActiveModel {
                project_id: Set(project_id),
                graph_id: Set(graph_id),
                name: Set(name.to_string()),
                projection_type: Set(projection_type),
                settings_json: Set(settings_json),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
        };

        let projection = if let Some(existing_id) = config.projection_id {
            let existing = projections::Entity::find_by_id(existing_id)
                .one(&self.db)
                .await?;

            match existing {
                Some(record) if record.project_id == project_id => {
                    let mut active: projections::ActiveModel = record.into();
                    active.graph_id = Set(graph_source_id);
                    active.name = Set(projection_name.clone());
                    active.projection_type = Set(projection_type.clone());
                    active.settings_json = Set(settings_json.clone());
                    active.updated_at = Set(Utc::now());
                    active.update(&self.db).await?
                }
                Some(record) => {
                    warn!(
                        "Projection {} belongs to project {} (expected {}), creating new for node {}",
                        existing_id, record.project_id, project_id, node_id
                    );
                    create_projection(
                        graph_source_id,
                        project_id,
                        &projection_name,
                        projection_type.clone(),
                        settings_json.clone(),
                    )
                    .insert(&self.db)
                    .await?
                }
                None => {
                    warn!(
                        "Projection {} not found for node {}; creating a new projection",
                        existing_id, node_id
                    );
                    create_projection(
                        graph_source_id,
                        project_id,
                        &projection_name,
                        projection_type.clone(),
                        settings_json.clone(),
                    )
                    .insert(&self.db)
                    .await?
                }
            }
        } else {
            // Reuse an existing projection for this node/name if present, otherwise create one
            let maybe_existing = projections::Entity::find()
                .filter(projections::Column::ProjectId.eq(project_id))
                .filter(projections::Column::GraphId.eq(graph_source_id))
                .filter(projections::Column::Name.eq(&projection_name))
                .one(&self.db)
                .await?;

            if let Some(existing) = maybe_existing {
                let mut active: projections::ActiveModel = existing.into();
                active.graph_id = Set(graph_source_id);
                active.name = Set(projection_name.clone());
                active.projection_type = Set(projection_type.clone());
                active.settings_json = Set(settings_json.clone());
                active.updated_at = Set(Utc::now());
                active.update(&self.db).await?
            } else {
                create_projection(
                    graph_source_id,
                    project_id,
                    &projection_name,
                    projection_type,
                    settings_json,
                )
                .insert(&self.db)
                .await?
            }
        };

        info!(
            "ProjectionNode {} linked to graph_data {} as projection {}",
            node_id, graph_source_id, projection.id
        );

        // Write projectionId back to node config if it wasn't already set
        if config.projection_id.is_none() || config.projection_id != Some(projection.id) {
            let mut updated_config = serde_json::from_str::<serde_json::Value>(&node.config_json)?;
            if let Some(obj) = updated_config.as_object_mut() {
                obj.insert("projectionId".to_string(), json!(projection.id));
            }

            let mut active_node: plan_dag_nodes::ActiveModel = node.clone().into();
            active_node.config_json = Set(serde_json::to_string(&updated_config)?);
            active_node.update(&self.db).await?;

            info!(
                "Updated ProjectionNode {} config with projectionId {}",
                node_id, projection.id
            );
        }

        if let Some(ctx) = context {
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

        let graph = match cached_graph {
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
                    annotations: None,
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
                            attributes: node_val.get("attributes").cloned(),
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
                            attributes: edge_val.get("attributes").cloned(),
                        };
                        graph.edges.push(edge);
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

        let graph_record = self
            .get_or_create_graph_record(
                project_id,
                node_id,
                node_name,
                GraphRecordOptions {
                    metadata: metadata.clone(),
                    source_type: "dataset".to_string(),
                    file_format: Some(data_set.file_format.clone()),
                    origin: Some(data_set.origin.clone()),
                    filename: Some(data_set.filename.clone()),
                    file_size: Some(data_set.file_size),
                },
            )
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
        self.graph_data_builder
            .graph_data_service
            .mark_processing(graph_record.id)
            .await?;

        // TODO: Publish execution status change for graph_data
        // #[cfg(feature = "graphql")]
        // crate::graphql::execution_events::publish_graph_data_status(
        //     &self.db,
        //     project_id,
        //     node_id,
        //     &graph_record,
        // )
        // .await;

        // Persist graph contents to graph_data tables (nodes, edges with layer info in attributes)
        self.persist_graph_contents(graph_record.id, &graph).await?;

        // Update to completed state with source hash
        self.graph_data_builder
            .graph_data_service
            .mark_complete(graph_record.id, dataset_hash)
            .await?;

        // TODO: Publish completion status for graph_data
        // #[cfg(feature = "graphql")]
        // crate::graphql::execution_events::publish_graph_data_status(
        //     &self.db,
        //     project_id,
        //     node_id,
        //     &graph_record,
        // )
        // .await;

        info!(
            "DataSetNode {} materialized from data_set {} with nodes:{}, edges:{}, layers:{}",
            node_id,
            data_set_id,
            graph.nodes.len(),
            graph.edges.len(),
            graph.layers.len(),
        );

        if let Some(ctx) = context {
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
        upstream_node_id: &str,
        graph: &crate::graph::Graph,
    ) -> Result<()> {
        // Try to load upstream graph metadata to include in hash computation
        // Try graph_data first, then fall back to legacy
        let upstream_graph_for_hash = if let Some(gd) = self
            .graph_data_builder
            .graph_data_service
            .get_by_dag_node(upstream_node_id)
            .await?
        {
            // Create a minimal graphs::Model compatible structure for hash computation
            // Note: This is a temporary solution during migration
            // graph_data doesn't have execution_state, but for hash computation we assume "completed"
            graphs::Model {
                id: gd.id,
                project_id,
                node_id: upstream_node_id.to_string(),
                name: gd.name,
                execution_state: "completed".to_string(),
                computed_date: gd.computed_date,
                source_hash: gd.source_hash,
                node_count: gd.node_count,
                edge_count: gd.edge_count,
                error_message: None, // graph_data doesn't track error_message separately
                metadata: gd.metadata,
                annotations: gd
                    .annotations
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                last_edit_sequence: gd.last_edit_sequence,
                has_pending_edits: gd.has_pending_edits,
                last_replay_at: gd.last_replay_at,
                created_at: gd.created_at,
                updated_at: gd.updated_at,
            }
        } else {
            // Fall back to legacy graphs table
            GraphEntity::find()
                .filter(GraphColumn::ProjectId.eq(project_id))
                .filter(GraphColumn::NodeId.eq(upstream_node_id))
                .one(&self.db)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "No graph metadata found for upstream node {} in either schema",
                        upstream_node_id
                    )
                })?
        };

        let metadata = Some(json!({
            "transforms": config.transforms,
            "upstreamGraphId": upstream_graph_for_hash.id,
            "upstreamNodeId": upstream_node_id,
        }));

        let graph_record = self
            .get_or_create_graph_record(
                project_id,
                node_id,
                node_name,
                GraphRecordOptions {
                    metadata: metadata.clone(),
                    source_type: "computed".to_string(),
                    file_format: None,
                    origin: None,
                    filename: None,
                    file_size: None,
                },
            )
            .await?;

        let transform_hash =
            self.compute_transform_hash(node_id, &upstream_graph_for_hash, config)?;

        // Compare annotations (graph_data has JsonValue, graph has String)
        let graph_annotations_json = graph
            .annotations
            .as_ref()
            .map(|s| serde_json::from_str::<JsonValue>(s).unwrap_or(JsonValue::String(s.clone())));
        let annotations_changed = graph_record.annotations != graph_annotations_json;
        let hash_matches = graph_record.source_hash.as_deref() == Some(&transform_hash);
        if hash_matches && !annotations_changed {
            return Ok(());
        }

        // Set to processing state
        self.graph_data_builder
            .graph_data_service
            .mark_processing(graph_record.id)
            .await?;

        // TODO: Publish execution status change for graph_data
        // #[cfg(feature = "graphql")]
        // crate::graphql::execution_events::publish_graph_data_status(
        //     &self.db,
        //     project_id,
        //     node_id,
        //     &graph_record,
        // )
        // .await;

        // Persist graph contents to graph_data tables
        self.persist_graph_contents(graph_record.id, graph).await?;

        // Update metadata, annotations, and mark as complete
        let mut active: graph_data::ActiveModel = graph_record.into();
        active.metadata = Set(metadata);
        active.annotations = Set(graph_annotations_json);
        active.source_hash = Set(Some(transform_hash));
        active.computed_date = Set(Some(Utc::now()));
        active.node_count = Set(graph.nodes.len() as i32);
        active.edge_count = Set(graph.edges.len() as i32);
        active.status = Set(graph_data::GraphDataStatus::Active.into());
        active.updated_at = Set(Utc::now());
        let _updated = active.update(&self.db).await?;

        // TODO: Publish completion status for graph_data
        // #[cfg(feature = "graphql")]
        // crate::graphql::execution_events::publish_graph_data_status(
        //     &self.db, project_id, node_id, &updated,
        // )
        // .await;

        Ok(())
    }

    async fn persist_filtered_graph(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        config: &FilterNodeConfig,
        upstream_node_id: &str,
        graph: &crate::graph::Graph,
    ) -> Result<()> {
        // Try to load upstream graph metadata to include in hash computation
        // Try graph_data first, then fall back to legacy
        let upstream_graph_for_hash = if let Some(gd) = self
            .graph_data_builder
            .graph_data_service
            .get_by_dag_node(upstream_node_id)
            .await?
        {
            // Create a minimal graphs::Model compatible structure for hash computation
            // graph_data doesn't have execution_state, but for hash computation we assume "completed"
            graphs::Model {
                id: gd.id,
                project_id,
                node_id: upstream_node_id.to_string(),
                name: gd.name,
                execution_state: "completed".to_string(),
                computed_date: gd.computed_date,
                source_hash: gd.source_hash,
                node_count: gd.node_count,
                edge_count: gd.edge_count,
                error_message: None, // graph_data doesn't track error_message separately
                metadata: gd.metadata,
                annotations: gd
                    .annotations
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                last_edit_sequence: gd.last_edit_sequence,
                has_pending_edits: gd.has_pending_edits,
                last_replay_at: gd.last_replay_at,
                created_at: gd.created_at,
                updated_at: gd.updated_at,
            }
        } else {
            // Fall back to legacy graphs table
            GraphEntity::find()
                .filter(GraphColumn::ProjectId.eq(project_id))
                .filter(GraphColumn::NodeId.eq(upstream_node_id))
                .one(&self.db)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "No graph metadata found for upstream node {} in either schema",
                        upstream_node_id
                    )
                })?
        };

        let metadata = Some(json!({
            "query": config.query,
            "upstreamGraphId": upstream_graph_for_hash.id,
            "upstreamNodeId": upstream_node_id,
        }));

        let graph_record = self
            .get_or_create_graph_record(
                project_id,
                node_id,
                node_name,
                GraphRecordOptions {
                    metadata: metadata.clone(),
                    source_type: "computed".to_string(),
                    file_format: None,
                    origin: None,
                    filename: None,
                    file_size: None,
                },
            )
            .await?;

        let filter_hash = self.compute_filter_hash(node_id, &upstream_graph_for_hash, config)?;

        // Normalize annotations for comparison (graph_data stores JSON, graph stores string)
        let graph_annotations_json = graph
            .annotations
            .as_ref()
            .map(|s| serde_json::from_str::<JsonValue>(s).unwrap_or(JsonValue::String(s.clone())));
        let annotations_changed = graph_record.annotations != graph_annotations_json;
        let hash_matches = graph_record.source_hash.as_deref() == Some(&filter_hash);

        if hash_matches && !annotations_changed {
            return Ok(());
        }

        // Set to processing state in graph_data
        self.graph_data_builder
            .graph_data_service
            .mark_processing(graph_record.id)
            .await?;

        self.persist_graph_contents(graph_record.id, graph).await?;

        // Update metadata, annotations, and mark as complete
        let mut active: graph_data::ActiveModel = graph_record.into();
        active.metadata = Set(metadata);
        active.annotations = Set(graph_annotations_json);
        active.source_hash = Set(Some(filter_hash));
        active.computed_date = Set(Some(Utc::now()));
        active.node_count = Set(graph.nodes.len() as i32);
        active.edge_count = Set(graph.edges.len() as i32);
        active.status = Set(graph_data::GraphDataStatus::Active.into());
        active.updated_at = Set(Utc::now());
        let _updated = active.update(&self.db).await?;

        // TODO: Publish graph_data execution status change when graph_data events are wired
        // #[cfg(feature = "graphql")]
        // crate::graphql::execution_events::publish_graph_data_status(
        //     &self.db, project_id, node_id, &updated,
        // )
        // .await;

        Ok(())
    }

    /// Get or create a graph_data record for a computed graph (transform/filter output)
    /// Returns the graph_data record, creating it if it doesn't exist
    async fn get_or_create_graph_record(
        &self,
        project_id: i32,
        node_id: &str,
        node_name: &str,
        options: GraphRecordOptions,
    ) -> Result<graph_data::Model> {
        let GraphRecordOptions {
            metadata,
            source_type,
            file_format,
            origin,
            filename,
            file_size,
        } = options;

        // Query by dag_node_id
        if let Some(mut graph_data) = self
            .graph_data_builder
            .graph_data_service
            .get_by_dag_node(node_id)
            .await?
        {
            // Update if name or metadata changed
            let mut needs_update = false;
            let mut active: graph_data::ActiveModel = graph_data.clone().into();

            if graph_data.name != node_name {
                active.name = Set(node_name.to_string());
                needs_update = true;
            }

            if graph_data.metadata != metadata {
                active.metadata = Set(metadata.clone());
                needs_update = true;
            }

            if graph_data.source_type != source_type {
                active.source_type = Set(source_type.clone());
                needs_update = true;
            }

            if graph_data.file_format.as_deref() != file_format.as_deref() {
                active.file_format = Set(file_format.clone());
                needs_update = true;
            }

            if graph_data.origin.as_deref() != origin.as_deref() {
                active.origin = Set(origin.clone());
                needs_update = true;
            }

            if graph_data.filename.as_deref() != filename.as_deref() {
                active.filename = Set(filename.clone());
                needs_update = true;
            }

            if graph_data.file_size != file_size {
                active.file_size = Set(file_size);
                needs_update = true;
            }

            if needs_update {
                active.updated_at = Set(Utc::now());
                graph_data = active.update(&self.db).await?;
            }

            Ok(graph_data)
        } else {
            // Create new computed graph_data entry
            use crate::services::graph_data_service::GraphDataCreate;

            let graph_data = self
                .graph_data_builder
                .graph_data_service
                .create(GraphDataCreate {
                    project_id,
                    name: node_name.to_string(),
                    source_type: source_type.clone(),
                    dag_node_id: Some(node_id.to_string()),
                    file_format: file_format.clone(),
                    origin: origin.clone(),
                    filename: filename.clone(),
                    blob: None,
                    file_size,
                    processed_at: None,
                    source_hash: None,
                    computed_date: None,
                    last_edit_sequence: Some(0),
                    has_pending_edits: Some(false),
                    last_replay_at: None,
                    metadata,
                    annotations: None,
                    status: Some(graph_data::GraphDataStatus::Processing),
                })
                .await?;

            Ok(graph_data)
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

    fn compute_merge_hash(
        &self,
        node_id: &str,
        upstream_nodes: &[String],
        graph: &crate::graph::Graph,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        for id in upstream_nodes {
            hasher.update(id.as_bytes());
        }
        hasher.update(graph.nodes.len().to_le_bytes());
        hasher.update(graph.edges.len().to_le_bytes());
        for node in &graph.nodes {
            hasher.update(node.id.as_bytes());
            hasher.update(node.layer.as_bytes());
            hasher.update(node.weight.to_le_bytes());
        }
        for edge in &graph.edges {
            hasher.update(edge.id.as_bytes());
            hasher.update(edge.source.as_bytes());
            hasher.update(edge.target.as_bytes());
            hasher.update(edge.weight.to_le_bytes());
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Persist graph contents to graph_data schema
    /// Replaces nodes and edges for the given graph_data_id
    /// Stores layer information in node attributes
    async fn persist_graph_contents(
        &self,
        graph_data_id: i32,
        graph: &crate::graph::Graph,
    ) -> Result<()> {
        // Convert nodes and edges to graph_data format
        let node_inputs = nodes_to_graph_data_inputs(&graph.nodes);
        let edge_inputs = edges_to_graph_data_inputs(&graph.edges);

        // Replace nodes using GraphDataService (handles deletion + insertion + count update)
        self.graph_data_builder
            .graph_data_service
            .replace_nodes(graph_data_id, node_inputs)
            .await?;

        // Replace edges using GraphDataService (handles deletion + insertion + count update)
        self.graph_data_builder
            .graph_data_service
            .replace_edges(graph_data_id, edge_inputs)
            .await?;

        // Note: Layer information is now stored in graph_data_nodes.attributes
        // No separate layer table persistence needed
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
            let span = debug_span!(
                "dag_execute_node",
                project_id,
                plan_id,
                node_id = node_id.as_str()
            );
            self.execute_node(
                project_id,
                plan_id,
                &node_id,
                nodes,
                edges,
                context.as_mut(),
            )
            .instrument(span)
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
            let span = debug_span!(
                "dag_execute_affected_node",
                project_id,
                plan_id,
                node_id = node_id.as_str(),
                changed_node = changed_node_id
            );
            self.execute_node(
                project_id,
                plan_id,
                &node_id,
                nodes,
                edges,
                context.as_mut(),
            )
            .instrument(span)
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
            let span = debug_span!(
                "dag_execute_target_node",
                project_id,
                plan_id,
                target_node = target_node_id,
                node_id = node_id.as_str()
            );
            self.execute_node(
                project_id,
                plan_id,
                &node_id,
                nodes,
                edges,
                context.as_mut(),
            )
            .instrument(span)
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

impl DagExecutor {
    /// Load a graph by DAG node ID, trying graph_data first, then falling back to legacy graphs table.
    /// Returns the Graph struct and a metadata tuple (graph_data_id OR legacy_graph_id, is_from_graph_data)
    async fn load_graph_by_dag_node(
        &self,
        project_id: i32,
        dag_node_id: &str,
    ) -> Result<(crate::graph::Graph, (i32, bool))> {
        // Try graph_data first (new schema)
        if let Some(graph_data) = self
            .graph_data_builder
            .graph_data_service
            .get_by_dag_node(dag_node_id)
            .await?
        {
            tracing::debug!(
                "Loading graph for node {} from graph_data (id: {})",
                dag_node_id,
                graph_data.id
            );

            let (gd, nodes, edges) = self
                .graph_data_builder
                .graph_data_service
                .load_full(graph_data.id)
                .await
                .with_context(|| {
                    format!(
                        "Failed to load graph_data {} for node {}",
                        graph_data.id, dag_node_id
                    )
                })?;

            // Convert graph_data entities to Graph struct
            use crate::graph::{Edge, Layer, Node};
            use std::collections::HashMap;

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

            // Extract layers from nodes (similar to projection.rs:144-194)
            let mut layer_map: HashMap<String, Layer> = HashMap::new();
            for node in &graph_nodes {
                if !layer_map.contains_key(&node.layer) {
                    // Extract layer styling from node attributes
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
                            label: node.layer.clone(), // Use layer ID as label by default
                            background_color: bg_color.unwrap_or_else(|| "#FFFFFF".to_string()),
                            text_color: text_color.unwrap_or_else(|| "#000000".to_string()),
                            border_color: border_color.unwrap_or_else(|| "#000000".to_string()),
                            alias: None,
                            dataset: node.dataset,
                            attributes: node.attributes.clone(),
                        },
                    );
                }
            }

            let layers: Vec<Layer> = layer_map.into_values().collect();

            let graph = crate::graph::Graph {
                name: gd.name.clone(),
                nodes: graph_nodes,
                edges: graph_edges,
                layers,
                annotations: gd
                    .annotations
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
            };

            return Ok((graph, (gd.id, true)));
        }

        // Fall back to legacy graphs table
        tracing::debug!(
            "Graph for node {} not found in graph_data, trying legacy graphs table",
            dag_node_id
        );

        let legacy_graph = GraphEntity::find()
            .filter(GraphColumn::ProjectId.eq(project_id))
            .filter(GraphColumn::NodeId.eq(dag_node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "No graph output found for node {} in either graph_data or legacy graphs table",
                    dag_node_id
                )
            })?;

        let graph_service = GraphService::new(self.db.clone());
        let graph = graph_service
            .build_graph_from_dag_graph(legacy_graph.id)
            .await
            .with_context(|| {
                format!(
                    "Failed to materialize graph from legacy table for node {}",
                    dag_node_id
                )
            })?;

        Ok((graph, (legacy_graph.id, false)))
    }

    async fn persist_story_context(
        &self,
        project_id: i32,
        node_id: &str,
        story_id: i32,
        context: &SequenceStoryContext,
    ) -> Result<()> {
        use sequence_contexts::ActiveModel as SequenceContextActiveModel;
        use sequence_contexts::Column as SequenceContextColumn;
        use sequence_contexts::Entity as SequenceContextEntity;

        let serialized =
            serde_json::to_string(context).context("Failed to serialize sequence context")?;
        let existing = SequenceContextEntity::find()
            .filter(SequenceContextColumn::NodeId.eq(node_id))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            let mut active: SequenceContextActiveModel = model.into();
            active.context_json = Set(serialized);
            active.story_id = Set(story_id);
            active.project_id = Set(project_id);
            active.updated_at = Set(Utc::now().into());
            active.update(&self.db).await?;
        } else {
            let active = SequenceContextActiveModel {
                project_id: Set(project_id),
                node_id: Set(node_id.to_string()),
                story_id: Set(story_id),
                context_json: Set(serialized),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
                ..Default::default()
            };
            active.insert(&self.db).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
