//! Transformation engine for executing graph transformations

use anyhow::{Result, anyhow};
use std::time::Instant;
use tracing::{debug, info, warn, error};

use crate::graph::Graph;
use super::{
    TransformationPipeline, 
    TransformationRule, 
    TransformationResult, 
    TransformationStatistics,
    TransformationType,
};
use super::operations::TransformationOperations;
use super::validation::TransformationValidator;

/// Main transformation engine
pub struct TransformationEngine {
    validator: TransformationValidator,
    operations: TransformationOperations,
    dry_run: bool,
}

impl TransformationEngine {
    /// Create a new transformation engine
    pub fn new() -> Self {
        Self {
            validator: TransformationValidator::new(),
            operations: TransformationOperations::new(),
            dry_run: false,
        }
    }
    
    /// Enable or disable dry run mode
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }
    
    /// Execute a complete transformation pipeline
    pub fn execute_pipeline(
        &self, 
        pipeline: &TransformationPipeline, 
        graph: Graph
    ) -> Result<Vec<TransformationResult>> {
        info!("Executing transformation pipeline: {}", pipeline.name);
        
        let mut results = Vec::new();
        let mut current_graph = graph;
        
        // Validate pipeline if enabled
        if pipeline.validation_enabled {
            let validation = self.validator.validate_pipeline(pipeline, &current_graph)?;
            if !validation.valid {
                return Err(anyhow!("Pipeline validation failed: {}", validation.errors.join(", ")));
            }
        }
        
        // Execute each enabled rule in sequence
        for rule in pipeline.enabled_rules() {
            info!("Executing rule: {}", rule.name);
            
            let result = self.execute_rule(rule, current_graph.clone())?;
            
            if result.success {
                if let Some(transformed) = &result.transformed_graph {
                    current_graph = transformed.clone();
                }
                info!("Rule {} completed successfully", rule.name);
            } else {
                error!("Rule {} failed: {}", rule.name, result.error.as_deref().unwrap_or("Unknown error"));
                
                if pipeline.rollback_enabled {
                    warn!("Rolling back transformation pipeline");
                    break;
                }
            }
            
            results.push(result);
        }
        
        info!("Pipeline execution completed with {} results", results.len());
        Ok(results)
    }
    
    /// Execute a single transformation rule
    pub fn execute_rule(
        &self, 
        rule: &TransformationRule, 
        graph: Graph
    ) -> Result<TransformationResult> {
        let start_time = Instant::now();
        let original_graph = if self.dry_run { Some(graph.clone()) } else { None };
        
        debug!("Executing transformation rule: {} ({})", rule.name, rule.id);
        
        // Check rule conditions
        if !self.check_conditions(rule, &graph)? {
            return Ok(TransformationResult {
                success: false,
                rule_id: rule.id.clone(),
                original_graph,
                transformed_graph: None,
                error: Some("Rule conditions not met".to_string()),
                statistics: TransformationStatistics::default(),
            });
        }
        
        // Execute the transformation operation
        let result = match self.apply_operation(&rule.operation, graph) {
            Ok((transformed_graph, stats)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                let mut final_stats = stats;
                final_stats.execution_time_ms = execution_time;
                
                TransformationResult {
                    success: true,
                    rule_id: rule.id.clone(),
                    original_graph,
                    transformed_graph: Some(transformed_graph),
                    error: None,
                    statistics: final_stats,
                }
            },
            Err(e) => {
                TransformationResult {
                    success: false,
                    rule_id: rule.id.clone(),
                    original_graph,
                    transformed_graph: None,
                    error: Some(e.to_string()),
                    statistics: TransformationStatistics::default(),
                }
            }
        };
        
        debug!("Rule execution completed in {}ms", result.statistics.execution_time_ms);
        Ok(result)
    }
    
    /// Check if rule conditions are satisfied
    fn check_conditions(&self, rule: &TransformationRule, graph: &Graph) -> Result<bool> {
        if rule.conditions.is_empty() {
            return Ok(true);
        }
        
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, graph)? {
                debug!("Condition failed: {}", condition);
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Evaluate a single condition against the graph
    fn evaluate_condition(&self, condition: &str, graph: &Graph) -> Result<bool> {
        // Simple condition evaluation - can be extended with a proper expression parser
        if condition.starts_with("node_count >") {
            let threshold: usize = condition.trim_start_matches("node_count >")
                .trim()
                .parse()
                .map_err(|_| anyhow!("Invalid node_count condition: {}", condition))?;
            return Ok(graph.nodes.len() > threshold);
        }
        
        if condition.starts_with("edge_count >") {
            let threshold: usize = condition.trim_start_matches("edge_count >")
                .trim()
                .parse()
                .map_err(|_| anyhow!("Invalid edge_count condition: {}", condition))?;
            return Ok(graph.edges.len() > threshold);
        }
        
        if condition.starts_with("layer_exists") {
            let layer_id = condition.trim_start_matches("layer_exists")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            return Ok(graph.layers.iter().any(|l| l.id == layer_id));
        }
        
        // Default: assume condition is met if we can't parse it
        warn!("Unknown condition format: {}", condition);
        Ok(true)
    }
    
    /// Apply a transformation operation to the graph
    fn apply_operation(
        &self, 
        operation: &TransformationType, 
        graph: Graph
    ) -> Result<(Graph, TransformationStatistics)> {
        match operation {
            TransformationType::NodeFilter(op) => {
                self.operations.filter_nodes(graph, op)
            },
            TransformationType::NodeTransform(op) => {
                self.operations.transform_nodes(graph, op)
            },
            TransformationType::NodeCreate(op) => {
                self.operations.create_nodes(graph, op)
            },
            TransformationType::NodeDelete(op) => {
                self.operations.delete_nodes(graph, op)
            },
            TransformationType::EdgeFilter(op) => {
                self.operations.filter_edges(graph, op)
            },
            TransformationType::EdgeTransform(op) => {
                self.operations.transform_edges(graph, op)
            },
            TransformationType::EdgeCreate(op) => {
                self.operations.create_edges(graph, op)
            },
            TransformationType::EdgeDelete(op) => {
                self.operations.delete_edges(graph, op)
            },
            TransformationType::LayerFilter(op) => {
                self.operations.filter_layers(graph, op)
            },
            TransformationType::LayerTransform(op) => {
                self.operations.transform_layers(graph, op)
            },
            TransformationType::LayerCreate(op) => {
                self.operations.create_layers(graph, op)
            },
            TransformationType::LayerDelete(op) => {
                self.operations.delete_layers(graph, op)
            },
            TransformationType::GraphMerge(op) => {
                Err(anyhow!("Graph merge operation requires additional graphs - not supported in single-graph mode"))
            },
            TransformationType::GraphSplit(op) => {
                self.operations.split_graph(graph, op)
            },
            TransformationType::GraphCluster(op) => {
                self.operations.cluster_graph(graph, op)
            },
            
            // Advanced operations
            TransformationType::NodeCluster(op) => {
                self.operations.cluster_nodes(graph, op)
            },
            TransformationType::EdgeWeightNormalize(op) => {
                self.operations.normalize_edge_weights(graph, op)
            },
            TransformationType::LayerMerge(op) => {
                self.operations.merge_layers(graph, op)
            },
            TransformationType::GraphAnalyze(op) => {
                self.operations.analyze_graph(graph, op)
            },
            TransformationType::GraphLayout(op) => {
                self.operations.layout_graph(graph, op)
            },
            TransformationType::SubgraphExtract(op) => {
                self.operations.extract_subgraph(graph, op)
            },
            TransformationType::CentralityCalculation(op) => {
                self.operations.calculate_centrality(graph, op)
            },
            
            // Placeholder implementations for future operations
            TransformationType::PathFinding(_op) => {
                Err(anyhow!("Path finding operation not yet implemented"))
            },
            TransformationType::CommunityDetection(_op) => {
                Err(anyhow!("Community detection operation not yet implemented"))
            },
        }
    }
    
    /// Validate a transformation without executing it
    pub fn validate_transformation(&self, graph: &Graph, transformation: &TransformationType) -> Result<()> {
        debug!("Validating transformation: {:?}", transformation);
        
        // Use the validator to check if the transformation is valid
        self.validator.validate_transformation(graph, transformation)
    }
    
    /// Execute a single transformation on a graph
    pub async fn execute_transformation(&self, graph: Graph, transformation: TransformationType) -> Result<TransformationResult> {
        let start_time = Instant::now();
        info!("Executing transformation: {:?}", transformation);
        
        // Validate first
        if let Err(e) = self.validate_transformation(&graph, &transformation) {
            return Ok(TransformationResult {
                success: false,
                rule_id: "single_transformation".to_string(),
                original_graph: Some(graph.clone()),
                transformed_graph: Some(graph),
                error: Some(format!("Validation failed: {}", e)),
                statistics: TransformationStatistics::default(),
            });
        }
        
        // Execute the transformation
        match self.apply_operation(&transformation, graph.clone()) {
            Ok((transformed_graph, mut stats)) => {
                stats.execution_time_ms = start_time.elapsed().as_millis() as u64;
                
                Ok(TransformationResult {
                    success: true,
                    rule_id: "single_transformation".to_string(),
                    original_graph: Some(graph),
                    transformed_graph: Some(transformed_graph),
                    error: None,
                    statistics: stats,
                })
            },
            Err(e) => {
                Ok(TransformationResult {
                    success: false,
                    rule_id: "single_transformation".to_string(),
                    original_graph: Some(graph.clone()),
                    transformed_graph: Some(graph),
                    error: Some(e.to_string()),
                    statistics: TransformationStatistics::default(),
                })
            }
        }
    }
    
    /// Create a rollback operation for a given transformation
    pub fn create_rollback(&self, result: &TransformationResult) -> Option<Graph> {
        result.original_graph.clone()
    }
}

impl Default for TransformationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Node, Edge, Layer};
    use super::super::{NodeFilterOp, TransformationRule};
    
    fn create_test_graph() -> Graph {
        Graph {
            name: "Test Graph".to_string(),
            nodes: vec![
                Node {
                    id: "node1".to_string(),
                    label: Some("Node 1".to_string()),
                    layer_id: Some("layer1".to_string()),
                    ..Default::default()
                },
                Node {
                    id: "node2".to_string(),
                    label: Some("Node 2".to_string()),
                    layer_id: Some("layer1".to_string()),
                    ..Default::default()
                },
            ],
            edges: vec![
                Edge {
                    id: "edge1".to_string(),
                    source_id: "node1".to_string(),
                    target_id: "node2".to_string(),
                    ..Default::default()
                }
            ],
            layers: vec![
                Layer {
                    id: "layer1".to_string(),
                    name: "Layer 1".to_string(),
                    ..Default::default()
                }
            ],
        }
    }
    
    #[test]
    fn test_condition_evaluation() {
        let engine = TransformationEngine::new();
        let graph = create_test_graph();
        
        assert!(engine.evaluate_condition("node_count > 1", &graph).unwrap());
        assert!(!engine.evaluate_condition("node_count > 5", &graph).unwrap());
        assert!(engine.evaluate_condition("edge_count > 0", &graph).unwrap());
        assert!(engine.evaluate_condition("layer_exists \"layer1\"", &graph).unwrap());
        assert!(!engine.evaluate_condition("layer_exists \"layer2\"", &graph).unwrap());
    }
    
    #[test]
    fn test_pipeline_execution() {
        let engine = TransformationEngine::new();
        let graph = create_test_graph();
        let mut pipeline = TransformationPipeline::new("Test Pipeline".to_string());
        
        // Add a simple filter rule
        let filter_op = NodeFilterOp {
            condition: "label = \"Node 1\"".to_string(),
            keep_connected: false,
        };
        let rule = TransformationRule::new(
            "Filter Rule".to_string(),
            TransformationType::NodeFilter(filter_op)
        );
        pipeline.add_rule(rule);
        
        let results = engine.execute_pipeline(&pipeline, graph).unwrap();
        assert_eq!(results.len(), 1);
    }
}