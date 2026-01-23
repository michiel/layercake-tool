# Graph Edit Tracking System

## Overview

The Graph Edit Tracking System preserves user modifications to graphs when upstream data sources are refreshed or regenerated. It provides a complete audit trail of changes, automatic replay of edits, and a user interface for managing edit history.

**Status**: ✅ Implemented (v2.1)

**Key Features**:
- Automatic tracking of node, edge, and layer modifications
- Sequence-based edit ordering for deterministic replay
- Optimistic UI updates with server synchronisation
- Edit history timeline with visual diff display
- Selective replay of unapplied edits
- Integration with graph regeneration pipeline

## Problem Statement

When graphs are built from external data sources (CSV files, databases, APIs), any manual changes made by users would be lost when the graph is regenerated from upstream data. This creates a poor user experience where:

1. Users customise graph node labels, layers, or properties
2. Upstream data is refreshed (e.g., new nodes added)
3. Graph is rebuilt, discarding all user customisations
4. Users must manually reapply their changes

The edit tracking system solves this by:
- Recording all user modifications as structured edit records
- Automatically replaying edits when graphs are rebuilt
- Handling conflicts gracefully (skip edits that can't be applied)
- Providing visibility into what changes were made and when

## Architecture

### High-Level Flow

```
┌─────────────────┐
│  User Action    │
│  (Edit Graph)   │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────┐
│  GraphQL Mutation               │
│  - updateGraphNode              │
│  - updateLayerProperties        │
└────────┬────────────────────────┘
         │
         ├─────────────────────┬──────────────────────┐
         ▼                     ▼                      ▼
┌─────────────────┐   ┌─────────────────┐   ┌──────────────────┐
│  Apply Change   │   │  Create Edit    │   │  Update Graph    │
│  to Graph       │   │  Record         │   │  Metadata        │
└─────────────────┘   └─────────────────┘   └──────────────────┘
                               │
                               ▼
                      ┌─────────────────┐
                      │  graph_edits    │
                      │  table          │
                      └─────────────────┘

┌─────────────────┐
│  Graph Rebuild  │
│  from Upstream  │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────┐
│  Check has_pending_edits flag   │
└────────┬────────────────────────┘
         │
         ▼ (if true)
┌─────────────────────────────────┐
│  Replay Edits in Sequence Order │
│  - Apply each edit              │
│  - Mark as applied              │
│  - Skip if can't apply          │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│  Update Graph                   │
│  - Clear has_pending_edits      │
│  - Update last_replay_at        │
└─────────────────────────────────┘
```

### Component Layers

```
┌──────────────────────────────────────────────────────┐
│                    Frontend (React)                   │
│  - EditHistoryModal                                   │
│  - GraphEditorPage                                    │
│  - Optimistic UI updates                              │
└───────────────────────┬──────────────────────────────┘
                        │ GraphQL
┌───────────────────────┴──────────────────────────────┐
│              GraphQL API (async-graphql)              │
│  Queries:                                             │
│    - graphEdits(graphId, unappliedOnly)              │
│    - graphEditCount(graphId, unappliedOnly)          │
│  Mutations:                                           │
│    - updateGraphNode(...) → auto-tracks edit         │
│    - updateLayerProperties(...) → auto-tracks edit   │
│    - replayGraphEdits(graphId)                       │
│    - clearGraphEdits(graphId)                        │
└───────────────────────┬──────────────────────────────┘
                        │
┌───────────────────────┴──────────────────────────────┐
│                  Business Logic Layer                 │
│  - GraphEditService: CRUD operations for edits       │
│  - GraphEditApplicator: Apply edits to graph         │
│  - GraphService: Graph data operations               │
└───────────────────────┬──────────────────────────────┘
                        │
┌───────────────────────┴──────────────────────────────┐
│               Database (SQLite via SeaORM)            │
│  Tables:                                              │
│    - graph_edits: Edit records                       │
│    - graphs: Graph metadata (has_pending_edits, etc) │
│    - graph_nodes: Node data                          │
│    - layers: Layer data                              │
└───────────────────────────────────────────────────────┘
```

## Data Model

### Database Schema

#### `graph_edits` Table

```sql
CREATE TABLE graph_edits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    graph_id INTEGER NOT NULL,           -- FK to graphs table
    target_type TEXT NOT NULL,           -- 'node', 'edge', 'layer'
    target_id TEXT NOT NULL,             -- ID of the target entity
    operation TEXT NOT NULL,             -- 'create', 'update', 'delete'
    field_name TEXT,                     -- Field being updated (for updates)
    old_value TEXT,                      -- JSON: previous value
    new_value TEXT,                      -- JSON: new value
    sequence_number INTEGER NOT NULL,    -- Auto-incrementing per graph
    applied BOOLEAN NOT NULL DEFAULT 0,  -- Whether edit has been applied
    created_at TIMESTAMP NOT NULL,
    created_by INTEGER,                  -- FK to users table (optional)
    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE
);

CREATE INDEX idx_graph_edits_graph_id ON graph_edits(graph_id);
CREATE INDEX idx_graph_edits_sequence ON graph_edits(graph_id, sequence_number);
CREATE INDEX idx_graph_edits_applied ON graph_edits(graph_id, applied);
```

**Key Fields**:
- `sequence_number`: Auto-incrementing per graph, ensures deterministic replay order
- `applied`: `false` when created, `true` after replay applies the edit
- `target_type`: Discriminator for which entity type is being edited
- `old_value`/`new_value`: JSON values allowing any data type to be tracked

#### `graphs` Table Extensions

```sql
-- Added columns to existing graphs table:
ALTER TABLE graphs ADD COLUMN has_pending_edits BOOLEAN DEFAULT 0;
ALTER TABLE graphs ADD COLUMN last_edit_sequence INTEGER DEFAULT 0;
ALTER TABLE graphs ADD COLUMN last_replay_at TIMESTAMP;
```

**Metadata Fields**:
- `has_pending_edits`: Quick flag to check if replay is needed
- `last_edit_sequence`: Latest sequence number (for auto-increment)
- `last_replay_at`: Timestamp of last replay operation

### TypeScript/Rust Types

#### Rust (Backend)

```rust
// layercake-core/src/database/entities/graph_edits.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_edits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub graph_id: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub field_name: Option<String>,
    pub old_value: Option<String>,  // JSON serialized
    pub new_value: Option<String>,  // JSON serialized
    pub sequence_number: i32,
    pub applied: bool,
    pub created_at: ChronoDateTime,
    pub created_by: Option<i32>,
}

// layercake-core/src/graphql/types/graph_edit.rs
pub struct GraphEdit {
    pub id: i32,
    pub graph_id: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub field_name: Option<String>,
    pub old_value: Option<JSON>,
    pub new_value: Option<JSON>,
    pub sequence_number: i32,
    pub applied: bool,
    pub created_at: String,
    pub created_by: Option<i32>,
}

pub struct ReplaySummary {
    pub total: i32,
    pub applied: i32,
    pub skipped: i32,
    pub failed: i32,
    pub details: Vec<EditResult>,
}
```

#### TypeScript (Frontend)

```typescript
// frontend/src/graphql/graphs.ts
export interface GraphEdit {
  id: number
  graphId: number
  targetType: string
  targetId: string
  operation: string
  fieldName?: string
  oldValue?: any
  newValue?: any
  sequenceNumber: number
  applied: boolean
  createdAt: string
  createdBy?: number
}

export interface ReplaySummary {
  total: number
  applied: number
  skipped: number
  failed: number
  details: EditResult[]
}
```

## Core Components

### 1. GraphEditService

**Location**: `layercake-core/src/services/graph_edit_service.rs`

**Responsibilities**:
- Create edit records with auto-incrementing sequence numbers
- Retrieve edits with filtering (unapplied only, by graph, etc.)
- Orchestrate replay operations
- Update graph metadata (`has_pending_edits`, `last_edit_sequence`)

**Key Methods**:

```rust
impl GraphEditService {
    // Create a new edit record
    pub async fn create_edit(
        &self,
        graph_id: i32,
        target_type: String,
        target_id: String,
        operation: String,
        field_name: Option<String>,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        created_by: Option<i32>,
    ) -> Result<graph_edits::Model>

    // Get all edits for a graph (optionally filter to unapplied only)
    pub async fn get_edits(
        &self,
        graph_id: i32,
        unapplied_only: bool,
    ) -> Result<Vec<graph_edits::Model>>

    // Count edits for a graph
    pub async fn get_edit_count(
        &self,
        graph_id: i32,
        unapplied_only: bool,
    ) -> Result<u64>

    // Replay all unapplied edits in sequence order
    pub async fn replay_edits(
        &self,
        graph_id: i32,
    ) -> Result<ReplaySummary>

    // Delete all edits for a graph
    pub async fn clear_edits(&self, graph_id: i32) -> Result<u64>
}
```

**Sequence Number Generation**:
```rust
async fn get_next_sequence_number(&self, graph_id: i32) -> Result<i32> {
    let last_edit = GraphEdits::find()
        .filter(graph_edits::Column::GraphId.eq(graph_id))
        .order_by_desc(graph_edits::Column::SequenceNumber)
        .one(&self.db)
        .await?;

    Ok(last_edit.map(|e| e.sequence_number + 1).unwrap_or(1))
}
```

### 2. GraphEditApplicator

**Location**: `layercake-core/src/services/graph_edit_applicator.rs`

**Responsibilities**:
- Apply individual edit records to graph entities
- Handle different target types (node, edge, layer)
- Handle different operations (create, update, delete)
- Return success/skip/fail status for each edit

**Key Methods**:

```rust
impl GraphEditApplicator {
    // Apply a single edit to the graph
    pub async fn apply_edit(
        &self,
        edit: &graph_edits::Model,
    ) -> Result<ApplyResult>
}

pub enum ApplyResult {
    Applied,   // Successfully applied
    Skipped,   // Target not found or edit not applicable
    Failed(String),  // Error occurred
}
```

**Application Logic**:

```rust
match (edit.target_type.as_str(), edit.operation.as_str()) {
    ("node", "update") => {
        // Find the node
        let node = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(edit.graph_id))
            .filter(graph_nodes::Column::Id.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        // If node doesn't exist, skip (was deleted upstream)
        let Some(node) = node else {
            return Ok(ApplyResult::Skipped);
        };

        // Apply the field change
        let mut active_node: graph_nodes::ActiveModel = node.into();
        match edit.field_name.as_deref() {
            Some("label") => {
                if let Some(new_val) = &edit.new_value {
                    let label: String = serde_json::from_value(new_val.clone())?;
                    active_node.label = Set(Some(label));
                }
            }
            Some("layer") => { /* ... */ }
            // ... handle other fields
            _ => return Ok(ApplyResult::Skipped),
        }

        active_node.update(&self.db).await?;
        Ok(ApplyResult::Applied)
    }

    ("layer", "update") => { /* ... */ }
    // ... handle other target_type/operation combinations

    _ => Ok(ApplyResult::Skipped),
}
```

### 3. GraphQL API

**Location**: `layercake-core/src/graphql/mutations/mod.rs`, `layercake-core/src/graphql/queries/mod.rs`

**Queries**:

```graphql
# Get edit history for a graph
query GetGraphEdits($graphId: Int!, $unappliedOnly: Boolean) {
  graphEdits(graphId: $graphId, unappliedOnly: $unappliedOnly) {
    id
    graphId
    targetType
    targetId
    operation
    fieldName
    oldValue
    newValue
    sequenceNumber
    applied
    createdAt
    createdBy
  }
}

# Get count of edits
query GetGraphEditCount($graphId: Int!, $unappliedOnly: Boolean) {
  graphEditCount(graphId: $graphId, unappliedOnly: $unappliedOnly)
}
```

**Mutations**:

```graphql
# Update a graph node (auto-tracks edit)
mutation UpdateGraphNode(
  $graphId: Int!
  $nodeId: String!
  $label: String
  $layer: String
  $attrs: JSON
) {
  updateGraphNode(
    graphId: $graphId
    nodeId: $nodeId
    label: $label
    layer: $layer
    attrs: $attrs
  ) {
    id
    label
    layer
    attrs
  }
}

# Update layer properties (auto-tracks edit)
mutation UpdateLayerProperties(
  $id: Int!
  $name: String
  $properties: JSON
) {
  updateLayerProperties(id: $id, name: $name, properties: $properties) {
    id
    layerId
    name
    properties
  }
}

# Replay all unapplied edits
mutation ReplayGraphEdits($graphId: Int!) {
  replayGraphEdits(graphId: $graphId) {
    total
    applied
    skipped
    failed
    details {
      sequenceNumber
      targetType
      targetId
      operation
      result
      message
    }
  }
}

# Clear all edit history
mutation ClearGraphEdits($graphId: Int!) {
  clearGraphEdits(graphId: $graphId)
}
```

**Auto-Tracking Implementation**:

```rust
// layercake-core/src/graphql/mutations/mod.rs:1440
async fn update_graph_node(
    &self,
    ctx: &Context<'_>,
    graph_id: i32,
    node_id: String,
    label: Option<String>,
    layer: Option<String>,
    attrs: Option<JSON>,
) -> Result<crate::graphql::types::graph_node::GraphNode> {
    let context = ctx.data::<GraphQLContext>()?;
    let graph_service = GraphService::new(context.db.clone());
    let edit_service = GraphEditService::new(context.db.clone());

    // Fetch old node to compare values
    let old_node = graph_service.get_graph_node(graph_id, node_id.clone()).await?;

    // Apply the update
    let updated_node = graph_service
        .update_graph_node(graph_id, node_id.clone(), label.clone(), layer.clone(), attrs.clone())
        .await?;

    // Create edit records for changed fields
    if let Some(old_node) = old_node {
        // Track label change
        if let Some(new_label) = &label {
            let old_label_value = old_node.label.clone().unwrap_or_default();
            if &old_label_value != new_label {
                edit_service.create_edit(
                    graph_id,
                    "node".to_string(),
                    node_id.clone(),
                    "update".to_string(),
                    Some("label".to_string()),
                    Some(serde_json::json!(old_label_value)),
                    Some(serde_json::json!(new_label)),
                    None,
                ).await?;
            }
        }

        // Track layer change
        if let Some(new_layer) = &layer {
            let old_layer_value = old_node.layer.clone().unwrap_or_default();
            if &old_layer_value != new_layer {
                edit_service.create_edit(
                    graph_id,
                    "node".to_string(),
                    node_id.clone(),
                    "update".to_string(),
                    Some("layer".to_string()),
                    if old_layer_value.is_empty() { None } else { Some(serde_json::json!(old_layer_value)) },
                    Some(serde_json::json!(new_layer)),
                    None,
                ).await?;
            }
        }
    }

    Ok(updated_node)
}
```

### 4. Graph Regeneration Integration

**Location**: `layercake-core/src/pipeline/graph_builder.rs`

**Automatic Replay on Rebuild**:

When a graph is rebuilt from upstream data sources (via `executeNode` mutation), the system automatically replays pending edits:

```rust
// After successful graph build
if graph.has_pending_edits {
    tracing::info!("Graph has pending edits, triggering automatic replay");

    let edit_service = GraphEditService::new(db.clone());
    let summary = edit_service.replay_edits(graph.id).await?;

    tracing::info!(
        "Edit replay complete: {} applied, {} skipped, {} failed",
        summary.applied,
        summary.skipped,
        summary.failed
    );

    if summary.failed > 0 {
        tracing::warn!("Some edits failed to apply during replay");
    }

    // Refresh graph to get updated metadata
    graph = graphs::Entity::find_by_id(graph.id)
        .one(&db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Graph not found after replay"))?;
}
```

### 5. Frontend Components

#### EditHistoryModal

**Location**: `frontend/src/components/graphs/EditHistoryModal.tsx`

**Features**:
- Timeline visualization of edit history
- Color-coded operation badges (create=green, update=blue, delete=red)
- Old/new value diff display with JSON formatting
- Toggle between "all edits" and "unapplied only"
- Replay button with confirmation dialog
- Clear history button with confirmation dialog
- Real-time loading and error states

**Key UI Elements**:

```tsx
<Timeline>
  {edits.map((edit: GraphEdit) => (
    <Timeline.Item
      bullet={getTargetTypeIcon(edit.targetType)}  // ⬢=node, →=edge, ▦=layer
      title={
        <Group>
          <Badge color={getOperationColor(edit.operation)}>
            {edit.operation}
          </Badge>
          <Badge variant="light">{edit.targetType}</Badge>
          <Text>{edit.targetId}</Text>
          {edit.applied && <Badge color="green">Applied</Badge>}
        </Group>
      }
    >
      {/* Display old/new values, timestamps, sequence number */}
    </Timeline.Item>
  ))}
</Timeline>
```

#### GraphEditorPage Integration

**Location**: `frontend/src/pages/GraphEditorPage.tsx:314-332`

**Features**:
- Pending edits badge showing count
- Live polling (10-second interval) for edit count updates
- History icon button to open EditHistoryModal
- Optimistic UI updates for immediate feedback

```tsx
// Poll for edit count updates
const { data: editCountData, refetch: refetchEditCount } = useQuery(
  GET_GRAPH_EDIT_COUNT,
  {
    variables: { graphId: parseInt(graphId || '0'), unappliedOnly: true },
    skip: !graphId,
    pollInterval: 10000,  // Poll every 10 seconds
  }
);

// Show badge when edits exist
{hasEdits && (
  <Badge color="yellow" variant="light">
    {editCount} pending {editCount === 1 ? 'edit' : 'edits'}
  </Badge>
)}
```

## Workflows

### 1. Creating an Edit (User Modification)

```
User Changes Node Label
         ↓
Frontend: handleNodeUpdate()
         ↓
GraphQL: updateGraphNode mutation
         ↓
Backend: GraphService.update_graph_node()
    ├─→ Update graph_nodes table
    └─→ Compare old vs new values
         ↓
    GraphEditService.create_edit()
    ├─→ Get next sequence number
    ├─→ Insert into graph_edits table (applied=false)
    └─→ Update graphs table (has_pending_edits=true)
         ↓
Frontend: Optimistic UI update + refetch
```

### 2. Viewing Edit History

```
User Clicks History Button
         ↓
Frontend: setEditHistoryOpen(true)
         ↓
EditHistoryModal opens
         ↓
GraphQL: GET_GRAPH_EDITS query
         ↓
Backend: GraphEditService.get_edits()
    ├─→ Query graph_edits table
    ├─→ Filter by graph_id
    ├─→ Optionally filter by applied=false
    └─→ Order by sequence_number ASC
         ↓
Frontend: Render timeline with edits
```

### 3. Replaying Edits (Manual)

```
User Clicks Replay Button
         ↓
Frontend: Confirmation dialog
         ↓
GraphQL: REPLAY_GRAPH_EDITS mutation
         ↓
Backend: GraphEditService.replay_edits()
    ├─→ Get all unapplied edits (sequence order)
    ├─→ For each edit:
    │   ├─→ GraphEditApplicator.apply_edit()
    │   │   ├─→ Find target entity
    │   │   ├─→ Apply change
    │   │   └─→ Return Applied/Skipped/Failed
    │   └─→ Mark edit as applied=true
    ├─→ Count applied/skipped/failed
    └─→ Update graphs table
        ├─→ has_pending_edits = (any unapplied remain)
        └─→ last_replay_at = now()
         ↓
Frontend: Show alert with summary
         ↓
Frontend: Refetch edit count + graph data
```

### 4. Automatic Replay on Graph Rebuild

```
User Executes DAG Node (rebuild graph)
         ↓
GraphQL: executeNode mutation
         ↓
Backend: Pipeline execution
    ├─→ Load upstream data sources
    ├─→ Build graph (GraphBuilder)
    │   ├─→ Clear existing nodes/edges
    │   ├─→ Insert new data
    │   └─→ Create/update layers
    ├─→ Check has_pending_edits flag
    └─→ If true:
        └─→ GraphEditService.replay_edits()
            ├─→ Apply all unapplied edits
            └─→ Update graph metadata
         ↓
User's customisations preserved!
```

### 5. Clearing Edit History

```
User Clicks Clear Button
         ↓
Frontend: Confirmation dialog
         ↓
GraphQL: CLEAR_GRAPH_EDITS mutation
         ↓
Backend: GraphEditService.clear_edits()
    ├─→ DELETE FROM graph_edits WHERE graph_id = ?
    ├─→ Update graphs table:
    │   ├─→ has_pending_edits = false
    │   └─→ last_edit_sequence = 0
    └─→ Return count of deleted edits
         ↓
Frontend: Show success message
         ↓
Frontend: Close modal + refetch data
```

## Usage Examples

### Example 1: Tracking a Label Change

**Scenario**: User changes a node label from "DB Server" to "PostgreSQL Database"

```sql
-- Before edit
SELECT id, label FROM graph_nodes WHERE id = 'node_42';
-- Result: node_42 | DB Server

-- User makes change via UI
-- Backend creates edit record:
INSERT INTO graph_edits (
    graph_id, target_type, target_id, operation,
    field_name, old_value, new_value,
    sequence_number, applied, created_at
) VALUES (
    1, 'node', 'node_42', 'update',
    'label', '"DB Server"', '"PostgreSQL Database"',
    1, false, '2025-10-11T00:00:00Z'
);

-- After edit
SELECT id, label FROM graph_nodes WHERE id = 'node_42';
-- Result: node_42 | PostgreSQL Database

SELECT * FROM graph_edits WHERE graph_id = 1;
-- Shows the edit record with applied=false
```

### Example 2: Graph Rebuild Preserves Changes

```sql
-- Step 1: User has customised a node
SELECT id, label FROM graph_nodes WHERE id = 'node_42';
-- Result: node_42 | PostgreSQL Database

SELECT * FROM graph_edits WHERE graph_id = 1 AND applied = false;
-- Shows: 1 unapplied edit (label change)

-- Step 2: Upstream CSV is updated with new data
-- Step 3: User executes DAG node to rebuild graph

-- During rebuild:
-- 1. Graph is cleared and rebuilt from CSV
SELECT id, label FROM graph_nodes WHERE id = 'node_42';
-- Result: node_42 | DB Server  (back to original!)

-- 2. Automatic replay detects has_pending_edits=true
-- 3. Edits are replayed in sequence order:
--    - Apply edit #1: Change label to "PostgreSQL Database"

-- After replay:
SELECT id, label FROM graph_nodes WHERE id = 'node_42';
-- Result: node_42 | PostgreSQL Database  (customisation preserved!)

SELECT * FROM graph_edits WHERE graph_id = 1;
-- Shows: 1 edit with applied=true
```

### Example 3: Layer Color Customisation

**Scenario**: User changes VM layer background colour from blue to pink

```javascript
// Frontend code
handleLayerColorChange('vm', 'background', 'ff33cf')

// Sends GraphQL mutation:
mutation {
  updateLayerProperties(
    id: 188,
    properties: {
      background_color: "ff33cf",
      border_color: "dddddd",
      text_color: "ffffff"
    }
  ) {
    id
    properties
  }
}

// Backend creates edit:
INSERT INTO graph_edits (
    graph_id, target_type, target_id, operation,
    field_name, old_value, new_value,
    sequence_number, applied
) VALUES (
    22, 'layer', 'vm', 'update',
    'properties',
    '{"background_color":"2c9ee6","border_color":"dddddd","text_color":"ffffff"}',
    '{"background_color":"ff33cf","border_color":"dddddd","text_color":"ffffff"}',
    1, false
);
```

### Example 4: Handling Skipped Edits

**Scenario**: Edit targets a node that was removed from upstream data

```sql
-- Initial state: node exists
SELECT id FROM graph_nodes WHERE id = 'temp_node';
-- Result: temp_node

-- User customises the node
INSERT INTO graph_edits (...) VALUES (...);  -- Edit created

-- Upstream data changes: temp_node removed from CSV
-- Graph is rebuilt

-- During replay:
-- GraphEditApplicator tries to find 'temp_node'
SELECT id FROM graph_nodes WHERE graph_id = 1 AND id = 'temp_node';
-- Result: (empty) - node doesn't exist

-- Applicator returns ApplyResult::Skipped
-- Edit is marked as applied=true (to prevent retry)
-- Summary reports: 0 applied, 1 skipped, 0 failed
```

## Edge Cases and Considerations

### 1. Concurrent Edits

**Scenario**: Multiple users editing the same graph simultaneously

**Current Behavior**: Last write wins
- Each edit gets a unique sequence number
- Both edits are recorded
- Replay applies edits in sequence order
- Later edit overwrites earlier edit

**Future Enhancement**: Could add conflict detection/resolution

### 2. Deleted Entities

**Scenario**: Edit targets an entity that no longer exists

**Behavior**:
- Applicator returns `ApplyResult::Skipped`
- Edit is marked as `applied=true` (to prevent infinite retry)
- User can view in history that edit was skipped
- Summary shows skip count

### 3. Schema Changes

**Scenario**: Field type or structure changes in database schema

**Behavior**:
- Edits store values as JSON (flexible)
- If field no longer exists, applicator skips
- If field type changed, may fail with error
- Failed edits are logged but don't block replay

**Mitigation**:
- Use semantic versioning for schema changes
- Provide migration scripts for edit records
- Consider edit "expiration" after schema changes

### 4. Large Edit Histories

**Scenario**: Graph accumulates thousands of edits over time

**Current Behavior**:
- All edits are stored indefinitely
- Replay processes all unapplied edits (could be slow)
- Frontend limits display (pagination not implemented)

**Recommendations**:
- Periodically clear old applied edits (user-initiated or automated)
- Consider "compaction": merge sequential edits to same field
- Add pagination to EditHistoryModal for large histories
- Index optimization on `(graph_id, applied, sequence_number)`

### 5. Null vs Empty Values

**Scenario**: Distinguishing between null, empty string, and undefined

**Behavior**:
- `old_value` and `new_value` are nullable JSON columns
- `null` in database = field was null
- Empty string = `""` in JSON
- Missing field = not tracked

**Frontend Handling**:
```typescript
// When properties is undefined, use empty object
const updatedProperties = {
  ...(layer.properties || {}),
  [`${colorType}_color`]: color,
};

// Skip mutation if value hasn't changed
const oldColor = layer.properties?.[`${colorType}_color`];
if (oldColor === color) {
  return;  // No change
}
```

### 6. Edit Granularity

**Current**: One edit per field change
- `label` change = 1 edit
- `layer` change = 1 edit
- `label` + `layer` change in same mutation = 2 edits

**Alternative**: One edit per mutation
- Would require storing multiple field changes in single edit
- Would need more complex applicator logic

**Decision**: Current approach provides finer-grained history and simpler replay logic

### 7. Performance Considerations

**Database**:
- Indexed on `(graph_id, sequence_number)` for fast ordered retrieval
- Indexed on `(graph_id, applied)` for counting unapplied edits
- Cascade delete when graph is deleted (prevents orphaned edits)

**Frontend**:
- 10-second polling for edit count (not real-time)
- Optimistic updates for immediate UI feedback
- Edit history modal uses ScrollArea (virtual scrolling would be better for 1000+ edits)

**Backend**:
- Replay is synchronous (blocks until complete)
- For large edit counts, consider async job queue
- No transaction wrapping (edits applied individually)

## Testing

### Unit Tests

```rust
// layercake-core/tests/graph_edit_service_test.rs
#[tokio::test]
async fn test_create_edit_with_sequence_number() {
    // Verifies sequence number auto-increments
}

#[tokio::test]
async fn test_replay_edits_in_sequence_order() {
    // Verifies edits applied in correct order
}

#[tokio::test]
async fn test_skip_missing_target() {
    // Verifies skipped when target doesn't exist
}

#[tokio::test]
async fn test_update_graph_metadata() {
    // Verifies has_pending_edits flag updated
}
```

### Integration Tests

Test complete workflows:
1. Create node → edit label → rebuild graph → verify label preserved
2. Edit layer → clear edits → verify history cleared
3. Multiple edits → replay → verify correct order

### Manual Testing Checklist

- [ ] Edit node label, verify edit created
- [ ] Edit node layer, verify edit created
- [ ] Edit layer colour, verify edit created
- [ ] View edit history modal, verify timeline displays
- [ ] Toggle "Show All Edits" vs "Show Unapplied Only"
- [ ] Replay edits manually, verify success
- [ ] Clear edits, verify history emptied
- [ ] Rebuild graph, verify edits auto-replayed
- [ ] Edit non-existent node, verify skipped
- [ ] Make same change twice, verify second skipped
- [ ] Check edit count badge updates

## Debugging

### Enable Tracing

```bash
RUST_LOG=debug cargo run
```

Look for these log messages:
```
Layer properties update - old_props: ..., new_properties: ...
Properties are equal: false
Creating edit for layer properties change
Edit replay complete: 3 applied, 1 skipped, 0 failed
```

### Query Edit History

```sql
-- View all edits for a graph
SELECT
    id, target_type, target_id, operation, field_name,
    old_value, new_value, sequence_number, applied
FROM graph_edits
WHERE graph_id = 22
ORDER BY sequence_number;

-- Count pending edits
SELECT COUNT(*) FROM graph_edits
WHERE graph_id = 22 AND applied = false;

-- Check graph metadata
SELECT id, name, has_pending_edits, last_edit_sequence, last_replay_at
FROM graphs
WHERE id = 22;
```

### Frontend Console Logs

```javascript
// Look for these in browser console:
"Layer color change: { layerId: 'vm', oldColor: '000000', newColor: 'ff33cf', ... }"
"Color unchanged, skipping mutation: { layerId: 'vm', ... }"
```

## Future Enhancements

### Planned
- [ ] User attribution (created_by tracking)
- [ ] Edit comments/descriptions
- [ ] Undo/redo functionality
- [ ] Selective replay (choose which edits to apply)
- [ ] Edit diff viewer with visual comparison

### Under Consideration
- [ ] Real-time edit notifications (WebSocket/SSE)
- [ ] Conflict resolution UI for concurrent edits
- [ ] Edit branching/merging (like git)
- [ ] Export/import edit history
- [ ] Edit analytics (most-edited nodes, etc.)
- [ ] Scheduled auto-cleanup of old edits

## References

### Implementation Commits

- Phase 1: Database schema - commits a90f34de-346e78a9
- Phase 2: Replay engine - commits within Phase 1-2
- Phase 3: GraphQL API - commits 88d2d8cc, ea963234
- Phase 4: Auto-replay on regeneration - commit 346e78a9
- Phase 5: Edit history UI - commit 52941c5c

### Related Documentation

- `docs/ARCHITECTURE.md` - Overall system architecture
- `docs/dual-edit-system-prototype.md` - Early design exploration
- `docs/edit-reproducibility-mechanics.md` - Replay mechanism design

### Code Locations

**Backend**:
- Database entities: `layercake-core/src/database/entities/graph_edits.rs`
- Services: `layercake-core/src/services/graph_edit_service.rs`
- Applicator: `layercake-core/src/services/graph_edit_applicator.rs`
- GraphQL types: `layercake-core/src/graphql/types/graph_edit.rs`
- GraphQL mutations: `layercake-core/src/graphql/mutations/mod.rs`
- GraphQL queries: `layercake-core/src/graphql/queries/mod.rs`
- Graph builder: `layercake-core/src/pipeline/graph_builder.rs`

**Frontend**:
- Types/queries: `frontend/src/graphql/graphs.ts`
- Edit history modal: `frontend/src/components/graphs/EditHistoryModal.tsx`
- Graph editor page: `frontend/src/pages/GraphEditorPage.tsx`

## Changelog

### v2.1 (2025-10-11)
- ✅ Initial implementation of complete edit tracking system
- ✅ Database schema with graph_edits table
- ✅ GraphEditService and GraphEditApplicator
- ✅ GraphQL queries and mutations
- ✅ Auto-tracking in updateGraphNode and updateLayerProperties
- ✅ Automatic replay on graph regeneration
- ✅ EditHistoryModal with timeline visualization
- ✅ Pending edits badge and live polling
- ✅ Fix: Skip mutations when values unchanged
