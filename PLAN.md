# Layercake Codebase Refactoring Plan

**Date:** 2025-10-10
**Status:** Draft
**Priority:** Medium

## Executive Summary

This document outlines identified code quality issues and provides a structured plan for refactoring the Layercake codebase. The analysis found several areas of dead code, duplication, and inconsistency that reduce maintainability.

## Key Findings

### 1. Dead Code - Database Entities ✅ COMPLETED

**Issue:** Obsolete entity files exist but are no longer used.

**Files Removed:**
- `/layercake-core/src/database/entities/nodes.rs` - ✅ Deleted
- `/layercake-core/src/database/entities/edges.rs` - ✅ Deleted
- `/layercake-core/src/graphql/types/node.rs` - ✅ Deleted (unused GraphQL type)
- `/layercake-core/src/graphql/types/edge.rs` - ✅ Deleted (unused GraphQL type)
- `/layercake-core/src/database/entities/layers.rs` - ✅ Kept (actively used by pipeline builders)

**Impact:**
- Removed ~185 lines of dead code
- Clearer codebase structure
- No build errors after removal

**Completed:** 2025-10-10
**Actual Effort:** 1 hour
**Commit:** 6a2986f5

---

### 2. Entity Naming Inconsistency ⚠️ HIGH PRIORITY

**Issue:** Two nearly identical entity files with different naming conventions.

**Files:**
- `/layercake-core/src/database/entities/datasources.rs` - 4 references
- `/layercake-core/src/database/entities/data_sources.rs` - 24 references

**Impact:**
- Confusion about which entity to use
- Potential for bugs if wrong one is imported
- Inconsistent naming patterns (snake_case vs camelCase)

**Action Items:**
1. Audit all 4 references to `datasources.rs`
2. Migrate them to use `data_sources.rs`
3. Remove `datasources.rs`
4. Standardize on `data_sources` naming pattern throughout codebase

**Estimated Effort:** 3 hours

---

### 3. Duplicate LayerData Struct ✅ COMPLETED

**Issue:** Identical `LayerData` struct defined in two separate files.

**Solution:**
- ✅ Created `/layercake-core/src/pipeline/types.rs` for shared pipeline types
- ✅ Extracted `LayerData` struct to types module
- ✅ Updated `merge_builder.rs` to import from shared module
- ✅ Updated `graph_builder.rs` to import from shared module
- ✅ Removed duplicate struct definitions

**Impact:**
- Eliminated code duplication
- Single source of truth for LayerData type
- Easier maintenance and evolution of type
- Foundation for extracting other shared types

**Completed:** 2025-10-10
**Actual Effort:** 30 minutes
**Commit:** 6e4eca4c

---

### 4. Dead Code - Frontend Dialog Component ✅ COMPLETED

**Issue:** Duplicate dialog file exists but is not used.

**Files Removed:**
- `/frontend/src/components/editors/PlanVisualEditor/dialogs/NodeConfigDialog.tsx` - ✅ Deleted (642 lines)
- `/frontend/src/components/editors/PlanVisualEditor/NodeConfigDialog.tsx` - ✅ Kept (119 lines, actively used)

**Impact:**
- Removed 642 lines of dead code
- Clearer directory structure
- No risk of incorrect imports

**Completed:** 2025-10-10
**Actual Effort:** 15 minutes
**Commit:** 120687b0

---

### 5. Multiple Collaboration/Presence Implementations 🟡 LOW PRIORITY

**Issue:** Three separate but related collaboration hooks exist.

**Files:**
- `/frontend/src/hooks/useCollaborationV2.ts` - 6.6KB
- `/frontend/src/hooks/usePresence.ts` - 4.9KB
- `/frontend/src/hooks/useWebSocketCollaboration.ts` - 8.3KB

**Impact:**
- Unclear which hook to use in different scenarios
- Potential logic duplication
- May have inconsistent behavior

**Action Items:**
1. Document the purpose and use case for each hook
2. Determine if they can be consolidated
3. If consolidation not feasible, add JSDoc comments explaining when to use each
4. Consider creating a facade hook that delegates to appropriate implementation

**Estimated Effort:** 4 hours (requires careful analysis)

---

### 6. GraphQL Type Inconsistencies 🟡 LOW PRIORITY

**Issue:** Inconsistent field naming between database and GraphQL (some use camelCase, some use snake_case with rename).

**Example from `/layercake-core/src/graphql/types/data_source.rs`:**
```rust
#[graphql(name = "projectId")]
pub project_id: i32,

#[graphql(name = "fileFormat")]
pub file_format: String,
```

**Impact:**
- Requires manual field name mapping
- Increases chance of typos
- Makes schema harder to generate/maintain

**Action Items:**
1. Establish consistent naming convention (prefer GraphQL-native camelCase throughout)
2. Consider using automatic field renaming via async-graphql config
3. Create a style guide for new GraphQL types

**Estimated Effort:** 8 hours (affects many files)

---

### 7. Service Layer Organization 🟡 LOW PRIORITY

**Issue:** Services are organized by feature but lack clear architectural boundaries.

**Current Structure:**
```
services/
├── auth_service.rs
├── data_source_service.rs
├── graph_service.rs
├── project_service.rs
├── plan_dag_service.rs
├── import_service.rs
├── export_service.rs
└── ...
```

**Observations:**
- Some services are very large (e.g., `data_source_service.rs` likely has many responsibilities)
- No clear separation between domain logic and data access
- Validation logic scattered across services

**Action Items:**
1. Audit service responsibilities
2. Consider extracting to layered architecture:
   - Domain layer (business rules)
   - Repository layer (data access)
   - Service layer (orchestration)
3. Extract validation logic to dedicated validation module
4. Document service boundaries and responsibilities

**Estimated Effort:** 16 hours (large refactoring)

---

### 8. Pipeline Builder Code Duplication 🔴 MEDIUM PRIORITY

**Issue:** `merge_builder.rs` and `graph_builder.rs` have similar layer extraction logic.

**Files:**
- `/layercake-core/src/pipeline/merge_builder.rs` - 25KB, 664 lines
- `/layercake-core/src/pipeline/graph_builder.rs` - 34KB, 872 lines

**Duplicate Patterns:**
- Layer extraction from JSON
- Layer extraction from database
- Layer insertion logic
- JSON structure validation

**Impact:**
- High maintenance cost
- Bug fixes must be applied twice
- Risk of logic divergence

**Action Items:**
1. Extract common layer operations to `/pipeline/layer_operations.rs`:
   - `extract_layers_from_json()`
   - `extract_layers_from_graph()`
   - `insert_layers_to_db()`
2. Extract common JSON parsing to `/pipeline/json_parser.rs`
3. Consider introducing a `LayerRepository` for database operations
4. Create integration tests to ensure behavior remains identical

**Estimated Effort:** 6 hours

---

### 9. Frontend YAML Conversion Duplication ✅ RESOLVED

**Status:** Already fixed in recent commits

**Previous Issue:** YAML conversion logic was duplicated between `App.tsx` and `PlanVisualEditor.tsx`

**Resolution:** Removed from `PlanVisualEditor.tsx`, now only exists in `App.tsx` on the project detail page.

---

## Implementation Priority

### Phase 1: Quick Wins ✅ COMPLETED (Target: 6-8 hours, Actual: 2.25 hours)
1. ✅ Remove dead entity files (nodes.rs, edges.rs) - 1 hour
2. ✅ Remove dead frontend dialog component - 15 minutes
3. ✅ Extract duplicate LayerData struct - 30 minutes

**Progress:** 3/3 items completed, 827 lines of code removed/refactored

### Phase 2: Medium Refactoring (12-16 hours) 🚧 IN PROGRESS
4. Resolve entity naming inconsistency - NEXT
5. Extract common pipeline builder logic
6. Document/consolidate collaboration hooks

### Phase 3: Architectural Improvements (24+ hours)
7. Standardize GraphQL naming conventions
8. Refactor service layer architecture

---

## Testing Strategy

For each refactoring:
1. **Before Changes:**
   - Run full test suite
   - Document current behavior
   - Create regression tests if needed

2. **During Changes:**
   - Use compiler to catch breaking changes
   - Run tests frequently
   - Commit incrementally

3. **After Changes:**
   - Run full test suite
   - Manual testing of affected features
   - Performance benchmarks for critical paths

---

## Rollout Strategy

- **Branch Strategy:** Create feature branches for each phase
- **Review Process:** All refactorings require code review
- **Deployment:** Deploy phases incrementally, monitor for issues
- **Documentation:** Update architecture docs with each phase

---

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Breaking existing functionality | High | Comprehensive tests before each change |
| Team velocity reduction | Medium | Limit refactoring to 20% of sprint capacity |
| Merge conflicts with ongoing work | Medium | Coordinate with team, communicate early |
| Incomplete refactoring | Low | Focus on self-contained modules first |

---

## Success Metrics

- [ ] Reduce total line count by removing dead code
- [ ] Eliminate all identified code duplication
- [ ] Improve code search relevance (fewer false positives)
- [ ] Reduce onboarding time for new developers
- [ ] Achieve 100% test coverage on refactored modules

---

## Additional Observations

### Positive Patterns Found

1. **Good CQRS Separation:** Frontend uses separated Command and Query services
2. **Strong Type Safety:** Rust and TypeScript provide excellent compile-time checks
3. **Modern Stack:** Good use of async-graphql, SeaORM, React patterns
4. **Subscription Architecture:** Delta-based updates with JSON Patch are efficient

### Areas for Future Investigation

1. **Bundle Size:** Frontend bundle is large (4.3MB), could benefit from code splitting
2. **Database Indexes:** Verify all foreign keys and frequently queried fields are indexed
3. **GraphQL N+1 Queries:** Check if DataLoader pattern would help
4. **Error Handling:** Standardize error types and responses across services

---

## Conclusion

The codebase is generally well-structured with modern patterns, but has accumulated technical debt through incremental development. The proposed refactoring plan addresses the most impactful issues first while maintaining system stability. The total estimated effort is 50-60 hours, which can be spread across 3-4 sprints.

The highest priority items (Phase 1) should be completed first as they provide immediate maintenance benefits with minimal risk.
