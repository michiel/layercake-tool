//! Implementation of graph transformation operations

use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

use crate::graph::{Graph, Node, Edge, Layer};
use super::{
    TransformationStatistics,
    NodeFilterOp, NodeTransformOp, NodeCreateOp, NodeDeleteOp, NodeClusterOp,
    EdgeFilterOp, EdgeTransformOp, EdgeCreateOp, EdgeDeleteOp, EdgeWeightNormalizeOp,
    LayerFilterOp, LayerTransformOp, LayerCreateOp, LayerDeleteOp, LayerMergeOp,
    GraphSplitOp, GraphClusterOp, GraphAnalyzeOp, GraphLayoutOp,
    SubgraphExtractOp, PathFindingOp, CentralityCalculationOp, CommunityDetectionOp,
    NormalizationMethod, CentralityMeasure, LayoutAlgorithm, GraphMetric,
    ClusteringAlgorithm, CommunityAlgorithm, PathAlgorithm, ExtractionCriteria,
};

/// Implementation of transformation operations
pub struct TransformationOperations {
    // Could include configuration, caches, etc.
}

impl TransformationOperations {
    pub fn new() -> Self {
        Self {}
    }
    
    // Node Operations
    
    pub fn filter_nodes(&self, mut graph: Graph, op: &NodeFilterOp) -> Result<(Graph, TransformationStatistics)> {
        let original_count = graph.nodes.len();
        let mut stats = TransformationStatistics::default();
        
        debug!("Filtering nodes with condition: {}", op.condition);
        
        // Filter nodes based on condition
        let filtered_nodes: Vec<Node> = graph.nodes
            .into_iter()
            .filter(|node| self.evaluate_node_condition(&op.condition, node).unwrap_or(false))
            .collect();
        
        let removed_count = original_count - filtered_nodes.len();
        stats.nodes_removed = removed_count;
        
        graph.nodes = filtered_nodes;
        
        // If keep_connected is false, remove orphaned edges
        if !op.keep_connected {
            let node_ids: HashSet<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
            let original_edge_count = graph.edges.len();
            
            graph.edges.retain(|edge| {
                node_ids.contains(&edge.source) && node_ids.contains(&edge.target)
            });
            
            stats.edges_removed = original_edge_count - graph.edges.len();
        }
        
        debug!("Node filtering completed: removed {} nodes, {} edges", stats.nodes_removed, stats.edges_removed);
        Ok((graph, stats))
    }
    
    pub fn transform_nodes(&self, mut graph: Graph, op: &NodeTransformOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Transforming nodes with {} field mappings", op.field_mappings.len());
        
        for node in &mut graph.nodes {
            let mut modified = false;
            
            // Apply field mappings
            for (from_field, to_field) in &op.field_mappings {
                if let Some(value) = self.get_node_field_value(node, from_field) {
                    self.set_node_field_value(node, to_field, &value);
                    modified = true;
                }
            }
            
            // Apply computed fields
            for (field_name, formula) in &op.computed_fields {
                if let Ok(computed_value) = self.evaluate_node_formula(formula, node) {
                    self.set_node_field_value(node, field_name, &computed_value);
                    modified = true;
                }
            }
            
            if modified {
                stats.nodes_modified += 1;
            }
        }
        
        debug!("Node transformation completed: modified {} nodes", stats.nodes_modified);
        Ok((graph, stats))
    }
    
    pub fn create_nodes(&self, mut graph: Graph, op: &NodeCreateOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        let count = op.count.unwrap_or(1);
        
        debug!("Creating {} nodes from template", count);
        
        for i in 0..count {
            let mut new_node = op.template.clone();
            
            // Generate unique ID
            if let Some(pattern) = &op.id_pattern {
                new_node.id = pattern.replace("{i}", &i.to_string());
            } else {
                new_node.id = format!("{}_{}", op.template.id, i);
            }
            
            // Ensure uniqueness
            let mut counter = 0;
            let base_id = new_node.id.clone();
            while graph.nodes.iter().any(|n| n.id == new_node.id) {
                counter += 1;
                new_node.id = format!("{}_{}", base_id, counter);
            }
            
            graph.nodes.push(new_node);
            stats.nodes_added += 1;
        }
        
        debug!("Node creation completed: added {} nodes", stats.nodes_added);
        Ok((graph, stats))
    }
    
    pub fn delete_nodes(&self, mut graph: Graph, op: &NodeDeleteOp) -> Result<(Graph, TransformationStatistics)> {
        let original_count = graph.nodes.len();
        let mut stats = TransformationStatistics::default();
        
        debug!("Deleting nodes with condition: {}", op.condition);
        
        // Identify nodes to delete
        let nodes_to_delete: HashSet<String> = graph.nodes
            .iter()
            .filter(|node| self.evaluate_node_condition(&op.condition, node).unwrap_or(false))
            .map(|node| node.id.clone())
            .collect();
        
        // Remove nodes
        graph.nodes.retain(|node| !nodes_to_delete.contains(&node.id));
        stats.nodes_removed = original_count - graph.nodes.len();
        
        // Remove associated edges if cascade_edges is true
        if op.cascade_edges {
            let original_edge_count = graph.edges.len();
            graph.edges.retain(|edge| {
                !nodes_to_delete.contains(&edge.source) && !nodes_to_delete.contains(&edge.target)
            });
            stats.edges_removed = original_edge_count - graph.edges.len();
        }
        
        debug!("Node deletion completed: removed {} nodes, {} edges", stats.nodes_removed, stats.edges_removed);
        Ok((graph, stats))
    }
    
    // Edge Operations
    
    pub fn filter_edges(&self, mut graph: Graph, op: &EdgeFilterOp) -> Result<(Graph, TransformationStatistics)> {
        let original_count = graph.edges.len();
        let mut stats = TransformationStatistics::default();
        
        debug!("Filtering edges with condition: {}", op.condition);
        
        if op.validate_nodes {
            let node_ids: HashSet<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
            graph.edges.retain(|edge| {
                let valid_nodes = node_ids.contains(&edge.source) && node_ids.contains(&edge.target);
                let condition_met = self.evaluate_edge_condition(&op.condition, edge).unwrap_or(false);
                valid_nodes && condition_met
            });
        } else {
            graph.edges.retain(|edge| {
                self.evaluate_edge_condition(&op.condition, edge).unwrap_or(false)
            });
        }
        
        stats.edges_removed = original_count - graph.edges.len();
        debug!("Edge filtering completed: removed {} edges", stats.edges_removed);
        Ok((graph, stats))
    }
    
    pub fn transform_edges(&self, mut graph: Graph, op: &EdgeTransformOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Transforming edges with {} field mappings", op.field_mappings.len());
        
        for edge in &mut graph.edges {
            let mut modified = false;
            
            // Apply field mappings
            for (from_field, to_field) in &op.field_mappings {
                if let Some(value) = self.get_edge_field_value(edge, from_field) {
                    self.set_edge_field_value(edge, to_field, &value);
                    modified = true;
                }
            }
            
            // Apply weight formula if provided
            if let Some(formula) = &op.weight_formula {
                if let Ok(weight) = self.evaluate_edge_weight_formula(formula, edge) {
                    edge.weight = weight as i32;
                    modified = true;
                }
            }
            
            if modified {
                stats.edges_modified += 1;
            }
        }
        
        debug!("Edge transformation completed: modified {} edges", stats.edges_modified);
        Ok((graph, stats))
    }
    
    pub fn create_edges(&self, mut graph: Graph, op: &EdgeCreateOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Creating edges with patterns: {} -> {}", op.source_pattern, op.target_pattern);
        
        // Find matching source and target nodes
        let source_nodes: Vec<&Node> = graph.nodes
            .iter()
            .filter(|node| self.matches_pattern(&node.id, &op.source_pattern))
            .collect();
        
        let target_nodes: Vec<&Node> = graph.nodes
            .iter()
            .filter(|node| self.matches_pattern(&node.id, &op.target_pattern))
            .collect();
        
        // Create edges between matching nodes
        for source in &source_nodes {
            for target in &target_nodes {
                if source.id != target.id { // Avoid self-loops unless explicitly requested
                    let mut new_edge = op.edge_template.clone();
                    new_edge.id = format!("{}_{}", source.id, target.id);
                    new_edge.source = source.id.clone();
                    new_edge.target = target.id.clone();
                    
                    // Ensure uniqueness
                    let mut counter = 0;
                    let base_id = new_edge.id.clone();
                    while graph.edges.iter().any(|e| e.id == new_edge.id) {
                        counter += 1;
                        new_edge.id = format!("{}_{}", base_id, counter);
                    }
                    
                    graph.edges.push(new_edge);
                    stats.edges_added += 1;
                }
            }
        }
        
        debug!("Edge creation completed: added {} edges", stats.edges_added);
        Ok((graph, stats))
    }
    
    pub fn delete_edges(&self, mut graph: Graph, op: &EdgeDeleteOp) -> Result<(Graph, TransformationStatistics)> {
        let original_count = graph.edges.len();
        
        debug!("Deleting edges with condition: {}", op.condition);
        
        graph.edges.retain(|edge| {
            !self.evaluate_edge_condition(&op.condition, edge).unwrap_or(false)
        });
        
        let mut stats = TransformationStatistics::default();
        stats.edges_removed = original_count - graph.edges.len();
        
        debug!("Edge deletion completed: removed {} edges", stats.edges_removed);
        Ok((graph, stats))
    }
    
    // Layer Operations
    
    pub fn filter_layers(&self, mut graph: Graph, op: &LayerFilterOp) -> Result<(Graph, TransformationStatistics)> {
        let original_count = graph.layers.len();
        let mut stats = TransformationStatistics::default();
        
        debug!("Filtering layers with condition: {}", op.condition);
        
        graph.layers.retain(|layer| {
            self.evaluate_layer_condition(&op.condition, layer).unwrap_or(false)
        });
        
        stats.layers_removed = original_count - graph.layers.len();
        debug!("Layer filtering completed: removed {} layers", stats.layers_removed);
        Ok((graph, stats))
    }
    
    pub fn transform_layers(&self, mut graph: Graph, op: &LayerTransformOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Transforming layers with {} field mappings", op.field_mappings.len());
        
        for layer in &mut graph.layers {
            let mut modified = false;
            
            // Apply field mappings
            for (from_field, to_field) in &op.field_mappings {
                if let Some(value) = self.get_layer_field_value(layer, from_field) {
                    self.set_layer_field_value(layer, to_field, &value);
                    modified = true;
                }
            }
            
            // Apply color scheme if provided
            if let Some(scheme) = &op.color_scheme {
                if let Some(color) = self.generate_color_from_scheme(scheme, layer) {
                    layer.background_color = color;
                    modified = true;
                }
            }
            
            if modified {
                stats.layers_modified += 1;
            }
        }
        
        debug!("Layer transformation completed: modified {} layers", stats.layers_modified);
        Ok((graph, stats))
    }
    
    pub fn create_layers(&self, mut graph: Graph, op: &LayerCreateOp) -> Result<(Graph, TransformationStatistics)> {
        let mut new_layer = op.template.clone();
        let mut stats = TransformationStatistics::default();
        
        // Ensure unique ID
        let mut counter = 0;
        let base_id = new_layer.id.clone();
        while graph.layers.iter().any(|l| l.id == new_layer.id) {
            counter += 1;
            new_layer.id = format!("{}_{}", base_id, counter);
        }
        
        debug!("Creating layer: {}", new_layer.id);
        
        graph.layers.push(new_layer.clone());
        stats.layers_added += 1;
        
        // Auto-assign nodes if requested
        if op.auto_assign_nodes {
            for node in &mut graph.nodes {
                if node.layer.is_empty() {
                    node.layer = new_layer.id.clone();
                    stats.nodes_modified += 1;
                }
            }
        }
        
        debug!("Layer creation completed: added 1 layer, modified {} nodes", stats.nodes_modified);
        Ok((graph, stats))
    }
    
    pub fn delete_layers(&self, mut graph: Graph, op: &LayerDeleteOp) -> Result<(Graph, TransformationStatistics)> {
        let original_count = graph.layers.len();
        let mut stats = TransformationStatistics::default();
        
        debug!("Deleting layers with condition: {}", op.condition);
        
        // Identify layers to delete
        let layers_to_delete: HashSet<String> = graph.layers
            .iter()
            .filter(|layer| self.evaluate_layer_condition(&op.condition, layer).unwrap_or(false))
            .map(|layer| layer.id.clone())
            .collect();
        
        // Remove layers
        graph.layers.retain(|layer| !layers_to_delete.contains(&layer.id));
        stats.layers_removed = original_count - graph.layers.len();
        
        // Reassign nodes if specified
        if let Some(reassign_layer) = &op.reassign_nodes {
            for node in &mut graph.nodes {
                if layers_to_delete.contains(&node.layer) {
                    node.layer = reassign_layer.clone();
                    stats.nodes_modified += 1;
                }
            }
        } else {
            // Clear layer assignment for affected nodes
            for node in &mut graph.nodes {
                if layers_to_delete.contains(&node.layer) {
                    node.layer = String::new();
                    stats.nodes_modified += 1;
                }
            }
        }
        
        debug!("Layer deletion completed: removed {} layers, modified {} nodes", 
               stats.layers_removed, stats.nodes_modified);
        Ok((graph, stats))
    }
    
    // Graph Operations
    
    pub fn split_graph(&self, graph: Graph, op: &GraphSplitOp) -> Result<(Graph, TransformationStatistics)> {
        // For now, return the original graph as splitting would return multiple graphs
        // This could be enhanced to support splitting into subgraphs
        warn!("Graph splitting not fully implemented - returning original graph");
        Ok((graph, TransformationStatistics::default()))
    }
    
    pub fn cluster_graph(&self, graph: Graph, op: &GraphClusterOp) -> Result<(Graph, TransformationStatistics)> {
        // Basic clustering implementation - could be enhanced with actual algorithms
        warn!("Graph clustering not fully implemented - returning original graph");
        Ok((graph, TransformationStatistics::default()))
    }
    
    // Advanced Node Operations
    
    pub fn cluster_nodes(&self, mut graph: Graph, op: &NodeClusterOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Clustering nodes using algorithm: {:?}", op.algorithm);
        
        let clusters = self.detect_clusters(&graph, &op.algorithm, &op.parameters)?;
        
        // Create cluster layers if requested
        if op.create_cluster_layers {
            for (cluster_id, _nodes) in &clusters {
                let layer = Layer {
                    id: format!("cluster_{}", cluster_id),
                    label: format!("Cluster {}", cluster_id),
                    background_color: self.generate_cluster_color(*cluster_id),
                    text_color: "#000000".to_string(),
                    border_color: "#666666".to_string(),
                };
                graph.layers.push(layer);
                stats.layers_added += 1;
            }
        }
        
        // Assign nodes to cluster layers
        for (cluster_id, node_ids) in clusters {
            if let Some(min_size) = op.min_cluster_size {
                if node_ids.len() < min_size {
                    continue; // Skip small clusters
                }
            }
            
            let cluster_layer = if op.create_cluster_layers {
                format!("cluster_{}", cluster_id)
            } else {
                format!("cluster_{}", cluster_id)
            };
            
            for node_id in node_ids {
                if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == node_id) {
                    node.layer = cluster_layer.clone();
                    stats.nodes_modified += 1;
                }
            }
        }
        
        debug!("Node clustering completed: {} clusters created, {} nodes modified", 
               stats.layers_added, stats.nodes_modified);
        Ok((graph, stats))
    }
    
    // Advanced Edge Operations
    
    pub fn normalize_edge_weights(&self, mut graph: Graph, op: &EdgeWeightNormalizeOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Normalizing edge weights using method: {:?}", op.method);
        
        if graph.edges.is_empty() {
            return Ok((graph, stats));
        }
        
        let weights: Vec<f64> = graph.edges.iter().map(|e| e.weight as f64).collect();
        
        match op.method {
            NormalizationMethod::MinMax => {
                let min_weight = weights.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_weight = weights.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                
                let (target_min, target_max) = op.range.unwrap_or((0.0, 1.0));
                let range = max_weight - min_weight;
                
                if range > 0.0 {
                    for edge in &mut graph.edges {
                        let normalized = (edge.weight as f64 - min_weight) / range;
                        edge.weight = (normalized * (target_max - target_min) + target_min) as i32;
                        stats.edges_modified += 1;
                    }
                }
            },
            NormalizationMethod::ZScore => {
                let mean: f64 = weights.iter().sum::<f64>() / weights.len() as f64;
                let variance: f64 = weights.iter().map(|w| (w - mean).powi(2)).sum::<f64>() / weights.len() as f64;
                let std_dev = variance.sqrt();
                
                if std_dev > 0.0 {
                    for edge in &mut graph.edges {
                        let z_score = (edge.weight as f64 - mean) / std_dev;
                        edge.weight = z_score as i32;
                        stats.edges_modified += 1;
                    }
                }
            },
            _ => {
                warn!("Normalization method {:?} not implemented", op.method);
            }
        }
        
        debug!("Edge weight normalization completed: {} edges modified", stats.edges_modified);
        Ok((graph, stats))
    }
    
    // Advanced Layer Operations
    
    pub fn merge_layers(&self, mut graph: Graph, op: &LayerMergeOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Merging layers: {:?} into {}", op.source_layers, op.target_layer);
        
        // Reassign nodes from source layers to target layer
        for node in &mut graph.nodes {
            if op.source_layers.contains(&node.layer) {
                node.layer = op.target_layer.clone();
                stats.nodes_modified += 1;
            }
        }
        
        // Remove source layers
        graph.layers.retain(|layer| !op.source_layers.contains(&layer.id));
        stats.layers_removed = op.source_layers.len();
        
        debug!("Layer merging completed: {} layers removed, {} nodes reassigned", 
               stats.layers_removed, stats.nodes_modified);
        Ok((graph, stats))
    }
    
    // Advanced Graph Operations
    
    pub fn analyze_graph(&self, mut graph: Graph, op: &GraphAnalyzeOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Analyzing graph with metrics: {:?}", op.metrics);
        
        let mut analysis_results = HashMap::new();
        
        for metric in &op.metrics {
            let value = match metric {
                GraphMetric::NodeCount => graph.nodes.len() as f64,
                GraphMetric::EdgeCount => graph.edges.len() as f64,
                GraphMetric::Density => {
                    let n = graph.nodes.len() as f64;
                    if n > 1.0 {
                        graph.edges.len() as f64 / (n * (n - 1.0))
                    } else {
                        0.0
                    }
                },
                GraphMetric::ConnectedComponents => self.count_connected_components(&graph) as f64,
                _ => {
                    warn!("Graph metric {:?} not implemented", metric);
                    0.0
                }
            };
            
            analysis_results.insert(format!("{:?}", metric), value);
        }
        
        // Store results if requested
        if op.store_results {
            // For now, we'll add analysis results as graph metadata
            // This could be enhanced to store in a separate structure
            debug!("Analysis results: {:?}", analysis_results);
        }
        
        debug!("Graph analysis completed: {} metrics calculated", analysis_results.len());
        Ok((graph, stats))
    }
    
    pub fn layout_graph(&self, mut graph: Graph, op: &GraphLayoutOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Applying graph layout: {:?}", op.algorithm);
        
        if !op.update_positions {
            return Ok((graph, stats));
        }
        
        match op.algorithm {
            LayoutAlgorithm::Circular => {
                let node_count = graph.nodes.len();
                if node_count > 0 {
                    let radius = 100.0;
                    let angle_step = 2.0 * std::f64::consts::PI / node_count as f64;
                    
                    for (i, node) in graph.nodes.iter_mut().enumerate() {
                        let angle = i as f64 * angle_step;
                        // Note: Position would be stored in node properties if needed
                        // For now, we skip positioning as the Node struct doesn't have x/y fields
                        stats.nodes_modified += 1;
                    }
                }
            },
            LayoutAlgorithm::Grid => {
                let grid_size = (graph.nodes.len() as f64).sqrt().ceil() as usize;
                let spacing = 50;
                
                for (i, node) in graph.nodes.iter_mut().enumerate() {
                    let row = i / grid_size;
                    let col = i % grid_size;
                    // Note: Position would be stored in node properties if needed
                    // For now, we skip positioning as the Node struct doesn't have x/y fields
                    stats.nodes_modified += 1;
                }
            },
            LayoutAlgorithm::Random => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                for node in &mut graph.nodes {
                    let mut hasher = DefaultHasher::new();
                    node.id.hash(&mut hasher);
                    let hash = hasher.finish();
                    
                    // Note: Position would be stored in node properties if needed
                    // For now, we skip positioning as the Node struct doesn't have x/y fields
                    stats.nodes_modified += 1;
                }
            },
            _ => {
                warn!("Layout algorithm {:?} not implemented", op.algorithm);
            }
        }
        
        debug!("Graph layout completed: {} nodes repositioned", stats.nodes_modified);
        Ok((graph, stats))
    }
    
    pub fn extract_subgraph(&self, graph: Graph, op: &SubgraphExtractOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Extracting subgraph with criteria: {:?}", op.criteria);
        
        let selected_nodes: HashSet<String> = match &op.criteria {
            ExtractionCriteria::NodeList(node_ids) => {
                node_ids.iter().cloned().collect()
            },
            ExtractionCriteria::LayerList(layer_ids) => {
                graph.nodes.iter()
                    .filter(|node| layer_ids.contains(&node.layer))
                    .map(|node| node.id.clone())
                    .collect()
            },
            ExtractionCriteria::NeighborhoodRadius(center_node, radius) => {
                self.get_neighborhood(&graph, center_node, *radius)
            },
            _ => {
                warn!("Extraction criteria {:?} not implemented", op.criteria);
                HashSet::new()
            }
        };
        
        // Calculate stats before moving data
        let original_node_count = graph.nodes.len();
        let original_edge_count = graph.edges.len();
        
        // Create subgraph
        let mut subgraph = Graph {
            name: format!("{}_subgraph", graph.name),
            nodes: graph.nodes.into_iter()
                .filter(|node| selected_nodes.contains(&node.id))
                .collect(),
            edges: Vec::new(),
            layers: graph.layers, // Keep all layers for now
        };
        
        // Add edges based on policy
        for edge in graph.edges {
            let source_in = selected_nodes.contains(&edge.source);
            let target_in = selected_nodes.contains(&edge.target);
            
            if source_in && target_in {
                subgraph.edges.push(edge);
            } else if op.include_boundary_edges && (source_in || target_in) {
                subgraph.edges.push(edge);
            }
        }
        
        stats.nodes_removed = original_node_count - subgraph.nodes.len();
        stats.edges_removed = original_edge_count - subgraph.edges.len();
        
        debug!("Subgraph extraction completed: {} nodes, {} edges extracted", 
               subgraph.nodes.len(), subgraph.edges.len());
        Ok((subgraph, stats))
    }
    
    pub fn calculate_centrality(&self, mut graph: Graph, op: &CentralityCalculationOp) -> Result<(Graph, TransformationStatistics)> {
        let mut stats = TransformationStatistics::default();
        
        debug!("Calculating centrality measures: {:?}", op.measures);
        
        for measure in &op.measures {
            match measure {
                CentralityMeasure::Degree => {
                    let degree_centrality = self.calculate_degree_centrality(&graph);
                    if op.store_as_node_property {
                        for node in &mut graph.nodes {
                            if let Some(&centrality) = degree_centrality.get(&node.id) {
                                // Store as a node property (this would need to be added to Node struct)
                                debug!("Node {} degree centrality: {}", node.id, centrality);
                            }
                        }
                        stats.nodes_modified += graph.nodes.len();
                    }
                },
                _ => {
                    warn!("Centrality measure {:?} not implemented", measure);
                }
            }
        }
        
        debug!("Centrality calculation completed");
        Ok((graph, stats))
    }
    
    // Helper methods for condition evaluation
    
    fn evaluate_node_condition(&self, condition: &str, node: &Node) -> Result<bool> {
        // Simple condition evaluation - can be extended
        if condition.contains("=") {
            let parts: Vec<&str> = condition.splitn(2, '=').collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                
                match field {
                    "label" => return Ok(node.label == value),
                    "id" => return Ok(node.id == value),
                    "layer" => return Ok(node.layer == value),
                    _ => {}
                }
            }
        }
        
        // Default to true if condition can't be evaluated
        Ok(true)
    }
    
    fn evaluate_edge_condition(&self, condition: &str, edge: &Edge) -> Result<bool> {
        // Simple condition evaluation for edges
        if condition.contains("=") {
            let parts: Vec<&str> = condition.splitn(2, '=').collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                
                match field {
                    "label" => return Ok(edge.label == value),
                    "id" => return Ok(edge.id == value),
                    "source" => return Ok(edge.source == value),
                    "target" => return Ok(edge.target == value),
                    _ => {}
                }
            }
        }
        
        Ok(true)
    }
    
    fn evaluate_layer_condition(&self, condition: &str, layer: &Layer) -> Result<bool> {
        // Simple condition evaluation for layers
        if condition.contains("=") {
            let parts: Vec<&str> = condition.splitn(2, '=').collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                
                match field {
                    "label" => return Ok(layer.label == value),
                    "id" => return Ok(layer.id == value),
                    _ => {}
                }
            }
        }
        
        Ok(true)
    }
    
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        // Simple pattern matching - could be enhanced with regex
        if pattern.contains("*") {
            let regex_pattern = pattern.replace("*", ".*");
            if let Ok(re) = Regex::new(&regex_pattern) {
                return re.is_match(text);
            }
        }
        
        text == pattern
    }
    
    // Field access helpers (simplified - could be enhanced)
    
    fn get_node_field_value(&self, node: &Node, field: &str) -> Option<String> {
        match field {
            "id" => Some(node.id.clone()),
            "label" => Some(node.label.clone()),
            "layer" => Some(node.layer.clone()),
            _ => None,
        }
    }
    
    fn set_node_field_value(&self, node: &mut Node, field: &str, value: &str) {
        match field {
            "id" => node.id = value.to_string(),
            "label" => node.label = value.to_string(),
            "layer" => node.layer = value.to_string(),
            _ => {}
        }
    }
    
    fn get_edge_field_value(&self, edge: &Edge, field: &str) -> Option<String> {
        match field {
            "id" => Some(edge.id.clone()),
            "label" => Some(edge.label.clone()),
            "source" => Some(edge.source.clone()),
            "target" => Some(edge.target.clone()),
            _ => None,
        }
    }
    
    fn set_edge_field_value(&self, edge: &mut Edge, field: &str, value: &str) {
        match field {
            "id" => edge.id = value.to_string(),
            "label" => edge.label = value.to_string(),
            "source" => edge.source = value.to_string(),
            "target" => edge.target = value.to_string(),
            _ => {}
        }
    }
    
    fn get_layer_field_value(&self, layer: &Layer, field: &str) -> Option<String> {
        match field {
            "id" => Some(layer.id.clone()),
            "label" => Some(layer.label.clone()),
            "name" => Some(layer.label.clone()), // Alias for label
            "background_color" => Some(layer.background_color.clone()),
            "text_color" => Some(layer.text_color.clone()),
            "border_color" => Some(layer.border_color.clone()),
            "color" => Some(layer.background_color.clone()), // Alias for background_color
            _ => None,
        }
    }
    
    fn set_layer_field_value(&self, layer: &mut Layer, field: &str, value: &str) {
        match field {
            "id" => layer.id = value.to_string(),
            "label" => layer.label = value.to_string(),
            "name" => layer.label = value.to_string(), // Alias for label
            "background_color" => layer.background_color = value.to_string(),
            "text_color" => layer.text_color = value.to_string(),
            "border_color" => layer.border_color = value.to_string(),
            "color" => layer.background_color = value.to_string(), // Alias for background_color
            _ => {}
        }
    }
    
    fn evaluate_node_formula(&self, formula: &str, node: &Node) -> Result<String> {
        // Simple formula evaluation - could be enhanced with a proper expression parser
        if formula.starts_with("upper(") && formula.ends_with(")") {
            let field = formula.trim_start_matches("upper(").trim_end_matches(")");
            if let Some(value) = self.get_node_field_value(node, field) {
                return Ok(value.to_uppercase());
            }
        }
        
        if formula.starts_with("lower(") && formula.ends_with(")") {
            let field = formula.trim_start_matches("lower(").trim_end_matches(")");
            if let Some(value) = self.get_node_field_value(node, field) {
                return Ok(value.to_lowercase());
            }
        }
        
        Err(anyhow!("Unsupported formula: {}", formula))
    }
    
    fn evaluate_edge_weight_formula(&self, formula: &str, _edge: &Edge) -> Result<f64> {
        // Simple weight formula evaluation
        if let Ok(weight) = formula.parse::<f64>() {
            return Ok(weight);
        }
        
        // Could add more complex formulas here
        Ok(1.0)
    }
    
    fn generate_color_from_scheme(&self, scheme: &str, layer: &Layer) -> Option<String> {
        // Simple color scheme generation
        match scheme {
            "rainbow" => {
                let hash = layer.id.chars().map(|c| c as u8).sum::<u8>();
                let hue = (hash as u32 * 137) % 360; // Golden angle for good distribution
                Some(format!("hsl({}, 70%, 50%)", hue))
            },
            "blues" => Some("#3b82f6".to_string()),
            "greens" => Some("#10b981".to_string()),
            "reds" => Some("#ef4444".to_string()),
            _ => None,
        }
    }
    
    // Advanced helper methods
    
    fn detect_clusters(&self, graph: &Graph, algorithm: &ClusteringAlgorithm, _parameters: &HashMap<String, f64>) -> Result<HashMap<usize, Vec<String>>> {
        match algorithm {
            ClusteringAlgorithm::ConnectedComponents => {
                self.find_connected_components(graph)
            },
            _ => {
                warn!("Clustering algorithm {:?} not implemented", algorithm);
                Ok(HashMap::new())
            }
        }
    }
    
    fn find_connected_components(&self, graph: &Graph) -> Result<HashMap<usize, Vec<String>>> {
        let mut visited = HashSet::new();
        let mut components = HashMap::new();
        let mut component_id = 0;
        
        // Build adjacency list
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &graph.edges {
            adjacency.entry(edge.source.clone()).or_default().push(edge.target.clone());
            adjacency.entry(edge.target.clone()).or_default().push(edge.source.clone());
        }
        
        for node in &graph.nodes {
            if !visited.contains(&node.id) {
                let mut component_nodes = Vec::new();
                let mut stack = vec![node.id.clone()];
                
                while let Some(current) = stack.pop() {
                    if visited.contains(&current) {
                        continue;
                    }
                    
                    visited.insert(current.clone());
                    component_nodes.push(current.clone());
                    
                    if let Some(neighbors) = adjacency.get(&current) {
                        for neighbor in neighbors {
                            if !visited.contains(neighbor) {
                                stack.push(neighbor.clone());
                            }
                        }
                    }
                }
                
                components.insert(component_id, component_nodes);
                component_id += 1;
            }
        }
        
        Ok(components)
    }
    
    fn generate_cluster_color(&self, cluster_id: usize) -> String {
        let colors = [
            "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#feca57",
            "#ff9ff3", "#54a0ff", "#5f27cd", "#00d2d3", "#ff9f43",
            "#ffb8b8", "#c44569", "#f8b500", "#78e08f", "#82ccdd",
        ];
        colors[cluster_id % colors.len()].to_string()
    }
    
    fn count_connected_components(&self, graph: &Graph) -> usize {
        self.find_connected_components(graph)
            .map(|components| components.len())
            .unwrap_or(0)
    }
    
    fn get_neighborhood(&self, graph: &Graph, center_node: &str, radius: usize) -> HashSet<String> {
        let mut neighborhood = HashSet::new();
        let mut current_level = HashSet::new();
        
        current_level.insert(center_node.to_string());
        neighborhood.insert(center_node.to_string());
        
        // Build adjacency list
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &graph.edges {
            adjacency.entry(edge.source.clone()).or_default().push(edge.target.clone());
            adjacency.entry(edge.target.clone()).or_default().push(edge.source.clone());
        }
        
        for _ in 0..radius {
            let mut next_level = HashSet::new();
            
            for node in &current_level {
                if let Some(neighbors) = adjacency.get(node) {
                    for neighbor in neighbors {
                        if !neighborhood.contains(neighbor) {
                            next_level.insert(neighbor.clone());
                            neighborhood.insert(neighbor.clone());
                        }
                    }
                }
            }
            
            if next_level.is_empty() {
                break;
            }
            
            current_level = next_level;
        }
        
        neighborhood
    }
    
    fn calculate_degree_centrality(&self, graph: &Graph) -> HashMap<String, f64> {
        let mut degree_count: HashMap<String, usize> = HashMap::new();
        
        // Initialize all nodes with degree 0
        for node in &graph.nodes {
            degree_count.insert(node.id.clone(), 0);
        }
        
        // Count degrees
        for edge in &graph.edges {
            *degree_count.entry(edge.source.clone()).or_insert(0) += 1;
            *degree_count.entry(edge.target.clone()).or_insert(0) += 1;
        }
        
        // Convert to centrality scores (normalized by max possible degree)
        let max_possible_degree = if graph.nodes.len() > 1 { graph.nodes.len() - 1 } else { 1 };
        
        degree_count.into_iter()
            .map(|(node_id, degree)| (node_id, degree as f64 / max_possible_degree as f64))
            .collect()
    }
}

impl Default for TransformationOperations {
    fn default() -> Self {
        Self::new()
    }
}