# Phase 5 & 6 Cleanup Plan: Layer Storage & Legacy Code Removal

**Created**: 2025-12-10
**Status**: Planning

---

## Overview

This document outlines the plan for Phase 5 (Remove Layer Storage) and Phase 6 (Cleanup) of the dataset/graph refactoring.

**Goals**:
- Phase 5: Move layers from graph-specific tables to project palette exclusively
- Phase 6: Remove legacy tables, services, and code

**Estimated Effort**: 4-6 days combined

---

## Phase 5: Remove Layer Storage from graph_data

### Current State

Layers are currently stored in multiple places:
1. `graph_layers` table (legacy, for computed graphs)
2. `dataset_graph_layers` table (legacy, for datasets)
3. `project_layers` table (project palette - authoritative)

**Problem**: Redundant storage, synchronization issues, unclear ownership.

### Target State

Layers stored **only** in `project_layers` table:
- Import: Extract layers from files, add to project palette
- Rendering: Always use project palette
- Export: Include layers from project palette
- Validation: Check layer references against project palette

### Tasks

#### 5.1 Update Import Logic ✅
- [x] GraphDataBuilder already doesn't create graph_layers records
- [x] Legacy GraphBuilder creates graph_layers (will be removed in Phase 6)
- [ ] Ensure dataset import extracts layers to project palette (check DataSetService)

#### 5.2 Update Export Logic
- [ ] Check current export implementation
- [ ] Ensure layers exported from project_layers table
- [ ] Test export/import roundtrip

#### 5.3 Update Validation
- [ ] Validate node/edge layer references against project_layers
- [ ] Fail on missing layers (no auto-create)

#### 5.4 Remove Layer Queries from GraphQL
- [ ] Graph.graphLayers resolver can query project.layers instead
- [ ] Or deprecate and remove (prefer project.layers)

#### 5.5 Documentation
- [ ] Update migration guide about layer management
- [ ] Document that layers are project-scoped, not graph-scoped

---

## Phase 6: Cleanup Legacy Code

### 6.1 Database Tables to Drop

**Legacy Graph Tables**:
- `graphs` - migrated to graph_data (source_type="computed")
- `graph_nodes` - migrated to graph_data_nodes
- `graph_edges` - migrated to graph_data_edges
- `graph_layers` - removed (use project_layers)

**Legacy Dataset Tables**:
- `data_sets` - migrated to graph_data (source_type="dataset")
- `dataset_graph_nodes` - migrated to graph_data_nodes
- `dataset_graph_edges` - migrated to graph_data_edges
- `dataset_graph_layers` - removed (use project_layers)

**Total**: 8 tables to drop

### 6.2 Services to Remove/Merge

**Remove Entirely**:
- `GraphService` (~500 LOC) - functionality merged into GraphDataService
- `DataSetService` (~800 LOC) - functionality merged into GraphDataService

**Merge into GraphDataService**:
- Graph/DataSet CRUD operations (already done)
- Validation logic (check if needed)
- Export logic (check if needed)

### 6.3 Code References to Update

Found 20 files with graph_layers/dataset_graph_layers references:
1. graphql/types/graph.rs - Graph.graphLayers resolver
2. graphql/queries/mod.rs - queries using legacy tables
3. pipeline/graph_builder.rs - creates graph_layers (will be removed)
4. database/entities/mod.rs - exports legacy entities
5. services/data_set_service.rs - creates dataset_graph_layers
6. services/graph_service.rs - creates graph_layers
7-20. Various other files

**Strategy**:
- Remove references systematically
- Update queries to use project_layers
- Remove layer creation code

### 6.4 Migration Steps

#### Step 1: Create Migration SQL
```sql
-- Validate no data loss
SELECT COUNT(*) FROM graphs;
SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed';
-- Should match

-- Drop legacy tables
DROP TABLE IF EXISTS graph_layers;
DROP TABLE IF EXISTS dataset_graph_layers;
DROP TABLE IF EXISTS graph_edges;
DROP TABLE IF EXISTS graph_nodes;
DROP TABLE IF EXISTS graphs;
DROP TABLE IF EXISTS dataset_graph_edges;
DROP TABLE IF EXISTS dataset_graph_nodes;
DROP TABLE IF EXISTS data_sets;
```

#### Step 2: Remove Entity Files
- Delete `database/entities/graphs.rs`
- Delete `database/entities/graph_nodes.rs`
- Delete `database/entities/graph_edges.rs`
- Delete `database/entities/graph_layers.rs`
- Delete `database/entities/data_sets.rs`
- Delete `database/entities/dataset_graph_nodes.rs`
- Delete `database/entities/dataset_graph_edges.rs`
- Delete `database/entities/dataset_graph_layers.rs`
- Update `database/entities/mod.rs`

#### Step 3: Remove Services
- Delete `services/graph_service.rs`
- Delete `services/data_set_service.rs`
- Check for any remaining logic to preserve
- Update `services/mod.rs`

#### Step 4: Update GraphQL
- Keep DataSet/Graph facade types (for backward compatibility)
- Update resolvers to use graph_data internally
- Remove direct queries to legacy tables

#### Step 5: Update Pipeline
- Remove GraphBuilder (use GraphDataBuilder only)
- Remove graph_to_data_set conversion (already done)
- Update DagExecutor to always use GraphDataBuilder

### 6.5 Files to Modify

**High Priority** (breaks compilation if tables dropped):
- `database/entities/mod.rs`
- `services/mod.rs`
- `graphql/types/graph.rs` (Graph.graphLayers resolver)
- `pipeline/graph_builder.rs` (remove entirely)
- Any direct table queries

**Medium Priority** (references to clean up):
- `graphql/mutations/graph.rs`
- `graphql/mutations/plan_dag.rs`
- `graphql/queries/mod.rs`
- `app_context/graph_operations.rs`

**Low Priority** (optional cleanup):
- Test files
- Documentation updates

---

## Risk Assessment

### High Risk
1. **Data Loss**: Dropping tables before validation
   - Mitigation: Validate counts, backup database
   - Rollback: Keep migration reversible

2. **Breaking Frontend**: Queries fail after table removal
   - Mitigation: Facades keep working with graph_data
   - Testing: Test all GraphQL queries

### Medium Risk
1. **Missing Functionality**: Some GraphService/DataSetService logic needed
   - Mitigation: Audit services before removal
   - Preserve essential logic in GraphDataService

2. **Layer References Break**: Missing layers after removal
   - Mitigation: Ensure all layers in project palette
   - Validation: Check layer references

### Low Risk
1. **Performance**: graph_data queries slower than specialized tables
   - Mitigation: Already using indexes
   - Monitor: Track query performance

---

## Testing Strategy

### Pre-Migration Validation
- [ ] Count rows in all legacy tables
- [ ] Validate graph_data has all migrated data
- [ ] Check layer references (all exist in project_layers)
- [ ] Backup database

### Post-Migration Validation
- [ ] All tests pass
- [ ] GraphQL queries work (using facades)
- [ ] DAG execution works
- [ ] Import/export works
- [ ] Layer rendering works

### Integration Tests
- [ ] Create dataset, verify layers in project palette
- [ ] Build computed graph, verify no graph_layers created
- [ ] Edit replay works
- [ ] Export includes layers from project palette

---

## Rollback Plan

If issues found after migration:

1. **Before dropping tables**: Just revert code changes
2. **After dropping tables**: Restore from backup
3. **Emergency**: Re-run original migration from data_sets/graphs to graph_data

---

## Timeline

**Phase 5** (1-2 days):
- Day 1: Update import/export logic, layer validation
- Day 2: Testing, documentation

**Phase 6** (3-4 days):
- Day 1: Create migration, remove entity files
- Day 2: Remove services, update references
- Day 3: Update GraphQL, pipeline cleanup
- Day 4: Testing, documentation, validation

**Total**: 4-6 days

---

## Success Criteria

### Phase 5
- [ ] Layers only stored in project_layers
- [ ] Import extracts layers to project palette
- [ ] Export includes layers from project palette
- [ ] Validation checks against project palette
- [ ] No graph_layers/dataset_graph_layers creation

### Phase 6
- [ ] Legacy tables dropped
- [ ] Legacy services removed
- [ ] All tests pass
- [ ] GraphQL queries work via facades
- [ ] DAG execution uses GraphDataBuilder only
- [ ] No references to legacy code
- [ ] Documentation updated

---

## Next Steps

1. Review this plan
2. Start with Phase 5 (safer, no table drops)
3. Validate thoroughly before Phase 6
4. Execute Phase 6 with caution (irreversible table drops)

---

**Owner**: Backend Team
**Last Updated**: 2025-12-10
