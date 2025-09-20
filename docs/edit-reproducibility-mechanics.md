# Edit Reproducibility Mechanics Design

## Overview

The specification requires that "edits to graphs (via GraphVisualEditor or GraphSpreadsheetEditor) are tracked and reproducible, so re-applying inputs and re-running the DAG, will re-run the edits (if they are still applicable, removing them from tracking otherwise)".

This document designs the mechanics for tracking, validating, and replaying graph edit operations.

## Core Concepts

### 1. Edit Operation Granularity

Manual edits are captured at the atomic operation level:

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManualEdit {
    pub id: String,                    // Unique edit identifier
    pub graph_id: i32,                 // Target graph
    pub operation: GraphEditOperation, // From dual-edit system
    pub context: EditContext,          // When/how edit was applied
    pub dag_state_hash: String,        // DAG state when edit was made
    pub applicability_signature: String, // Context signature for validation
    pub created_at: DateTime<Utc>,
    pub last_applied_at: Option<DateTime<Utc>>,
    pub is_active: bool,               // Whether edit is still tracked
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditContext {
    pub user_id: i32,
    pub session_id: String,
    pub editor_type: EditorType,       // Visual or Spreadsheet
    pub edit_reason: Option<String>,   // User-provided reason
    pub parent_edits: Vec<String>,     // Dependencies on other edits
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EditorType {
    GraphVisualEditor,    // ReactFlow-based visual editor
    GraphSpreadsheetEditor, // Table-based editor
    PlanVisualEditor,     // Plan DAG editor (for plan changes)
    API,                  // Programmatic edits
}
```

### 2. DAG State Hashing

To determine when edits are applicable, we track the DAG state when edits were made:

```rust
pub struct DAGStateHasher;

impl DAGStateHasher {
    /// Create deterministic hash of DAG state relevant to graph generation
    pub fn hash_dag_state(plan_dag: &PlanDAG, target_graph_ref: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();

        // Hash the DAG structure that affects this graph
        let relevant_nodes = Self::get_nodes_affecting_graph(plan_dag, target_graph_ref);

        for node in relevant_nodes {
            // Hash node type and configuration
            hasher.update(node.id.as_bytes());
            hasher.update(node.node_type.to_string().as_bytes());
            hasher.update(serde_json::to_string(&node.config).unwrap().as_bytes());

            // Hash input connections
            let inputs = Self::get_node_inputs(plan_dag, &node.id);
            for input in inputs {
                hasher.update(input.as_bytes());
            }
        }

        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Get all nodes in DAG that can affect the specified graph
    fn get_nodes_affecting_graph(plan_dag: &PlanDAG, target_graph_ref: &str) -> Vec<&PlanDAGNode> {
        let mut affecting_nodes = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = vec![target_graph_ref.to_string()];

        while let Some(current_ref) = to_visit.pop() {
            if visited.contains(&current_ref) {
                continue;
            }
            visited.insert(current_ref.clone());

            // Find nodes that output to this reference
            for node in &plan_dag.nodes {
                if Self::node_outputs_to_ref(node, &current_ref) {
                    affecting_nodes.push(node);

                    // Add this node's inputs to visit list
                    let inputs = Self::get_node_inputs(plan_dag, &node.id);
                    to_visit.extend(inputs);
                }
            }
        }

        affecting_nodes
    }

    fn node_outputs_to_ref(node: &PlanDAGNode, target_ref: &str) -> bool {
        match &node.node_type {
            PlanDAGNodeType::InputNode => {
                if let Ok(config) = serde_json::from_value::<InputNodeConfig>(node.config.clone()) {
                    config.output_graph_ref == target_ref
                } else { false }
            },
            PlanDAGNodeType::TransformNode => {
                if let Ok(config) = serde_json::from_value::<TransformNodeConfig>(node.config.clone()) {
                    config.output_graph_ref == target_ref
                } else { false }
            },
            PlanDAGNodeType::MergeNode => {
                if let Ok(config) = serde_json::from_value::<MergeNodeConfig>(node.config.clone()) {
                    config.output_graph_ref == target_ref
                } else { false }
            },
            PlanDAGNodeType::CopyNode => {
                if let Ok(config) = serde_json::from_value::<CopyNodeConfig>(node.config.clone()) {
                    config.output_graph_ref == target_ref
                } else { false }
            },
            _ => false,
        }
    }

    fn get_node_inputs(plan_dag: &PlanDAG, node_id: &str) -> Vec<String> {
        plan_dag.edges.iter()
            .filter(|e| e.target == node_id)
            .map(|e| e.source.clone())
            .collect()
    }
}
```

### 3. Applicability Validation

Edits are applicable if the graph structure supports them:

```rust
pub struct EditApplicabilityValidator;

impl EditApplicabilityValidator {
    /// Check if an edit can still be applied to a graph
    pub fn is_edit_applicable(
        edit: &ManualEdit,
        current_graph: &Graph,
        current_dag_state: &str,
    ) -> ApplicabilityResult {
        // 1. Check DAG state compatibility
        if edit.dag_state_hash != current_dag_state {
            // DAG changed - need deeper validation
            if !Self::validate_dag_compatibility(&edit, current_dag_state) {
                return ApplicabilityResult::Inapplicable {
                    reason: "DAG structure changed incompatibly".to_string(),
                    suggestion: Some("Re-apply edit manually".to_string()),
                };
            }
        }

        // 2. Check graph structure compatibility
        match &edit.operation {
            GraphEditOperation::NodeCreate { node, .. } => {
                if current_graph.get_node_by_id(&node.id).is_some() {
                    ApplicabilityResult::Inapplicable {
                        reason: format!("Node {} already exists", node.id),
                        suggestion: Some("Update existing node instead".to_string()),
                    }
                } else {
                    ApplicabilityResult::Applicable
                }
            },
            GraphEditOperation::NodeUpdate { node_id, .. } => {
                if current_graph.get_node_by_id(node_id).is_some() {
                    ApplicabilityResult::Applicable
                } else {
                    ApplicabilityResult::Inapplicable {
                        reason: format!("Node {} no longer exists", node_id),
                        suggestion: Some("Create node first".to_string()),
                    }
                }
            },
            GraphEditOperation::NodeDelete { node, .. } => {
                if current_graph.get_node_by_id(&node.id).is_some() {
                    // Check if node has dependencies
                    let dependent_edges: Vec<&Edge> = current_graph.edges.iter()
                        .filter(|e| e.source == node.id || e.target == node.id)
                        .collect();

                    if dependent_edges.is_empty() {
                        ApplicabilityResult::Applicable
                    } else {
                        ApplicabilityResult::Conditional {
                            reason: "Node has dependent edges".to_string(),
                            required_actions: vec![
                                "Delete dependent edges first".to_string()
                            ],
                        }
                    }
                } else {
                    ApplicabilityResult::AlreadyApplied {
                        reason: "Node already deleted".to_string(),
                    }
                }
            },
            GraphEditOperation::EdgeCreate { edge, .. } => {
                // Check if source and target nodes exist
                let source_exists = current_graph.get_node_by_id(&edge.source).is_some();
                let target_exists = current_graph.get_node_by_id(&edge.target).is_some();
                let edge_exists = current_graph.edges.iter().any(|e| e.id == edge.id);

                match (source_exists, target_exists, edge_exists) {
                    (true, true, false) => ApplicabilityResult::Applicable,
                    (true, true, true) => ApplicabilityResult::AlreadyApplied {
                        reason: "Edge already exists".to_string(),
                    },
                    (false, _, _) => ApplicabilityResult::Inapplicable {
                        reason: format!("Source node {} not found", edge.source),
                        suggestion: Some("Create source node first".to_string()),
                    },
                    (_, false, _) => ApplicabilityResult::Inapplicable {
                        reason: format!("Target node {} not found", edge.target),
                        suggestion: Some("Create target node first".to_string()),
                    },
                }
            },
            // ... handle other edit types
            _ => ApplicabilityResult::Applicable, // Default for other types
        }
    }

    fn validate_dag_compatibility(edit: &ManualEdit, current_dag_state: &str) -> bool {
        // Compare DAG states to see if changes are compatible
        // For now, simple string comparison - could be enhanced with semantic comparison

        // If DAG states differ, we need to check if the differences affect this edit
        // For prototype: assume DAG changes make edits inapplicable
        // TODO: Implement semantic DAG comparison
        false
    }
}

#[derive(Debug, Clone)]
pub enum ApplicabilityResult {
    Applicable,                           // Edit can be applied as-is
    AlreadyApplied { reason: String },    // Edit already applied (skip)
    Conditional {
        reason: String,
        required_actions: Vec<String>
    },                                    // Can be applied with prerequisites
    Inapplicable {
        reason: String,
        suggestion: Option<String>
    },                                    // Cannot be applied
}
```

### 4. Edit Replay System

When DAG re-executes, edits are replayed:

```rust
pub struct EditReplayManager {
    db: DatabaseConnection,
    dual_edit_manager: Arc<DualEditManager>,
}

impl EditReplayManager {
    /// Replay all applicable edits for a graph after DAG execution
    pub async fn replay_edits_for_graph(
        &self,
        graph_id: i32,
        base_graph: Graph,
        current_dag_state: &str,
    ) -> Result<Graph, ReplayError> {
        // Get all tracked edits for this graph
        let tracked_edits = self.get_tracked_edits(graph_id).await?;

        let mut result_graph = base_graph;
        let mut applied_edits = Vec::new();
        let mut failed_edits = Vec::new();
        let mut inapplicable_edits = Vec::new();

        tracing::info!("Replaying {} tracked edits for graph {}", tracked_edits.len(), graph_id);

        for edit in tracked_edits {
            tracing::debug!("Checking applicability of edit: {}", edit.id);

            match EditApplicabilityValidator::is_edit_applicable(&edit, &result_graph, current_dag_state) {
                ApplicabilityResult::Applicable => {
                    tracing::info!("Applying edit: {}", edit.id);

                    match self.apply_edit_to_graph(&mut result_graph, &edit).await {
                        Ok(_) => {
                            applied_edits.push(edit.id.clone());
                            self.update_edit_last_applied(&edit.id).await?;
                        },
                        Err(e) => {
                            tracing::warn!("Failed to apply edit {}: {}", edit.id, e);
                            failed_edits.push((edit.id.clone(), e.to_string()));
                        }
                    }
                },
                ApplicabilityResult::AlreadyApplied { reason } => {
                    tracing::info!("Edit {} already applied: {}", edit.id, reason);
                    applied_edits.push(edit.id.clone());
                },
                ApplicabilityResult::Conditional { reason, required_actions } => {
                    tracing::warn!("Edit {} requires actions: {} - {}", edit.id, reason, required_actions.join(", "));

                    // Try to apply required actions automatically
                    if self.can_auto_resolve_conditions(&required_actions) {
                        if let Ok(_) = self.apply_conditional_edit(&mut result_graph, &edit, required_actions).await {
                            applied_edits.push(edit.id.clone());
                            self.update_edit_last_applied(&edit.id).await?;
                        } else {
                            failed_edits.push((edit.id.clone(), "Auto-resolution failed".to_string()));
                        }
                    } else {
                        failed_edits.push((edit.id.clone(), format!("Requires manual intervention: {}", reason)));
                    }
                },
                ApplicabilityResult::Inapplicable { reason, suggestion } => {
                    tracing::info!("Edit {} no longer applicable: {}", edit.id, reason);
                    inapplicable_edits.push(edit.id.clone());

                    // Mark edit as inactive
                    self.deactivate_edit(&edit.id, &reason).await?;

                    if let Some(suggestion) = suggestion {
                        tracing::info!("Suggestion for edit {}: {}", edit.id, suggestion);
                    }
                }
            }
        }

        tracing::info!(
            "Edit replay complete for graph {}: {} applied, {} failed, {} deactivated",
            graph_id, applied_edits.len(), failed_edits.len(), inapplicable_edits.len()
        );

        // Store replay summary for user review
        self.store_replay_summary(graph_id, ReplaySummary {
            applied_edits,
            failed_edits,
            inapplicable_edits,
            replay_timestamp: chrono::Utc::now(),
        }).await?;

        Ok(result_graph)
    }

    async fn apply_edit_to_graph(&self, graph: &mut Graph, edit: &ManualEdit) -> Result<(), ReplayError> {
        // Apply edit operation to graph
        match &edit.operation {
            GraphEditOperation::NodeCreate { node, .. } => {
                graph.set_node(node.clone());
            },
            GraphEditOperation::NodeUpdate { node_id, new_values, .. } => {
                if let Some(existing_node) = graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                    Self::apply_node_updates(existing_node, new_values)?;
                } else {
                    return Err(ReplayError::NodeNotFound(node_id.clone()));
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
                    Self::apply_edge_updates(existing_edge, new_values)?;
                } else {
                    return Err(ReplayError::EdgeNotFound(edge_id.clone()));
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
                    Self::apply_layer_updates(existing_layer, new_values)?;
                } else {
                    return Err(ReplayError::LayerNotFound(layer_id.clone()));
                }
            },
            GraphEditOperation::LayerDelete { layer, .. } => {
                graph.layers.retain(|l| l.id != layer.id);
            },
        }

        // Verify graph integrity after edit
        graph.verify_graph_integrity()
            .map_err(|errors| ReplayError::GraphIntegrityViolation(errors))?;

        Ok(())
    }

    async fn get_tracked_edits(&self, graph_id: i32) -> Result<Vec<ManualEdit>, DatabaseError> {
        // Query database for active manual edits
        let records = manual_edits::Entity::find()
            .filter(manual_edits::Column::GraphId.eq(graph_id))
            .filter(manual_edits::Column::IsActive.eq(true))
            .order_by_asc(manual_edits::Column::CreatedAt)
            .all(&self.db)
            .await?;

        let mut edits = Vec::new();
        for record in records {
            if let Ok(operation) = serde_json::from_str::<GraphEditOperation>(&record.operation_data) {
                if let Ok(context) = serde_json::from_str::<EditContext>(&record.context_data) {
                    edits.push(ManualEdit {
                        id: record.id,
                        graph_id: record.graph_id,
                        operation,
                        context,
                        dag_state_hash: record.dag_state_hash,
                        applicability_signature: record.applicability_signature,
                        created_at: record.created_at,
                        last_applied_at: record.last_applied_at,
                        is_active: record.is_active,
                    });
                }
            }
        }

        Ok(edits)
    }

    async fn deactivate_edit(&self, edit_id: &str, reason: &str) -> Result<(), DatabaseError> {
        manual_edits::Entity::update_many()
            .filter(manual_edits::Column::Id.eq(edit_id))
            .set(manual_edits::ActiveModel {
                is_active: Set(false),
                deactivation_reason: Set(Some(reason.to_string())),
                deactivated_at: Set(Some(chrono::Utc::now())),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn update_edit_last_applied(&self, edit_id: &str) -> Result<(), DatabaseError> {
        manual_edits::Entity::update_many()
            .filter(manual_edits::Column::Id.eq(edit_id))
            .set(manual_edits::ActiveModel {
                last_applied_at: Set(Some(chrono::Utc::now())),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ReplaySummary {
    pub applied_edits: Vec<String>,
    pub failed_edits: Vec<(String, String)>,
    pub inapplicable_edits: Vec<String>,
    pub replay_timestamp: DateTime<Utc>,
}
```

### 5. User Interface Integration

The edit reproducibility system integrates with the frontend:

```typescript
// Frontend integration for edit tracking
export interface EditTrackingManager {
  // Track manual edits from visual/spreadsheet editors
  trackEdit(editOperation: GraphEditOperation, context: EditContext): Promise<string>;

  // Get edit history for a graph
  getEditHistory(graphId: number): Promise<ManualEdit[]>;

  // Preview edit applicability before DAG re-execution
  previewEditApplicability(graphId: number): Promise<EditApplicabilityPreview>;

  // Handle edit replay conflicts
  handleReplayConflicts(conflicts: EditConflict[]): Promise<ConflictResolution[]>;
}

export interface EditApplicabilityPreview {
  totalEdits: number;
  applicableEdits: number;
  inapplicableEdits: Array<{
    editId: string;
    reason: string;
    suggestion?: string;
  }>;
  conditionalEdits: Array<{
    editId: string;
    requiredActions: string[];
  }>;
}

// React component for edit tracking notification
export function EditTrackingIndicator({ graphId }: { graphId: number }) {
  const [editCount, setEditCount] = useState(0);
  const [lastEdit, setLastEdit] = useState<ManualEdit | null>(null);

  // Subscribe to edit tracking updates
  useEffect(() => {
    const subscription = editTrackingManager.subscribeToEdits(graphId, (edit) => {
      setEditCount(prev => prev + 1);
      setLastEdit(edit);

      // Show notification
      showNotification({
        title: "Edit Tracked",
        message: `${edit.operation.type} will be replayed when DAG re-executes`,
        color: "blue",
        icon: <IconHistory size={16} />,
      });
    });

    return () => subscription.unsubscribe();
  }, [graphId]);

  if (editCount === 0) return null;

  return (
    <Badge color="blue" variant="light">
      <IconHistory size={12} />
      {editCount} tracked edits
    </Badge>
  );
}

// React component for replay preview
export function EditReplayPreview({ graphId }: { graphId: number }) {
  const [preview, setPreview] = useState<EditApplicabilityPreview | null>(null);
  const [loading, setLoading] = useState(false);

  const checkApplicability = async () => {
    setLoading(true);
    try {
      const result = await editTrackingManager.previewEditApplicability(graphId);
      setPreview(result);
    } finally {
      setLoading(false);
    }
  };

  if (!preview) {
    return (
      <Button onClick={checkApplicability} loading={loading}>
        Preview Edit Applicability
      </Button>
    );
  }

  return (
    <Stack>
      <Text size="sm" weight={500}>Edit Replay Preview</Text>

      <Group>
        <Badge color="green" variant="light">
          {preview.applicableEdits} applicable
        </Badge>

        {preview.conditionalEdits.length > 0 && (
          <Badge color="yellow" variant="light">
            {preview.conditionalEdits.length} conditional
          </Badge>
        )}

        {preview.inapplicableEdits.length > 0 && (
          <Badge color="red" variant="light">
            {preview.inapplicableEdits.length} inapplicable
          </Badge>
        )}
      </Group>

      {preview.inapplicableEdits.length > 0 && (
        <Alert color="orange" icon={<IconAlertTriangle size={16} />}>
          <Text size="sm" weight={500}>Some edits cannot be replayed:</Text>
          <List size="sm">
            {preview.inapplicableEdits.map(edit => (
              <List.Item key={edit.editId}>
                {edit.reason}
                {edit.suggestion && <Text color="dimmed"> - {edit.suggestion}</Text>}
              </List.Item>
            ))}
          </List>
        </Alert>
      )}
    </Stack>
  );
}
```

## Database Schema

```sql
-- Manual edits tracking table
CREATE TABLE manual_edits (
    id TEXT PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    operation_data TEXT NOT NULL,        -- JSON: GraphEditOperation
    context_data TEXT NOT NULL,          -- JSON: EditContext
    dag_state_hash TEXT NOT NULL,        -- DAG state when edit was made
    applicability_signature TEXT NOT NULL, -- For quick applicability checks
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_applied_at DATETIME,            -- When edit was last successfully applied
    is_active BOOLEAN DEFAULT TRUE,      -- Whether edit is still tracked
    deactivated_at DATETIME,             -- When edit was deactivated
    deactivation_reason TEXT,            -- Why edit was deactivated

    FOREIGN KEY (graph_id) REFERENCES graphs(id),

    INDEX idx_manual_edits_graph_active (graph_id, is_active),
    INDEX idx_manual_edits_dag_state (dag_state_hash),
    INDEX idx_manual_edits_created (graph_id, created_at)
);

-- Edit replay summaries for user review
CREATE TABLE edit_replay_summaries (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    replay_timestamp DATETIME NOT NULL,
    applied_edits TEXT NOT NULL,         -- JSON: Array of edit IDs
    failed_edits TEXT NOT NULL,          -- JSON: Array of {editId, reason}
    inapplicable_edits TEXT NOT NULL,    -- JSON: Array of edit IDs

    FOREIGN KEY (graph_id) REFERENCES graphs(id),

    INDEX idx_replay_summaries_graph (graph_id, replay_timestamp)
);
```

## Benefits of This Design

### 1. **Granular Tracking**
- Every atomic edit operation is tracked individually
- Full context preservation for debugging and audit

### 2. **Intelligent Applicability**
- DAG state hashing detects when edits may be invalid
- Detailed validation prevents errors during replay

### 3. **Graceful Degradation**
- Inapplicable edits are automatically deactivated
- Users get clear feedback on what happened and why

### 4. **Performance Optimization**
- Applicability signatures for quick checks
- Batch processing during replay

### 5. **User Experience**
- Real-time tracking notifications
- Preview functionality before DAG execution
- Clear conflict resolution workflows

This design ensures that manual graph edits are preserved and intelligently replayed when DAG re-execution occurs, providing a seamless experience for iterative graph development workflows.