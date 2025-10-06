use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use crate::database::entities::{
    nodes, edges, layers,
    nodes::Entity as Nodes,
    edges::Entity as Edges,
    layers::Entity as Layers,
    graph_nodes, graph_edges,
    graph_nodes::Entity as GraphNodes,
    graph_edges::Entity as GraphEdges,
};
use crate::graph::{Graph, Node, Edge, Layer};

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get database nodes for a project
    pub async fn get_nodes_for_project(&self, project_id: i32) -> Result<Vec<nodes::Model>> {
        let db_nodes = Nodes::find()
            .filter(nodes::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(db_nodes)
    }

    /// Get database edges for a project  
    pub async fn get_edges_for_project(&self, project_id: i32) -> Result<Vec<edges::Model>> {
        let db_edges = Edges::find()
            .filter(edges::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(db_edges)
    }

    /// Get database layers for a project
    pub async fn get_layers_for_project(&self, project_id: i32) -> Result<Vec<layers::Model>> {
        let db_layers = Layers::find()
            .filter(layers::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(db_layers)
    }

    /// Convert database entities to the existing Graph structure for use with export engine
    pub async fn build_graph_from_project(&self, project_id: i32) -> Result<Graph> {
        // Fetch all entities for the project
        let db_nodes = Nodes::find()
            .filter(nodes::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;

        let db_edges = Edges::find()
            .filter(edges::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;

        let db_layers = Layers::find()
            .filter(layers::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;

        // Convert database layers to Graph Layer structs
        let graph_layers: Vec<Layer> = db_layers
            .into_iter()
            .map(|db_layer| Layer {
                id: db_layer.layer_id,
                label: db_layer.name,
                background_color: db_layer.color.unwrap_or("FFFFFF".to_string()),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
            })
            .collect();

        // Convert database nodes to Graph Node structs
        let graph_nodes: Vec<Node> = db_nodes
            .into_iter()
            .map(|db_node| Node {
                id: db_node.node_id,
                label: db_node.label,
                layer: db_node.layer_id.unwrap_or("default".to_string()),
                is_partition: false, // Default for imported CSV data
                belongs_to: None,    // Default for imported CSV data
                weight: 1,           // Default weight
                comment: None,       // Default comment
            })
            .collect();

        // Convert database edges to Graph Edge structs
        let graph_edges: Vec<Edge> = db_edges
            .into_iter()
            .map(|db_edge| Edge {
                id: format!("{}_{}", db_edge.source_node_id, db_edge.target_node_id),
                source: db_edge.source_node_id,
                target: db_edge.target_node_id,
                label: "".to_string(),                // Default label
                layer: "default".to_string(),         // Default layer
                weight: 1,                            // Default weight
                comment: None,                        // Default comment
            })
            .collect();

        // Create the Graph with all entities
        Ok(Graph {
            name: format!("Project {}", project_id),
            nodes: graph_nodes,
            edges: graph_edges,
            layers: graph_layers,
        })
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
                belongs_to: None, // Could be extracted from attrs if needed
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
        use sea_orm::{Set, ActiveModelTrait};

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
        use sea_orm::{Set, ActiveModelTrait};

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

        graphs::Entity::delete_by_id(graph.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }
}