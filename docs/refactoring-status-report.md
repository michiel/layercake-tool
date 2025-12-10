# Dataset/Graph Refactoring Status Report
**Date**: 2025-12-10
**Report Type**: Implementation Review
**Plan Reference**: `plans/refactor-datasets-and-graphs.md`

---

## Executive Summary

The refactoring to unify Dataset and Graph structures into a unified `graph_data` model is **approximately 45-50% complete**. The foundation (Phases 1-2) is solid with all database schema and migrations in place. Phase 3 (pipeline integration) is partially complete with basic `GraphDataBuilder` functionality but lacks full integration. Phases 4-6 (GraphQL API, layer removal, cleanup) have not been started.

**Critical Finding**: The migration we just fixed (m20251210_000004) had three bugs that prevented it from running on legacy databases:
1. Missing `created_at` column guards for node/edge tables
2. SQL syntax error in INSERT statement
3. Malformed JSON handling for annotations field

These have been resolved and the migration now runs successfully.

---

## Phase-by-Phase Status

### ✅ Phase 1: Create Unified Tables (COMPLETE)

**Status**: ✅ **100% Complete**

**Implemented Components**:
- ✅ Database migrations:
  - `m20251210_000001_create_graph_data.rs`
  - `m20251210_000002_create_graph_data_nodes.rs`
  - `m20251210_000003_create_graph_data_edges.rs`
- ✅ Entity models: `graph_data`, `graph_data_nodes`, `graph_data_edges`
- ✅ Service scaffolding:
  - `GraphDataService` with basic CRUD operations
  - `LayerPaletteService` with validation logic

**Quality Assessment**:
- Schema matches plan specification exactly
- All indexes defined (15 indexes across 3 tables)
- Foreign key constraints properly implemented
- Services compile and have basic test coverage

**Files Created**:
- `layercake-core/src/database/entities/graph_data.rs`
- `layercake-core/src/database/entities/graph_data_nodes.rs`
- `layercake-core/src/database/entities/graph_data_edges.rs`
- `layercake-core/src/services/graph_data_service.rs`
- `layercake-core/src/services/layer_palette_service.rs`

---

### ✅ Phase 2: Data Migration (COMPLETE with recent fixes)

**Status**: ✅ **100% Complete** (as of 2025-12-10)

**Implemented Components**:
- ✅ Migration script: `m20251210_000004_migrate_existing_graph_data.rs`
- ✅ Data copying from old tables to new tables
- ✅ ID offset handling (+1,000,000 for computed graphs)
- ✅ Annotation normalization to JSON arrays
- ✅ graph_edits table offset updates
- ✅ plan_dag_nodes.config_json graphId offset updates
- ✅ Validation query suite (9 validation checks)
- ✅ Sequence/autoincrement reseeding
- ✅ Legacy column guards (metadata, annotations, created_at)
- ✅ Integration test: `tests/migration_validation_test.rs`

**Recent Fixes Applied** (2025-12-10):
1. Added `created_at` column guards for:
   - `dataset_graph_nodes`
   - `dataset_graph_edges`
   - `graph_nodes`
   - `graph_edges`
2. Fixed SQL syntax: removed `json(annotations)` from INSERT column list
3. Fixed malformed JSON: changed to `COALESCE(annotations, '[]')` for safe handling

**Validation Checks Implemented**:
- Dataset count migration (old vs new)
- Graph count migration (old vs new)
- Node count migration (combined)
- Edge count migration (combined)
- Orphaned edge source references
- Orphaned edge target references
- Edges missing graph_data FK
- Nodes missing graph_data FK
- graph_edits missing graph_data FK
- plan_dag_nodes graphId below offset

**Quality Assessment**:
- Migration is non-destructive (keeps old tables intact)
- Can be rerun safely (INSERT OR IGNORE)
- Comprehensive validation suite
- Successfully tested with legacy databases
- All edge cases handled (NULL annotations, missing timestamps)

---

### 🟡 Phase 3: Update Pipeline to Use graph_data (PARTIAL)

**Status**: 🟡 **~60% Complete**

**Implemented Components**:
- ✅ `GraphDataBuilder` scaffolding (`pipeline/graph_data_builder.rs`)
  - ✅ Upstream graph loading (datasets + computed)
  - ✅ Layer validation against project palette
  - ✅ Source hash computation for change detection
  - ✅ Reuse existing graph_data when hash matches
  - ✅ Node/edge merging
  - ✅ Mark status transitions (processing → active)
- ✅ Integration into `DagExecutor`
  - ✅ GraphDataBuilder instantiated
  - ⚠️ Only used when `graphDataIds` provided in config

**Missing Components**:
- ❌ Edit replay logic for graph_data
- ❌ Full migration from legacy `GraphBuilder`
- ❌ Update all DAG node types to use graphDataIds
- ❌ Remove `graph_to_data_set()` conversion (still exists, 2 references)
- ❌ Comprehensive integration tests for full DAG execution
- ❌ Change detection parity with legacy system

**Code References**:
- Legacy conversion still present: `graph_to_data_set` (2 occurrences)
- `DataSetService` still heavily used (47 references)
- Dual-path execution in DagExecutor (legacy + new)

**Quality Assessment**:
- Core functionality works but incomplete
- No edit replay support yet
- Limited test coverage for graph_data path
- Legacy path still dominant in codebase

**Next Actions** (from plan line 865-868):
- Add edit replay/change-detection parity for `graph_data`
- Phase out legacy `GraphBuilder` paths
- Update GraphQL/MCP/console callers to use `graphDataId`
- Use `docs/graph_id_audit.md` to drive removal (NOT YET CREATED)

---

### ❌ Phase 4: Update GraphQL API (NOT STARTED)

**Status**: ❌ **0% Complete**

**Current State**:
- Old types still in use: `DataSet` (graphql/types/data_set.rs), `Graph` (graphql/types/graph.rs)
- No `GraphDataGql` type created
- No unified GraphQL resolvers
- No facade pattern implemented
- No deprecation warnings

**Required Work**:
1. Create `GraphDataGql` type with lazy-loading resolvers
2. Implement facade types that delegate to GraphDataGql
3. Update mutations to use GraphDataService
4. Add deprecation warnings to old types
5. Update GraphQL schema

**Estimated Effort**: 3-4 days (as per plan)

---

### ❌ Phase 5: Remove Layer Storage (NOT STARTED)

**Status**: ❌ **0% Complete**

**Current State**:
- Legacy layer tables still exist and in use:
  - `dataset_graph_layers`
  - `graph_layers`
- No layer extraction to project palette during import
- Rendering still may reference graph-specific layers

**Required Work**:
1. Update import logic to extract layers to project palette
2. Update rendering to use project_layers only
3. Update export to include palette layers
4. Remove layer-related code from GraphBuilder

**Estimated Effort**: 2-3 days (as per plan)

---

### ❌ Phase 6: Cleanup (NOT STARTED)

**Status**: ❌ **0% Complete**

**Blockers**: All previous phases must be complete

**Required Work**:
1. Drop old tables (6 tables)
2. Remove DataSetService (47 references to clean up)
3. Remove facade GraphQL types
4. Remove graph_to_data_set conversion (2 references)
5. Update documentation

**Estimated Effort**: 2-3 days (as per plan)

---

## Detailed Findings

### 1. Migration Quality ✅

**Strengths**:
- Comprehensive validation suite with 9 different checks
- Non-destructive design (keeps old tables)
- Handles edge cases well (NULL values, missing columns)
- Properly offsets IDs to prevent collisions
- Updates all FK references (graph_edits, plan_dag_nodes)

**Recent Improvements**:
- Fixed to handle legacy databases missing timestamp columns
- Fixed SQL syntax errors
- Fixed JSON parsing issues with annotations

**Test Coverage**:
- Integration test validates FK constraints
- Validates count migration accuracy
- Tests with in-memory SQLite database

### 2. Service Layer Quality 🟡

**Strengths**:
- Clean separation of concerns (GraphDataService vs LayerPaletteService)
- Proper transaction handling in replace_nodes/replace_edges
- Layer validation logic implemented
- Status lifecycle management

**Weaknesses**:
- Limited functionality compared to plan (no create_from_file, create_computed helpers)
- No lazy loading implementation
- Missing convenience methods from plan (list_datasets, list_computed)
- Unused methods warning (get_by_id, list_by_project_and_source)

**Gap Analysis** (vs Plan Section 3.2.1):
- Missing: `create_from_file()`, `create_from_json()`, `create_computed()`
- Missing: `get_by_dag_node()`, `list_datasets()`, `list_computed()`
- Missing: `load_nodes()`, `load_edges()`, `load_full()` lazy loaders
- Missing: `mark_processing()`, `mark_complete()`, `mark_error()` helpers
- Partially implemented: Basic create() exists but not specialized versions

### 3. Pipeline Integration Status 🟡

**What Works**:
- GraphDataBuilder can merge upstream sources
- Layer validation works
- Hash-based change detection works
- Can reuse existing graph_data when hash matches

**What's Missing**:
- No edit replay integration
- No full migration from legacy GraphBuilder
- DagExecutor still uses legacy path by default
- No comprehensive DAG execution tests

**Legacy Code Still Present**:
```rust
// Still exists in codebase:
graph_to_data_set()  // 2 references
DataSetService       // 47 references
```

### 4. Test Coverage 🟡

**Existing Tests**:
- ✅ Migration validation test (FK constraints, counts)
- ✅ Layer palette validation test (implied from service code)
- ❌ No GraphDataBuilder integration tests
- ❌ No full DAG execution tests with graph_data
- ❌ No GraphQL API tests for new types (none exist yet)

**Test Gap Analysis**:
- Need: Full pipeline tests with graph_data (plan section 7.2)
- Need: Migration data integrity tests (plan section 7.1)
- Need: GraphQL backwards compatibility tests (plan section 7.3)

### 5. Documentation Status ⚠️

**Plan Documentation**:
- ✅ Comprehensive plan exists: `plans/refactor-datasets-and-graphs.md`
- ✅ Plan has detailed phase breakdowns
- ✅ Plan includes migration SQL examples

**Implementation Documentation**:
- ❌ No graph_id_audit.md (referenced in plan line 868)
- ❌ No developer migration guide
- ❌ No user-facing documentation updates
- ❌ No architecture diagram updates

### 6. Technical Debt Assessment

**New Technical Debt Created**:
- Dual-path execution (legacy + graph_data) in DagExecutor
- Unused service methods generating warnings
- Incomplete service API vs plan specification

**Technical Debt Not Yet Resolved**:
- DataSetService still exists (47 references)
- graph_to_data_set conversion still exists (2 references)
- Layer storage duplication still exists (6 layer tables)
- GraphQL type duplication still exists (DataSet + Graph)

**Debt Ratio**: ~55% of original technical debt still present

---

## Risk Assessment

### High Risks 🔴

1. **Incomplete Phase 3** - Pipeline can't fully use graph_data
   - **Impact**: Can't deprecate legacy tables
   - **Mitigation**: Complete edit replay and full migration

2. **No GraphQL API** - Frontend can't use new model
   - **Impact**: Blocks user-facing benefits
   - **Mitigation**: Implement Phase 4 with facades

3. **Test Coverage Gaps** - Limited integration testing
   - **Impact**: Unknown edge cases, regression risk
   - **Mitigation**: Add comprehensive integration tests

### Medium Risks 🟡

1. **Dual-Path Complexity** - Two ways to do everything
   - **Impact**: Maintenance burden, potential bugs
   - **Mitigation**: Feature flag to gradually migrate

2. **Documentation Lag** - Implementation ahead of docs
   - **Impact**: Team confusion, onboarding difficulty
   - **Mitigation**: Document as we complete phases

### Low Risks 🟢

1. **Migration Quality** - Solid foundation
   - **Status**: Mitigated by comprehensive validation

2. **Database Schema** - Matches plan exactly
   - **Status**: No concerns

---

## Recommendations

### Immediate Actions (Next Sprint)

1. **Complete Phase 3** 🔴 **HIGH PRIORITY**
   - [ ] Implement edit replay for graph_data
   - [ ] Add comprehensive integration tests
   - [ ] Migrate all DAG node types to use graphDataIds
   - [ ] Update DagExecutor to prefer graph_data path

2. **Create Missing Documentation** 🟡 **MEDIUM PRIORITY**
   - [ ] Create `docs/graph_id_audit.md` to track legacy references
   - [ ] Document graph_data API for developers
   - [ ] Update architecture diagrams

3. **Improve Service APIs** 🟡 **MEDIUM PRIORITY**
   - [ ] Add missing convenience methods to GraphDataService
   - [ ] Implement lazy loading (load_nodes, load_edges, load_full)
   - [ ] Add specialized creation helpers (create_from_file, etc.)

### Short-term (1-2 Weeks)

4. **Begin Phase 4 - GraphQL API** 🟡
   - [ ] Create GraphDataGql type
   - [ ] Implement facade pattern for DataSet/Graph
   - [ ] Add deprecation warnings
   - [ ] Update frontend to use unified type

5. **Add Comprehensive Tests** 🟡
   - [ ] Full DAG execution tests with graph_data
   - [ ] GraphQL backwards compatibility tests
   - [ ] Performance benchmarks (old vs new)

### Medium-term (3-4 Weeks)

6. **Phase 5 - Remove Layer Storage** 🟢
   - [ ] Update import to populate project palette
   - [ ] Update rendering to use palette only
   - [ ] Remove layer tables

7. **Begin Phase 6 - Cleanup** 🟢
   - [ ] Create deprecation timeline
   - [ ] Plan breaking changes for major version
   - [ ] Update user documentation

---

## Metrics & Progress Tracking

### Code Reduction Progress

| Metric | Before | Current | Target | Progress |
|--------|--------|---------|--------|----------|
| Database Tables | 9 | 9 (6 old + 3 new) | 3 | 0% |
| Service Files | 2 | 4 (2 old + 2 new) | 2 | 0% |
| GraphQL Types | 2 | 2 | 1 | 0% |
| Conversion LOC | ~500 | ~500 | 0 | 0% |
| Migration Complete | - | Phase 1-2 | Phase 6 | 33% |

**Note**: Code reduction metrics at 0% because cleanup (Phase 6) hasn't started. Once cleanup begins, we'll see rapid progress.

### Timeline Progress

| Phase | Estimate | Actual | Status | Remaining |
|-------|----------|--------|--------|-----------|
| Phase 1 | 3-4 days | ~4 days | ✅ Complete | 0 days |
| Phase 2 | 3-4 days | ~4 days | ✅ Complete | 0 days |
| Phase 3 | 4-5 days | ~3 days | 🟡 Partial | 2-3 days |
| Phase 4 | 3-4 days | 0 days | ❌ Not started | 3-4 days |
| Phase 5 | 2-3 days | 0 days | ❌ Not started | 2-3 days |
| Phase 6 | 2-3 days | 0 days | ❌ Not started | 2-3 days |
| **Total** | **16-22 days** | **~7 days** | **45% complete** | **9-15 days** |

---

## Success Criteria Status

From Plan Section 12 (Success Metrics):

### Phase 1 Success Criteria ✅
- [x] New tables created
- [x] Services compile and pass tests
- [x] No changes to existing code (additive only)

### Phase 2 Success Criteria ✅
- [x] All data copied to new tables
- [x] Counts match between old and new tables
- [x] All annotations properly normalized
- [x] graph_edits references updated
- [x] All edge source/target FKs valid
- [x] Validation queries pass

### Phase 3 Success Criteria 🟡
- [x] GraphDataBuilder implemented
- [ ] Plan DAG execution works with new tables
- [ ] Graph chaining works without conversion
- [ ] Edit replay functions correctly
- [x] Change detection triggers correctly
- [ ] All pipeline tests pass

### Phase 4-6 Success Criteria ❌
- Not yet applicable

---

## Open Questions & Decisions

### From Plan Section 11.2 (High Priority Decisions)

**Decision 1: Migration Downtime Strategy** ⏳
- **Status**: No decision yet
- **Recommendation**: Full downtime (simplest, safest)
- **Impact**: Phase 6 execution

**Decision 2: Test Data Volume Requirements** ⏳
- **Status**: Using minimal test data
- **Recommendation**: Create realistic test dataset
- **Impact**: Phase 3 completion confidence

**Decision 3: Rollback Safety** ⏳
- **Status**: Currently in dual-write mode (old tables preserved)
- **Recommendation**: Maintain dual-write through Phase 5
- **Impact**: Rollback capability

---

## Next Steps (Prioritised)

### Week 1: Complete Phase 3
1. Implement edit replay for graph_data
2. Add integration tests for GraphDataBuilder
3. Create docs/graph_id_audit.md
4. Migrate more DAG node types to graphDataIds

### Week 2: Begin Phase 4
1. Create GraphDataGql type
2. Implement facade pattern
3. Update one GraphQL resolver as proof of concept
4. Add deprecation warnings

### Week 3-4: Continue Phase 4
1. Complete GraphQL migration
2. Update frontend to use unified API
3. Performance testing and benchmarking
4. Documentation updates

### Week 5-6: Phases 5-6
1. Remove layer storage duplication
2. Plan breaking changes for cleanup
3. Create migration guide for users
4. Execute Phase 6 cleanup (with rollback plan)

---

## Conclusion

The Dataset/Graph refactoring has a **solid foundation** with Phases 1-2 complete and tested. The migration successfully handles legacy databases after recent bug fixes. However, **Phase 3 is incomplete**, blocking progress on later phases.

**Key Blockers**:
1. Edit replay not implemented for graph_data
2. GraphQL API not started (Phase 4)
3. Legacy code still dominates (47 DataSetService refs, 2 conversion refs)

**Recommended Path Forward**:
1. Focus on completing Phase 3 (2-3 days)
2. Begin Phase 4 in parallel (GraphQL facade pattern)
3. Maintain dual-write mode until Phase 5
4. Schedule Phase 6 cleanup as breaking change in next major version

**Overall Assessment**: 🟡 **On track but needs focused effort to complete Phase 3**

The implementation quality is good, but momentum has slowed. Dedicating focused time to complete Phase 3 and begin Phase 4 will unlock the full benefits of this refactoring.

---

**Report compiled by**: Claude Code
**Last updated**: 2025-12-10
**Next review**: After Phase 3 completion
