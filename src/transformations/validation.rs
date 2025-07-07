//! Validation module for transformation operations

use anyhow::Result;
use std::collections::HashSet;
use tracing::{debug, warn};

use crate::graph::Graph;
use super::{TransformationPipeline, TransformationRule, ValidationResult};

/// Validator for transformation operations
pub struct TransformationValidator {
    // Configuration options could go here
}

impl TransformationValidator {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Validate a single transformation type without a rule wrapper
    pub fn validate_transformation(&self, graph: &Graph, transformation: &super::TransformationType) -> Result<()> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        debug!("Validating transformation: {:?}", transformation);
        
        // Validate operation-specific constraints
        match transformation {
            super::TransformationType::NodeFilter(op) => {
                self.validate_node_filter_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::NodeTransform(op) => {
                self.validate_node_transform_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::NodeCreate(op) => {
                self.validate_node_create_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::NodeDelete(op) => {
                self.validate_node_delete_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeFilter(op) => {
                self.validate_edge_filter_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeTransform(op) => {
                self.validate_edge_transform_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeCreate(op) => {
                self.validate_edge_create_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeDelete(op) => {
                self.validate_edge_delete_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerFilter(op) => {
                self.validate_layer_filter_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerTransform(op) => {
                self.validate_layer_transform_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerCreate(op) => {
                self.validate_layer_create_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerDelete(op) => {
                self.validate_layer_delete_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::GraphMerge(op) => {
                self.validate_graph_merge_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::GraphSplit(op) => {
                self.validate_graph_split_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::GraphCluster(op) => {
                self.validate_graph_cluster_op(op, graph, &mut errors, &mut warnings);
            },
            
            // Advanced operations - basic validation for now
            super::TransformationType::NodeCluster(_op) => {
                // Basic validation - ensure graph has nodes
                if graph.nodes.is_empty() {
                    errors.push("Cannot cluster nodes: graph has no nodes".to_string());
                }
            },
            super::TransformationType::EdgeWeightNormalize(_op) => {
                // Basic validation - ensure graph has edges
                if graph.edges.is_empty() {
                    warnings.push("Edge weight normalization: graph has no edges".to_string());
                }
            },
            super::TransformationType::LayerMerge(op) => {
                // Validate source layers exist
                for layer_id in &op.source_layers {
                    if !graph.layers.iter().any(|l| l.id == *layer_id) {
                        errors.push(format!("Layer merge: source layer '{}' does not exist", layer_id));
                    }
                }
            },
            super::TransformationType::GraphAnalyze(_op) => {
                // Always valid - can analyze any graph
            },
            super::TransformationType::GraphLayout(_op) => {
                // Basic validation - ensure graph has nodes
                if graph.nodes.is_empty() {
                    warnings.push("Graph layout: graph has no nodes to layout".to_string());
                }
            },
            super::TransformationType::SubgraphExtract(_op) => {
                // Basic validation - ensure graph has content
                if graph.nodes.is_empty() {
                    errors.push("Subgraph extraction: source graph has no nodes".to_string());
                }
            },
            super::TransformationType::PathFinding(_op) => {
                // Basic validation - ensure graph has edges for path finding
                if graph.edges.is_empty() {
                    errors.push("Path finding: graph has no edges".to_string());
                }
            },
            super::TransformationType::CentralityCalculation(_op) => {
                // Basic validation - ensure graph has nodes and edges
                if graph.nodes.is_empty() {
                    errors.push("Centrality calculation: graph has no nodes".to_string());
                } else if graph.edges.is_empty() {
                    warnings.push("Centrality calculation: graph has no edges".to_string());
                }
            },
            super::TransformationType::CommunityDetection(_op) => {
                // Basic validation - ensure graph has edges for community detection
                if graph.edges.is_empty() {
                    errors.push("Community detection: graph has no edges".to_string());
                }
            },
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Validation failed: {}", errors.join(", ")))
        }
    }
    
    /// Validate an entire transformation pipeline
    pub fn validate_pipeline(&self, pipeline: &TransformationPipeline, graph: &Graph) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        debug!("Validating transformation pipeline: {}", pipeline.name);
        
        // Check pipeline structure
        if pipeline.rules.is_empty() {
            warnings.push("Pipeline contains no transformation rules".to_string());
        }
        
        // Check for duplicate rule IDs
        let mut rule_ids = HashSet::new();
        for rule in &pipeline.rules {
            if !rule_ids.insert(&rule.id) {
                errors.push(format!("Duplicate rule ID found: {}", rule.id));
            }
        }
        
        // Validate each rule
        for rule in &pipeline.rules {
            let rule_validation = self.validate_rule(rule, graph)?;
            errors.extend(rule_validation.errors);
            warnings.extend(rule_validation.warnings);
        }
        
        // Check for potential conflicts between rules
        self.check_rule_conflicts(&pipeline.rules, &mut warnings);
        
        // Validate rule execution order
        self.validate_execution_order(&pipeline.rules, &mut warnings);
        
        let valid = errors.is_empty();
        debug!("Pipeline validation completed: {} errors, {} warnings", errors.len(), warnings.len());
        
        Ok(ValidationResult {
            valid,
            errors,
            warnings,
        })
    }
    
    /// Validate a single transformation rule
    pub fn validate_rule(&self, rule: &TransformationRule, graph: &Graph) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        debug!("Validating transformation rule: {}", rule.name);
        
        // Check rule structure
        if rule.name.trim().is_empty() {
            errors.push("Rule name cannot be empty".to_string());
        }
        
        if rule.id.trim().is_empty() {
            errors.push("Rule ID cannot be empty".to_string());
        }
        
        // Validate conditions
        for condition in &rule.conditions {
            if let Err(e) = self.validate_condition(condition, graph) {
                errors.push(format!("Invalid condition '{}': {}", condition, e));
            }
        }
        
        // Validate operation-specific constraints
        match &rule.operation {
            super::TransformationType::NodeFilter(op) => {
                self.validate_node_filter_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::NodeTransform(op) => {
                self.validate_node_transform_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::NodeCreate(op) => {
                self.validate_node_create_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::NodeDelete(op) => {
                self.validate_node_delete_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeFilter(op) => {
                self.validate_edge_filter_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeTransform(op) => {
                self.validate_edge_transform_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeCreate(op) => {
                self.validate_edge_create_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::EdgeDelete(op) => {
                self.validate_edge_delete_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerFilter(op) => {
                self.validate_layer_filter_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerTransform(op) => {
                self.validate_layer_transform_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerCreate(op) => {
                self.validate_layer_create_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::LayerDelete(op) => {
                self.validate_layer_delete_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::GraphMerge(_) => {
                warnings.push("Graph merge operations require additional graphs".to_string());
            },
            super::TransformationType::GraphSplit(op) => {
                self.validate_graph_split_op(op, graph, &mut errors, &mut warnings);
            },
            super::TransformationType::GraphCluster(op) => {
                self.validate_graph_cluster_op(op, graph, &mut errors, &mut warnings);
            },
            
            // Advanced operations - basic validation for now
            super::TransformationType::NodeCluster(_op) => {
                // Basic validation - ensure graph has nodes
                if graph.nodes.is_empty() {
                    errors.push("Cannot cluster nodes: graph has no nodes".to_string());
                }
            },
            super::TransformationType::EdgeWeightNormalize(_op) => {
                // Basic validation - ensure graph has edges
                if graph.edges.is_empty() {
                    warnings.push("Edge weight normalization: graph has no edges".to_string());
                }
            },
            super::TransformationType::LayerMerge(op) => {
                // Validate source layers exist
                for layer_id in &op.source_layers {
                    if !graph.layers.iter().any(|l| l.id == *layer_id) {
                        errors.push(format!("Layer merge: source layer '{}' does not exist", layer_id));
                    }
                }
            },
            super::TransformationType::GraphAnalyze(_op) => {
                // Always valid - can analyze any graph
            },
            super::TransformationType::GraphLayout(_op) => {
                // Basic validation - ensure graph has nodes
                if graph.nodes.is_empty() {
                    warnings.push("Graph layout: graph has no nodes to layout".to_string());
                }
            },
            super::TransformationType::SubgraphExtract(_op) => {
                // Basic validation - ensure graph has content
                if graph.nodes.is_empty() {
                    errors.push("Subgraph extraction: source graph has no nodes".to_string());
                }
            },
            super::TransformationType::PathFinding(_op) => {
                // Basic validation - ensure graph has edges for path finding
                if graph.edges.is_empty() {
                    errors.push("Path finding: graph has no edges".to_string());
                }
            },
            super::TransformationType::CentralityCalculation(_op) => {
                // Basic validation - ensure graph has nodes and edges
                if graph.nodes.is_empty() {
                    errors.push("Centrality calculation: graph has no nodes".to_string());
                } else if graph.edges.is_empty() {
                    warnings.push("Centrality calculation: graph has no edges".to_string());
                }
            },
            super::TransformationType::CommunityDetection(_op) => {
                // Basic validation - ensure graph has edges for community detection
                if graph.edges.is_empty() {
                    errors.push("Community detection: graph has no edges".to_string());
                }
            },
        }
        
        let valid = errors.is_empty();
        debug!("Rule validation completed: {} errors, {} warnings", errors.len(), warnings.len());
        
        Ok(ValidationResult {
            valid,
            errors,
            warnings,
        })
    }
    
    /// Validate a condition string
    fn validate_condition(&self, condition: &str, graph: &Graph) -> Result<()> {
        // Basic syntax checking for conditions
        if condition.trim().is_empty() {
            return Err(anyhow::anyhow!("Condition cannot be empty"));
        }
        
        // Check for supported condition formats
        if condition.starts_with("node_count") || 
           condition.starts_with("edge_count") || 
           condition.starts_with("layer_exists") {
            return Ok(());
        }
        
        // Check for field-based conditions
        if condition.contains("=") || condition.contains(">") || condition.contains("<") {
            return Ok(());
        }
        
        warn!("Unknown condition format: {}", condition);
        Ok(())
    }
    
    // Node operation validators
    
    fn validate_node_filter_op(&self, op: &super::NodeFilterOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.condition.trim().is_empty() {
            errors.push("Node filter condition cannot be empty".to_string());
        }
        
        if graph.nodes.is_empty() {
            warnings.push("Graph contains no nodes to filter".to_string());
        }
    }
    
    fn validate_node_transform_op(&self, op: &super::NodeTransformOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.field_mappings.is_empty() && op.computed_fields.is_empty() {
            warnings.push("Node transform operation has no field mappings or computed fields".to_string());
        }
        
        if graph.nodes.is_empty() {
            warnings.push("Graph contains no nodes to transform".to_string());
        }
        
        // Check for circular mappings
        for (from, to) in &op.field_mappings {
            if from == to {
                warnings.push(format!("Circular field mapping detected: {} -> {}", from, to));
            }
        }
    }
    
    fn validate_node_create_op(&self, op: &super::NodeCreateOp, _graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.template.id.trim().is_empty() {
            errors.push("Node template must have a valid ID".to_string());
        }
        
        if let Some(count) = op.count {
            if count == 0 {
                warnings.push("Node create operation with count = 0 will not create any nodes".to_string());
            } else if count > 1000 {
                warnings.push(format!("Creating {} nodes may impact performance", count));
            }
        }
    }
    
    fn validate_node_delete_op(&self, op: &super::NodeDeleteOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.condition.trim().is_empty() {
            errors.push("Node delete condition cannot be empty".to_string());
        }
        
        if graph.nodes.is_empty() {
            warnings.push("Graph contains no nodes to delete".to_string());
        }
        
        if op.cascade_edges {
            warnings.push("Node deletion with cascade_edges=true will also remove connected edges".to_string());
        }
    }
    
    // Edge operation validators
    
    fn validate_edge_filter_op(&self, op: &super::EdgeFilterOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.condition.trim().is_empty() {
            errors.push("Edge filter condition cannot be empty".to_string());
        }
        
        if graph.edges.is_empty() {
            warnings.push("Graph contains no edges to filter".to_string());
        }
    }
    
    fn validate_edge_transform_op(&self, op: &super::EdgeTransformOp, graph: &Graph, _errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.field_mappings.is_empty() && op.weight_formula.is_none() {
            warnings.push("Edge transform operation has no field mappings or weight formula".to_string());
        }
        
        if graph.edges.is_empty() {
            warnings.push("Graph contains no edges to transform".to_string());
        }
    }
    
    fn validate_edge_create_op(&self, op: &super::EdgeCreateOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.source_pattern.trim().is_empty() {
            errors.push("Edge create operation must specify source pattern".to_string());
        }
        
        if op.target_pattern.trim().is_empty() {
            errors.push("Edge create operation must specify target pattern".to_string());
        }
        
        if op.edge_template.id.trim().is_empty() {
            errors.push("Edge template must have a valid ID".to_string());
        }
        
        if graph.nodes.is_empty() {
            warnings.push("Graph contains no nodes - edge creation will not produce any edges".to_string());
        }
    }
    
    fn validate_edge_delete_op(&self, op: &super::EdgeDeleteOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.condition.trim().is_empty() {
            errors.push("Edge delete condition cannot be empty".to_string());
        }
        
        if graph.edges.is_empty() {
            warnings.push("Graph contains no edges to delete".to_string());
        }
    }
    
    // Layer operation validators
    
    fn validate_layer_filter_op(&self, op: &super::LayerFilterOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.condition.trim().is_empty() {
            errors.push("Layer filter condition cannot be empty".to_string());
        }
        
        if graph.layers.is_empty() {
            warnings.push("Graph contains no layers to filter".to_string());
        }
    }
    
    fn validate_layer_transform_op(&self, op: &super::LayerTransformOp, graph: &Graph, _errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.field_mappings.is_empty() && op.color_scheme.is_none() {
            warnings.push("Layer transform operation has no field mappings or color scheme".to_string());
        }
        
        if graph.layers.is_empty() {
            warnings.push("Graph contains no layers to transform".to_string());
        }
    }
    
    fn validate_layer_create_op(&self, op: &super::LayerCreateOp, _graph: &Graph, errors: &mut Vec<String>, _warnings: &mut Vec<String>) {
        if op.template.id.trim().is_empty() {
            errors.push("Layer template must have a valid ID".to_string());
        }
        
        if op.template.label.trim().is_empty() {
            errors.push("Layer template must have a valid name".to_string());
        }
    }
    
    fn validate_layer_delete_op(&self, op: &super::LayerDeleteOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.condition.trim().is_empty() {
            errors.push("Layer delete condition cannot be empty".to_string());
        }
        
        if graph.layers.is_empty() {
            warnings.push("Graph contains no layers to delete".to_string());
        }
        
        if let Some(reassign_layer) = &op.reassign_nodes {
            if !graph.layers.iter().any(|l| l.id == *reassign_layer) {
                warnings.push(format!("Reassign layer '{}' does not exist in graph", reassign_layer));
            }
        }
    }
    
    // Graph operation validators
    
    fn validate_graph_merge_op(&self, op: &super::GraphMergeOp, graph: &Graph, _errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if graph.nodes.is_empty() {
            warnings.push("Graph is empty - merge operation may not be meaningful".to_string());
        }
        
        // Check merge strategy validity
        match op.merge_strategy {
            super::MergeStrategy::Union => {
                // Union merge is always valid
            },
            super::MergeStrategy::Intersection => {
                warnings.push("Intersection merge requires another graph - cannot validate without target graph".to_string());
            },
            super::MergeStrategy::LeftJoin | super::MergeStrategy::RightJoin => {
                warnings.push("Join merge requires another graph - cannot validate without target graph".to_string());
            },
        }
    }
    
    fn validate_graph_split_op(&self, op: &super::GraphSplitOp, graph: &Graph, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if op.split_criteria.trim().is_empty() {
            errors.push("Graph split criteria cannot be empty".to_string());
        }
        
        if graph.nodes.len() < 2 {
            warnings.push("Graph has fewer than 2 nodes - splitting may not be meaningful".to_string());
        }
    }
    
    fn validate_graph_cluster_op(&self, op: &super::GraphClusterOp, graph: &Graph, _errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        if graph.edges.is_empty() {
            warnings.push("Graph has no edges - clustering may not be meaningful".to_string());
        }
        
        match op.algorithm {
            super::ClusteringAlgorithm::KMeans => {
                if !op.parameters.contains_key("k") {
                    warnings.push("K-means clustering should specify 'k' parameter".to_string());
                }
            },
            super::ClusteringAlgorithm::Hierarchical => {
                if !op.parameters.contains_key("threshold") {
                    warnings.push("Hierarchical clustering should specify 'threshold' parameter".to_string());
                }
            },
            _ => {}
        }
    }
    
    /// Check for potential conflicts between rules in a pipeline
    fn check_rule_conflicts(&self, rules: &[TransformationRule], warnings: &mut Vec<String>) {
        // Check for conflicting operations
        let mut has_node_delete = false;
        let mut has_node_create = false;
        let mut has_edge_delete = false;
        let mut has_edge_create = false;
        
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            
            match &rule.operation {
                super::TransformationType::NodeDelete(_) => has_node_delete = true,
                super::TransformationType::NodeCreate(_) => has_node_create = true,
                super::TransformationType::EdgeDelete(_) => has_edge_delete = true,
                super::TransformationType::EdgeCreate(_) => has_edge_create = true,
                _ => {}
            }
        }
        
        if has_node_delete && has_edge_create {
            warnings.push("Pipeline contains both node deletion and edge creation - edge creation may fail if referenced nodes are deleted".to_string());
        }
        
        if has_edge_delete && has_node_create {
            warnings.push("Pipeline deletes edges before creating nodes - consider reordering operations".to_string());
        }
    }
    
    /// Validate the execution order of rules in a pipeline
    fn validate_execution_order(&self, rules: &[TransformationRule], warnings: &mut Vec<String>) {
        let enabled_rules: Vec<&TransformationRule> = rules.iter().filter(|r| r.enabled).collect();
        
        if enabled_rules.len() < 2 {
            return;
        }
        
        // Check for potentially problematic orderings
        for i in 0..enabled_rules.len() - 1 {
            let current = &enabled_rules[i];
            let next = &enabled_rules[i + 1];
            
            // Check for deletion followed by operations that might depend on deleted elements
            if matches!(current.operation, super::TransformationType::NodeDelete(_)) {
                if matches!(next.operation, super::TransformationType::EdgeCreate(_)) {
                    warnings.push(format!("Rule '{}' deletes nodes before rule '{}' creates edges - this may cause failures", current.name, next.name));
                }
            }
            
            if matches!(current.operation, super::TransformationType::LayerDelete(_)) {
                if matches!(next.operation, super::TransformationType::NodeTransform(_)) {
                    warnings.push(format!("Rule '{}' deletes layers before rule '{}' transforms nodes - nodes may lose layer assignments", current.name, next.name));
                }
            }
        }
    }
}

impl Default for TransformationValidator {
    fn default() -> Self {
        Self::new()
    }
}