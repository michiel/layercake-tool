use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, Set};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::database::entities::{
    nodes, edges, layers, graphs,
    nodes::Entity as Nodes,
    edges::Entity as Edges,
    layers::Entity as Layers,
    graphs::Entity as Graphs,
};
use crate::graph::{Graph, Node, Edge, Layer};

/// Graph statistics information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub nodes_per_layer: HashMap<String, usize>,
    pub edges_per_layer: HashMap<String, usize>,
    pub connected_components: usize,
    pub density: f64,
}

/// Graph comparison diff
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct GraphDiff {
    pub added_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub added_edges: Vec<(String, String)>,
    pub removed_edges: Vec<(String, String)>,
}

/// Graph validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct GraphValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

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

    /// Create a graph artifact and store it in the database
    pub async fn create_graph_artifact(
        &self,
        plan_id: i32,
        plan_node_id: &str,
        name: &str,
        graph: &Graph,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let graph_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // Serialize graph data to JSON
        let graph_data = serde_json::json!({
            "nodes": graph.nodes,
            "edges": graph.edges,
            "layers": graph.layers
        });

        let metadata_json = serde_json::to_value(metadata)?;

        // Create graph record
        let graph_active_model = graphs::ActiveModel {
            id: Set(graph_id.clone()),
            plan_id: Set(plan_id),
            plan_node_id: Set(plan_node_id.to_string()),
            name: Set(name.to_string()),
            description: Set(Some("Graph artifact created by DAG execution".to_string())),
            graph_data: Set(graph_data.to_string()),
            metadata: Set(Some(metadata_json.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        };

        graph_active_model.insert(&self.db).await?;
        Ok(graph_id)
    }

    /// Get a graph by its ID
    pub async fn get_graph_by_id(&self, graph_id: &str) -> Result<Graph> {
        let graph_model = Graphs::find()
            .filter(graphs::Column::Id.eq(graph_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found: {}", graph_id))?;

        // Parse graph data JSON
        let graph_data: serde_json::Value = serde_json::from_str(&graph_model.graph_data)?;
        
        let nodes: Vec<Node> = serde_json::from_value(
            graph_data.get("nodes").unwrap_or(&serde_json::json!([])).clone()
        )?;
        
        let edges: Vec<Edge> = serde_json::from_value(
            graph_data.get("edges").unwrap_or(&serde_json::json!([])).clone()
        )?;
        
        let layers: Vec<Layer> = serde_json::from_value(
            graph_data.get("layers").unwrap_or(&serde_json::json!([])).clone()
        )?;

        Ok(Graph {
            name: graph_model.name,
            nodes,
            edges,
            layers,
        })
    }

    /// Get all graphs for a plan
    pub async fn get_graphs_for_plan(&self, plan_id: i32) -> Result<Vec<graphs::Model>> {
        let graphs = Graphs::find()
            .filter(graphs::Column::PlanId.eq(plan_id))
            .all(&self.db)
            .await?;
        Ok(graphs)
    }

    /// Get graph by plan node ID
    pub async fn get_graph_by_plan_node(&self, plan_node_id: &str) -> Result<Option<Graph>> {
        if let Some(graph_model) = Graphs::find()
            .filter(graphs::Column::PlanNodeId.eq(plan_node_id))
            .one(&self.db)
            .await?
        {
            let graph = self.get_graph_by_id(&graph_model.id).await?;
            Ok(Some(graph))
        } else {
            Ok(None)
        }
    }

    /// Get graph artifact with metadata by ID
    pub async fn get_graph_artifact_with_metadata(&self, graph_id: &str) -> Result<(Graph, graphs::Model)> {
        let graph_model = Graphs::find()
            .filter(graphs::Column::Id.eq(graph_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found: {}", graph_id))?;

        let graph = self.get_graph_by_id(graph_id).await?;
        Ok((graph, graph_model))
    }

    /// Get all graph artifacts for a plan with metadata
    pub async fn get_plan_graph_artifacts(&self, plan_id: i32) -> Result<Vec<(Graph, graphs::Model)>> {
        let graph_models = Graphs::find()
            .filter(graphs::Column::PlanId.eq(plan_id))
            .all(&self.db)
            .await?;

        let mut results = Vec::new();
        for model in graph_models {
            let graph = self.get_graph_by_id(&model.id).await?;
            results.push((graph, model));
        }
        Ok(results)
    }

    /// Update graph artifact metadata
    pub async fn update_graph_metadata(
        &self,
        graph_id: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let metadata_json = serde_json::to_value(metadata)?;
        let now = chrono::Utc::now();

        let update_model = graphs::ActiveModel {
            id: Set(graph_id.to_string()),
            metadata: Set(Some(metadata_json.to_string())),
            updated_at: Set(now),
            ..Default::default()
        };

        update_model.update(&self.db).await?;
        Ok(())
    }

    /// Delete graph artifact
    pub async fn delete_graph_artifact(&self, graph_id: &str) -> Result<()> {
        Graphs::delete_many()
            .filter(graphs::Column::Id.eq(graph_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// Get graph statistics
    pub async fn get_graph_statistics(&self, graph_id: &str) -> Result<GraphStatistics> {
        let graph = self.get_graph_by_id(graph_id).await?;
        
        let node_count = graph.nodes.len();
        let edge_count = graph.edges.len();
        let layer_count = graph.layers.len();
        
        // Calculate nodes per layer
        let mut nodes_per_layer = HashMap::new();
        for node in &graph.nodes {
            *nodes_per_layer.entry(node.layer.clone()).or_insert(0) += 1;
        }

        // Calculate edges per layer
        let mut edges_per_layer = HashMap::new();
        for edge in &graph.edges {
            *edges_per_layer.entry(edge.layer.clone()).or_insert(0) += 1;
        }

        // Find connected components (simplified - just check if all nodes are connected)
        let connected_components = self.calculate_connected_components(&graph);
        
        Ok(GraphStatistics {
            node_count,
            edge_count,
            layer_count,
            nodes_per_layer,
            edges_per_layer,
            connected_components,
            density: self.calculate_density(&graph),
        })
    }

    /// Calculate graph density (edges / possible_edges)
    fn calculate_density(&self, graph: &Graph) -> f64 {
        let n = graph.nodes.len() as f64;
        let e = graph.edges.len() as f64;
        
        if n <= 1.0 {
            return 0.0;
        }
        
        // For directed graph: density = e / (n * (n-1))
        // For undirected graph: density = 2*e / (n * (n-1))
        // We'll assume directed for now
        e / (n * (n - 1.0))
    }

    /// Calculate connected components (simplified version)
    fn calculate_connected_components(&self, graph: &Graph) -> usize {
        let mut visited = std::collections::HashSet::new();
        let mut components = 0;

        // Build adjacency list
        let mut adj = HashMap::new();
        for node in &graph.nodes {
            adj.insert(node.id.clone(), Vec::new());
        }
        
        for edge in &graph.edges {
            if let Some(list) = adj.get_mut(&edge.source) {
                list.push(edge.target.clone());
            }
            if let Some(list) = adj.get_mut(&edge.target) {
                list.push(edge.source.clone());
            }
        }

        // DFS to find connected components
        for node in &graph.nodes {
            if !visited.contains(&node.id) {
                self.dfs_visit(&node.id, &adj, &mut visited);
                components += 1;
            }
        }

        components
    }

    /// DFS helper for connected components
    fn dfs_visit(
        &self,
        node_id: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        visited.insert(node_id.to_string());
        
        if let Some(neighbors) = adj.get(node_id) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_visit(neighbor, adj, visited);
                }
            }
        }
    }

    /// Compare two graphs and return diff information
    pub async fn compare_graphs(&self, graph_id1: &str, graph_id2: &str) -> Result<GraphDiff> {
        let graph1 = self.get_graph_by_id(graph_id1).await?;
        let graph2 = self.get_graph_by_id(graph_id2).await?;

        let nodes1: HashSet<_> = graph1.nodes.iter().map(|n| &n.id).collect();
        let nodes2: HashSet<_> = graph2.nodes.iter().map(|n| &n.id).collect();

        let edges1: HashSet<_> = graph1.edges.iter().map(|e| (&e.source, &e.target)).collect();
        let edges2: HashSet<_> = graph2.edges.iter().map(|e| (&e.source, &e.target)).collect();

        let added_nodes: Vec<_> = nodes2.difference(&nodes1).cloned().cloned().collect();
        let removed_nodes: Vec<_> = nodes1.difference(&nodes2).cloned().cloned().collect();

        let added_edges: Vec<_> = edges2.difference(&edges1).map(|(s, t)| ((*s).clone(), (*t).clone())).collect();
        let removed_edges: Vec<_> = edges1.difference(&edges2).map(|(s, t)| ((*s).clone(), (*t).clone())).collect();

        Ok(GraphDiff {
            added_nodes,
            removed_nodes,
            added_edges,
            removed_edges,
        })
    }

    /// Validate graph integrity
    pub async fn validate_graph(&self, graph_id: &str) -> Result<GraphValidationResult> {
        let graph = self.get_graph_by_id(graph_id).await?;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for dangling edges
        let node_ids: HashSet<_> = graph.nodes.iter().map(|n| &n.id).collect();
        for edge in &graph.edges {
            if !node_ids.contains(&edge.source) {
                errors.push(format!("Edge {} references non-existent source node: {}", edge.id, edge.source));
            }
            if !node_ids.contains(&edge.target) {
                errors.push(format!("Edge {} references non-existent target node: {}", edge.id, edge.target));
            }
        }

        // Check for duplicate node IDs
        let mut seen_nodes = HashSet::new();
        for node in &graph.nodes {
            if seen_nodes.contains(&node.id) {
                errors.push(format!("Duplicate node ID: {}", node.id));
            }
            seen_nodes.insert(&node.id);
        }

        // Check for duplicate edge IDs
        let mut seen_edges = HashSet::new();
        for edge in &graph.edges {
            if seen_edges.contains(&edge.id) {
                errors.push(format!("Duplicate edge ID: {}", edge.id));
            }
            seen_edges.insert(&edge.id);
        }

        // Check for self-loops
        for edge in &graph.edges {
            if edge.source == edge.target {
                warnings.push(format!("Self-loop detected: {}", edge.id));
            }
        }

        // Check for orphaned nodes (nodes with no edges)
        let connected_nodes: HashSet<_> = graph.edges.iter()
            .flat_map(|e| vec![&e.source, &e.target])
            .collect();
        
        for node in &graph.nodes {
            if !connected_nodes.contains(&node.id) {
                warnings.push(format!("Orphaned node (no edges): {}", node.id));
            }
        }

        let is_valid = errors.is_empty();
        
        Ok(GraphValidationResult {
            is_valid,
            errors,
            warnings,
        })
    }
}