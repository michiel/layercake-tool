//! Pre-built transformation rules and rule templates

use std::collections::HashMap;

use super::{
    TransformationRule, 
    TransformationType,
    NodeFilterOp, NodeTransformOp, NodeCreateOp, NodeDeleteOp,
    EdgeFilterOp, EdgeTransformOp, EdgeCreateOp, EdgeDeleteOp,
    LayerCreateOp, LayerTransformOp,
    GraphClusterOp, ClusteringAlgorithm,
};
use crate::graph::{Node, Edge, Layer};

/// Collection of pre-built transformation rules
pub struct TransformationRuleTemplates;

impl TransformationRuleTemplates {
    /// Create a rule to filter nodes by layer
    pub fn filter_nodes_by_layer(layer_id: &str) -> TransformationRule {
        let op = NodeFilterOp {
            condition: format!("layer_id = \"{}\"", layer_id),
            keep_connected: false,
        };
        
        TransformationRule::new(
            format!("Filter nodes by layer: {}", layer_id),
            TransformationType::NodeFilter(op)
        )
    }
    
    /// Create a rule to filter nodes by label pattern
    pub fn filter_nodes_by_label_pattern(pattern: &str) -> TransformationRule {
        let op = NodeFilterOp {
            condition: format!("label = \"{}\"", pattern),
            keep_connected: true,
        };
        
        TransformationRule::new(
            format!("Filter nodes by label: {}", pattern),
            TransformationType::NodeFilter(op)
        )
    }
    
    /// Create a rule to remove isolated nodes (nodes with no edges)
    pub fn remove_isolated_nodes() -> TransformationRule {
        let op = NodeDeleteOp {
            condition: "isolated = true".to_string(),
            cascade_edges: false,
        };
        
        TransformationRule::new(
            "Remove isolated nodes".to_string(),
            TransformationType::NodeDelete(op)
        )
    }
    
    /// Create a rule to normalize node labels (convert to uppercase)
    pub fn normalize_node_labels_uppercase() -> TransformationRule {
        let mut computed_fields = HashMap::new();
        computed_fields.insert("label".to_string(), "upper(label)".to_string());
        
        let op = NodeTransformOp {
            field_mappings: HashMap::new(),
            computed_fields,
        };
        
        TransformationRule::new(
            "Normalize node labels to uppercase".to_string(),
            TransformationType::NodeTransform(op)
        )
    }
    
    /// Create a rule to add prefix to node IDs
    pub fn add_node_id_prefix(prefix: &str) -> TransformationRule {
        let mut computed_fields = HashMap::new();
        computed_fields.insert("id".to_string(), format!("\"{}\" + id", prefix));
        
        let op = NodeTransformOp {
            field_mappings: HashMap::new(),
            computed_fields,
        };
        
        TransformationRule::new(
            format!("Add prefix '{}' to node IDs", prefix),
            TransformationType::NodeTransform(op)
        )
    }
    
    /// Create a rule to create a central hub node
    pub fn create_hub_node(hub_id: &str, hub_label: &str, layer_id: &str) -> TransformationRule {
        let template = Node {
            id: hub_id.to_string(),
            label: hub_label.to_string(),
            layer: layer_id.to_string(),
            is_partition: false,
            belongs_to: None,
            weight: 1,
            comment: None,
        };
        
        let op = NodeCreateOp {
            template,
            count: Some(1),
            id_pattern: None,
        };
        
        TransformationRule::new(
            format!("Create hub node: {}", hub_label),
            TransformationType::NodeCreate(op)
        )
    }
    
    /// Create a rule to connect all nodes in a layer to a hub
    pub fn connect_layer_to_hub(layer_pattern: &str, hub_id: &str) -> TransformationRule {
        let template = Edge {
            id: "auto".to_string(),
            source: "".to_string(),
            target: hub_id.to_string(),
            label: "connection".to_string(),
            layer: "".to_string(),
            weight: 1,
            comment: None,
        };
        
        let op = EdgeCreateOp {
            source_pattern: layer_pattern.to_string(),
            target_pattern: hub_id.to_string(),
            edge_template: template,
        };
        
        TransformationRule::new(
            format!("Connect layer {} to hub {}", layer_pattern, hub_id),
            TransformationType::EdgeCreate(op)
        )
    }
    
    /// Create a rule to remove weak edges (edges with low weight)
    pub fn remove_weak_edges(threshold: f64) -> TransformationRule {
        let op = EdgeDeleteOp {
            condition: format!("weight < {}", threshold),
        };
        
        TransformationRule::new(
            format!("Remove edges with weight < {}", threshold),
            TransformationType::EdgeDelete(op)
        )
    }
    
    /// Create a rule to set edge weights based on node similarity
    pub fn set_edge_weights_by_similarity() -> TransformationRule {
        let mut field_mappings = HashMap::new();
        field_mappings.insert("weight".to_string(), "similarity_score".to_string());
        
        let op = EdgeTransformOp {
            field_mappings,
            weight_formula: Some("1.0".to_string()),
        };
        
        TransformationRule::new(
            "Set edge weights based on node similarity".to_string(),
            TransformationType::EdgeTransform(op)
        )
    }
    
    /// Create a rule to filter edges by type
    pub fn filter_edges_by_type(edge_type: &str) -> TransformationRule {
        let op = EdgeFilterOp {
            condition: format!("type = \"{}\"", edge_type),
            validate_nodes: true,
        };
        
        TransformationRule::new(
            format!("Filter edges by type: {}", edge_type),
            TransformationType::EdgeFilter(op)
        )
    }
    
    /// Create a rule to create layers based on node properties
    pub fn create_layer_from_node_property(property: &str, layer_name: &str) -> TransformationRule {
        let template = Layer {
            id: format!("layer_{}", property.to_lowercase()),
            label: layer_name.to_string(),
            background_color: "#3b82f6".to_string(),
            text_color: "#ffffff".to_string(),
            border_color: "#2563eb".to_string(),
        };
        
        let op = LayerCreateOp {
            template,
            auto_assign_nodes: true,
        };
        
        TransformationRule::new(
            format!("Create layer '{}' from property '{}'", layer_name, property),
            TransformationType::LayerCreate(op)
        )
    }
    
    /// Create a rule to apply color scheme to layers
    pub fn apply_layer_color_scheme(scheme: &str) -> TransformationRule {
        let mut field_mappings = HashMap::new();
        
        let op = LayerTransformOp {
            field_mappings,
            color_scheme: Some(scheme.to_string()),
        };
        
        TransformationRule::new(
            format!("Apply color scheme: {}", scheme),
            TransformationType::LayerTransform(op)
        )
    }
    
    /// Create a rule to cluster nodes using modularity
    pub fn cluster_by_modularity() -> TransformationRule {
        let mut parameters = HashMap::new();
        parameters.insert("resolution".to_string(), 1.0);
        
        let op = GraphClusterOp {
            algorithm: ClusteringAlgorithm::Modularity,
            parameters,
        };
        
        TransformationRule::new(
            "Cluster nodes by modularity".to_string(),
            TransformationType::GraphCluster(op)
        )
    }
    
    /// Create a rule to cluster nodes using connected components
    pub fn cluster_by_connected_components() -> TransformationRule {
        let op = GraphClusterOp {
            algorithm: ClusteringAlgorithm::ConnectedComponents,
            parameters: HashMap::new(),
        };
        
        TransformationRule::new(
            "Cluster nodes by connected components".to_string(),
            TransformationType::GraphCluster(op)
        )
    }
}

/// Common transformation patterns
pub struct TransformationPatterns;

impl TransformationPatterns {
    /// Create a pipeline for cleaning and normalizing graph data
    pub fn data_cleaning_pipeline() -> Vec<TransformationRule> {
        vec![
            TransformationRuleTemplates::remove_isolated_nodes(),
            TransformationRuleTemplates::normalize_node_labels_uppercase(),
            TransformationRuleTemplates::remove_weak_edges(0.1),
        ]
    }
    
    /// Create a pipeline for hierarchical analysis
    pub fn hierarchical_analysis_pipeline() -> Vec<TransformationRule> {
        vec![
            TransformationRuleTemplates::create_hub_node("root_hub", "Root Hub", "analysis"),
            TransformationRuleTemplates::connect_layer_to_hub("layer*", "root_hub"),
            TransformationRuleTemplates::cluster_by_modularity(),
        ]
    }
    
    /// Create a pipeline for network simplification
    pub fn network_simplification_pipeline() -> Vec<TransformationRule> {
        vec![
            TransformationRuleTemplates::remove_isolated_nodes(),
            TransformationRuleTemplates::remove_weak_edges(0.3),
            TransformationRuleTemplates::cluster_by_connected_components(),
        ]
    }
    
    /// Create a pipeline for visual enhancement
    pub fn visual_enhancement_pipeline() -> Vec<TransformationRule> {
        vec![
            TransformationRuleTemplates::apply_layer_color_scheme("rainbow"),
            TransformationRuleTemplates::normalize_node_labels_uppercase(),
            TransformationRuleTemplates::set_edge_weights_by_similarity(),
        ]
    }
}

/// Transformation rule builder for creating custom rules
pub struct TransformationRuleBuilder {
    name: String,
    description: Option<String>,
    conditions: Vec<String>,
}

impl TransformationRuleBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            conditions: Vec::new(),
        }
    }
    
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    pub fn condition(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }
    
    pub fn build_node_filter(self, condition: &str, keep_connected: bool) -> TransformationRule {
        let op = NodeFilterOp {
            condition: condition.to_string(),
            keep_connected,
        };
        
        let mut rule = TransformationRule::new(self.name, TransformationType::NodeFilter(op));
        rule.description = self.description;
        rule.conditions = self.conditions;
        rule
    }
    
    pub fn build_edge_filter(self, condition: &str, validate_nodes: bool) -> TransformationRule {
        let op = EdgeFilterOp {
            condition: condition.to_string(),
            validate_nodes,
        };
        
        let mut rule = TransformationRule::new(self.name, TransformationType::EdgeFilter(op));
        rule.description = self.description;
        rule.conditions = self.conditions;
        rule
    }
    
    pub fn build_node_transform(self, field_mappings: HashMap<String, String>) -> TransformationRule {
        let op = NodeTransformOp {
            field_mappings,
            computed_fields: HashMap::new(),
        };
        
        let mut rule = TransformationRule::new(self.name, TransformationType::NodeTransform(op));
        rule.description = self.description;
        rule.conditions = self.conditions;
        rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_nodes_by_layer() {
        let rule = TransformationRuleTemplates::filter_nodes_by_layer("web_tier");
        assert_eq!(rule.name, "Filter nodes by layer: web_tier");
        
        if let TransformationType::NodeFilter(op) = rule.operation {
            assert_eq!(op.condition, "layer_id = \"web_tier\"");
            assert!(!op.keep_connected);
        } else {
            panic!("Expected NodeFilter operation");
        }
    }
    
    #[test]
    fn test_rule_builder() {
        let rule = TransformationRuleBuilder::new("Test Rule")
            .description("A test rule")
            .condition("node_count > 5")
            .build_node_filter("label = \"test\"", true);
        
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.description, Some("A test rule".to_string()));
        assert_eq!(rule.conditions, vec!["node_count > 5"]);
    }
    
    #[test]
    fn test_data_cleaning_pipeline() {
        let pipeline = TransformationPatterns::data_cleaning_pipeline();
        assert_eq!(pipeline.len(), 3);
        assert_eq!(pipeline[0].name, "Remove isolated nodes");
        assert_eq!(pipeline[1].name, "Normalize node labels to uppercase");
        assert!(pipeline[2].name.contains("Remove edges with weight"));
    }
}