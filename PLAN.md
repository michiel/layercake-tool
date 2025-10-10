# Implementation Plan: LcGraphEdit - Graph Change Tracking System

## Overview

Implement a change tracking system that records all edits made to graph instances (nodes, edges, layers) and allows these edits to be replayed when upstream data sources are refreshed. This ensures user modifications are preserved even when the underlying graph data is regenerated.

---

## Architecture

### Core Concept

**Problem**: When a graph is regenerated from upstream sources (e.g., refreshed CSV data), all manual edits to nodes, edges, and layers are lost.

**Solution**: Track every edit as a discrete GraphEdit operation that can be replayed on newly generated graph data.

**Key Principles**:
1. **Ordered replay**: Edits must be applied in chronological order
2. **Idempotent operations**: Same edit can be safely applied multiple times
3. **Graceful degradation**: If target entity doesn't exist, skip the edit
4. **Minimal storage**: Store only the delta (what changed), not full state

### Data Flow

```
User Edit → Create GraphEdit → Store in DB → Mark Graph as Modified
                                                       ↓
Upstream Refresh → Regenerate Graph → Replay GraphEdits → Updated Graph with Edits
```

---

## Database Schema

### LcGraphEdit Table

```sql
CREATE TABLE graph_edits (
    id SERIAL PRIMARY KEY,
    graph_id INTEGER NOT NULL REFERENCES graphs(id) ON DELETE CASCADE,

    -- Target identification
    target_type VARCHAR(20) NOT NULL,  -- 'node', 'edge', 'layer'
    target_id VARCHAR(255) NOT NULL,   -- ID of the affected entity

    -- Operation details
    operation VARCHAR(50) NOT NULL,    -- 'create', 'update', 'delete'
    field_name VARCHAR(100),           -- For updates: which field changed
    old_value JSONB,                   -- Previous value (for rollback/undo)
    new_value JSONB,                   -- New value to apply

    -- Metadata
    sequence_number INTEGER NOT NULL,  -- Order of operations (auto-increment per graph)
    applied BOOLEAN DEFAULT FALSE,     -- Has this edit been applied to current graph state?
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),

    -- Indexing
    UNIQUE(graph_id, sequence_number),
    INDEX idx_graph_edits_graph_id (graph_id),
    INDEX idx_graph_edits_target (graph_id, target_type, target_id),
    INDEX idx_graph_edits_sequence (graph_id, sequence_number)
);
```

### Enhanced Graphs Table

```sql
ALTER TABLE graphs
ADD COLUMN last_edit_sequence INTEGER DEFAULT 0,
ADD COLUMN has_pending_edits BOOLEAN DEFAULT FALSE,
ADD COLUMN last_replay_at TIMESTAMP;
```

---

## GraphEdit Operation Types

### Node Operations

```typescript
// Create node
{
  target_type: 'node',
  target_id: 'new-node-123',
  operation: 'create',
  field_name: null,
  old_value: null,
  new_value: {
    label: 'New Node',
    layer: 'default',
    isPartition: false,
    attrs: {}
  }
}

// Update node label
{
  target_type: 'node',
  target_id: 'node-abc',
  operation: 'update',
  field_name: 'label',
  old_value: 'Old Label',
  new_value: 'New Label'
}

// Update node layer
{
  target_type: 'node',
  target_id: 'node-abc',
  operation: 'update',
  field_name: 'layer',
  old_value: 'layer1',
  new_value: 'layer2'
}

// Delete node
{
  target_type: 'node',
  target_id: 'node-abc',
  operation: 'delete',
  field_name: null,
  old_value: { /* full node data */ },
  new_value: null
}
```

### Edge Operations

```typescript
// Create edge
{
  target_type: 'edge',
  target_id: 'edge-123',
  operation: 'create',
  field_name: null,
  old_value: null,
  new_value: {
    source: 'node-a',
    target: 'node-b',
    label: 'connects',
    layer: 'default'
  }
}

// Update edge label
{
  target_type: 'edge',
  target_id: 'edge-123',
  operation: 'update',
  field_name: 'label',
  old_value: 'old label',
  new_value: 'new label'
}

// Delete edge
{
  target_type: 'edge',
  target_id: 'edge-123',
  operation: 'delete',
  field_name: null,
  old_value: { /* full edge data */ },
  new_value: null
}
```

### Layer Operations

```typescript
// Create layer
{
  target_type: 'layer',
  target_id: 'layer-new',
  operation: 'create',
  field_name: null,
  old_value: null,
  new_value: {
    layerId: 'layer-new',
    name: 'New Layer',
    properties: {
      background_color: 'ffffff',
      border_color: '000000',
      text_color: '000000'
    }
  }
}

// Update layer properties
{
  target_type: 'layer',
  target_id: 'layer1',
  operation: 'update',
  field_name: 'properties',
  old_value: { background_color: 'ffffff' },
  new_value: { background_color: 'ff0000' }
}

// Delete layer
{
  target_type: 'layer',
  target_id: 'layer1',
  operation: 'delete',
  field_name: null,
  old_value: { /* full layer data */ },
  new_value: null
}
```

---

## Implementation Phases

### Phase 1: Database & Core Infrastructure (8-10 hours)

**Goal**: Create database schema and base Rust service layer

**Tasks**:

1. **Database Migration** (1 hour)
   - Create `graph_edits` table migration
   - Add new columns to `graphs` table
   - Create indexes
   - Test migration up/down

2. **SeaORM Entity** (1-2 hours)
   - Generate entity for `graph_edits`
   - Define relationships (LcGraph has_many LcGraphEdit)
   - Add JSON serialization for `old_value`/`new_value`
   - Create enum for `target_type` and `operation`

3. **GraphEdit Service** (3-4 hours)
   - Create `GraphEditService` struct
   - Implement `create_edit()` - record new edit with auto-increment sequence
   - Implement `get_edits_for_graph()` - fetch ordered list
   - Implement `mark_edit_applied()` - update applied flag
   - Implement `clear_graph_edits()` - remove all edits for graph
   - Implement sequence number auto-increment logic

4. **Unit Tests** (2-3 hours)
   - Test edit creation with sequence ordering
   - Test fetching edits in order
   - Test cascade delete when graph is deleted
   - Test concurrent edit creation

**Acceptance Criteria**:
- Migration runs successfully
- Can create GraphEdit records via service
- GraphEdits are properly ordered by sequence number
- Relationships work correctly
- All tests pass

---

### Phase 2: Replay Engine (10-12 hours)

**Goal**: Implement logic to replay edits on refreshed graph data

**Tasks**:

1. **GraphEdit Applicator** (4-5 hours)
   - Create `GraphEditApplicator` struct
   - Implement `apply_edit()` - apply single edit to graph data
     * Node create: add node to graph
     * Node update: update field if node exists
     * Node delete: remove node if exists
     * Edge create: add edge if source/target exist
     * Edge update: update field if edge exists
     * Edge delete: remove edge if exists
     * Layer create: add layer
     * Layer update: update properties if layer exists
     * Layer delete: remove layer if exists
   - Handle missing targets gracefully (log and skip)
   - Return ApplyResult (success, skipped, error)

2. **Replay Orchestrator** (3-4 hours)
   - Create `replay_graph_edits()` function
   - Fetch all unapplied edits for graph in sequence order
   - Apply each edit sequentially
   - Track which edits succeeded/failed/skipped
   - Mark successfully applied edits
   - Update `last_replay_at` timestamp
   - Return replay summary (applied: X, skipped: Y, failed: Z)

3. **Conflict Resolution** (2 hours)
   - Handle case where edit targets deleted entity
   - Handle case where field no longer exists
   - Handle case where new data conflicts with edit
   - Log all skipped edits with reason

4. **Integration Tests** (1-2 hours)
   - Test full replay scenario
   - Test partial replay (some edits skip)
   - Test replay idempotence
   - Test edit ordering matters

**Acceptance Criteria**:
- Edits can be applied to graph data structure
- Replay processes all edits in order
- Gracefully handles missing targets
- Returns accurate summary of replay
- Idempotent (can replay multiple times safely)

---

### Phase 3: Frontend Integration - Graph Editor (8-10 hours)

**Goal**: Capture edits from graph editor UI and create GraphEdit records

**Tasks**:

1. **GraphQL Mutations** (2-3 hours)
   - Add `createGraphEdit` mutation
   - Add `getGraphEdits` query
   - Add `replayGraphEdits` mutation
   - Add `clearGraphEdits` mutation
   - Update existing mutations (updateGraphNode, etc.) to create GraphEdit

2. **Update Existing Mutations** (3-4 hours)
   - `updateGraphNode`: Create GraphEdit after successful update
   - `updateLayerProperties`: Create GraphEdit after successful update
   - Create/delete operations: Create corresponding GraphEdit
   - Extract common edit creation logic

3. **GraphEditor Hook** (2 hours)
   - Create `useGraphEdits` hook
   - Provides `createEdit()` function
   - Automatically increments sequence number
   - Handles optimistic updates

4. **UI Indicators** (1-2 hours)
   - Show "Modified" badge on graphs with pending edits
   - Display edit count in graph metadata
   - Add "View Edit History" button
   - Add "Clear Edits" action (with confirmation)

**Acceptance Criteria**:
- All graph edits create GraphEdit records
- Sequence numbers auto-increment correctly
- UI shows when graph has edits
- Can view edit history
- Optimistic updates work correctly

---

### Phase 4: Graph Regeneration Integration (6-8 hours)

**Goal**: Trigger replay when graph is regenerated from upstream sources

**Tasks**:

1. **Detect Graph Regeneration** (2 hours)
   - Modify graph builder to set `has_pending_edits` flag check
   - After building new graph, check for existing edits
   - Trigger replay if edits exist

2. **Replay on Regeneration** (2-3 hours)
   - Call `replay_graph_edits()` after graph build
   - Apply edits to newly built graph
   - Save updated graph with edits applied
   - Log replay summary

3. **User Notification** (1-2 hours)
   - Show notification after replay with summary
   - "Applied 15 edits, skipped 3 (targets not found)"
   - Allow user to review skipped edits
   - Option to clear skipped edits

4. **Testing** (1-2 hours)
   - Test full cycle: edit → regenerate → replay
   - Test with skipped edits
   - Test with no edits
   - Test replay failure recovery

**Acceptance Criteria**:
- Graph regeneration triggers replay automatically
- Edits are correctly applied to new graph data
- User is notified of replay results
- Skipped edits are logged and shown to user
- System is resilient to replay failures

---

### Phase 5: Edit History UI (6-8 hours)

**Goal**: Create UI for viewing, managing, and understanding graph edits

**Tasks**:

1. **Edit History Panel** (3-4 hours)
   - Create GraphEditHistoryPanel component
   - Display list of all edits for graph
   - Show: sequence, timestamp, operation, target, user
   - Group by date/time
   - Show applied vs pending status
   - Color code by type (create=green, update=blue, delete=red)

2. **Edit Details View** (2 hours)
   - Click edit to see full details
   - Show old_value vs new_value diff
   - Show JSON diff for complex values
   - Show target entity current state (if exists)

3. **Edit Management Actions** (1-2 hours)
   - Button: "Replay Edits" (manual trigger)
   - Button: "Clear All Edits" (with confirmation)
   - Button: "Clear Skipped Edits"
   - Checkbox: "Auto-replay on regeneration" (graph setting)

4. **Integration** (1 hour)
   - Add "Edit History" tab to graph editor
   - Add edit count badge to graph card
   - Add quick replay button to graph toolbar

**Acceptance Criteria**:
- Can view full edit history for any graph
- Edit details are clear and informative
- Can manually trigger replay
- Can clear edits (all or skipped only)
- UI is intuitive and helpful

---

### Phase 6: Advanced Features (Optional, 8-10 hours)

**Goal**: Add advanced edit management capabilities

**Tasks**:

1. **Edit Undo/Redo** (3-4 hours)
   - Implement undo: create reverse edit
   - Implement redo: reapply edit
   - Maintain undo/redo stack
   - UI buttons in graph editor

2. **Edit Branches** (3-4 hours)
   - Allow multiple edit branches per graph
   - Switch between branches
   - Merge branches (conflict resolution)
   - Store branch metadata

3. **Edit Export/Import** (2 hours)
   - Export edits as JSON
   - Import edits from JSON
   - Apply imported edits to graph
   - Use case: transfer edits between environments

**Acceptance Criteria**:
- Undo/redo works correctly
- Can create and switch between branches
- Can export/import edit history
- All features well tested

---

## Technical Specifications

### GraphEdit Rust Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdit {
    pub id: i32,
    pub graph_id: i32,
    pub target_type: TargetType,
    pub target_id: String,
    pub operation: Operation,
    pub field_name: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub sequence_number: i32,
    pub applied: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetType {
    Node,
    Edge,
    Layer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub total_edits: usize,
    pub applied: usize,
    pub skipped: usize,
    pub failed: usize,
    pub skipped_details: Vec<SkippedEdit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedEdit {
    pub sequence_number: i32,
    pub target_type: TargetType,
    pub target_id: String,
    pub reason: String,
}
```

### GraphQL Schema

```graphql
type GraphEdit {
  id: Int!
  graphId: Int!
  targetType: String!
  targetId: String!
  operation: String!
  fieldName: String
  oldValue: JSON
  newValue: JSON
  sequenceNumber: Int!
  applied: Boolean!
  createdAt: DateTime!
  createdBy: Int
}

type ReplayResult {
  totalEdits: Int!
  applied: Int!
  skipped: Int!
  failed: Int!
  skippedDetails: [SkippedEditDetail!]!
}

type SkippedEditDetail {
  sequenceNumber: Int!
  targetType: String!
  targetId: String!
  reason: String!
}

extend type Query {
  graphEdits(graphId: Int!): [GraphEdit!]!
  graphEditHistory(graphId: Int!, limit: Int, offset: Int): [GraphEdit!]!
}

extend type Mutation {
  createGraphEdit(
    graphId: Int!
    targetType: String!
    targetId: String!
    operation: String!
    fieldName: String
    oldValue: JSON
    newValue: JSON
  ): GraphEdit!

  replayGraphEdits(graphId: Int!): ReplayResult!
  clearGraphEdits(graphId: Int!): Boolean!
  clearSkippedEdits(graphId: Int!): Boolean!
}
```

---

## Testing Strategy

### Unit Tests

1. **GraphEdit Creation**
   - Test sequence number auto-increment
   - Test all operation types
   - Test all target types
   - Test JSON serialization

2. **Edit Application**
   - Test each operation type individually
   - Test missing target handling
   - Test field update logic
   - Test delete operations

3. **Replay Logic**
   - Test ordered replay
   - Test partial replay (some skip)
   - Test idempotence
   - Test error handling

### Integration Tests

1. **End-to-End Edit Cycle**
   - Create graph → edit → record edit → verify
   - Regenerate graph → replay → verify edits applied

2. **Complex Scenarios**
   - Multiple edits to same entity
   - Edits that conflict with new data
   - Large edit history (100+ edits)
   - Concurrent edit creation

3. **Error Recovery**
   - Invalid edit data
   - Database connection failure during replay
   - Corrupted graph data
   - Missing graph entities

### Performance Tests

1. **Replay Performance**
   - 1000+ edits replay time
   - Memory usage during replay
   - Database query optimization

2. **Edit Creation Performance**
   - Concurrent edit creation
   - High-frequency edits
   - Bulk edit operations

---

## Migration Strategy

### Phase 1: Add alongside existing system
- Deploy GraphEdit table and services
- Don't auto-create edits yet
- Manual testing and validation

### Phase 2: Opt-in recording
- Add feature flag for edit recording
- Enable for test graphs only
- Monitor performance and issues

### Phase 3: Auto-record for new edits
- Automatically create GraphEdit for all new edits
- Existing graphs continue without edits
- Allow manual "Start Tracking" action

### Phase 4: Full rollout
- All graph edits create GraphEdit records
- Auto-replay on regeneration (with opt-out)
- Monitor and optimize

---

## Risks & Mitigations

### Risk 1: Large edit history bloat
**Mitigation**:
- Add edit pruning (keep last N edits)
- Archive old edits to separate table
- Add edit compression

### Risk 2: Replay performance degradation
**Mitigation**:
- Batch edit application
- Optimize database queries
- Add replay progress indicator
- Allow background replay

### Risk 3: Complex conflict scenarios
**Mitigation**:
- Start with simple operations
- Add conflict detection
- Provide conflict resolution UI
- Allow manual edit before replay

### Risk 4: Data consistency issues
**Mitigation**:
- Use database transactions
- Add validation before replay
- Create backup before replay
- Allow replay rollback

---

## Success Metrics

1. **Functionality**: 95%+ of edits successfully replay after regeneration
2. **Performance**: Replay completes in <5 seconds for 100 edits
3. **Reliability**: 0 data loss incidents from replay
4. **Usability**: Users understand and use edit history feature
5. **Adoption**: 80%+ of graphs have edit tracking enabled

---

## Timeline Estimate

- **Phase 1** (Database & Infrastructure): 8-10 hours
- **Phase 2** (Replay Engine): 10-12 hours
- **Phase 3** (Frontend Integration): 8-10 hours
- **Phase 4** (Regeneration Integration): 6-8 hours
- **Phase 5** (Edit History UI): 6-8 hours
- **Phase 6** (Advanced Features): 8-10 hours (optional)

**Total**: 46-58 hours (6-7 working days)
**With Phase 6**: 54-68 hours (7-9 working days)

---

## Future Enhancements

1. **Collaborative Editing**: Show which user made each edit
2. **Edit Comments**: Allow users to annotate edits
3. **Edit Templates**: Save common edit patterns
4. **Diff View**: Visual diff of graph before/after edits
5. **Edit Suggestions**: ML-based suggestions for common edits
6. **Bulk Edit Operations**: Apply same edit to multiple entities
7. **Edit Macros**: Record and replay sequences of edits
8. **Version Control**: Git-like branching and merging for graphs

---

## References

- SPECIFICATION.md - LcGraph and LcGraphEdit relationship
- Database schema: layercake-core/src/database/entities/
- Graph service: layercake-core/src/services/graph_service.rs
- Frontend graph editor: frontend/src/components/graphs/
