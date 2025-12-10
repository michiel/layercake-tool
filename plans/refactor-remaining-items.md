# Refactoring Plan: Remaining Open Items

**Status**: Issues 1-7 from review have been fixed in the plan
**Date**: 2025-12-10

The following items still require decisions or clarification before implementation:

---

## 8. Large Graph Memory Risk (Medium Priority)

**Issue**: `GraphDataService::load_full()` loads entire graph into memory.

**Impact**:
- Graphs with 1M+ nodes could consume 2.5+ GB RAM
- Concurrent operations could exhaust memory
- No limits or pagination currently defined

**Questions**:
1. What is the expected maximum graph size to support?
2. Should we enforce hard limits on graph size?
3. Do we need streaming or pagination for large graphs?

**Options**:
- **Option A**: Document limit (e.g., "recommended max 100k nodes")
- **Option B**: Add max_nodes parameter to load_full() with error if exceeded
- **Option C**: Implement pagination/streaming for all load operations
- **Option D**: Add separate `load_summary()` that doesn't load nodes/edges

**Recommendation**: Start with Option A+B (document + enforce limit), add Option C if needed later.

**Required Decision**: Maximum supported graph size (nodes + edges)?

---

## 9. Concurrent Access During Migration (High Priority)

**Issue**: No strategy defined for system availability during migration.

**Impact**:
- User experience during migration
- Data consistency requirements
- Migration window constraints

**Questions**:
1. Can we take the system offline for migration?
2. If not, how do we handle reads/writes during migration?
3. What's the acceptable downtime window?

**Options**:

### Option A: Full Downtime
```
1. Put system in maintenance mode
2. Complete migration (estimated: 2-6 hours depending on data volume)
3. Bring system back online
```
- **Pros**: Simplest, safest, data consistent
- **Cons**: Downtime required

### Option B: Dual-Write
```
1. Phase 3: Write to both old and new tables
2. Gradually migrate data in background
3. Switch reads to new tables when complete
4. Phase 6: Stop writing to old tables
```
- **Pros**: No downtime
- **Cons**: Complex, requires careful synchronization, double storage

### Option C: Read-Only During Migration
```
1. Set system to read-only mode
2. Run migration (reads continue to work)
3. Bring system back online for writes
```
- **Pros**: Users can still view data
- **Cons**: Can't edit during migration

**Recommendation**: Start with Option A for initial migration. Consider Option B if downtime is unacceptable.

**Required Decision**: 
- Is downtime acceptable? If so, how long?
- If not, dual-write or other strategy?

---

## 10. Test Data Volume Requirements (High Priority)

**Issue**: "Production-like data" not defined.

**Impact**:
- Migration testing could miss edge cases
- Performance issues not caught before production
- Unclear success criteria

**Current State**: No specific requirements.

**Proposed Requirements**:

### Minimal Test Data (Smoke Test)
```
- 5 projects
- 25 graphs (15 datasets, 10 computed)
- 100-1,000 nodes/edges per graph
- 50 graph_edits
- 10 layers per project
- All status types represented
- All source_types represented
```
- Purpose: Verify migration logic works
- Runtime: ~1 minute

### Realistic Test Data (Integration Test)
```
- 50 projects
- 250 graphs (150 datasets, 100 computed)
- 1,000-10,000 nodes/edges per graph
- 500 graph_edits
- 20 layers per project
- Complex DAG chains (3-5 levels deep)
- All annotation formats (null/empty/json/text)
```
- Purpose: Verify performance acceptable
- Runtime: ~10 minutes

### Stress Test Data (Optional)
```
- 500 projects
- 2,500 graphs
- 50,000-100,000 nodes/edges in largest graphs
- 5,000 graph_edits
- Test specific edge cases:
  * Graph with 1M nodes (memory test)
  * 100 graphs in single DAG chain
  * 1000 layers in project palette
```
- Purpose: Find breaking points
- Runtime: ~1 hour

**Recommendation**: Require Minimal for Phase 2, Realistic for Phase 3, Stress optional.

**Required Decision**: Which test data tiers are mandatory before each phase?

---

## 11. Backward Compatibility Duration (Medium Priority)

**Issue**: How long do GraphQL facade types remain?

**Impact**:
- Frontend migration timeline
- API versioning strategy
- Support burden for dual APIs

**Questions**:
1. When can we remove `DataSet` and `Graph` facade types?
2. What's the deprecation schedule?
3. How do we communicate to API consumers?

**Options**:

### Option A: Remove After Frontend Migration
```
Timeline:
- Phase 4: Add facades with deprecation notices
- Phase 5-6: Complete
- Wait 2-3 months for frontend migration
- Phase 7: Remove facades (new phase)
```

### Option B: Keep Until v2.0
```
- Keep facades indefinitely in v1.x
- Remove in v2.0 (major version bump)
- Give 6-12 months notice
```

### Option C: Feature Flag
```
- Add feature flag to disable old types
- Default: enabled (backward compatible)
- Deprecate in release notes
- Remove in next major version
```

**Recommendation**: Option A - Remove after frontend fully migrated (2-3 months).

**Communication Strategy**:
1. Add deprecation warnings in GraphQL schema
2. Update API documentation
3. Announce in release notes
4. Provide migration guide for API consumers

**Required Decision**: Deprecation timeline and communication plan?

---

## 12. Rollback After Phase 4 (High Priority)

**Issue**: Can we rollback after GraphQL API changes?

**Impact**:
- Risk tolerance for later phases
- Production incident response
- Migration reversibility

**Problem**:
- Phase 3 writes only to new tables
- Phase 4 changes API to use new tables
- Rollback requires reconstructing old tables from new

**Current Plan**: "Keep old tables until Phase 6" but writing stops in Phase 3.

**Proposal**: Make Phase 3 dual-write:

```rust
// Phase 3: Dual-write period
impl GraphDataService {
    pub async fn create_computed(&self, ...) -> Result<GraphData> {
        // 1. Write to new graph_data tables
        let graph_data = self.insert_into_graph_data(...).await?;
        
        // 2. Also write to old graphs tables (for rollback)
        if cfg!(feature = "dual_write_rollback") {
            self.mirror_to_old_schema(&graph_data).await?;
        }
        
        Ok(graph_data)
    }
}
```

**Rollback Windows**:
- **Phase 1-2**: Full rollback possible (just drop new tables)
- **Phase 3**: Rollback possible if dual-write enabled
- **Phase 4-5**: Rollback requires effort (restore old GraphQL types)
- **Phase 6**: No rollback (old tables dropped) - Point of no return

**Recommendation**: 
1. Enable dual-write in Phase 3 as safety net
2. Disable after Phase 5 proves stable (1-2 weeks)
3. Phase 6 requires explicit approval after validation

**Required Decision**: 
- Implement dual-write in Phase 3? (Recommendation: Yes)
- How long to maintain dual-write? (Recommendation: 2 weeks)

---

## 13. Foreign Key Cascade Strategy (Low Priority)

**Issue**: Should `source_dataset_id` CASCADE or SET NULL on delete?

**Current Schema**:
```sql
source_dataset_id INTEGER,  -- No FK constraint defined
```

**Question**: If source dataset deleted, what happens to computed graphs referencing it?

**Options**:

### Option A: CASCADE (Delete computed graphs)
```sql
source_dataset_id INTEGER,
FOREIGN KEY (source_dataset_id) 
    REFERENCES graph_data(id) ON DELETE CASCADE
```
- **Pros**: Clean data, no orphaned references
- **Cons**: Deleting dataset could cascade-delete many graphs

### Option B: SET NULL (Preserve graphs, lose traceability)
```sql
source_dataset_id INTEGER,
FOREIGN KEY (source_dataset_id) 
    REFERENCES graph_data(id) ON DELETE SET NULL
```
- **Pros**: Graphs survive even if source deleted
- **Cons**: Lose traceability, can't reconstruct provenance

### Option C: RESTRICT (Prevent deletion)
```sql
source_dataset_id INTEGER,
FOREIGN KEY (source_dataset_id) 
    REFERENCES graph_data(id) ON DELETE RESTRICT
```
- **Pros**: Prevents accidental deletion of used datasets
- **Cons**: Have to manually delete all dependent graphs first

### Option D: No FK (Current)
- **Pros**: Flexible, no constraints
- **Cons**: No referential integrity, orphaned references possible

**Recommendation**: **Option B (SET NULL)** - Preserve computed graphs, accept loss of traceability.

**Rationale**:
- Computed graphs are valuable derived artifacts
- User may want to keep them even if source deleted
- Traceability is nice-to-have, not critical
- Can still track via metadata/annotations if needed

**Alternative**: Option C if we want to prevent accidental deletion (more conservative).

**Required Decision**: Which cascade strategy for source_dataset_id?

---

## Summary of Required Decisions

### High Priority (Before Phase 2)
1. ✅ **Issue 9**: Migration downtime strategy - **DECISION NEEDED**
2. ✅ **Issue 10**: Test data volume requirements - **DECISION NEEDED**
3. ✅ **Issue 12**: Dual-write for rollback safety - **DECISION NEEDED**

### Medium Priority (Before Phase 3)
4. **Issue 8**: Maximum supported graph size - **DECISION NEEDED**
5. **Issue 11**: Deprecation timeline for facades - **DECISION NEEDED**

### Low Priority (Can defer)
6. **Issue 13**: Foreign key cascade strategy - **DECISION NEEDED**

---

## Recommendations Summary

| Issue | Recommendation | Rationale |
|-------|---------------|-----------|
| 8. Memory | Document limit + enforce max 100k nodes | Balances usability with safety |
| 9. Migration | Full downtime (2-6 hours) | Simplest, safest for initial migration |
| 10. Test Data | Minimal (Phase 2), Realistic (Phase 3) | Adequate coverage without over-testing |
| 11. Facades | Remove after 2-3 months | Allows frontend migration time |
| 12. Rollback | Dual-write in Phase 3, 2 weeks | Safety net for production |
| 13. FK Cascade | SET NULL on source_dataset_id | Preserve graphs, accept lost traceability |

---

## Next Steps

1. Review these items with stakeholders
2. Make decisions on each item
3. Update refactoring plan with decisions
4. Proceed with Phase 1 implementation

Once all decisions made, append them to main refactoring plan and remove this file.
