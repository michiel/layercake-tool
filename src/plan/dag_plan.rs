use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Position data for visual layout in DAG editor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

impl Default for NodePosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Plan node types in the DAG
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlanNodeType {
    Import,
    Transform,
    Export,
}

/// Configuration for import plan nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportNodeConfig {
    /// Source type: "csv", "json", "graphml", etc.
    pub source_type: String,
    /// File path or URL for the import source
    pub source_path: Option<String>,
    /// Import options specific to the source type
    pub import_options: HashMap<String, serde_json::Value>,
    /// Field mappings for data transformation during import
    pub field_mappings: Option<HashMap<String, String>>,
}

/// Configuration for transformation plan nodes  
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformNodeConfig {
    /// Transformation type: "filter", "map", "merge", "split", etc.
    pub transform_type: String,
    /// Transformation parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Script or code for custom transformations
    pub script: Option<String>,
    /// Script language: "javascript", "python", etc.
    pub script_language: Option<String>,
}

/// Configuration for export plan nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportNodeConfig {
    /// Export format: "json", "csv", "dot", "plantuml", etc.
    pub format: String,
    /// Output file path or template
    pub output_path: Option<String>,
    /// Export options specific to the format
    pub export_options: HashMap<String, serde_json::Value>,
    /// Custom template for rendering (e.g., Handlebars)
    pub template: Option<String>,
}

/// Plan node configuration (union type)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "config")]
pub enum PlanNodeConfig {
    #[serde(rename = "import")]
    Import(ImportNodeConfig),
    #[serde(rename = "transform")]
    Transform(TransformNodeConfig),
    #[serde(rename = "export")]
    Export(ExportNodeConfig),
}

/// A single node in the DAG plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagPlanNode {
    /// Unique identifier for the node
    pub id: String,
    /// Human-readable name for the node
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Node type and configuration
    pub config: PlanNodeConfig,
    /// Visual position for DAG editor
    pub position: NodePosition,
    /// Optional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl DagPlanNode {
    pub fn new_import(name: String, config: ImportNodeConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            config: PlanNodeConfig::Import(config),
            position: NodePosition::default(),
            metadata: None,
        }
    }

    pub fn new_transform(name: String, config: TransformNodeConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            config: PlanNodeConfig::Transform(config),
            position: NodePosition::default(),
            metadata: None,
        }
    }

    pub fn new_export(name: String, config: ExportNodeConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            config: PlanNodeConfig::Export(config),
            position: NodePosition::default(),
            metadata: None,
        }
    }

    pub fn node_type(&self) -> PlanNodeType {
        match &self.config {
            PlanNodeConfig::Import(_) => PlanNodeType::Import,
            PlanNodeConfig::Transform(_) => PlanNodeType::Transform,
            PlanNodeConfig::Export(_) => PlanNodeType::Export,
        }
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.position = NodePosition { x, y };
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Edge connecting two nodes in the DAG
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagPlanEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Optional label for the edge
    pub label: Option<String>,
    /// Optional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl DagPlanEdge {
    pub fn new(source: String, target: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            target,
            label: None,
            metadata: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

/// Execution context for the plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionContext {
    /// Variables available during execution
    pub variables: HashMap<String, serde_json::Value>,
    /// Working directory for file operations
    pub working_directory: Option<String>,
    /// Timeout for the entire plan execution (seconds)
    pub timeout_seconds: Option<u32>,
    /// Maximum memory usage (MB)
    pub max_memory_mb: Option<u32>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            working_directory: None,
            timeout_seconds: Some(300), // 5 minutes default
            max_memory_mb: Some(1024),  // 1GB default
        }
    }
}

/// Complete DAG-based plan definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagPlan {
    /// Plan metadata
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    
    /// DAG structure
    pub nodes: Vec<DagPlanNode>,
    pub edges: Vec<DagPlanEdge>,
    
    /// Execution configuration
    pub execution_context: ExecutionContext,
    
    /// Plan metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl DagPlan {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            version: "1.0".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            execution_context: ExecutionContext::default(),
            metadata: None,
        }
    }

    pub fn add_node(&mut self, node: DagPlanNode) -> &mut Self {
        self.nodes.push(node);
        self
    }

    pub fn add_edge(&mut self, edge: DagPlanEdge) -> &mut Self {
        self.edges.push(edge);
        self
    }

    pub fn connect_nodes(&mut self, source_id: &str, target_id: &str) -> Result<&mut Self, String> {
        // Validate that both nodes exist
        let source_exists = self.nodes.iter().any(|n| n.id == source_id);
        let target_exists = self.nodes.iter().any(|n| n.id == target_id);
        
        if !source_exists {
            return Err(format!("Source node '{}' not found", source_id));
        }
        if !target_exists {
            return Err(format!("Target node '{}' not found", target_id));
        }
        
        let edge = DagPlanEdge::new(source_id.to_string(), target_id.to_string());
        self.edges.push(edge);
        Ok(self)
    }

    pub fn get_node(&self, id: &str) -> Option<&DagPlanNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut DagPlanNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Check for duplicate node IDs
        let mut node_ids = std::collections::HashSet::new();
        for node in &self.nodes {
            if !node_ids.insert(&node.id) {
                errors.push(format!("Duplicate node ID: {}", node.id));
            }
        }
        
        // Check edge references
        for edge in &self.edges {
            if !self.nodes.iter().any(|n| n.id == edge.source) {
                errors.push(format!("Edge references non-existent source node: {}", edge.source));
            }
            if !self.nodes.iter().any(|n| n.id == edge.target) {
                errors.push(format!("Edge references non-existent target node: {}", edge.target));
            }
        }
        
        // Check for cycles (basic check - could be more sophisticated)
        if self.has_cycles() {
            errors.push("Plan contains cycles, which are not allowed in DAG".to_string());
        }
        
        // Validate that there's at least one import node
        if !self.nodes.iter().any(|n| matches!(n.config, PlanNodeConfig::Import(_))) {
            errors.push("Plan must contain at least one import node".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Simple cycle detection using DFS
    fn has_cycles(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        for node in &self.nodes {
            if !visited.contains(&node.id) {
                if self.has_cycle_dfs(&node.id, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }
    
    fn has_cycle_dfs(
        &self,
        node_id: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());
        
        // Visit all adjacent nodes
        for edge in &self.edges {
            if edge.source == node_id {
                if !visited.contains(&edge.target) {
                    if self.has_cycle_dfs(&edge.target, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&edge.target) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node_id);
        false
    }

    /// Get nodes in topological order for execution
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize in-degree count and adjacency list
        for node in &self.nodes {
            in_degree.insert(node.id.clone(), 0);
            adj_list.insert(node.id.clone(), Vec::new());
        }
        
        for edge in &self.edges {
            adj_list.get_mut(&edge.source).unwrap().push(edge.target.clone());
            *in_degree.get_mut(&edge.target).unwrap() += 1;
        }
        
        // Kahn's algorithm
        let mut queue = std::collections::VecDeque::new();
        let mut result = Vec::new();
        
        // Find all nodes with no incoming edges
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }
        
        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());
            
            // For each neighbor of the current node
            if let Some(neighbors) = adj_list.get(&node_id) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        if result.len() != self.nodes.len() {
            Err("Graph contains a cycle".to_string())
        } else {
            Ok(result)
        }
    }
}