use crate::database::entities::{
    graph_edges, graph_edges::Entity as GraphEdges, graph_layers, graph_layers::Entity as Layers,
    graph_nodes, graph_nodes::Entity as GraphNodes, plan_dag_edges, plan_dag_nodes, project_layers,
};
use crate::errors::{GraphError, GraphResult};
use crate::graph::{Edge, Graph, Layer, Node};
use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashSet;

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn get_project_layers_palette(&self, project_id: i32) -> GraphResult<Vec<Layer>> {
        let db_layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::Enabled.eq(true))
            .order_by_asc(project_layers::Column::SourceDatasetId)
            .order_by_asc(project_layers::Column::Id)
            .all(&self.db)
            .await
            .map_err(GraphError::Database)?;

        let mut seen = HashSet::new();
        let mut palette = Vec::new();

        for db_layer in db_layers {
            if seen.contains(&db_layer.layer_id) {
                continue;
            }
            seen.insert(db_layer.layer_id.clone());
            palette.push(Layer {
                id: db_layer.layer_id,
                label: db_layer.name,
                background_color: db_layer.background_color,
                text_color: db_layer.text_color,
                border_color: db_layer.border_color,
                dataset: db_layer.source_dataset_id,
            });
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

    /// Build a Graph from a DAG-built graph in the graphs table
    pub async fn build_graph_from_dag_graph(&self, graph_id: i32) -> GraphResult<Graph> {
        // Fetch the graph metadata
        use crate::database::entities::graphs::Entity as Graphs;
        let graph_meta = Graphs::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or(GraphError::NotFound(graph_id))?;

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
        let graph_layers: Vec<Layer> = {
            let palette = self
                .get_project_layers_palette(graph_meta.project_id)
                .await?;
            if !palette.is_empty() {
                palette
            } else {
                let db_layers = Layers::find()
                    .filter(graph_layers::Column::GraphId.eq(graph_id))
                    .all(&self.db)
                    .await
                    .map_err(GraphError::Database)?;

                db_layers
                    .into_iter()
                    .map(|db_layer| Layer {
                        id: db_layer.layer_id,
                        label: db_layer.name,
                        background_color: db_layer
                            .background_color
                            .unwrap_or_else(|| "FFFFFF".to_string()),
                        text_color: db_layer.text_color.unwrap_or_else(|| "000000".to_string()),
                        border_color: db_layer
                            .border_color
                            .unwrap_or_else(|| "000000".to_string()),
                        dataset: db_layer.dataset_id,
                    })
                    .collect()
            }
        };

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

                Node {
                    id: db_node.id,
                    label,
                    layer,
                    is_partition: db_node.is_partition,
                    belongs_to: db_node.belongs_to,
                    weight: db_node.weight.unwrap_or(1.0) as i32,
                    comment: None, // Could be extracted from attrs if needed
                    dataset: db_node.dataset_id,
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
                }
            })
            .collect();

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
            layers: graph_layers,
            annotations: graph_meta.annotations,
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
        source_dataset_id: Option<i32>,
        enabled: bool,
    ) -> GraphResult<project_layers::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let existing = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id.clone()))
            .filter(project_layers::Column::SourceDatasetId.eq(source_dataset_id))
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
            active.enabled = Set(enabled);
            active.updated_at = Set(now);

            active.update(&self.db).await.map_err(GraphError::Database)
        } else {
            let active = project_layers::ActiveModel {
                id: Set(0),
                project_id: Set(project_id),
                layer_id: Set(layer_id),
                name: Set(name),
                background_color: Set(background_color),
                text_color: Set(text_color),
                border_color: Set(border_color),
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
        let result = project_layers::Entity::delete_many()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.eq(layer_id))
            .filter(project_layers::Column::SourceDatasetId.eq(source_dataset_id))
            .exec(&self.db)
            .await
            .map_err(GraphError::Database)?;

        Ok(result.rows_affected)
    }

    pub async fn set_layer_dataset_enabled(
        &self,
        project_id: i32,
        dataset_id: i32,
        enabled: bool,
    ) -> GraphResult<usize> {
        use crate::database::entities::data_sets;
        use sea_orm::Set;

        let data_set = data_sets::Entity::find_by_id(dataset_id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or_else(|| GraphError::Validation(format!("Dataset {} not found", dataset_id)))?;

        if data_set.project_id != project_id {
            return Err(GraphError::Validation(format!(
                "Dataset {} does not belong to project {}",
                dataset_id, project_id
            )));
        }

        let parsed_graph: crate::graph::Graph =
            serde_json::from_str(&data_set.graph_json).unwrap_or_default();
        let mut updated = 0;

        for layer in parsed_graph.layers {
            let _ = self
                .upsert_project_layer(
                    project_id,
                    layer.id.clone(),
                    layer.label.clone(),
                    layer.background_color.clone(),
                    layer.text_color.clone(),
                    layer.border_color.clone(),
                    Some(dataset_id),
                    enabled,
                )
                .await?;
            updated += 1;
        }

        // If no layers were found, still flip any existing rows for this dataset
        if updated == 0 {
            let now = chrono::Utc::now();
            let result = project_layers::Entity::update_many()
                .col_expr(project_layers::Column::Enabled, Expr::value(enabled))
                .col_expr(project_layers::Column::UpdatedAt, Expr::value(now))
                .filter(project_layers::Column::ProjectId.eq(project_id))
                .filter(project_layers::Column::SourceDatasetId.eq(Some(dataset_id)))
                .exec(&self.db)
                .await
                .map_err(GraphError::Database)?;
            updated = result.rows_affected as usize;
        }

        Ok(updated)
    }

    pub async fn missing_layers(&self, project_id: i32) -> GraphResult<Vec<String>> {
        use crate::database::entities::graphs::Entity as Graphs;

        let palette = self.get_project_layers_palette(project_id).await?;
        let mut known: HashSet<String> = palette.into_iter().map(|l| l.id).collect();

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
}
