use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use crate::database::entities::{
    nodes, edges, layers,
    nodes::Entity as Nodes,
    edges::Entity as Edges,
    layers::Entity as Layers,
};
use crate::graph::{Graph, Node, Edge, Layer};

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Convert database entities to the existing Graph structure for use with export engine
    pub async fn build_graph_for_project(&self, project_id: i32) -> Result<Graph> {
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
}