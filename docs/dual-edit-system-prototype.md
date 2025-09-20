# Dual-Edit System Prototype Design

## Architecture Overview

The dual-edit system combines real-time CRDT collaboration with reproducible operation tracking for DAG re-execution. This design integrates with the existing `Graph` structure while adding new capabilities.

## Core Components

### 1. Edit Operation Types

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GraphEditOperation {
    NodeCreate {
        node: Node,
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    NodeUpdate {
        node_id: String,
        old_values: HashMap<String, serde_json::Value>,
        new_values: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    NodeDelete {
        node: Node, // Store full node for rollback
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    EdgeCreate {
        edge: Edge,
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    EdgeUpdate {
        edge_id: String,
        old_values: HashMap<String, serde_json::Value>,
        new_values: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    EdgeDelete {
        edge: Edge, // Store full edge for rollback
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    LayerCreate {
        layer: Layer,
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    LayerUpdate {
        layer_id: String,
        old_values: HashMap<String, serde_json::Value>,
        new_values: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
    LayerDelete {
        layer: Layer, // Store full layer for rollback
        timestamp: DateTime<Utc>,
        user_id: i32,
    },
}

impl GraphEditOperation {
    pub fn get_operation_id(&self) -> String {
        // Generate deterministic ID based on content and timestamp
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(self).unwrap());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    pub fn get_affected_entity(&self) -> (String, String) {
        match self {
            Self::NodeCreate { node, .. } => ("node".to_string(), node.id.clone()),
            Self::NodeUpdate { node_id, .. } => ("node".to_string(), node_id.clone()),
            Self::NodeDelete { node, .. } => ("node".to_string(), node.id.clone()),
            Self::EdgeCreate { edge, .. } => ("edge".to_string(), edge.id.clone()),
            Self::EdgeUpdate { edge_id, .. } => ("edge".to_string(), edge_id.clone()),
            Self::EdgeDelete { edge, .. } => ("edge".to_string(), edge.id.clone()),
            Self::LayerCreate { layer, .. } => ("layer".to_string(), layer.id.clone()),
            Self::LayerUpdate { layer_id, .. } => ("layer".to_string(), layer_id.clone()),
            Self::LayerDelete { layer, .. } => ("layer".to_string(), layer.id.clone()),
        }
    }
}
```

### 2. Dual-Edit Manager

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DualEditManager {
    // Real-time collaboration via CRDT
    crdt_manager: Arc<CRDTManager>,

    // Reproducible operations tracking
    operation_tracker: Arc<OperationTracker>,

    // Current graph state
    current_graph: Arc<RwLock<Graph>>,

    // Database connection
    db: DatabaseConnection,

    // Graph ID
    graph_id: i32,
}

impl DualEditManager {
    pub fn new(
        graph_id: i32,
        initial_graph: Graph,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            crdt_manager: Arc::new(CRDTManager::new(graph_id)),
            operation_tracker: Arc::new(OperationTracker::new(graph_id, db.clone())),
            current_graph: Arc::new(RwLock::new(initial_graph)),
            db,
            graph_id,
        }
    }

    /// Apply edit operation in both real-time and tracked mode
    pub async fn apply_edit(
        &self,
        operation: GraphEditOperation,
        context: EditContext,
    ) -> Result<EditResult, DualEditError> {
        let operation_id = operation.get_operation_id();

        // 1. Validate operation applicability
        self.validate_operation(&operation).await?;

        // 2. Apply to current graph state
        let mut graph = self.current_graph.write().await;
        self.apply_operation_to_graph(&mut graph, &operation)?;

        // 3. Apply via CRDT for real-time collaboration
        let crdt_result = self.crdt_manager
            .apply_operation(&operation, context.clone())
            .await?;

        // 4. Track operation for reproducibility (if not from DAG replay)
        if !context.is_dag_replay {
            self.operation_tracker
                .record_operation(&operation, context.dag_state_hash.clone())
                .await?;
        }

        // 5. Broadcast to collaborators
        self.broadcast_operation(&operation, &crdt_result).await?;

        Ok(EditResult {
            operation_id,
            crdt_vector: crdt_result.vector,
            conflicts: crdt_result.conflicts,
            graph_state_hash: self.compute_graph_hash(&graph),
        })
    }

    /// Re-execute DAG with tracked operations
    pub async fn replay_operations_for_dag(
        &self,
        base_graph: Graph,
        dag_state_hash: String,
    ) -> Result<Graph, DualEditError> {
        // Get applicable operations for this DAG state
        let operations = self.operation_tracker
            .get_applicable_operations(&dag_state_hash)
            .await?;

        let mut result_graph = base_graph;
        let mut applied_operations = Vec::new();
        let mut failed_operations = Vec::new();

        for operation in operations {
            // Check if operation is still applicable
            if self.is_operation_applicable(&result_graph, &operation) {
                // Apply operation with DAG replay context
                let context = EditContext {
                    user_id: operation.get_user_id(),
                    session_id: "dag_replay".to_string(),
                    is_dag_replay: true,
                    dag_state_hash: dag_state_hash.clone(),
                };

                match self.apply_operation_to_graph(&mut result_graph, &operation) {
                    Ok(_) => applied_operations.push(operation),
                    Err(e) => {
                        tracing::warn!("Failed to apply operation during DAG replay: {}", e);
                        failed_operations.push(operation);
                    }
                }
            } else {
                tracing::info!("Operation no longer applicable, removing from tracking");
                failed_operations.push(operation.clone());

                // Remove non-applicable operations from tracking
                self.operation_tracker
                    .mark_operation_inapplicable(&operation.get_operation_id())
                    .await?;
            }
        }

        tracing::info!(
            "DAG replay complete: {} applied, {} failed/removed",
            applied_operations.len(),
            failed_operations.len()
        );

        Ok(result_graph)
    }

    fn apply_operation_to_graph(
        &self,
        graph: &mut Graph,
        operation: &GraphEditOperation,
    ) -> Result<(), DualEditError> {
        match operation {
            GraphEditOperation::NodeCreate { node, .. } => {
                graph.set_node(node.clone());
            },
            GraphEditOperation::NodeUpdate { node_id, new_values, .. } => {
                if let Some(existing_node) = graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                    self.apply_field_updates_to_node(existing_node, new_values)?;
                } else {
                    return Err(DualEditError::NodeNotFound(node_id.clone()));
                }
            },
            GraphEditOperation::NodeDelete { node, .. } => {
                graph.remove_node(node.id.clone());
            },
            GraphEditOperation::EdgeCreate { edge, .. } => {
                graph.edges.push(edge.clone());
            },
            GraphEditOperation::EdgeUpdate { edge_id, new_values, .. } => {
                if let Some(existing_edge) = graph.edges.iter_mut().find(|e| e.id == *edge_id) {
                    self.apply_field_updates_to_edge(existing_edge, new_values)?;
                } else {
                    return Err(DualEditError::EdgeNotFound(edge_id.clone()));
                }
            },
            GraphEditOperation::EdgeDelete { edge, .. } => {
                graph.edges.retain(|e| e.id != edge.id);
            },
            GraphEditOperation::LayerCreate { layer, .. } => {
                graph.add_layer(layer.clone());
            },
            GraphEditOperation::LayerUpdate { layer_id, new_values, .. } => {
                if let Some(existing_layer) = graph.layers.iter_mut().find(|l| l.id == *layer_id) {
                    self.apply_field_updates_to_layer(existing_layer, new_values)?;
                } else {
                    return Err(DualEditError::LayerNotFound(layer_id.clone()));
                }
            },
            GraphEditOperation::LayerDelete { layer, .. } => {
                graph.layers.retain(|l| l.id != layer.id);
            },
        }

        // Verify graph integrity after operation
        graph.verify_graph_integrity()
            .map_err(|errors| DualEditError::GraphIntegrityViolation(errors))?;

        Ok(())
    }

    fn is_operation_applicable(&self, graph: &Graph, operation: &GraphEditOperation) -> bool {
        match operation {
            GraphEditOperation::NodeCreate { node, .. } => {
                // Check if node doesn't already exist
                graph.get_node_by_id(&node.id).is_none()
            },
            GraphEditOperation::NodeUpdate { node_id, .. } => {
                // Check if node exists
                graph.get_node_by_id(node_id).is_some()
            },
            GraphEditOperation::NodeDelete { node, .. } => {
                // Check if node exists and can be safely deleted
                graph.get_node_by_id(&node.id).is_some()
            },
            GraphEditOperation::EdgeCreate { edge, .. } => {
                // Check if edge doesn't exist and source/target nodes exist
                !graph.edges.iter().any(|e| e.id == edge.id) &&
                graph.get_node_by_id(&edge.source).is_some() &&
                graph.get_node_by_id(&edge.target).is_some()
            },
            GraphEditOperation::EdgeUpdate { edge_id, .. } => {
                // Check if edge exists
                graph.edges.iter().any(|e| e.id == *edge_id)
            },
            GraphEditOperation::EdgeDelete { edge, .. } => {
                // Check if edge exists
                graph.edges.iter().any(|e| e.id == edge.id)
            },
            GraphEditOperation::LayerCreate { layer, .. } => {
                // Check if layer doesn't already exist
                !graph.layers.iter().any(|l| l.id == layer.id)
            },
            GraphEditOperation::LayerUpdate { layer_id, .. } => {
                // Check if layer exists
                graph.layers.iter().any(|l| l.id == *layer_id)
            },
            GraphEditOperation::LayerDelete { layer, .. } => {
                // Check if layer exists and no nodes/edges depend on it
                graph.layers.iter().any(|l| l.id == layer.id) &&
                !graph.nodes.iter().any(|n| n.layer == layer.id) &&
                !graph.edges.iter().any(|e| e.layer == layer.id)
            },
        }
    }
}
```

### 3. CRDT Manager

```rust
pub struct CRDTManager {
    graph_id: i32,
    // Using Yrs (Yjs port to Rust) for mature CRDT implementation
    doc: Arc<RwLock<yrs::Doc>>,

    // WebSocket connections for real-time sync
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
}

impl CRDTManager {
    pub fn new(graph_id: i32) -> Self {
        let doc = yrs::Doc::new();

        // Initialize shared maps for graph data
        let nodes_map = doc.get_or_insert_map("nodes");
        let edges_map = doc.get_or_insert_map("edges");
        let layers_map = doc.get_or_insert_map("layers");

        Self {
            graph_id,
            doc: Arc::new(RwLock::new(doc)),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn apply_operation(
        &self,
        operation: &GraphEditOperation,
        context: EditContext,
    ) -> Result<CRDTResult, CRDTError> {
        let mut doc = self.doc.write().await;

        // Apply operation to CRDT document
        let vector = match operation {
            GraphEditOperation::NodeCreate { node, .. } => {
                let nodes_map = doc.get_or_insert_map("nodes");
                let node_map = nodes_map.insert(&mut doc, &node.id, yrs::Map::new());

                // Set node properties
                node_map.insert(&mut doc, "id", &node.id);
                node_map.insert(&mut doc, "label", &node.label);
                node_map.insert(&mut doc, "layer", &node.layer);
                node_map.insert(&mut doc, "is_partition", node.is_partition);
                if let Some(belongs_to) = &node.belongs_to {
                    node_map.insert(&mut doc, "belongs_to", belongs_to);
                }
                node_map.insert(&mut doc, "weight", node.weight as f64);

                doc.state_vector()
            },
            GraphEditOperation::NodeUpdate { node_id, new_values, .. } => {
                let nodes_map = doc.get_or_insert_map("nodes");
                if let Some(node_map) = nodes_map.get(&node_id) {
                    if let Some(node_map) = node_map.cast::<yrs::Map>() {
                        for (key, value) in new_values {
                            match value {
                                serde_json::Value::String(s) => {
                                    node_map.insert(&mut doc, key, s);
                                },
                                serde_json::Value::Number(n) => {
                                    if let Some(f) = n.as_f64() {
                                        node_map.insert(&mut doc, key, f);
                                    }
                                },
                                serde_json::Value::Bool(b) => {
                                    node_map.insert(&mut doc, key, *b);
                                },
                                _ => {
                                    node_map.insert(&mut doc, key, value.to_string());
                                }
                            }
                        }
                    }
                }
                doc.state_vector()
            },
            // ... handle other operation types
            _ => doc.state_vector(),
        };

        Ok(CRDTResult {
            vector: base64::encode(vector),
            conflicts: Vec::new(), // TODO: Implement conflict detection
        })
    }
}
```

### 4. Operation Tracker

```rust
pub struct OperationTracker {
    graph_id: i32,
    db: DatabaseConnection,
}

impl OperationTracker {
    pub fn new(graph_id: i32, db: DatabaseConnection) -> Self {
        Self { graph_id, db }
    }

    pub async fn record_operation(
        &self,
        operation: &GraphEditOperation,
        dag_state_hash: String,
    ) -> Result<(), DatabaseError> {
        let operation_id = operation.get_operation_id();
        let (entity_type, entity_id) = operation.get_affected_entity();

        let new_record = graph_edit_operations::ActiveModel {
            id: Set(operation_id),
            graph_id: Set(self.graph_id),
            operation_type: Set(entity_type),
            entity_id: Set(entity_id),
            operation_data: Set(serde_json::to_string(operation)?),
            dag_state_hash: Set(dag_state_hash),
            applied_at: Set(chrono::Utc::now()),
            created_by_user: Set(operation.get_user_id()),
            is_reproducible: Set(true),
            ..Default::default()
        };

        new_record.insert(&self.db).await?;
        Ok(())
    }

    pub async fn get_applicable_operations(
        &self,
        dag_state_hash: &str,
    ) -> Result<Vec<GraphEditOperation>, DatabaseError> {
        let records = graph_edit_operations::Entity::find()
            .filter(graph_edit_operations::Column::GraphId.eq(self.graph_id))
            .filter(graph_edit_operations::Column::DagStateHash.eq(dag_state_hash))
            .filter(graph_edit_operations::Column::IsReproducible.eq(true))
            .order_by_asc(graph_edit_operations::Column::AppliedAt)
            .all(&self.db)
            .await?;

        let mut operations = Vec::new();
        for record in records {
            if let Ok(operation) = serde_json::from_str::<GraphEditOperation>(&record.operation_data) {
                operations.push(operation);
            }
        }

        Ok(operations)
    }

    pub async fn mark_operation_inapplicable(
        &self,
        operation_id: &str,
    ) -> Result<(), DatabaseError> {
        graph_edit_operations::Entity::update_many()
            .filter(graph_edit_operations::Column::Id.eq(operation_id))
            .set(graph_edit_operations::ActiveModel {
                is_reproducible: Set(false),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
```

## Database Schema Addition

```sql
-- Add to existing schema
CREATE TABLE graph_edit_operations (
    id TEXT PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    operation_type TEXT NOT NULL,    -- 'node', 'edge', 'layer'
    entity_id TEXT NOT NULL,         -- node_id, edge_id, layer_id
    operation_data TEXT NOT NULL,    -- JSON serialized GraphEditOperation
    dag_state_hash TEXT NOT NULL,    -- DAG state when operation was applied
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by_user INTEGER,
    is_reproducible BOOLEAN DEFAULT TRUE,

    FOREIGN KEY (graph_id) REFERENCES graphs(id),
    FOREIGN KEY (created_by_user) REFERENCES users(id),

    INDEX idx_graph_operations_dag (graph_id, dag_state_hash),
    INDEX idx_graph_operations_time (graph_id, applied_at),
    INDEX idx_operations_reproducible (graph_id, is_reproducible)
);
```

## Integration Points

### 1. With Existing Graph Structure
- Extends current `Graph`, `Node`, `Edge`, `Layer` structs
- Maintains compatibility with existing transformation system
- Uses existing integrity verification

### 2. With Current Database
- Builds on existing SeaORM entities
- Adds operation tracking table
- Maintains referential integrity

### 3. With Plan DAG System
- DAG state hash tracks when operations were applied
- Operations can be replayed when DAG re-executes
- Non-applicable operations are automatically pruned

## Next Steps

1. **Prototype CRDT Integration**: Test Yrs library with graph data structures
2. **Operation Serialization**: Validate JSON serialization of all operation types
3. **Conflict Resolution**: Design UI patterns for operation conflicts
4. **Performance Testing**: Benchmark operation replay with 1000+ operations

This design provides a foundation for both real-time collaboration and reproducible DAG execution while building on the existing robust graph processing system.