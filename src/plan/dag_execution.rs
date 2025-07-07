use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use sea_orm::DatabaseConnection;

use crate::graph::Graph;
use super::dag_plan::*;
use crate::services::{GraphService, ImportService, ExportService};

/// Execution status for individual plan nodes
#[derive(Debug, Clone, PartialEq)]
pub enum NodeExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

/// Result of executing a plan node
#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    pub node_id: String,
    pub status: NodeExecutionStatus,
    pub graph_id: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub error_message: Option<String>,
}

/// Overall execution state for a DAG plan
#[derive(Debug, Clone)]
pub struct DagExecutionState {
    pub execution_id: String,
    pub plan_name: String,
    pub status: ExecutionStatus,
    pub node_results: HashMap<String, NodeExecutionResult>,
    pub execution_order: Vec<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_nodes: usize,
    pub completed_nodes: usize,
    pub failed_nodes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl DagExecutionState {
    pub fn new(execution_id: String, plan: &DagPlan) -> Self {
        let execution_order = plan.topological_sort().unwrap_or_default();
        
        Self {
            execution_id,
            plan_name: plan.name.clone(),
            status: ExecutionStatus::Pending,
            node_results: HashMap::new(),
            execution_order,
            started_at: chrono::Utc::now(),
            completed_at: None,
            total_nodes: plan.nodes.len(),
            completed_nodes: 0,
            failed_nodes: 0,
        }
    }

    pub fn update_node_result(&mut self, result: NodeExecutionResult) {
        match result.status {
            NodeExecutionStatus::Completed => {
                self.completed_nodes += 1;
            }
            NodeExecutionStatus::Failed(_) => {
                self.failed_nodes += 1;
            }
            _ => {}
        }
        
        self.node_results.insert(result.node_id.clone(), result);
        
        // Update overall status
        if self.failed_nodes > 0 {
            self.status = ExecutionStatus::Failed;
        } else if self.completed_nodes == self.total_nodes {
            self.status = ExecutionStatus::Completed;
            self.completed_at = Some(chrono::Utc::now());
        }
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_nodes == 0 {
            return 100.0;
        }
        (self.completed_nodes as f64 / self.total_nodes as f64) * 100.0
    }
}

/// Context available to nodes during execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub plan_id: i32,
    pub working_directory: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub graph_artifacts: HashMap<String, String>, // node_id -> graph_id
}

/// DAG execution engine
pub struct DagExecutionEngine {
    db: DatabaseConnection,
    graph_service: Arc<GraphService>,
    import_service: Arc<ImportService>,
    export_service: Arc<ExportService>,
    execution_states: Arc<RwLock<HashMap<String, DagExecutionState>>>,
}

impl DagExecutionEngine {
    pub fn new(
        db: DatabaseConnection,
        graph_service: Arc<GraphService>,
        import_service: Arc<ImportService>,
        export_service: Arc<ExportService>,
    ) -> Self {
        Self {
            db,
            graph_service,
            import_service,
            export_service,
            execution_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a DAG plan
    pub async fn execute_plan(
        &self,
        plan: DagPlan,
        plan_id: i32,
        context: ExecutionContext,
    ) -> Result<String> {
        // Validate plan first
        plan.validate().map_err(|errors| {
            anyhow!("Plan validation failed: {}", errors.join(", "))
        })?;

        let execution_id = Uuid::new_v4().to_string();
        info!("Starting DAG execution {} for plan: {}", execution_id, plan.name);

        // Initialize execution state
        let mut execution_state = DagExecutionState::new(execution_id.clone(), &plan);
        execution_state.status = ExecutionStatus::Running;

        // Store execution state
        {
            let mut states = self.execution_states.write().await;
            states.insert(execution_id.clone(), execution_state.clone());
        }

        // Get execution order
        let execution_order = plan.topological_sort().map_err(|e| {
            anyhow!("Failed to determine execution order: {}", e)
        })?;

        debug!("Execution order: {:?}", execution_order);

        // Execute nodes in topological order
        let mut node_graphs: HashMap<String, String> = HashMap::new(); // node_id -> graph_id
        
        for node_id in &execution_order {
            let node = plan.get_node(node_id).ok_or_else(|| {
                anyhow!("Node {} not found in plan", node_id)
            })?;

            info!("Executing node: {} ({})", node.name, node_id);
            let start_time = std::time::Instant::now();

            let result = self.execute_node(
                node,
                &plan,
                plan_id,
                &context,
                &node_graphs,
            ).await;

            let execution_time = start_time.elapsed().as_millis() as u64;

            let node_result = match result {
                Ok(graph_id) => {
                    if let Some(ref gid) = graph_id {
                        node_graphs.insert(node_id.clone(), gid.clone());
                    }
                    
                    NodeExecutionResult {
                        node_id: node_id.clone(),
                        status: NodeExecutionStatus::Completed,
                        graph_id,
                        execution_time_ms: execution_time,
                        metadata: HashMap::new(),
                        error_message: None,
                    }
                }
                Err(e) => {
                    error!("Node {} failed: {}", node_id, e);
                    NodeExecutionResult {
                        node_id: node_id.clone(),
                        status: NodeExecutionStatus::Failed(e.to_string()),
                        graph_id: None,
                        execution_time_ms: execution_time,
                        metadata: HashMap::new(),
                        error_message: Some(e.to_string()),
                    }
                }
            };

            // Update execution state
            {
                let mut states = self.execution_states.write().await;
                if let Some(state) = states.get_mut(&execution_id) {
                    state.update_node_result(node_result.clone());
                }
            }

            // Stop execution if a node failed (fail-fast strategy)
            if matches!(node_result.status, NodeExecutionStatus::Failed(_)) {
                warn!("Stopping execution due to node failure: {}", node_id);
                break;
            }
        }

        // Get final execution state
        let final_state = {
            let states = self.execution_states.read().await;
            states.get(&execution_id).cloned()
        };

        if let Some(state) = final_state {
            info!(
                "DAG execution {} completed with status: {:?}, progress: {:.1}%",
                execution_id,
                state.status,
                state.progress_percentage()
            );
        }

        Ok(execution_id)
    }

    /// Execute a single node in the DAG
    async fn execute_node(
        &self,
        node: &DagPlanNode,
        plan: &DagPlan,
        plan_id: i32,
        context: &ExecutionContext,
        node_graphs: &HashMap<String, String>,
    ) -> Result<Option<String>> {
        match &node.config {
            PlanNodeConfig::Import(config) => {
                self.execute_import_node(node, config, plan_id, context).await
            }
            PlanNodeConfig::Transform(config) => {
                self.execute_transform_node(node, config, plan, plan_id, context, node_graphs).await
            }
            PlanNodeConfig::Export(config) => {
                self.execute_export_node(node, config, plan, plan_id, context, node_graphs).await
            }
        }
    }

    /// Execute an import node
    async fn execute_import_node(
        &self,
        node: &DagPlanNode,
        config: &ImportNodeConfig,
        plan_id: i32,
        context: &ExecutionContext,
    ) -> Result<Option<String>> {
        debug!("Executing import node: {} with source type: {}", node.name, config.source_type);

        // For now, we'll create a simple import based on the source type
        match config.source_type.as_str() {
            "csv_nodes" | "csv_edges" | "csv_layers" => {
                // In a real implementation, this would load from the specified path
                // For now, we'll create an empty graph as a placeholder
                let graph = Graph::default();
                
                // Store the graph in the database
                let graph_id = self.graph_service.create_graph_artifact(
                    plan_id,
                    &node.id,
                    &format!("Imported graph from {}", node.name),
                    &graph,
                    HashMap::new(),
                ).await?;

                info!("Import node {} created graph artifact: {}", node.name, graph_id);
                Ok(Some(graph_id))
            }
            "migrated_from_project_data" => {
                // This is data that was already migrated, so we should find the existing graph
                if let Some(graph_id) = &node.id.get(0..36) { // Extract potential UUID
                    debug!("Import node {} references migrated data", node.name);
                    // In a real implementation, we'd verify the graph exists
                    return Ok(Some(graph_id.to_string()));
                }
                Err(anyhow!("Migrated data not found for import node: {}", node.name))
            }
            _ => {
                Err(anyhow!("Unsupported import source type: {}", config.source_type))
            }
        }
    }

    /// Execute a transform node
    async fn execute_transform_node(
        &self,
        node: &DagPlanNode,
        config: &TransformNodeConfig,
        plan: &DagPlan,
        plan_id: i32,
        context: &ExecutionContext,
        node_graphs: &HashMap<String, String>,
    ) -> Result<Option<String>> {
        debug!("Executing transform node: {} with type: {}", node.name, config.transform_type);

        // Find input graphs from predecessor nodes
        let mut input_graphs = Vec::new();
        for edge in &plan.edges {
            if edge.target == node.id {
                if let Some(input_graph_id) = node_graphs.get(&edge.source) {
                    let graph = self.graph_service.get_graph_by_id(input_graph_id).await?;
                    input_graphs.push(graph);
                }
            }
        }

        if input_graphs.is_empty() {
            return Err(anyhow!("Transform node {} has no input graphs", node.name));
        }

        // Apply transformation based on type
        let output_graph = match config.transform_type.as_str() {
            "filter" => {
                // Apply filtering transformation
                self.apply_filter_transform(&input_graphs[0], &config.parameters).await?
            }
            "merge" => {
                // Merge multiple input graphs
                self.apply_merge_transform(&input_graphs, &config.parameters).await?
            }
            "map" => {
                // Apply field mapping transformation
                self.apply_map_transform(&input_graphs[0], &config.parameters).await?
            }
            _ => {
                return Err(anyhow!("Unsupported transform type: {}", config.transform_type));
            }
        };

        // Store the transformed graph
        let graph_id = self.graph_service.create_graph_artifact(
            plan_id,
            &node.id,
            &format!("Transformed graph from {}", node.name),
            &output_graph,
            HashMap::new(),
        ).await?;

        info!("Transform node {} created graph artifact: {}", node.name, graph_id);
        Ok(Some(graph_id))
    }

    /// Execute an export node
    async fn execute_export_node(
        &self,
        node: &DagPlanNode,
        config: &ExportNodeConfig,
        plan: &DagPlan,
        plan_id: i32,
        context: &ExecutionContext,
        node_graphs: &HashMap<String, String>,
    ) -> Result<Option<String>> {
        debug!("Executing export node: {} with format: {}", node.name, config.format);

        // Find input graphs from predecessor nodes
        let mut input_graphs = Vec::new();
        for edge in &plan.edges {
            if edge.target == node.id {
                if let Some(input_graph_id) = node_graphs.get(&edge.source) {
                    let graph = self.graph_service.get_graph_by_id(input_graph_id).await?;
                    input_graphs.push(graph);
                }
            }
        }

        if input_graphs.is_empty() {
            return Err(anyhow!("Export node {} has no input graphs", node.name));
        }

        // Use the first input graph for export
        let graph = &input_graphs[0];

        // Perform export based on format
        let default_path = format!("output_{}.{}", node.id, config.format);
        let output_path = config.output_path.as_ref()
            .unwrap_or(&default_path);

        match config.format.as_str() {
            "json" => {
                // Export as JSON
                self.export_service.export_as_json(graph, output_path).await?;
            }
            "csv" => {
                // Export as CSV
                self.export_service.export_as_csv(graph, output_path).await?;
            }
            "dot" => {
                // Export as DOT
                self.export_service.export_as_dot(graph, output_path).await?;
            }
            _ => {
                return Err(anyhow!("Unsupported export format: {}", config.format));
            }
        }

        info!("Export node {} exported to: {}", node.name, output_path);
        
        // Export nodes don't produce graph artifacts, just output files
        Ok(None)
    }

    /// Apply filter transformation
    async fn apply_filter_transform(
        &self,
        input_graph: &Graph,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<Graph> {
        let mut output_graph = input_graph.clone();
        
        // Example: filter nodes by layer
        if let Some(layer_filter) = parameters.get("layer") {
            if let Some(layer_name) = layer_filter.as_str() {
                output_graph.nodes.retain(|node| {
                    node.layer == layer_name
                });
                
                // Remove edges that reference filtered nodes
                let node_ids: HashSet<_> = output_graph.nodes.iter().map(|n| &n.id).collect();
                output_graph.edges.retain(|edge| {
                    node_ids.contains(&edge.source) && node_ids.contains(&edge.target)
                });
            }
        }
        
        Ok(output_graph)
    }

    /// Apply merge transformation
    async fn apply_merge_transform(
        &self,
        input_graphs: &[Graph],
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<Graph> {
        let mut output_graph = Graph::default();
        
        // Merge all nodes and edges from input graphs
        for graph in input_graphs {
            for node in &graph.nodes {
                // Avoid duplicates by checking if node already exists
                if !output_graph.nodes.iter().any(|n| n.id == node.id) {
                    output_graph.nodes.push(node.clone());
                }
            }
            
            for edge in &graph.edges {
                // Avoid duplicate edges
                if !output_graph.edges.iter().any(|e| 
                    e.source == edge.source && e.target == edge.target
                ) {
                    output_graph.edges.push(edge.clone());
                }
            }
            
            for layer in &graph.layers {
                // Avoid duplicate layers
                if !output_graph.layers.iter().any(|l| l.id == layer.id) {
                    output_graph.layers.push(layer.clone());
                }
            }
        }
        
        Ok(output_graph)
    }

    /// Apply field mapping transformation
    async fn apply_map_transform(
        &self,
        input_graph: &Graph,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<Graph> {
        let mut output_graph = input_graph.clone();
        
        // Example: rename node labels based on mapping
        if let Some(mappings) = parameters.get("label_mappings") {
            if let Some(mappings_obj) = mappings.as_object() {
                for node in &mut output_graph.nodes {
                    if let Some(new_label) = mappings_obj.get(&node.label) {
                        if let Some(new_label_str) = new_label.as_str() {
                            node.label = new_label_str.to_string();
                        }
                    }
                }
            }
        }
        
        Ok(output_graph)
    }

    /// Get execution state
    pub async fn get_execution_state(&self, execution_id: &str) -> Option<DagExecutionState> {
        let states = self.execution_states.read().await;
        states.get(execution_id).cloned()
    }

    /// Cancel execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<()> {
        let mut states = self.execution_states.write().await;
        if let Some(state) = states.get_mut(execution_id) {
            state.status = ExecutionStatus::Cancelled;
            info!("Execution {} cancelled", execution_id);
            Ok(())
        } else {
            Err(anyhow!("Execution {} not found", execution_id))
        }
    }

    /// List all execution states
    pub async fn list_executions(&self) -> Vec<DagExecutionState> {
        let states = self.execution_states.read().await;
        states.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::dag_plan::{DagPlan, DagPlanNode, ImportNodeConfig, ExportNodeConfig};

    #[test]
    fn test_execution_state_creation() {
        let mut plan = DagPlan::new("Test Plan".to_string());
        
        let import_node = DagPlanNode::new_import(
            "Import Node".to_string(),
            ImportNodeConfig {
                source_type: "csv".to_string(),
                source_path: Some("test.csv".to_string()),
                import_options: HashMap::new(),
                field_mappings: None,
            }
        );
        
        plan.add_node(import_node);
        
        let state = DagExecutionState::new("test-execution".to_string(), &plan);
        
        assert_eq!(state.execution_id, "test-execution");
        assert_eq!(state.plan_name, "Test Plan");
        assert_eq!(state.status, ExecutionStatus::Pending);
        assert_eq!(state.total_nodes, 1);
        assert_eq!(state.completed_nodes, 0);
    }

    #[test]
    fn test_execution_state_progress() {
        let plan = DagPlan::new("Test Plan".to_string());
        let mut state = DagExecutionState::new("test-execution".to_string(), &plan);
        
        assert_eq!(state.progress_percentage(), 100.0); // Empty plan is 100% complete
        
        // Add a mock node result
        state.total_nodes = 2;
        let result = NodeExecutionResult {
            node_id: "node-1".to_string(),
            status: NodeExecutionStatus::Completed,
            graph_id: Some("graph-1".to_string()),
            execution_time_ms: 100,
            metadata: HashMap::new(),
            error_message: None,
        };
        
        state.update_node_result(result);
        assert_eq!(state.progress_percentage(), 50.0);
        assert_eq!(state.completed_nodes, 1);
    }
}