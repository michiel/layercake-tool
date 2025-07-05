use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{Result, Context};
use sea_orm::*;
use tracing::{info, debug};
use serde::{Deserialize, Serialize};

use crate::database::entities::{nodes, edges, layers};

/// Service for advanced graph analysis operations
#[derive(Clone)]
pub struct GraphAnalysisService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub density: f64,
    pub average_degree: f64,
    pub max_degree: usize,
    pub min_degree: usize,
    pub connected_components: usize,
    pub diameter: Option<usize>,
    pub average_path_length: Option<f64>,
    pub clustering_coefficient: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub betweenness_centrality: f64,
    pub closeness_centrality: f64,
    pub eigenvector_centrality: f64,
    pub pagerank: f64,
    pub clustering_coefficient: f64,
    pub component_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    pub source: String,
    pub target: String,
    pub path: Vec<String>,
    pub length: usize,
    pub weight: f64,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityResult {
    pub components: Vec<ConnectedComponent>,
    pub component_count: usize,
    pub largest_component_size: usize,
    pub is_connected: bool,
    pub articulation_points: Vec<String>,
    pub bridges: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedComponent {
    pub id: usize,
    pub nodes: Vec<String>,
    pub size: usize,
    pub is_largest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDetectionResult {
    pub communities: Vec<Community>,
    pub modularity: f64,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: usize,
    pub nodes: Vec<String>,
    pub size: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerAnalysis {
    pub layer_id: String,
    pub layer_name: String,
    pub node_count: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub density: f64,
    pub connectivity_to_other_layers: HashMap<String, usize>,
}

impl GraphAnalysisService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get comprehensive graph metrics
    pub async fn analyze_graph(&self, project_id: i32) -> Result<GraphMetrics> {
        info!("Analyzing graph for project {}", project_id);

        let (nodes, edges) = self.load_graph_data(project_id).await?;
        let layers = self.load_layers(project_id).await?;
        
        let node_count = nodes.len();
        let edge_count = edges.len();
        let layer_count = layers.len();

        if node_count == 0 {
            return Ok(GraphMetrics {
                node_count: 0,
                edge_count: 0,
                layer_count,
                density: 0.0,
                average_degree: 0.0,
                max_degree: 0,
                min_degree: 0,
                connected_components: 0,
                diameter: None,
                average_path_length: None,
                clustering_coefficient: 0.0,
            });
        }

        // Build adjacency list
        let adjacency = self.build_adjacency_list(&nodes, &edges);
        
        // Calculate basic metrics
        let degrees: Vec<usize> = nodes
            .iter()
            .map(|node| adjacency.get(&node.node_id).map_or(0, |adj| adj.len()))
            .collect();

        let total_degree: usize = degrees.iter().sum();
        let max_degree = *degrees.iter().max().unwrap_or(&0);
        let min_degree = *degrees.iter().min().unwrap_or(&0);
        let average_degree = if node_count > 0 {
            total_degree as f64 / node_count as f64
        } else {
            0.0
        };

        // Calculate density
        let max_edges = if node_count > 1 {
            node_count * (node_count - 1) // Directed graph
        } else {
            0
        };
        let density = if max_edges > 0 {
            edge_count as f64 / max_edges as f64
        } else {
            0.0
        };

        // Find connected components
        let components = self.find_connected_components(&nodes, &adjacency);
        let connected_components = components.len();

        // Calculate diameter and average path length for largest component
        let (diameter, average_path_length) = if !components.is_empty() {
            let largest_component = components
                .iter()
                .max_by_key(|c| c.nodes.len())
                .unwrap();
            
            if largest_component.nodes.len() > 1 {
                self.calculate_distance_metrics(&largest_component.nodes, &adjacency)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Calculate clustering coefficient
        let clustering_coefficient = self.calculate_global_clustering_coefficient(&nodes, &adjacency);

        Ok(GraphMetrics {
            node_count,
            edge_count,
            layer_count,
            density,
            average_degree,
            max_degree,
            min_degree,
            connected_components,
            diameter,
            average_path_length,
            clustering_coefficient,
        })
    }

    /// Calculate detailed metrics for individual nodes
    pub async fn analyze_nodes(&self, project_id: i32) -> Result<Vec<NodeMetrics>> {
        let (nodes, edges) = self.load_graph_data(project_id).await?;
        
        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        let adjacency = self.build_adjacency_list(&nodes, &edges);
        let reverse_adjacency = self.build_reverse_adjacency_list(&nodes, &edges);
        let components = self.find_connected_components(&nodes, &adjacency);
        
        // Create component mapping
        let mut node_to_component = HashMap::new();
        for (comp_id, component) in components.iter().enumerate() {
            for node_id in &component.nodes {
                node_to_component.insert(node_id.clone(), comp_id);
            }
        }

        let mut node_metrics = Vec::new();

        for node in &nodes {
            let degree = adjacency.get(&node.node_id).map_or(0, |adj| adj.len());
            let out_degree = degree;
            let in_degree = reverse_adjacency.get(&node.node_id).map_or(0, |adj| adj.len());
            
            let betweenness_centrality = self.calculate_betweenness_centrality(&node.node_id, &nodes, &adjacency);
            let closeness_centrality = self.calculate_closeness_centrality(&node.node_id, &nodes, &adjacency);
            let eigenvector_centrality = 0.0; // TODO: Implement eigenvector centrality
            let pagerank = self.calculate_pagerank(&node.node_id, &nodes, &adjacency);
            let clustering_coefficient = self.calculate_local_clustering_coefficient(&node.node_id, &adjacency);
            let component_id = node_to_component.get(&node.node_id).copied().unwrap_or(0);

            node_metrics.push(NodeMetrics {
                node_id: node.node_id.clone(),
                degree,
                in_degree,
                out_degree,
                betweenness_centrality,
                closeness_centrality,
                eigenvector_centrality,
                pagerank,
                clustering_coefficient,
                component_id,
            });
        }

        Ok(node_metrics)
    }

    /// Find shortest path between two nodes
    pub async fn find_shortest_path(
        &self,
        project_id: i32,
        source: String,
        target: String,
    ) -> Result<PathResult> {
        let (nodes, edges) = self.load_graph_data(project_id).await?;
        let adjacency = self.build_adjacency_list(&nodes, &edges);

        // Verify nodes exist
        let source_exists = nodes.iter().any(|n| n.node_id == source);
        let target_exists = nodes.iter().any(|n| n.node_id == target);

        if !source_exists || !target_exists {
            return Ok(PathResult {
                source: source.clone(),
                target: target.clone(),
                path: Vec::new(),
                length: 0,
                weight: 0.0,
                exists: false,
            });
        }

        // BFS for shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(source.clone());
        visited.insert(source.clone());

        let mut found = false;
        while let Some(current) = queue.pop_front() {
            if current == target {
                found = true;
                break;
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if !found {
            return Ok(PathResult {
                source,
                target,
                path: Vec::new(),
                length: 0,
                weight: 0.0,
                exists: false,
            });
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut current = target.clone();
        
        while current != source {
            path.push(current.clone());
            current = parent[&current].clone();
        }
        path.push(source.clone());
        path.reverse();

        Ok(PathResult {
            source,
            target,
            length: path.len() - 1,
            weight: (path.len() - 1) as f64,
            path,
            exists: true,
        })
    }

    /// Analyze graph connectivity
    pub async fn analyze_connectivity(&self, project_id: i32) -> Result<ConnectivityResult> {
        let (nodes, edges) = self.load_graph_data(project_id).await?;
        let adjacency = self.build_adjacency_list(&nodes, &edges);
        
        let components = self.find_connected_components(&nodes, &adjacency);
        let largest_component_size = components
            .iter()
            .map(|c| c.size)
            .max()
            .unwrap_or(0);
        
        let is_connected = components.len() <= 1;
        
        // Find articulation points and bridges
        let articulation_points = self.find_articulation_points(&nodes, &adjacency);
        let bridges = self.find_bridges(&nodes, &adjacency);

        Ok(ConnectivityResult {
            component_count: components.len(),
            largest_component_size,
            is_connected,
            components,
            articulation_points,
            bridges,
        })
    }

    /// Detect communities using simple modularity-based approach
    pub async fn detect_communities(&self, project_id: i32) -> Result<CommunityDetectionResult> {
        let (nodes, edges) = self.load_graph_data(project_id).await?;
        let adjacency = self.build_adjacency_list(&nodes, &edges);
        
        // Use connected components as initial communities
        let components = self.find_connected_components(&nodes, &adjacency);
        
        let mut communities = Vec::new();
        for (id, component) in components.into_iter().enumerate() {
            let internal_edges = self.count_internal_edges(&component.nodes, &adjacency);
            let external_edges = self.count_external_edges(&component.nodes, &adjacency);
            let density = if component.size > 1 {
                let max_internal = component.size * (component.size - 1);
                internal_edges as f64 / max_internal as f64
            } else {
                0.0
            };

            communities.push(Community {
                id,
                nodes: component.nodes,
                size: component.size,
                internal_edges,
                external_edges,
                density,
            });
        }

        // Calculate modularity (simplified)
        let total_edges = edges.len();
        let modularity = if total_edges > 0 {
            communities
                .iter()
                .map(|c| {
                    let internal_ratio = c.internal_edges as f64 / total_edges as f64;
                    let degree_sum: usize = c.nodes
                        .iter()
                        .map(|n| adjacency.get(n).map_or(0, |adj| adj.len()))
                        .sum();
                    let expected = (degree_sum as f64 / (2.0 * total_edges as f64)).powi(2);
                    internal_ratio - expected
                })
                .sum()
        } else {
            0.0
        };

        Ok(CommunityDetectionResult {
            communities,
            modularity,
            algorithm: "Connected Components".to_string(),
        })
    }

    /// Analyze individual layers
    pub async fn analyze_layers(&self, project_id: i32) -> Result<Vec<LayerAnalysis>> {
        let (nodes, edges) = self.load_graph_data(project_id).await?;
        let layers = self.load_layers(project_id).await?;
        
        let mut layer_analyses = Vec::new();
        
        for layer in &layers {
            // Get nodes in this layer
            let layer_nodes: Vec<_> = nodes
                .iter()
                .filter(|n| n.layer_id.as_ref() == Some(&layer.layer_id))
                .collect();
            
            let node_count = layer_nodes.len();
            
            // Count internal and external edges
            let mut internal_edges = 0;
            let mut external_edges = 0;
            let mut connectivity_to_other_layers = HashMap::new();
            
            for edge in &edges {
                let source_in_layer = layer_nodes.iter().any(|n| n.node_id == edge.source_node_id);
                let target_in_layer = layer_nodes.iter().any(|n| n.node_id == edge.target_node_id);
                
                if source_in_layer && target_in_layer {
                    internal_edges += 1;
                } else if source_in_layer || target_in_layer {
                    external_edges += 1;
                    
                    // Track connectivity to other layers
                    let other_node_id = if source_in_layer {
                        &edge.target_node_id
                    } else {
                        &edge.source_node_id
                    };
                    
                    if let Some(other_node) = nodes.iter().find(|n| n.node_id == *other_node_id) {
                        if let Some(other_layer_id) = &other_node.layer_id {
                            *connectivity_to_other_layers.entry(other_layer_id.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
            
            // Calculate density
            let max_internal_edges = if node_count > 1 {
                node_count * (node_count - 1)
            } else {
                0
            };
            let density = if max_internal_edges > 0 {
                internal_edges as f64 / max_internal_edges as f64
            } else {
                0.0
            };
            
            layer_analyses.push(LayerAnalysis {
                layer_id: layer.layer_id.clone(),
                layer_name: layer.name.clone(),
                node_count,
                internal_edges,
                external_edges,
                density,
                connectivity_to_other_layers,
            });
        }
        
        Ok(layer_analyses)
    }

    // Helper methods
    
    async fn load_graph_data(&self, project_id: i32) -> Result<(Vec<nodes::Model>, Vec<edges::Model>)> {
        let nodes = nodes::Entity::find()
            .filter(nodes::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .context("Failed to load nodes")?;
            
        let edges = edges::Entity::find()
            .filter(edges::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .context("Failed to load edges")?;
            
        Ok((nodes, edges))
    }
    
    async fn load_layers(&self, project_id: i32) -> Result<Vec<layers::Model>> {
        layers::Entity::find()
            .filter(layers::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .context("Failed to load layers")
    }
    
    fn build_adjacency_list(
        &self,
        nodes: &[nodes::Model],
        edges: &[edges::Model],
    ) -> HashMap<String, Vec<String>> {
        let mut adjacency = HashMap::new();
        
        // Initialize with all nodes
        for node in nodes {
            adjacency.insert(node.node_id.clone(), Vec::new());
        }
        
        // Add edges
        for edge in edges {
            adjacency
                .entry(edge.source_node_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.target_node_id.clone());
        }
        
        adjacency
    }
    
    fn build_reverse_adjacency_list(
        &self,
        nodes: &[nodes::Model],
        edges: &[edges::Model],
    ) -> HashMap<String, Vec<String>> {
        let mut reverse_adjacency = HashMap::new();
        
        // Initialize with all nodes
        for node in nodes {
            reverse_adjacency.insert(node.node_id.clone(), Vec::new());
        }
        
        // Add reverse edges
        for edge in edges {
            reverse_adjacency
                .entry(edge.target_node_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.source_node_id.clone());
        }
        
        reverse_adjacency
    }
    
    fn find_connected_components(
        &self,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> Vec<ConnectedComponent> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();
        let mut component_id = 0;
        
        for node in nodes {
            if !visited.contains(&node.node_id) {
                let mut component_nodes = Vec::new();
                let mut stack = vec![node.node_id.clone()];
                
                while let Some(current) = stack.pop() {
                    if visited.contains(&current) {
                        continue;
                    }
                    
                    visited.insert(current.clone());
                    component_nodes.push(current.clone());
                    
                    // Add undirected neighbors
                    if let Some(neighbors) = adjacency.get(&current) {
                        for neighbor in neighbors {
                            if !visited.contains(neighbor) {
                                stack.push(neighbor.clone());
                            }
                        }
                    }
                    
                    // Add reverse neighbors for undirected connectivity
                    for (source, targets) in adjacency {
                        if targets.contains(&current) && !visited.contains(source) {
                            stack.push(source.clone());
                        }
                    }
                }
                
                let size = component_nodes.len();
                components.push(ConnectedComponent {
                    id: component_id,
                    nodes: component_nodes,
                    size,
                    is_largest: false, // Will be set later
                });
                component_id += 1;
            }
        }
        
        // Mark largest component
        if let Some(largest_size) = components.iter().map(|c| c.size).max() {
            for component in &mut components {
                component.is_largest = component.size == largest_size;
            }
        }
        
        components
    }
    
    fn calculate_distance_metrics(
        &self,
        component_nodes: &[String],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> (Option<usize>, Option<f64>) {
        if component_nodes.len() < 2 {
            return (None, None);
        }
        
        let mut max_distance = 0;
        let mut total_distance = 0;
        let mut path_count = 0;
        
        for source in component_nodes {
            let distances = self.bfs_distances(source, adjacency);
            
            for target in component_nodes {
                if source != target {
                    if let Some(&distance) = distances.get(target) {
                        max_distance = max_distance.max(distance);
                        total_distance += distance;
                        path_count += 1;
                    }
                }
            }
        }
        
        let diameter = if max_distance > 0 { Some(max_distance) } else { None };
        let average_path_length = if path_count > 0 {
            Some(total_distance as f64 / path_count as f64)
        } else {
            None
        };
        
        (diameter, average_path_length)
    }
    
    fn bfs_distances(
        &self,
        start: &str,
        adjacency: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, usize> {
        let mut distances = HashMap::new();
        let mut queue = VecDeque::new();
        
        distances.insert(start.to_string(), 0);
        queue.push_back(start.to_string());
        
        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];
            
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !distances.contains_key(neighbor) {
                        distances.insert(neighbor.clone(), current_distance + 1);
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        distances
    }
    
    fn calculate_global_clustering_coefficient(
        &self,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> f64 {
        if nodes.is_empty() {
            return 0.0;
        }
        
        let total_coefficient: f64 = nodes
            .iter()
            .map(|node| self.calculate_local_clustering_coefficient(&node.node_id, adjacency))
            .sum();
            
        total_coefficient / nodes.len() as f64
    }
    
    fn calculate_local_clustering_coefficient(
        &self,
        node_id: &str,
        adjacency: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let neighbors = adjacency.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
        let k = neighbors.len();
        
        if k < 2 {
            return 0.0;
        }
        
        let mut edges_between_neighbors = 0;
        for i in 0..neighbors.len() {
            for j in i + 1..neighbors.len() {
                if let Some(neighbor_adj) = adjacency.get(&neighbors[i]) {
                    if neighbor_adj.contains(&neighbors[j]) {
                        edges_between_neighbors += 1;
                    }
                }
            }
        }
        
        let possible_edges = k * (k - 1) / 2;
        edges_between_neighbors as f64 / possible_edges as f64
    }
    
    fn calculate_betweenness_centrality(
        &self,
        node_id: &str,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> f64 {
        // Simplified betweenness centrality calculation
        // For a complete implementation, we would use Brandes' algorithm
        
        let mut betweenness = 0.0;
        let node_ids: Vec<_> = nodes.iter().map(|n| &n.node_id).collect();
        
        for source in &node_ids {
            for target in &node_ids {
                if source != target && *source != node_id && *target != node_id {
                    // Check if shortest path from source to target goes through node_id
                    // This is a simplified approximation
                    let path_through_node = self.has_path_through_node(source, target, node_id, adjacency);
                    if path_through_node {
                        betweenness += 1.0;
                    }
                }
            }
        }
        
        // Normalize
        let n = nodes.len();
        if n > 2 {
            betweenness / ((n - 1) * (n - 2)) as f64
        } else {
            0.0
        }
    }
    
    fn has_path_through_node(
        &self,
        source: &str,
        target: &str,
        intermediate: &str,
        adjacency: &HashMap<String, Vec<String>>,
    ) -> bool {
        // Check if path from source to target goes through intermediate
        // This is a simplified heuristic
        
        let source_to_intermediate = self.has_path(source, intermediate, adjacency);
        let intermediate_to_target = self.has_path(intermediate, target, adjacency);
        
        source_to_intermediate && intermediate_to_target
    }
    
    fn has_path(
        &self,
        source: &str,
        target: &str,
        adjacency: &HashMap<String, Vec<String>>,
    ) -> bool {
        if source == target {
            return true;
        }
        
        let mut visited = HashSet::new();
        let mut stack = vec![source.to_string()];
        
        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            
            if current == target {
                return true;
            }
            
            visited.insert(current.clone());
            
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }
        
        false
    }
    
    fn calculate_closeness_centrality(
        &self,
        node_id: &str,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let distances = self.bfs_distances(node_id, adjacency);
        let reachable_nodes = distances.len() - 1; // Exclude the node itself
        
        if reachable_nodes == 0 {
            return 0.0;
        }
        
        let total_distance: usize = distances.values().sum();
        let avg_distance = total_distance as f64 / reachable_nodes as f64;
        
        if avg_distance > 0.0 {
            1.0 / avg_distance
        } else {
            0.0
        }
    }
    
    fn calculate_pagerank(
        &self,
        node_id: &str,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> f64 {
        // Simplified PageRank calculation
        // For a complete implementation, we would use iterative algorithm
        
        let in_degree = adjacency
            .values()
            .flat_map(|neighbors| neighbors.iter())
            .filter(|&neighbor| neighbor == node_id)
            .count();
            
        let total_nodes = nodes.len();
        if total_nodes > 0 {
            0.15 / total_nodes as f64 + 0.85 * (in_degree as f64 / total_nodes as f64)
        } else {
            0.0
        }
    }
    
    fn find_articulation_points(
        &self,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        // Simplified articulation point detection
        let mut articulation_points = Vec::new();
        
        for node in nodes {
            // Remove node and check if graph becomes more disconnected
            let original_components = self.count_components_excluding_node(nodes, adjacency, &node.node_id);
            let without_node_components = self.count_components_excluding_node(nodes, adjacency, &node.node_id);
            
            if without_node_components > original_components {
                articulation_points.push(node.node_id.clone());
            }
        }
        
        articulation_points
    }
    
    fn find_bridges(
        &self,
        _nodes: &[nodes::Model],
        _adjacency: &HashMap<String, Vec<String>>,
    ) -> Vec<(String, String)> {
        // Simplified bridge detection
        // For a complete implementation, we would use Tarjan's algorithm
        Vec::new()
    }
    
    fn count_components_excluding_node(
        &self,
        nodes: &[nodes::Model],
        adjacency: &HashMap<String, Vec<String>>,
        excluded_node: &str,
    ) -> usize {
        let filtered_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.node_id != excluded_node)
            .collect();
            
        let filtered_adjacency: HashMap<String, Vec<String>> = adjacency
            .iter()
            .filter(|(k, _)| *k != excluded_node)
            .map(|(k, v)| {
                let filtered_v: Vec<String> = v
                    .iter()
                    .filter(|neighbor| *neighbor != excluded_node)
                    .cloned()
                    .collect();
                (k.clone(), filtered_v)
            })
            .collect();
            
        let filtered_nodes_vec: Vec<nodes::Model> = filtered_nodes.into_iter().cloned().collect();
        self.find_connected_components(&filtered_nodes_vec, &filtered_adjacency).len()
    }
    
    fn count_internal_edges(
        &self,
        community_nodes: &[String],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> usize {
        let node_set: HashSet<_> = community_nodes.iter().collect();
        let mut internal_edges = 0;
        
        for node in community_nodes {
            if let Some(neighbors) = adjacency.get(node) {
                for neighbor in neighbors {
                    if node_set.contains(&neighbor) {
                        internal_edges += 1;
                    }
                }
            }
        }
        
        internal_edges
    }
    
    fn count_external_edges(
        &self,
        community_nodes: &[String],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> usize {
        let node_set: HashSet<_> = community_nodes.iter().collect();
        let mut external_edges = 0;
        
        for node in community_nodes {
            if let Some(neighbors) = adjacency.get(node) {
                for neighbor in neighbors {
                    if !node_set.contains(&neighbor) {
                        external_edges += 1;
                    }
                }
            }
        }
        
        external_edges
    }
}
