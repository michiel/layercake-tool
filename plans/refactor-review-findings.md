# Refactoring Plan Review: Findings and Recommendations

**Date**: 2025-12-10
**Reviewer**: Analysis of updated refactor-datasets-and-graphs.md
**Status**: Critical issues identified requiring resolution before Phase 1

---

## Executive Summary

The updated plan addresses several important clarifications (surrogate PKs, strict layer validation, canonical status). However, **critical ambiguities and unaddressed risks remain** that could derail the migration, particularly around:

1. **GraphEdit system compatibility** with surrogate PKs
2. **Primary key contradictions** in schema design
3. **Missing execution state mapping** for migration
4. **Foreign key strategy** for edge source/target references
5. **Annotation format migration** edge cases

**Recommendation**: Address critical issues (Section 1-3) before beginning Phase 1 implementation.

---

## 1. Critical Ambiguities

### 1.1 Primary Key Contradiction

**Location**: Section 3.1.1 - Schema definition

**Issue**: The schema contains contradictory primary key definitions:

```sql
CREATE TABLE graph_data_nodes (
    id INTEGER PRIMARY KEY,     -- Line 157: Single column PK
    graph_data_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    ...
    PRIMARY KEY (id, graph_data_id),  -- Line 170: Composite PK
    ...
);
```

**Problem**: Cannot have both single-column PK and composite PK.

**Impact**:
- Schema invalid, won't create
- Unclear which approach to use
- Affects query patterns and performance

**Options**:

**Option A: Composite PK (id, graph_data_id)**
```sql
CREATE TABLE graph_data_nodes (
    id INTEGER NOT NULL,  -- Part of composite PK
    graph_data_id INTEGER NOT NULL,  -- Part of composite PK
    external_id TEXT NOT NULL,
    ...
    PRIMARY KEY (id, graph_data_id),
    UNIQUE (graph_data_id, external_id)
);
```
- Pros: Mirrors current schema pattern
- Cons: More complex queries, larger indexes

**Option B: Autoincrement Surrogate PK**
```sql
CREATE TABLE graph_data_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- Surrogate PK
    graph_data_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    ...
    UNIQUE (graph_data_id, external_id)
);
```
- Pros: Simple, efficient joins, standard pattern
- Cons: Adds extra column

**Recommendation**: **Option B** - Use autoincrement surrogate PK. Simpler, more efficient, clearer semantics.

**Required Changes**:
- Update schema in Section 3.1.1
- Update migration SQL in Appendix C
- Clarify in GraphNode/GraphEdge structs (db_id is the PK)

---

### 1.2 Execution State to Status Mapping

**Location**: Phase 2 migration script (line 667)

**Issue**: States "map prior pipeline states into canonical status" but provides no mapping.

**Current execution_state enum values** (from codebase):
```rust
pub enum ExecutionState {
    NotStarted,
    Pending,
    Processing,
    Completed,
    Error,
}
```

**Problem**: How do these map to the 3 canonical statuses ('active', 'processing', 'error')?

**Proposed Mapping**:
```rust
// Migration logic
fn map_execution_state_to_status(exec_state: &str) -> &str {
    match exec_state {
        "Completed" => "active",
        "Processing" => "processing",
        "Error" => "error",
        "NotStarted" => "processing",  // ❓ Or should this be a new status?
        "Pending" => "processing",
        _ => "error"  // Unknown states treated as error
    }
}
```

**Ambiguities**:
- What happens to "NotStarted" graphs? Should they be "processing" or need a new "pending" status?
- Do we lose information by collapsing 5 states to 3?
- Can we reconstruct execution_state from status if needed?

**Recommendation**:
1. Expand canonical status to include "pending": `'pending' | 'processing' | 'active' | 'error'`
2. Document explicit mapping in Phase 2
3. Add validation query to verify no unmapped states exist

**Required Changes**:
```sql
-- Add to migration validation (Appendix C)
SELECT DISTINCT execution_state
FROM graphs
WHERE execution_state NOT IN ('NotStarted', 'Pending', 'Processing', 'Completed', 'Error');
-- Should return 0 rows

-- Update status mapping in migration
CASE
    WHEN execution_state = 'Completed' THEN 'active'
    WHEN execution_state IN ('Processing') THEN 'processing'
    WHEN execution_state IN ('NotStarted', 'Pending') THEN 'pending'
    WHEN execution_state = 'Error' THEN 'error'
    ELSE 'error'
END AS status
```

---

### 1.3 Edge Source/Target Reference Strategy

**Location**: Section 3.1.1 - graph_data_edges schema

**Issue**: Edges reference nodes via TEXT fields (external_id), but no foreign key constraint:

```sql
CREATE TABLE graph_data_edges (
    ...
    source TEXT NOT NULL,       -- references node external_id
    target TEXT NOT NULL,       -- references node external_id
    ...
);
```

**Problem**: No referential integrity enforcement.

**Impact**:
- Orphaned edges if nodes deleted
- Invalid edge references possible
- Harder to maintain data consistency

**Options**:

**Option A: Keep TEXT, No FK (Current)**
- Pros: Flexible, simple schema
- Cons: No integrity enforcement, orphaned edges possible

**Option B: Add FK to external_id**
```sql
CREATE TABLE graph_data_edges (
    ...
    source TEXT NOT NULL,
    target TEXT NOT NULL,
    ...
    FOREIGN KEY (graph_data_id, source)
        REFERENCES graph_data_nodes(graph_data_id, external_id),
    FOREIGN KEY (graph_data_id, target)
        REFERENCES graph_data_nodes(graph_data_id, external_id)
);
```
- Pros: Referential integrity enforced
- Cons: Requires index on (graph_data_id, external_id)

**Option C: Reference surrogate id (db_id)**
```sql
CREATE TABLE graph_data_edges (
    ...
    source_node_id INTEGER NOT NULL,  -- FK to graph_data_nodes.id
    target_node_id INTEGER NOT NULL,
    source_external_id TEXT NOT NULL,  -- Denormalised for convenience
    target_external_id TEXT NOT NULL,
    ...
    FOREIGN KEY (source_node_id) REFERENCES graph_data_nodes(id),
    FOREIGN KEY (target_node_id) REFERENCES graph_data_nodes(id)
);
```
- Pros: Strong integrity, efficient joins
- Cons: Denormalisation, more complex

**Recommendation**: **Option B** - Add FK constraints on (graph_data_id, external_id). Provides integrity without denormalisation.

**Required Changes**:
- Update schema in Section 3.1.1
- Add indexes: `CREATE INDEX idx_nodes_external ON graph_data_nodes(graph_data_id, external_id)`
- Update migration to verify edge references are valid before inserting

---

## 2. Unaddressed Risks

### 2.1 GraphEdit System Incompatibility ⚠️ CRITICAL

**Location**: Not mentioned in plan

**Current Implementation**:
```rust
// graph_edits table
pub struct Model {
    pub graph_id: i32,           // FK to graphs.id
    pub target_type: String,     // 'node', 'edge', 'layer'
    pub target_id: String,       // References node/edge/layer ID
    pub operation: String,       // 'create', 'update', 'delete'
    ...
}
```

**Problem**: `target_id` currently references node/edge IDs directly (what will become external_id). If we change to surrogate PKs, edit replay logic may break.

**Impact**:
- **HIGH RISK**: Edit replay is a core feature
- User edits could be lost during migration
- No rollback if edits fail to replay on new schema

**Questions**:
1. Does `target_id` reference the old `graph_nodes.id` (TEXT) or the new `graph_data_nodes.id` (INTEGER)?
2. After migration, should `target_id` reference `external_id` (user-facing) or `db_id` (internal)?
3. How do we migrate existing graph_edits to work with new schema?

**Recommendation**:
1. **Clarify**: Confirm that graph_edits.target_id references external_id (user-provided IDs)
2. **Migrate**: Update graph_edits.graph_id to reference graph_data.id (new table)
3. **Test**: Comprehensive edit replay tests before and after migration
4. **Phase**: Add explicit task to Phase 2: "Migrate graph_edits table and validate replay"

**Required Changes**:
```sql
-- Phase 2 migration addition
-- Update graph_edits to reference graph_data
UPDATE graph_edits
SET graph_id = (
    SELECT graph_data.id
    FROM graph_data
    WHERE graph_data.id = graph_edits.graph_id
      AND graph_data.source_type = 'computed'
);

-- Validate all edits have valid references
SELECT COUNT(*) FROM graph_edits e
WHERE NOT EXISTS (
    SELECT 1 FROM graph_data_nodes n
    WHERE n.graph_data_id = e.graph_id
      AND n.external_id = e.target_id
      AND e.target_type = 'node'
)
AND e.target_type = 'node';
-- Should return 0
```

---

### 2.2 ID Collision Risk During Migration

**Location**: Phase 2 - Data Migration

**Issue**: Merging datasets and graphs into single `graph_data` table. What if:
- Dataset ID 42 exists
- Graph ID 42 exists
- Both try to claim `graph_data.id = 42`

**Current Plan**: "Reseed autoincrement sequences to avoid ID collisions" (Appendix C, line 1367)

**Problem**: Not specific enough. How exactly do we avoid collisions?

**Impact**: Migration could fail with PRIMARY KEY violation

**Options**:

**Option A: Offset Dataset IDs**
```sql
-- Migrate datasets with offset
INSERT INTO graph_data (id, ...)
SELECT id + 1000000, ...  -- Offset by 1M
FROM data_sets;

-- Migrate graphs as-is
INSERT INTO graph_data (id, ...)
SELECT id, ...
FROM graphs;
```

**Option B: Let autoincrement assign new IDs**
```sql
-- Don't specify id, let DB assign
INSERT INTO graph_data (project_id, name, ...)
SELECT project_id, name, ...
FROM data_sets;

-- Map old IDs to new IDs
CREATE TEMP TABLE id_mapping (
    old_id INTEGER,
    new_id INTEGER,
    source_type TEXT
);
```
- Problem: Breaks FK references from other tables

**Option C: Separate ID sequences by source_type**
```sql
-- Datasets: 1-999,999
-- Graphs: 1,000,000-1,999,999
-- Manual: 2,000,000+

INSERT INTO graph_data (id, ...)
SELECT
    id,  -- datasets already < 1M
    ...
FROM data_sets;

INSERT INTO graph_data (id, ...)
SELECT
    id + 1000000,  -- graphs offset by 1M
    ...
FROM graphs;
```

**Recommendation**: **Option C** with validation:
1. Verify max dataset ID < 1,000,000
2. Offset graph IDs by 1,000,000
3. Update all FK references (graph_edits, plan_dag_nodes, etc.)

**Required Changes**:
```sql
-- Add to Phase 2 migration

-- 1. Validate no ID collisions
SELECT
    CASE WHEN MAX(id) >= 1000000 THEN 'ERROR' ELSE 'OK' END AS dataset_check
FROM data_sets;

-- 2. Migrate with offset
INSERT INTO graph_data (id, ...)
SELECT
    CASE source_table
        WHEN 'dataset' THEN id
        WHEN 'graph' THEN id + 1000000
    END,
    ...

-- 3. Update FK references
UPDATE graph_edits
SET graph_id = graph_id + 1000000;

UPDATE plan_dag_nodes
SET config_json = json_set(
    config_json,
    '$.graphId',
    json_extract(config_json, '$.graphId') + 1000000
)
WHERE node_type = 'GraphNode';
```

---

### 2.3 Annotation Format Migration

**Location**: Appendix C, line 1279

**Current Migration**:
```sql
json(annotations),  -- normalize to JSON array
```

**Problem**: What if annotations is already JSON? Or NULL? Or empty string?

**Edge Cases**:
1. `annotations = NULL` → `json(NULL)` = `NULL` ✓
2. `annotations = ""` → `json("")` = SQL error ✗
3. `annotations = '["foo"]'` → `json('["foo"]')` = double-encoded `"[\"foo\"]"` ✗
4. `annotations = "line1\nline2"` → `json("line1\nline2")` = SQL error ✗

**Impact**: Migration could fail or corrupt annotation data

**Recommendation**: Add annotation normalization function:

```sql
-- Add to migration
CREATE TEMP TABLE annotation_types AS
SELECT
    id,
    CASE
        WHEN annotations IS NULL THEN 'null'
        WHEN annotations = '' THEN 'empty'
        WHEN json_valid(annotations) THEN 'json'
        ELSE 'text'
    END AS type
FROM data_sets;

-- Migrate with proper handling
INSERT INTO graph_data (annotations, ...)
SELECT
    CASE t.type
        WHEN 'null' THEN NULL
        WHEN 'empty' THEN '[]'
        WHEN 'json' THEN ds.annotations  -- Already JSON
        WHEN 'text' THEN json_array(ds.annotations)  -- Wrap in array
    END,
    ...
FROM data_sets ds
JOIN annotation_types t ON t.id = ds.id;
```

---

### 2.4 Missing Index Strategy

**Location**: Not addressed in plan

**Problem**: No discussion of indexes for new schema. Queries will be slow without proper indexes.

**Critical Indexes Needed**:
```sql
-- graph_data
CREATE INDEX idx_graph_data_project ON graph_data(project_id);
CREATE INDEX idx_graph_data_dag_node ON graph_data(dag_node_id);
CREATE INDEX idx_graph_data_source_type ON graph_data(project_id, source_type);

-- graph_data_nodes
CREATE INDEX idx_nodes_graph ON graph_data_nodes(graph_data_id);
CREATE INDEX idx_nodes_external ON graph_data_nodes(graph_data_id, external_id);
CREATE INDEX idx_nodes_layer ON graph_data_nodes(layer);  -- For palette queries
CREATE INDEX idx_nodes_belongs_to ON graph_data_nodes(belongs_to);  -- For hierarchy

-- graph_data_edges
CREATE INDEX idx_edges_graph ON graph_data_edges(graph_data_id);
CREATE INDEX idx_edges_external ON graph_data_edges(graph_data_id, external_id);
CREATE INDEX idx_edges_source ON graph_data_edges(source);  -- For path queries
CREATE INDEX idx_edges_target ON graph_data_edges(target);
CREATE INDEX idx_edges_source_target ON graph_data_edges(source, target);  -- For existence checks
CREATE INDEX idx_edges_layer ON graph_data_edges(layer);
```

**Recommendation**: Add Section 3.1.4 "Index Strategy" before Phase 1 implementation.

---

### 2.5 Large Graph Memory Risk

**Location**: Section 3.2.1 - GraphDataService::load_full()

**Issue**: `load_full()` loads all nodes and edges into memory.

**Problem**: For large graphs (100k+ nodes), this could consume gigabytes of RAM.

**Example**:
- 100,000 nodes × ~500 bytes/node = ~50 MB
- 500,000 edges × ~400 bytes/edge = ~200 MB
- Total: ~250 MB per graph (acceptable)

BUT:
- 1,000,000 nodes = ~500 MB
- 5,000,000 edges = ~2 GB
- Total: ~2.5 GB per graph (problematic)

**Questions**:
1. What's the expected max graph size?
2. Should `load_full()` have pagination or streaming?
3. Can we lazy-load nodes/edges only when needed?

**Recommendation**:
1. Add `max_nodes` limit to load_full() with error if exceeded
2. Add `load_nodes_paginated()` and `load_edges_paginated()`
3. Update Phase 3 to handle large graphs
4. Document recommended max graph size

---

## 3. Critical Open Questions

### 3.1 Backward Compatibility Duration

**Question**: How long do GraphQL facade types (`DataSet`, `Graph`) remain?

**Impact**:
- Frontend migration timeline
- API versioning strategy
- Support burden

**Needs Decision**:
- Phase 4 creates facades
- When do we remove them?
- What's the deprecation schedule?
- How do we communicate to API consumers?

**Recommendation**: Add to plan:
- Facades remain until frontend fully migrated (estimate: 2-3 months post-Phase 4)
- Deprecation warnings in Phase 4
- Sunset in Phase 7 (new phase after Phase 6)

---

### 3.2 Concurrent Access During Migration

**Question**: Can the system operate while migration is running?

**Scenarios**:
1. **Downtime migration**: Take system offline, migrate, bring back up
2. **Hot migration**: System runs, migration happens in background
3. **Dual-write**: Write to both old and new tables during transition

**Current Plan**: Silent on this. Phase 2 says "Run migration on test database" but not production strategy.

**Impact**:
- User experience during migration
- Data consistency requirements
- Rollback complexity

**Recommendation**:
1. **Phase 1-2**: Can run with downtime (schema changes only)
2. **Phase 3**: Requires feature flag for gradual rollout
3. **Phase 4-6**: Can run hot with facades
4. Document downtime requirements in Phase timeline

---

### 3.3 Test Data Volume Requirements

**Question**: What constitutes "production-like data" for testing?

**Current Plan**: "Test with production-like data" (Phase 2, line 671)

**Needs Specificity**:
- How many projects?
- How many graphs per project?
- How many nodes/edges per graph?
- How many edits to replay?

**Recommendation**: Define test data requirements:
```
Minimal:
- 10 projects
- 50 graphs (mix of datasets and computed)
- 1,000-10,000 nodes/edges per graph
- 100 graph_edits

Realistic:
- 100 projects
- 500 graphs
- 10,000-100,000 nodes/edges per graph
- 1,000 graph_edits

Stress:
- 1,000 projects
- 5,000 graphs
- 100,000-1,000,000 nodes/edges in largest graphs
- 10,000 graph_edits
```

---

### 3.4 Rollback After Phase 4

**Question**: Can we rollback after GraphQL API changes in Phase 4?

**Current Plan**: "Keep old tables until Phase 6" but:
- Phase 3 writes only to new tables
- Phase 4 changes API
- Rollback requires reconstructing old tables from new

**Problem**: Rollback complexity increases significantly after Phase 3.

**Recommendation**:
1. Phase 3 should be "reversible checkpoint"
2. Dual-write to both old and new tables during Phase 3-5
3. Phase 6 becomes point of no return (drop old tables)
4. Document explicit rollback procedures for each phase

---

### 3.5 Foreign Key Cascade Strategy

**Location**: Section 3.1.1, lines 171 and 189

**Current**:
```sql
FOREIGN KEY (graph_data_id) REFERENCES graph_data(id) ON DELETE CASCADE
```

**Question**: Should `source_dataset_id` also cascade or SET NULL?

```sql
-- Current (no constraint)
source_dataset_id INTEGER,

-- Option A: Cascade
source_dataset_id INTEGER,
FOREIGN KEY (source_dataset_id) REFERENCES graph_data(id) ON DELETE CASCADE

-- Option B: SET NULL
source_dataset_id INTEGER,
FOREIGN KEY (source_dataset_id) REFERENCES graph_data(id) ON DELETE SET NULL
```

**Impact**:
- If dataset deleted, what happens to computed graphs that reference it?
- Cascade: Computed graphs also deleted (may not want this)
- SET NULL: Traceability lost but graphs remain

**Recommendation**: **SET NULL** - preserve computed graphs even if source dataset deleted.

---

## 4. Minor Ambiguities

### 4.1 Status vs execution_state Terminology

**Issue**: Plan says "remove execution_state from schema" but GraphQL section mentions:

```rust
pub execution_state: Option<String>,  // Computed-specific (NULL for datasets)
```

**Clarification Needed**: Is this:
1. Removed from database but derived in GraphQL? OR
2. A typo that should be removed?

**Recommendation**: Remove from GraphQL type definition, use `status` only.

---

### 4.2 Metadata Schema Validation

**Issue**: `metadata JSON` has no schema or validation mentioned.

**Questions**:
- Are there common/expected metadata fields?
- Should we validate structure?
- Migration preserves as-is?

**Recommendation**: Document that metadata is freeform, preserve as-is in migration. Can add schema validation later if needed.

---

### 4.3 Layer Validation Failure UX

**Issue**: Changed to "Strict validation; missing layers fail the operation" but no UX guidance.

**Questions**:
- What error message does user see?
- Can they retry after adding layers?
- Should we batch-validate before starting import?

**Recommendation**: Add to Phase 5:
- Pre-import validation that lists all missing layers
- User can choose: add to palette, skip import, or cancel
- Clear error messages with layer IDs

---

## 5. Recommendations Summary

### Priority 1 (Must Fix Before Phase 1)

1. **[Critical]** Fix Primary Key contradiction (Section 1.1)
2. **[Critical]** Document GraphEdit migration strategy (Section 2.1)
3. **[Critical]** Define execution_state to status mapping (Section 1.2)
4. **[High]** Add foreign key strategy for edges (Section 1.3)
5. **[High]** Define ID collision prevention (Section 2.2)

### Priority 2 (Must Fix Before Phase 2)

6. **[High]** Add annotation format handling (Section 2.3)
7. **[High]** Define index strategy (Section 2.4)
8. **[Medium]** Clarify concurrent access during migration (Section 3.2)
9. **[Medium]** Define test data volume requirements (Section 3.3)

### Priority 3 (Should Address Before Implementation)

10. **[Medium]** Define backward compatibility duration (Section 3.1)
11. **[Medium]** Add large graph memory handling (Section 2.5)
12. **[Medium]** Document rollback procedures per phase (Section 3.4)
13. **[Low]** Define foreign key cascade strategy (Section 3.5)
14. **[Low]** Clarify status terminology (Section 4.1)

---

## 6. Suggested Plan Updates

### Add Section 3.1.4: Index Strategy

```markdown
#### 3.1.4 Index Strategy

Critical indexes for performance:
- graph_data: project_id, dag_node_id, (project_id, source_type)
- graph_data_nodes: graph_data_id, (graph_data_id, external_id), layer, belongs_to
- graph_data_edges: graph_data_id, (graph_data_id, external_id), source, target, (source, target), layer

[Full DDL provided in Section 2.4 above]
```

### Update Section 4.2: Add GraphEdit Migration

```markdown
### 4.2 Phase 2: Data Migration (Batch)

**Tasks**:
...
7. **Migrate graph_edits table**:
   - Update graph_id references (apply offset if using Option C for ID collision)
   - Validate all target_id references match external_id in new tables
   - Test edit replay on migrated data
8. Add validation queries...
```

### Add Section 4.7: Rollback Procedures

```markdown
### 4.7 Rollback Procedures

**Phase 1**: Drop new tables, rollback migration
**Phase 2**: Restore from backup, verify old tables intact
**Phase 3**: Stop writes to new tables, resume old pipeline (requires dual-write)
**Phase 4-5**: Rollback GraphQL changes, frontend uses old types
**Phase 6**: Point of no return (old tables dropped)

Each phase requires specific rollback script documented before execution.
```

---

## 7. Conclusion

The updated plan significantly improves clarity on surrogate PKs, strict layer validation, and canonical status. However, **critical ambiguities remain** that must be resolved before implementation:

**Showstoppers**:
- Primary key contradiction in schema
- GraphEdit system compatibility not addressed
- Execution state mapping undefined
- ID collision strategy vague

**High Risk**:
- Missing index strategy will cause performance issues
- Annotation migration edge cases could corrupt data
- No rollback procedures defined

**Recommended Next Steps**:
1. Address Priority 1 issues (showstoppers)
2. Update plan with fixes from this review
3. Create detailed Phase 1 implementation spec
4. Review Phase 1 spec before coding begins

Once Priority 1 and 2 issues are resolved, the plan will be ready for implementation.
