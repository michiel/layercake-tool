//! Implementation of graph transformation operations

use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

use crate::graph::{Graph, Node, Edge, Layer};
use super::{
    TransformationStatistics,
    NodeFilterOp, NodeTransformOp, NodeCreateOp, NodeDeleteOp,
    EdgeFilterOp, EdgeTransformOp, EdgeCreateOp, EdgeDeleteOp,
    LayerFilterOp, LayerTransformOp, LayerCreateOp, LayerDeleteOp,
    GraphSplitOp, GraphClusterOp,
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
}

impl Default for TransformationOperations {
    fn default() -> Self {
        Self::new()
    }
}