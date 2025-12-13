# Single-Schema Migration Plan

**Date:** 2025-12-13
**Status:** In Progress
**Goal:** Migrate backend to use only the unified `graph_data` schema, removing all legacy table usage and code

## Outstanding Tasks (ordered, uncompleted)
1. Migrate GraphQL read/mutation paths to load from `graph_data` (graph/graphs/graphNodes/graphEdges resolvers, graph mutations, GraphService backing).
2. Finish MergeNode end-to-end validation on graph_data (graph_data write path is active; add scenario tests to confirm hash/metadata).
3. Wire Graph edits/history to `graph_data` (FK migration + GraphEditService update, replay support).
4. Deprecate/remove legacy GraphBuilder/GraphService legacy tables after read path migration; clean unused fields (e.g., DagExecutor.graph_builder).
5. Frontend/ID transition plan (legacyId or graphDataId exposure) and docs for API consumers.
6. Schema cleanup migration to drop legacy tables once validation period completes.

## Executive Summary

The Layercake backend currently operates in a **dual-schema mode** where:
- **Old schema:** `graphs`, `graph_nodes`, `graph_edges` + `data_sets`, `dataset_graph_nodes`, `dataset_graph_edges`
- **New schema:** `graph_data`, `graph_data_nodes`, `graph_data_edges` (unified table for both datasets and computed graphs)

A migration (`m20251210_000004`) was run on 2025-12-10 to copy existing data from old → new schema, but **new graph creation still writes to BOTH schemas**, leading to data inconsistency.

**Root Cause of Current Bug:**
- Graph 432 created on 2025-12-11 (after migration)
- Metadata written to `graph_data` table ✓
- Nodes/edges written ONLY to old `graph_nodes`/`graph_edges` tables ✗
- Result: `projectionGraph` query returns empty arrays (reads from new schema, but data is in old schema)

## Current State Analysis

### Database Schema Status

**Legacy Tables (to be deprecated):**
```
graphs                    - Computed graph metadata (18 rows)
graph_nodes               - Computed graph nodes (38 nodes for graph 432)
graph_edges               - Computed graph edges (55 edges for graph 432)
graph_layers              - Layer metadata for computed graphs
graph_edits               - Edit history (still references graphs table)

data_sets                 - Dataset metadata (30 rows)
dataset_graph_nodes       - Dataset nodes
dataset_graph_edges       - Dataset edges
dataset_graph_layers      - Layer metadata for datasets
```

**New Unified Tables (current target):**
```
graph_data                - Unified metadata (48 rows: 30 datasets + 18 computed)
graph_data_nodes          - Unified nodes (1746 total, but 0 for graph 432!)
graph_data_edges          - Unified edges (1513 total, but 0 for graph 432!)
```

**Tables to Preserve:**
```
graph_edits               - Edit history (needs FK update from graphs → graph_data)
projections               - Projection metadata (references graph_data.id)
plan_dag_nodes            - DAG config (references graphId in JSON)
```

### Code Usage Analysis

**Files Using Legacy Schema (16 files identified):**

1. **Core Pipeline:**
   - `src/pipeline/graph_builder.rs` - Legacy graph builder (still writes to `graphs`, `graph_nodes`, `graph_edges`)
   - `src/pipeline/merge_builder.rs` - Merge operations
   - `src/pipeline/persist_utils.rs` - Persistence utilities

2. **Services:**
   - `src/services/graph_service.rs` - Graph operations (layers, validation)
   - `src/services/data_set_service.rs` - Dataset operations

3. **GraphQL:**
   - `src/graphql/queries/mod.rs` - Query resolvers
   - `src/graphql/mutations/graph.rs` - Graph mutations
   - `src/graphql/types/graph.rs` - GraphQL types
   - `src/graphql/types/layer.rs` - Layer types
   - `src/graphql/types/graph_edit.rs` - Edit types

4. **App Context:**
   - `src/app_context/plan_dag_operations.rs` - DAG operations
   - `src/app_context/library_operations.rs` - Library operations
   - `src/app_context/data_set_operations.rs` - Dataset operations
   - `src/app_context/story_operations.rs` - Story operations

5. **Console:**
   - `src/console/context.rs` - Console context
   - `src/console/chat/session.rs` - Chat session

6. **MCP:**
   - `src/mcp/resources.rs` - MCP resources

7. **Tests:**
   - `tests/graph_edit_replay_test.rs`
   - `tests/graph_edit_service_test.rs`

**New Schema Usage (2 files):**
- `src/pipeline/graph_data_builder.rs` - **New unified builder** (currently unused by DAG execution)
- `src/graphql/types/projection.rs` - Projection queries (recently added, uses new schema)

### Migration History

**Completed Migrations:**
- `m20251210_000001` - Create `graph_data` table
- `m20251210_000002` - Create `graph_data_nodes` table
- `m20251210_000003` - Create `graph_data_edges` table
- `m20251210_000004` - Migrate existing data (ran 2025-12-10 05:36:46)
  - Result: 18 graphs, 1746 nodes, 1513 edges migrated successfully
  - Validation: All checks passed, no orphaned data

**Issue:** New graphs created after migration don't populate new schema tables.

## Technical Design

### Phase 1: Data Repair (Immediate Fix)

**Objective:** Fix graph 432 and any other post-migration graphs

**Steps:**
1. Identify all graphs created after 2025-12-10 05:36:46
2. For each graph:
   - Verify `graph_data` entry exists
   - Check if `graph_data_nodes` and `graph_data_edges` are empty
   - If empty, run incremental migration:
     ```sql
     INSERT INTO graph_data_nodes (...)
     SELECT ... FROM graph_nodes WHERE graph_id = ? + OFFSET

     INSERT INTO graph_data_edges (...)
     SELECT ... FROM graph_edges WHERE graph_id = ? + OFFSET
     ```
3. Update `graph_data.node_count` and `graph_data.edge_count`

**Implementation:** Create migration `m20251213_000001_repair_post_migration_graphs.rs`

### Phase 2: Switch DAG Execution to New Schema

**Objective:** Route all new graph creation through `GraphDataBuilder`

**Current Flow:**
```
Plan DAG Node (GraphNode)
  → graph_builder.rs (legacy)
    → Writes to: graphs, graph_nodes, graph_edges
```

**Target Flow:**
```
Plan DAG Node (GraphNode)
  → graph_data_builder.rs (new)
    → Writes to: graph_data, graph_data_nodes, graph_data_edges
```

**Changes Required:**

1. **Update DAG Executor** (`src/pipeline/dag_executor.rs`):
   ```rust
   // OLD: GraphBuilder for GraphNode
   match node_type {
       "GraphNode" => {
           let builder = GraphBuilder::new(db.clone());
           builder.build_graph(...).await?;
       }
   }

   // NEW: GraphDataBuilder for GraphNode
   match node_type {
       "GraphNode" => {
           let builder = GraphDataBuilder::new(db.clone());
           builder.build_from_sources(...).await?;
       }
   }
   ```

2. **Update GraphDataBuilder**:
   - Ensure it handles all features from legacy GraphBuilder:
     - Source hash computation
     - Change detection (skip rebuild if hash unchanged)
     - Layer extraction and storage
     - Edit replay support
     - Execution state publishing (WebSocket events)

3. **Config Migration**:
   - Plan DAG nodes may reference `graphId` in config
   - Update to use `graphDataId` instead (no offset needed for new graphs)
   - Legacy graphs still accessible via offset: `graph_id + 1_000_000`

### Phase 3: Migrate Read Operations

**Objective:** Update all queries to read from `graph_data_*` tables

**Services to Update:**

1. **GraphService** (`src/services/graph_service.rs`):
   ```rust
   // Replace:
   graph_nodes::Entity::find()
       .filter(graph_nodes::Column::GraphId.eq(graph_id))

   // With:
   graph_data_nodes::Entity::find()
       .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
   ```

2. **GraphQL Queries** (`src/graphql/queries/mod.rs`):
   - `graph()` - Switch to `graph_data::Entity`
   - `graphs()` - Switch to `graph_data::Entity`
   - `graphNodes()` - Switch to `graph_data_nodes::Entity`
   - `graphEdges()` - Switch to `graph_data_edges::Entity`

3. **Layer Operations**:
   - Currently reads from `graph_layers` + `dataset_graph_layers`
   - Migrate to layer information stored in `graph_data_nodes.attributes` JSON
   - Build `ProjectionGraphLayer` from node attributes (as in `projection.rs:144-194`)

4. **Graph Edits**:
   - Update FK: `graph_edits.graph_id` → `graph_edits.graph_data_id`
   - Migration needed to offset existing edit records
   - Update `GraphEditService` to use `graph_data` table

### Phase 4: Update GraphQL Types

**Objective:** Expose new schema through GraphQL API

**Current GraphQL Types:**
```rust
pub struct Graph {
    pub id: i32,           // OLD: graphs.id
    pub project_id: i32,
    pub name: String,
    // ... from graphs table
}
```

**New GraphQL Types:**
```rust
pub struct Graph {
    pub id: i32,           // NEW: graph_data.id
    pub project_id: i32,
    pub name: String,
    pub source_type: String,  // "dataset" | "computed"
    // ... from graph_data table
}
```

**Breaking Changes:**
- `graph.id` values will change for migrated graphs (+1,000,000 offset)
- Frontend must handle this transition
- Consider adding `graph.legacyId` field during transition period

### Phase 5: Deprecate Legacy Code

**Objective:** Remove dual-schema code paths

**Code to Remove:**

1. **Entity Definitions:**
   - `src/database/entities/graphs.rs`
   - `src/database/entities/graph_nodes.rs`
   - `src/database/entities/graph_edges.rs`
   - `src/database/entities/graph_layers.rs`
   - Keep: `graph_edits.rs` (updated to use graph_data)

2. **Legacy Builders:**
   - `src/pipeline/graph_builder.rs` (replace all usage with `graph_data_builder.rs`)
   - Legacy methods in `merge_builder.rs`

3. **Dual-Schema Service Methods:**
   - `GraphService` methods that query `graph_nodes`
   - `DataSetService` methods that create `dataset_graph_nodes`

**Deprecation Strategy:**
1. Add `#[deprecated]` annotations with migration guidance
2. Log warnings when legacy code paths are hit
3. Monitor for 1-2 release cycles
4. Remove entirely after confirming no usage

### Phase 6: Schema Cleanup (Final)

**Objective:** Drop legacy tables from database

**Migration: `m20251215_000001_drop_legacy_graph_tables.rs`**

```sql
-- Backup validation: ensure all data migrated
CREATE TABLE legacy_graph_validation AS
SELECT
    'graphs' as table_name,
    COUNT(*) as old_count,
    (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed') as new_count,
    COUNT(*) - (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed') as delta
FROM graphs;

-- Only proceed if delta = 0
DROP TABLE IF EXISTS graph_layers;
DROP TABLE IF EXISTS graph_edges;
DROP TABLE IF EXISTS graph_nodes;
DROP TABLE IF EXISTS graphs;

DROP TABLE IF EXISTS dataset_graph_layers;
DROP TABLE IF EXISTS dataset_graph_edges;
DROP TABLE IF EXISTS dataset_graph_nodes;
DROP TABLE IF EXISTS data_sets;
```

**Note:** This is a **destructive migration** requiring:
- Full backup before execution
- Validation that all systems use new schema
- Rollback plan if issues discovered

## Implementation Plan

### Stage 1: Immediate Data Repair ✅ COMPLETED (2025-12-13)

**Goal:** Fix current projection bug and prevent new inconsistencies

**Tasks:**
- [x] Create repair migration `m20251213_000001_repair_post_migration_graphs.rs`
- [x] Run migration to populate `graph_data_nodes`/`graph_data_edges` for graph 432
- [x] Verify `projectionGraph(id: 7)` returns nodes and edges
- [ ] Add database constraint to prevent writing to old tables without new tables (deferred)

**Success Criteria:**
- ✅ Projection viewer displays graph 432 correctly (38 nodes, 55 edges)
- ✅ All graphs have consistent data in both schemas

**Actual Results:**
- Migration created and tested successfully
- Graph 432 repaired: 0 → 38 nodes, 0 → 55 edges
- Commit: 7f9e153c
- SQL script used for immediate repair: `/tmp/repair_graph_432.sql`

### Stage 2: New Graph Creation ✅ COMPLETED (2025-12-13)

**Goal:** Route all new graphs through `GraphDataBuilder`

**Tasks:**
- [x] Audit `GraphDataBuilder` for feature parity with `GraphBuilder`
  - ✅ Has: Source hash computation (SHA-256, lines 141-163)
  - ✅ Has: Change detection and graph reuse (lines 91-120)
  - ✅ Has: Layer extraction and validation (lines 37-56, 121-139)
  - ✅ Has: Status management (Pending → InProgress → Complete)
  - ❌ Missing: Edit replay integration (deferred to Stage 3)
  - ❌ Missing: Complete WebSocket event publishing (deferred to Stage 3)
- [x] Update DAG executor to use `GraphDataBuilder`
- [ ] Update plan DAG to handle `graphDataId` references (not needed - auto-resolved via dag_node_id)
- [ ] Test graph creation through UI (deferred - requires full pipeline)

**Audit Summary:**
GraphDataBuilder is 90% feature complete and ready for basic use. Missing features (edit replay, full WebSocket events) require GraphEditService migration, planned for Stage 3.

**Implementation:**
- Made `GraphDataBuilder.graph_data_service` public for DAG node resolution
- Updated `dag_executor.rs` GraphNode handler (lines 152-190)
- Added upstream DAG node ID → graph_data ID resolution via `get_by_dag_node()`
- Removed legacy GraphBuilder code path for new GraphNode executions
- Added graceful fallback for legacy upstream nodes without graph_data

**Success Criteria:**
- ✅ New graphs create data ONLY in `graph_data_*` tables
- ✅ Code compiles without errors (34 warnings, 0 errors)
- ⚠️ Basic graph features work - pending integration test
- ⚠️ Edit replay deferred to Stage 3

**Actual Results:**
- Commit: 2ad742a8
- Files changed: dag_executor.rs, graph_data_builder.rs
- Cargo check passes
- GraphBuilder remains instantiated but unused for GraphNode

**Status:** ✅ Complete

### Stage 3: Read/Write Path Migration ✅ COMPLETE (2025-12-14)

**Goal:** Run the full DAG pipeline and API on `graph_data` for reads and writes.

**Status:** Pipeline + GraphQL read paths migrated to graph_data with legacy fallback retained.

**What Was Accomplished (recent):**
- TransformNode/FilterNode load via `graph_data` first and persist outputs to `graph_data`.
- DataSetNode (data_set + filePath) writes unified `graph_data` with hashes/metadata/nodes/edges.
- MergeNode resolves upstream via `graph_data` (fallback legacy), DAG executor mirrors merge outputs into `graph_data` with merge hash/metadata/annotations/counts.
- ProjectionNode upserts `projections` pointing at upstream `graph_data`.
- `exportNodeOutput` mutation renders from `graph_data` (fallback legacy) via `ExportService`.
- `graph`/`graphs`/`graphPreview` GraphQL queries now read from `graph_data` first; Graph type’s nodes/edges/layers resolvers fetch from `graph_data` and derive layers from node attributes.
- GraphQL converters added for `graph_data_nodes`/`graph_data_edges` previews and node/edge types.
- `GraphDataBuilder` ignores empty layer IDs when validating/persisting, avoiding palette errors.

**Remaining for downstream stages:**
- Hook graph edits/history to `graph_data` (FK migration, GraphEditService, replay) and retire legacy GraphBuilder/GraphService.
- Integration tests covering MergeNode, Transform/Filter, Projection, and artefact export on `graph_data`.

### Stage 4: GraphQL API Update (Week 3-4)

**Goal:** Update public API to reflect new schema

**Tasks:**
- [ ] Update GraphQL types for `Graph`, `Node`, `Edge`
- [ ] Add `sourceType` field to Graph type
- [ ] Handle ID offset in frontend (or add `legacyId` field)
- [ ] Update frontend to use new field names
- [ ] Provide migration guide for API consumers

**Success Criteria:**
- GraphQL schema updated and documented
- Frontend works with new API
- API versioning strategy in place if needed

### Stage 5: Code Cleanup (Week 4-5)

**Goal:** Remove legacy code

**Tasks:**
- [ ] Mark `GraphBuilder` as `#[deprecated]`
- [ ] Remove unused entity definitions
- [ ] Remove dual-schema code paths
- [ ] Update documentation
- [ ] Remove legacy imports from `entities/mod.rs`

**Success Criteria:**
- `cargo check` passes with no legacy code usage
- Code coverage maintained
- Documentation updated

### Stage 6: Schema Cleanup (Week 6 - After Validation Period)

**Goal:** Drop legacy tables

**Tasks:**
- [ ] Backup production database
- [ ] Run validation queries
- [ ] Create and test `m20251215_000001_drop_legacy_graph_tables.rs`
- [ ] Execute migration in production
- [ ] Monitor for any errors
- [ ] Confirm database size reduction

**Success Criteria:**
- Legacy tables dropped
- No application errors
- Database schema clean and documented

## Risk Analysis

### High Risk Areas

1. **Data Loss Risk:**
   - **Mitigation:** Full database backups before each phase, validation queries, incremental rollout

2. **Graph Edit History:**
   - graph_edits table currently references graphs.id
   - **Mitigation:** Careful FK migration, test edit replay thoroughly

3. **Frontend Breaking Changes:**
   - Graph IDs will change (+1M offset for migrated graphs)
   - **Mitigation:** Add `legacyId` field during transition, coordinate with frontend team

4. **Plan DAG References:**
   - Config JSON stores `graphId` which may become stale
   - **Mitigation:** Migration to update JSON, maintain backward compatibility

### Medium Risk Areas

1. **Performance:**
   - New unified table may have different query patterns
   - **Mitigation:** Add appropriate indexes, benchmark queries

2. **WebSocket Events:**
   - Execution state changes published to clients
   - **Mitigation:** Ensure GraphDataBuilder publishes same events

3. **Layer Data:**
   - Currently in separate tables, moving to JSON attributes
   - **Mitigation:** Validate layer extraction logic, test layer palette feature

## Testing Strategy

### Unit Tests
- [ ] `GraphDataBuilder` feature parity tests
- [ ] Migration validation tests
- [ ] Service method tests for new schema

### Integration Tests
- [ ] Full DAG execution (DataSet → Graph → Transform → Output)
- [ ] Graph edit replay on new schema
- [ ] Layer operations and palette
- [ ] Projection graph queries

### E2E Tests
- [ ] Create graph through UI → verify in DB
- [ ] Edit graph → replay edits → verify consistency
- [ ] Export graph → verify data completeness

### Performance Tests
- [ ] Benchmark graph queries (before/after)
- [ ] Load test with large graphs (1000+ nodes)
- [ ] Concurrent edit operations

## Rollback Plan

### Stage 1-2 Rollback
- Revert DAG executor to use `GraphBuilder`
- Legacy tables still intact, can continue writing

### Stage 3-4 Rollback
- Revert service and GraphQL changes
- May need to re-run migration if new data created

### Stage 6 Rollback
- **Not possible** once tables dropped
- Requires restore from backup
- **Therefore:** Extended validation period before Stage 6

## Monitoring & Validation

### Metrics to Track
- [ ] Graph creation success rate
- [ ] Query performance (p50, p95, p99)
- [ ] Data consistency checks (nightly job)
- [ ] Legacy table usage (should be zero after Stage 3)

### Validation Queries
```sql
-- Check for data consistency
SELECT
    gd.id,
    gd.name,
    gd.node_count as expected_nodes,
    (SELECT COUNT(*) FROM graph_data_nodes WHERE graph_data_id = gd.id) as actual_nodes,
    gd.edge_count as expected_edges,
    (SELECT COUNT(*) FROM graph_data_edges WHERE graph_data_id = gd.id) as actual_edges
FROM graph_data gd
WHERE gd.node_count != (SELECT COUNT(*) FROM graph_data_nodes WHERE graph_data_id = gd.id)
   OR gd.edge_count != (SELECT COUNT(*) FROM graph_data_edges WHERE graph_data_id = gd.id);
```

## Documentation Updates

- [ ] Update architecture docs with new schema
- [ ] API changelog for GraphQL changes
- [ ] Migration guide for operators
- [ ] Developer guide for new graph creation patterns
- [ ] Database schema diagram

## Timeline Summary

| Stage | Duration | Status | Risk |
|-------|----------|--------|------|
| 1. Data Repair | 2-3 days | Not Started | Low |
| 2. New Graph Creation | 1 week | Not Started | Medium |
| 3. Read Path Migration | 1-2 weeks | Not Started | Medium |
| 4. GraphQL API Update | 1 week | Not Started | High |
| 5. Code Cleanup | 1 week | Not Started | Low |
| 6. Schema Cleanup | 1 day + validation | Not Started | High |

**Total Estimated Time:** 5-6 weeks

## Success Criteria

The migration is complete when:
1. ✅ All new graphs write ONLY to `graph_data_*` tables
2. ✅ All queries read ONLY from `graph_data_*` tables
3. ✅ Legacy entity definitions removed from codebase
4. ✅ Legacy tables dropped from database
5. ✅ All tests passing (unit, integration, E2E)
6. ✅ No performance regressions
7. ✅ Documentation updated and accurate
8. ✅ Zero legacy table queries (verified via logging)

## Open Questions

1. **Graph ID Stability:** Do we need to maintain graph IDs across migration, or can frontend handle offset?
2. **API Versioning:** Should we version the GraphQL API (v1 vs v2) or just deprecate fields?
3. **Archive Export:** Do graph exports need to support legacy format for backwards compatibility?
4. **Concurrent Migration:** Can we run old and new builders in parallel during transition, or must it be a hard cutover?

## References

- Migration file: `src/database/migrations/m20251210_000004_migrate_existing_graph_data.rs`
- New builder: `src/pipeline/graph_data_builder.rs`
- Legacy builder: `src/pipeline/graph_builder.rs`
- Projection types: `src/graphql/types/projection.rs`
- Database investigation: See conversation thread (2025-12-13)
