use crate::database::entities::{
    graph_edges, graph_edges::Entity as GraphEdges, graph_nodes, graph_nodes::Entity as GraphNodes,
    layers, layers::Entity as Layers, plan_dag_edges, plan_dag_nodes,
};
use crate::graph::{Edge, Graph, Layer, Node};
use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get database layers for a graph
    pub async fn get_layers_for_graph(&self, graph_id: i32) -> Result<Vec<layers::Model>> {
        let db_layers = Layers::find()
            .filter(layers::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;
        Ok(db_layers)
    }

    /// Build a Graph from a DAG-built graph in the graphs table
    pub async fn build_graph_from_dag_graph(&self, graph_id: i32) -> Result<Graph> {
        // Fetch the graph metadata
        use crate::database::entities::graphs::Entity as Graphs;
        let graph_meta = Graphs::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph {} not found", graph_id))?;

        // Fetch graph nodes
        let db_graph_nodes = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        // Fetch graph edges
        let db_graph_edges = GraphEdges::find()
            .filter(graph_edges::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        // Get unique layers from nodes
        use std::collections::HashSet;
        let unique_layers: HashSet<String> = db_graph_nodes
            .iter()
            .filter_map(|n| n.layer.clone())
            .collect();

        // Create default layers
        let graph_layers: Vec<Layer> = unique_layers
            .into_iter()
            .map(|layer_id| Layer {
                id: layer_id.clone(),
                label: layer_id,
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
            })
            .collect();

        // Convert to Graph Node structs
        let graph_nodes: Vec<Node> = db_graph_nodes
            .into_iter()
            .map(|db_node| Node {
                id: db_node.id,
                label: db_node.label.unwrap_or_default(),
                layer: db_node.layer.unwrap_or_else(|| "default".to_string()),
                is_partition: db_node.is_partition,
                belongs_to: db_node.belongs_to,
                weight: db_node.weight.unwrap_or(1.0) as i32,
                comment: None, // Could be extracted from attrs if needed
            })
            .collect();

        // Convert to Graph Edge structs
        let graph_edges: Vec<Edge> = db_graph_edges
            .into_iter()
            .map(|db_edge| Edge {
                id: db_edge.id.clone(),
                source: db_edge.source,
                target: db_edge.target,
                label: db_edge.label.unwrap_or_default(),
                layer: db_edge.layer.unwrap_or_else(|| "default".to_string()),
                weight: db_edge.weight.unwrap_or(1.0) as i32,
                comment: None,
            })
            .collect();

        Ok(Graph {
            name: graph_meta.name,
            nodes: graph_nodes,
            edges: graph_edges,
            layers: graph_layers,
        })
    }

    pub async fn create_graph(
        &self,
        project_id: i32,
        name: String,
    ) -> Result<crate::database::entities::graphs::Model> {
        use crate::database::entities::graphs;
        use sea_orm::{ActiveModelTrait, Set};

        // Generate a placeholder node_id for now
        let node_id = format!("graphnode_{}", uuid::Uuid::new_v4().to_string());

        let graph = graphs::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            node_id: Set(node_id),
            ..Default::default()
        };

        let graph = graph.insert(&self.db).await?;

        Ok(graph)
    }

    pub async fn update_graph(
        &self,
        id: i32,
        name: Option<String>,
    ) -> Result<crate::database::entities::graphs::Model> {
        use crate::database::entities::graphs;
        use sea_orm::{ActiveModelTrait, Set};

        let graph = graphs::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;

        let mut active_model: graphs::ActiveModel = graph.into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(&self.db).await?;
        Ok(updated)
    }

    pub async fn delete_graph(&self, id: i32) -> Result<()> {
        use crate::database::entities::graphs;

        let graph = graphs::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;

        // Find and delete all plan_dag_nodes that reference this graph by node_id
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::Id.eq(&graph.node_id))
            .all(&self.db)
            .await?;

        for dag_node in dag_nodes {
            // Delete connected edges first
            plan_dag_edges::Entity::delete_many()
                .filter(plan_dag_edges::Column::SourceNodeId.eq(&dag_node.id))
                .exec(&self.db)
                .await?;

            plan_dag_edges::Entity::delete_many()
                .filter(plan_dag_edges::Column::TargetNodeId.eq(&dag_node.id))
                .exec(&self.db)
                .await?;

            // Delete the node
            plan_dag_nodes::Entity::delete_by_id(&dag_node.id)
                .exec(&self.db)
                .await?;
        }

        // Delete the graph itself
        graphs::Entity::delete_by_id(graph.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn update_graph_node(
        &self,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attrs: Option<serde_json::Value>,
    ) -> Result<graph_nodes::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let node = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .filter(graph_nodes::Column::Id.eq(&node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph node not found"))?;

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

        let updated = active_model.update(&self.db).await?;
        Ok(updated)
    }

    pub async fn update_layer_properties(
        &self,
        layer_id: i32,
        name: Option<String>,
        properties: Option<serde_json::Value>,
    ) -> Result<layers::Model> {
        use sea_orm::{ActiveModelTrait, Set};

        let layer = Layers::find_by_id(layer_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Layer not found"))?;

        let mut active_model: layers::ActiveModel = layer.into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }

        if let Some(properties) = properties {
            let properties_string = serde_json::to_string(&properties)?;
            active_model.properties = Set(Some(properties_string));
        }

        let updated = active_model.update(&self.db).await?;
        Ok(updated)
    }
}
