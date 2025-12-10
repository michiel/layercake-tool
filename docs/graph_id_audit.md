# Legacy Graph/Dataset ID Usage Audit
**Purpose**: Track all references to legacy `graphs` and `data_sets` tables to guide Phase 3-6 migration
**Created**: 2025-12-10
**Last Updated**: 2025-12-10

---

## Overview

This document tracks all remaining references to the legacy graph and dataset infrastructure that need to be migrated to the unified `graph_data` model. Use this as a checklist for Phase 3-6 cleanup.

---

## Summary Statistics

| Category | Count | Priority |
|----------|-------|----------|
| GraphService references | 47+ | 🔴 High |
| DataSetService references | 47+ | 🔴 High |
| graph_to_data_set conversion | 2 | 🔴 High |
| graphs::Entity usage | 20+ | 🟡 Medium |
| data_sets::Entity usage | 15+ | 🟡 Medium |
| graph_id FK references | 10+ | 🟡 Medium |
| GraphQL mutations (legacy) | 8 | 🟢 Low |
| GraphQL queries (legacy) | 6 | 🟢 Low |

**Total Estimated Cleanup Effort**: 8-10 days

---

## 1. Entity Relationships (Database Layer)

### Legacy Entities with graph_id FKs

**File**: `layercake-core/src/database/entities/graph_edits.rs`
```rust
pub graph_id: i32  // ⚠️ Already offset by +1M in migration
belongs_to = "super::graphs::Entity"
```
- **Status**: ⚠️ Partially migrated (IDs offset, but still references graphs table)
- **Action**: Update to reference graph_data table once edit replay is implemented
- **Blocker**: Edit replay for graph_data not yet implemented

**File**: `layercake-core/src/database/entities/graph_nodes.rs`
```rust
pub graph_id: i32
belongs_to = "super::graphs::Entity"
```
- **Status**: ❌ Legacy table, migrated to graph_data_nodes but old table still active
- **Action**: Phase 6 cleanup - drop table and remove entity

**File**: `layercake-core/src/database/entities/graph_edges.rs`
```rust
pub graph_id: i32
belongs_to = "super::graphs::Entity"
```
- **Status**: ❌ Legacy table, migrated to graph_data_edges but old table still active
- **Action**: Phase 6 cleanup - drop table and remove entity

**File**: `layercake-core/src/database/entities/graph_layers.rs`
```rust
pub graph_id: i32
belongs_to = "super::graphs::Entity"
```
- **Status**: ❌ Legacy table, will be removed entirely in Phase 5
- **Action**: Phase 5 - remove layer storage from graph-specific tables

### Legacy Entities with dataset_id FKs

**File**: `layercake-core/src/database/entities/project_layers.rs`
```rust
belongs_to = "super::data_sets::Entity"
source_dataset_id: Option<i32>
```
- **Status**: ⚠️ Should reference graph_data instead
- **Action**: Update FK to graph_data.id, keep for traceability

**File**: `layercake-core/src/database/entities/projects.rs`
```rust
has_many = "super::data_sets::Entity"
```
- **Status**: ❌ Should have has_many = "graph_data::Entity"
- **Action**: Update relationship in Phase 6

---

## 2. Service Layer References

### GraphService Usage

**High-impact files** (require refactoring to use GraphDataService):

1. **layercake-core/src/services/graph_service.rs**
   - Full service file (~500+ LOC)
   - **Action**: Merge relevant logic into GraphDataService
   - **Priority**: 🔴 High (Phase 3)

2. **layercake-core/src/graphql/mutations/graph.rs**
   ```rust
   let graph_service = GraphService::new(context.db.clone());
   ```
   - **Action**: Replace with GraphDataService
   - **Priority**: 🔴 High (Phase 4)

3. **layercake-core/src/graphql/mutations/layer.rs**
   ```rust
   GraphService::upsert_project_layer
   GraphService::delete_project_layer
   GraphService::set_layer_dataset_enabled
   GraphService::reset_project_layers
   ```
   - **Action**: Move to LayerPaletteService
   - **Priority**: 🔴 High (Phase 5)

4. **layercake-core/src/graphql/mutations/graph_edit.rs**
   ```rust
   async fn replay_graph_edits(&self, ctx: &Context<'_>, graph_id: i32)
   async fn clear_graph_edits(&self, ctx: &Context<'_>, graph_id: i32)
   ```
   - **Action**: Implement for graph_data
   - **Priority**: 🔴 High (Phase 3)

### DataSetService Usage

**High-impact files**:

1. **layercake-core/src/services/data_set_service.rs**
   - Full service file (~800+ LOC)
   - **Action**: Merge relevant logic into GraphDataService
   - **Priority**: 🔴 High (Phase 3-4)

2. **layercake-core/src/graphql/queries/mod.rs**
   ```rust
   DataSetService::get_graph_summary
   DataSetService::get_graph_page
   data_sets::Entity::find_by_id
   ```
   - **Action**: Replace with GraphDataService equivalents
   - **Priority**: 🔴 High (Phase 4)

3. **layercake-core/src/services/library_item_service.rs**
   ```rust
   ) -> Result<data_sets::Model>
   ) -> Result<Vec<data_sets::Model>>
   ```
   - **Action**: Change return types to graph_data::Model
   - **Priority**: 🟡 Medium (Phase 4)

---

## 3. Pipeline Layer References

### graph_to_data_set Conversion

**File**: `layercake-core/src/pipeline/graph_builder.rs`
```rust
async fn graph_to_data_set(&self, graph: &graphs::Model) -> Result<data_sets::Model>
```
- **Location**: Lines ~350-400 (estimated)
- **Usage**: Called when chaining computed graphs
- **Action**: REMOVE - GraphDataBuilder eliminates need for conversion
- **Priority**: 🔴 High (Phase 3)
- **Blocker**: Must complete GraphDataBuilder integration first

### Legacy GraphBuilder Usage

**File**: `layercake-core/src/pipeline/graph_builder.rs`
```rust
fn compute_data_set_hash(&self, data_sets: &[data_sets::Model])
async fn build_graph(...) // Uses graphs::Entity
```
- **Action**: Gradually migrate all DAG node types to use GraphDataBuilder
- **Priority**: 🔴 High (Phase 3)
- **Status**: Currently dual-path (legacy + new)

**File**: `layercake-core/src/pipeline/merge_builder.rs`
```rust
cache: &mut HashMap<i32, data_sets::Model>
DataSet(data_sets::Model)
```
- **Action**: Update to use graph_data::Model
- **Priority**: 🟡 Medium (Phase 3)

**File**: `layercake-core/src/pipeline/dag_executor.rs`
- Currently uses both GraphBuilder and GraphDataBuilder
- **Action**: Migrate all node types to GraphDataBuilder
- **Priority**: 🔴 High (Phase 3)

---

## 4. GraphQL Layer References

### Mutations Using Legacy Models

1. **plan_dag.rs**
   ```rust
   graphs::Entity::find()
   graphs::Entity::delete_many()
   graphs::Entity::update()
   ```
   - **Action**: Update to graph_data::Entity
   - **Priority**: 🟡 Medium (Phase 4)

2. **graph_edit.rs**
   ```rust
   replay_graph_edits(graph_id: i32)
   clear_graph_edits(graph_id: i32)
   ```
   - **Action**: Support both legacy graph_id and new graph_data_id during transition
   - **Priority**: 🔴 High (Phase 3)

### Queries Using Legacy Models

**File**: `layercake-core/src/graphql/queries/mod.rs`
```rust
data_sets::Entity::find_by_id(id)  // 3 occurrences
```
- **Action**: Create facade that queries graph_data with source_type filter
- **Priority**: 🟡 Medium (Phase 4)

### Type Definitions

**Files to update in Phase 4**:
- `layercake-core/src/graphql/types/data_set.rs` (facade)
- `layercake-core/src/graphql/types/graph.rs` (facade)

---

## 5. Migration Path & Priority

### Phase 3 (Current Focus - 2-3 days)

**Critical Path Items**:
1. ✅ GraphDataBuilder basic functionality (DONE)
2. ✅ Edit replay for graph_data (DONE 2025-12-10)
3. ✅ Remove graph_to_data_set conversion (DONE 2025-12-10)
4. ❌ Migrate DagExecutor to prefer GraphDataBuilder (OPTIONAL - dual path works)

**Files Modified** (2025-12-10):
- [x] `services/graph_data_service.rs` - Added edit replay methods (replay_edits, clear_edits, update_edit_metadata, etc.)
- [x] `services/graph_data_edit_applicator.rs` - NEW: Applies edits to graph_data_nodes/edges
- [x] `tests/graph_data_builder_test.rs` - Added 3 comprehensive edit replay tests
- [x] `pipeline/graph_builder.rs` - Removed graph_to_data_set (90 lines), now fails with migration error for chaining
- [ ] `pipeline/dag_executor.rs` - Currently dual-path (uses graphDataIds when available, legacy otherwise)
- [ ] `graphql/mutations/graph_edit.rs` - Support graph_data (optional Phase 4)

### Phase 4 (GraphQL API - 3-4 days)

**Files to Create**:
- [ ] `graphql/types/graph_data.rs` - New unified type
- [ ] Update `graphql/types/data_set.rs` - Facade pattern
- [ ] Update `graphql/types/graph.rs` - Facade pattern

**Files to Modify**:
- [ ] `graphql/queries/mod.rs` - Use graph_data::Entity
- [ ] `graphql/mutations/*.rs` - Use GraphDataService

### Phase 5 (Remove Layer Storage - 2-3 days)

**Files to Modify**:
- [ ] Drop `graph_layers` table
- [ ] Drop `dataset_graph_layers` table
- [ ] `graphql/mutations/layer.rs` - Use LayerPaletteService only

### Phase 6 (Cleanup - 2-3 days)

**Files to Delete**:
- [ ] `database/entities/graphs.rs`
- [ ] `database/entities/graph_nodes.rs`
- [ ] `database/entities/graph_edges.rs`
- [ ] `database/entities/graph_layers.rs`
- [ ] `database/entities/data_sets.rs`
- [ ] `database/entities/dataset_graph_nodes.rs`
- [ ] `database/entities/dataset_graph_edges.rs`
- [ ] `database/entities/dataset_graph_layers.rs`
- [ ] `services/graph_service.rs` (merge into GraphDataService)
- [ ] `services/data_set_service.rs` (merge into GraphDataService)

**Migrations to Add**:
- [ ] Drop old tables migration
- [ ] Update FK constraints migration

---

## 6. Testing Requirements

### Integration Tests Needed

**Phase 3**:
- [ ] GraphDataBuilder full DAG execution test
- [ ] Edit replay with graph_data
- [ ] Change detection hash comparison

**Phase 4**:
- [ ] GraphQL backwards compatibility (DataSet → GraphData facade)
- [ ] GraphQL backwards compatibility (Graph → GraphData facade)
- [ ] Unified GraphData queries

**Phase 5**:
- [ ] Layer extraction from imports
- [ ] Layer validation strict mode
- [ ] Export with project palette layers

---

## 7. Risk Mitigation

### High-Risk Changes

1. **Edit Replay Migration** 🔴
   - **Risk**: Existing edits reference legacy graph IDs
   - **Mitigation**: graph_edits already offset (+1M) in migration
   - **Validation**: Test replay on migrated graphs

2. **GraphQL Breaking Changes** 🟡
   - **Risk**: Frontend depends on DataSet/Graph types
   - **Mitigation**: Use facade pattern for 2-3 months
   - **Validation**: Parallel testing with old/new queries

3. **Data Loss During Cleanup** 🟡
   - **Risk**: Dropping old tables before validation
   - **Mitigation**: Keep backup, validate count equality
   - **Validation**: Run validation queries from migration

---

## 8. Progress Tracking

### Completed ✅
- [x] Create unified schema (graph_data, nodes, edges)
- [x] Data migration with validation
- [x] GraphDataService scaffolding
- [x] LayerPaletteService scaffolding
- [x] GraphDataBuilder basic implementation
- [x] Migration bug fixes (created_at, JSON handling)

### In Progress 🟡
- [ ] Complete GraphDataService API
- [ ] Edit replay for graph_data
- [ ] Integration testing

### Not Started ❌
- [ ] GraphQL unified API (Phase 4)
- [ ] Remove layer storage (Phase 5)
- [ ] Drop old tables (Phase 6)

---

## 9. Quick Reference Commands

### Find Legacy References
```bash
# Find all GraphService references
grep -r "GraphService" --include="*.rs" layercake-core/src | wc -l

# Find all DataSetService references
grep -r "DataSetService" --include="*.rs" layercake-core/src | wc -l

# Find graph_to_data_set conversion
grep -r "graph_to_data_set" --include="*.rs" layercake-core/src

# Find graphs::Entity usage
grep -r "graphs::Entity" --include="*.rs" layercake-core/src

# Find data_sets::Entity usage
grep -r "data_sets::Entity" --include="*.rs" layercake-core/src
```

### Validate Migration
```sql
-- Check row counts match
SELECT
    (SELECT COUNT(*) FROM data_sets) as old_datasets,
    (SELECT COUNT(*) FROM graph_data WHERE source_type = 'dataset') as new_datasets,
    (SELECT COUNT(*) FROM graphs) as old_graphs,
    (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed') as new_graphs;

-- Check validation table
SELECT * FROM graph_data_migration_validation ORDER BY check_name;
```

---

## 10. Next Review

**Schedule**: After Phase 3 completion
**Reviewer**: Developer completing Phase 3
**Update**: This document should be updated as references are removed

---

**Document Owner**: Development Team
**Last Audit**: 2025-12-10
**Next Audit**: After Phase 3 completion
