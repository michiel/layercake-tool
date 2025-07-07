//! Graph transformation pipeline module
//!
//! This module provides a comprehensive transformation system for graph data,
//! including node and edge operations, validation, and rollback capabilities.

pub mod engine;
pub mod operations;
pub mod rules;
pub mod validation;

pub use engine::TransformationEngine;
pub use operations::TransformationOperations;
pub use validation::TransformationValidator;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::graph::{Graph, Node, Edge, Layer};

/// Transformation operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    /// Node operations
    NodeFilter(NodeFilterOp),
    NodeTransform(NodeTransformOp),
    NodeCreate(NodeCreateOp),
    NodeDelete(NodeDeleteOp),
    NodeCluster(NodeClusterOp),
    
    /// Edge operations
    EdgeFilter(EdgeFilterOp),
    EdgeTransform(EdgeTransformOp),
    EdgeCreate(EdgeCreateOp),
    EdgeDelete(EdgeDeleteOp),
    EdgeWeightNormalize(EdgeWeightNormalizeOp),
    
    /// Layer operations
    LayerFilter(LayerFilterOp),
    LayerTransform(LayerTransformOp),
    LayerCreate(LayerCreateOp),
    LayerDelete(LayerDeleteOp),
    LayerMerge(LayerMergeOp),
    
    /// Graph operations
    GraphMerge(GraphMergeOp),
    GraphSplit(GraphSplitOp),
    GraphCluster(GraphClusterOp),
    GraphAnalyze(GraphAnalyzeOp),
    GraphLayout(GraphLayoutOp),
    
    /// Advanced operations
    SubgraphExtract(SubgraphExtractOp),
    PathFinding(PathFindingOp),
    CentralityCalculation(CentralityCalculationOp),
    CommunityDetection(CommunityDetectionOp),
}

/// Node filtering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFilterOp {
    pub condition: String,
    pub keep_connected: bool,
}

/// Node transformation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTransformOp {
    pub field_mappings: HashMap<String, String>,
    pub computed_fields: HashMap<String, String>,
}

/// Node creation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCreateOp {
    pub template: Node,
    pub count: Option<usize>,
    pub id_pattern: Option<String>,
}

/// Node deletion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDeleteOp {
    pub condition: String,
    pub cascade_edges: bool,
}

/// Edge filtering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeFilterOp {
    pub condition: String,
    pub validate_nodes: bool,
}

/// Edge transformation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTransformOp {
    pub field_mappings: HashMap<String, String>,
    pub weight_formula: Option<String>,
}

/// Edge creation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCreateOp {
    pub source_pattern: String,
    pub target_pattern: String,
    pub edge_template: Edge,
}

/// Edge deletion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDeleteOp {
    pub condition: String,
}

/// Layer filtering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerFilterOp {
    pub condition: String,
}

/// Layer transformation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerTransformOp {
    pub field_mappings: HashMap<String, String>,
    pub color_scheme: Option<String>,
}

/// Layer creation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerCreateOp {
    pub template: Layer,
    pub auto_assign_nodes: bool,
}

/// Layer deletion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDeleteOp {
    pub condition: String,
    pub reassign_nodes: Option<String>,
}

/// Graph merging operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMergeOp {
    pub merge_strategy: MergeStrategy,
    pub conflict_resolution: ConflictResolution,
}

/// Graph splitting operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSplitOp {
    pub split_criteria: String,
    pub preserve_edges: bool,
}

/// Graph clustering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphClusterOp {
    pub algorithm: ClusteringAlgorithm,
    pub parameters: HashMap<String, f64>,
}

/// Merge strategies for combining graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    Union,
    Intersection,
    LeftJoin,
    RightJoin,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepFirst,
    KeepLast,
    Merge,
    Error,
}

/// Clustering algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusteringAlgorithm {
    ConnectedComponents,
    Modularity,
    KMeans,
    Hierarchical,
    Louvain,
    LabelPropagation,
}

/// Node clustering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeClusterOp {
    pub algorithm: ClusteringAlgorithm,
    pub parameters: HashMap<String, f64>,
    pub create_cluster_layers: bool,
    pub min_cluster_size: Option<usize>,
}

/// Edge weight normalization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeWeightNormalizeOp {
    pub method: NormalizationMethod,
    pub range: Option<(f64, f64)>,
    pub preserve_zero: bool,
}

/// Layer merging operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMergeOp {
    pub source_layers: Vec<String>,
    pub target_layer: String,
    pub merge_strategy: LayerMergeStrategy,
}

/// Graph analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalyzeOp {
    pub metrics: Vec<GraphMetric>,
    pub store_results: bool,
    pub output_format: AnalysisOutputFormat,
}

/// Graph layout operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLayoutOp {
    pub algorithm: LayoutAlgorithm,
    pub parameters: HashMap<String, f64>,
    pub iterations: Option<usize>,
    pub update_positions: bool,
}

/// Subgraph extraction operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphExtractOp {
    pub criteria: ExtractionCriteria,
    pub preserve_structure: bool,
    pub include_boundary_edges: bool,
}

/// Path finding operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathFindingOp {
    pub algorithm: PathAlgorithm,
    pub source: String,
    pub target: Option<String>,
    pub max_depth: Option<usize>,
    pub create_path_edges: bool,
}

/// Centrality calculation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityCalculationOp {
    pub measures: Vec<CentralityMeasure>,
    pub normalize: bool,
    pub store_as_node_property: bool,
}

/// Community detection operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDetectionOp {
    pub algorithm: CommunityAlgorithm,
    pub resolution: Option<f64>,
    pub create_community_layers: bool,
    pub min_community_size: Option<usize>,
}

/// Normalization methods for edge weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    MinMax,
    ZScore,
    Robust,
    UnitVector,
}

/// Layer merge strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerMergeStrategy {
    Union,
    Intersection,
    FirstWins,
    LastWins,
}

/// Graph metrics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphMetric {
    NodeCount,
    EdgeCount,
    Density,
    AverageClusteringCoefficient,
    AverageShortestPath,
    Diameter,
    ConnectedComponents,
    StronglyConnectedComponents,
}

/// Analysis output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisOutputFormat {
    NodeProperties,
    SeparateGraph,
    Report,
    Json,
}

/// Layout algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    ForceDirected,
    Circular,
    Hierarchical,
    Grid,
    Random,
    Fruchterman,
    Kamada,
}

/// Extraction criteria for subgraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionCriteria {
    NodeList(Vec<String>),
    LayerList(Vec<String>),
    NodeProperty(String, String),
    NeighborhoodRadius(String, usize),
    ConnectedComponent(String),
}

/// Path finding algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathAlgorithm {
    Dijkstra,
    BellmanFord,
    FloydWarshall,
    AStar,
    BFS,
    DFS,
}

/// Centrality measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CentralityMeasure {
    Degree,
    Betweenness,
    Closeness,
    Eigenvector,
    PageRank,
    Katz,
}

/// Community detection algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunityAlgorithm {
    Louvain,
    LabelPropagation,
    WalkTrap,
    EdgeBetweenness,
    Modularity,
}

/// A single transformation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub operation: TransformationType,
    pub enabled: bool,
    pub conditions: Vec<String>,
}

/// A transformation pipeline containing multiple rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationPipeline {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<TransformationRule>,
    pub validation_enabled: bool,
    pub rollback_enabled: bool,
}

/// Result of applying a transformation
#[derive(Debug, Clone)]
pub struct TransformationResult {
    pub success: bool,
    pub rule_id: String,
    pub original_graph: Option<Graph>,
    pub transformed_graph: Option<Graph>,
    pub error: Option<String>,
    pub statistics: TransformationStatistics,
}

/// Statistics about transformation operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformationStatistics {
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub nodes_modified: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub edges_modified: usize,
    pub layers_added: usize,
    pub layers_removed: usize,
    pub layers_modified: usize,
    pub execution_time_ms: u64,
}

/// Validation result for transformations
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl TransformationPipeline {
    /// Create a new empty pipeline
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            rules: Vec::new(),
            validation_enabled: true,
            rollback_enabled: true,
        }
    }
    
    /// Add a rule to the pipeline
    pub fn add_rule(&mut self, rule: TransformationRule) {
        self.rules.push(rule);
    }
    
    /// Remove a rule from the pipeline
    pub fn remove_rule(&mut self, rule_id: &str) {
        self.rules.retain(|r| r.id != rule_id);
    }
    
    /// Get all enabled rules
    pub fn enabled_rules(&self) -> Vec<&TransformationRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }
}

impl TransformationRule {
    /// Create a new transformation rule
    pub fn new(name: String, operation: TransformationType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            operation,
            enabled: true,
            conditions: Vec::new(),
        }
    }
    
    /// Add a condition to the rule
    pub fn add_condition(&mut self, condition: String) {
        self.conditions.push(condition);
    }
}

impl Default for TransformationPipeline {
    fn default() -> Self {
        Self::new("Unnamed Pipeline".to_string())
    }
}