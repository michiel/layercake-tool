use std::collections::HashMap;
use layercake::plan::dag_plan::*;
use layercake::transformations::*;

#[test]
fn test_advanced_transformation_node_creation() {
    // Test creating a node clustering transformation
    let clustering_op = NodeClusterOp {
        algorithm: ClusteringAlgorithm::Louvain,
        parameters: [("resolution".to_string(), 1.0)].iter().cloned().collect(),
        create_cluster_layers: true,
        min_cluster_size: Some(3),
    };
    
    let advanced_op = AdvancedTransformOperation::NodeCluster(clustering_op);
    
    let transform_config = TransformNodeConfig {
        transform_type: "node_cluster".to_string(),
        parameters: HashMap::new(),
        script: None,
        script_language: None,
        advanced_operation: Some(advanced_op),
    };
    
    let node = DagPlanNode::new_transform(
        "Cluster Analysis".to_string(),
        transform_config,
    );
    
    assert_eq!(node.name, "Cluster Analysis");
    
    if let PlanNodeConfig::Transform(config) = &node.config {
        assert_eq!(config.transform_type, "node_cluster");
        assert!(config.advanced_operation.is_some());
    } else {
        panic!("Expected transform node config");
    }
}

#[test]
fn test_edge_weight_normalization_node() {
    let normalization_op = EdgeWeightNormalizeOp {
        method: NormalizationMethod::MinMax,
        range: Some((0.0, 1.0)),
        preserve_zero: false,
    };
    
    let advanced_op = AdvancedTransformOperation::EdgeWeightNormalize(normalization_op);
    
    let transform_config = TransformNodeConfig {
        transform_type: "edge_weight_normalize".to_string(),
        parameters: HashMap::new(),
        script: None,
        script_language: None,
        advanced_operation: Some(advanced_op),
    };
    
    let node = DagPlanNode::new_transform(
        "Normalize Weights".to_string(),
        transform_config,
    );
    
    assert_eq!(node.name, "Normalize Weights");
}

#[test]
fn test_graph_analysis_node() {
    let analysis_op = GraphAnalyzeOp {
        metrics: vec![
            GraphMetric::NodeCount,
            GraphMetric::EdgeCount,
            GraphMetric::Density,
        ],
        store_results: true,
        output_format: AnalysisOutputFormat::NodeProperties,
    };
    
    let advanced_op = AdvancedTransformOperation::GraphAnalyze(analysis_op);
    
    let transform_config = TransformNodeConfig {
        transform_type: "graph_analyze".to_string(),
        parameters: HashMap::new(),
        script: None,
        script_language: None,
        advanced_operation: Some(advanced_op),
    };
    
    let node = DagPlanNode::new_transform(
        "Graph Analysis".to_string(),
        transform_config,
    );
    
    assert_eq!(node.name, "Graph Analysis");
}

#[test]
fn test_dag_plan_with_transformation_nodes() {
    let mut plan = DagPlan::new("Advanced Transformation Test".to_string());
    
    // Add import node
    let import_config = ImportNodeConfig {
        source_type: "csv_nodes".to_string(),
        source_path: Some("test_nodes.csv".to_string()),
        import_options: HashMap::new(),
        field_mappings: None,
    };
    
    let import_node = DagPlanNode::new_import(
        "Import Test Data".to_string(),
        import_config,
    );
    
    // Add clustering transformation
    let clustering_op = NodeClusterOp {
        algorithm: ClusteringAlgorithm::ConnectedComponents,
        parameters: HashMap::new(),
        create_cluster_layers: false,
        min_cluster_size: None,
    };
    
    let cluster_transform = DagPlanNode::new_transform(
        "Cluster Nodes".to_string(),
        TransformNodeConfig {
            transform_type: "node_cluster".to_string(),
            parameters: HashMap::new(),
            script: None,
            script_language: None,
            advanced_operation: Some(AdvancedTransformOperation::NodeCluster(clustering_op)),
        },
    );
    
    plan.add_node(import_node.clone());
    plan.add_node(cluster_transform.clone());
    
    // Connect nodes
    let _ = plan.connect_nodes(&import_node.id, &cluster_transform.id);
    
    assert_eq!(plan.nodes.len(), 2);
    assert_eq!(plan.edges.len(), 1);
    
    // Test topological sort
    let execution_order = plan.topological_sort().expect("Valid DAG structure");
    assert_eq!(execution_order.len(), 2);
    assert_eq!(execution_order[0], import_node.id);
    assert_eq!(execution_order[1], cluster_transform.id);
}

#[test]
fn test_transformation_type_conversion() {
    let clustering_op = NodeClusterOp {
        algorithm: ClusteringAlgorithm::Louvain,
        parameters: HashMap::new(),
        create_cluster_layers: true,
        min_cluster_size: Some(2),
    };
    
    let advanced_op = AdvancedTransformOperation::NodeCluster(clustering_op);
    let transformation_type = advanced_op.to_transformation_type();
    
    // Verify it converts to the correct TransformationType
    if let TransformationType::NodeCluster(op) = transformation_type {
        assert_eq!(op.algorithm, ClusteringAlgorithm::Louvain);
        assert_eq!(op.create_cluster_layers, true);
        assert_eq!(op.min_cluster_size, Some(2));
    } else {
        panic!("Expected NodeCluster transformation type");
    }
}