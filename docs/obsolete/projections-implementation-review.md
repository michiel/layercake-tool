# Projections Implementation Review

**Date**: 2025-12-11
**Status**: Partially Complete (Backend ~80%, Frontend ~60%)

---

## Executive Summary

The projections feature implementation is **functional but incomplete**. Core backend infrastructure and GraphQL API are solid, but several key requirements from the plan remain unimplemented or inconsistent.

**Major Gaps**:
1. ❌ Using 2D `force-graph` instead of 3D `3d-force-graph` as specified
2. ❌ No `layer3d` scaffolding or implementation
3. ⚠️ Inconsistent field naming between plan and implementation
4. ⚠️ Tauri window support incomplete/untested
5. ⚠️ Export functionality uses fallback implementation (no built projection assets)
6. ⚠️ No tests for projections functionality

---

## Phase-by-Phase Analysis

### Phase 1: Backend Foundation ✅ **Complete** (~95%)

#### ✅ Implemented
- **Database Migration** (`m20251210_000005_create_projections.rs`):layercake-core/src/database/migrations/m20251210_000005_create_projections.rs:1
  - ✅ Table created with all required fields
  - ✅ Foreign keys to `projects` and `graph_data` (correctly uses unified graph_data)
  - ✅ Indexes on project_id and graph_id
  - ✅ Migration registered in migrations/mod.rs

- **Entity Model** (`projections.rs`):layercake-core/src/database/entities/projections.rs:1
  - ✅ SeaORM entity with all required fields
  - ✅ Relationships to projects and graph_data defined
  - ✅ No legacy references (using graph_data, not old graphs table) ✨

- **ProjectionService** (`layercake-projections/src/service.rs`):
  - ✅ CRUD operations (create, get, list, update, delete)
  - ✅ Graph loading from graph_data_nodes/edges
  - ✅ State management with in-memory store
  - ✅ Broadcast channels for subscriptions
  - ✅ Export payload generation
  - ✅ Export bundle creation with ZIP
  - ✅ Graph/project validation (ensure_graph_in_project)

- **GraphQL API** (`layercake-projections/src/graphql.rs`):
  - ✅ All queries: projections, projection, projectionGraph, projectionState
  - ✅ All mutations: createProjection, updateProjection, deleteProjection, saveProjectionState, refreshProjectionGraph, exportProjection
  - ✅ All subscriptions: projectionStateUpdated, projectionGraphUpdated
  - ✅ Proper input/output types

- **Server Integration** (`layercake-core/src/server/app.rs`):layercake-core/src/server/app.rs:32-46
  - ✅ ProjectionService instantiated
  - ✅ Projections GraphQL schema built
  - ✅ Routes mounted at `/projections/graphql` and `/projections/graphql/ws`
  - ✅ Static assets served from `projections-frontend/dist`
  - ✅ Proper WebSocket handling

#### ⚠️ Issues

1. **Export Implementation Uses Fallback**:layercake-projections/src/service.rs:394-432
   - Current: Falls back to hardcoded HTML template if no built assets found
   - Expected: Use built projections-frontend assets (preferred path)
   - Impact: Exported projections may have inconsistent UI with live version

2. **No Service-Level Tests**:
   - No unit/integration tests for ProjectionService
   - No tests for graph cloning, state persistence, subscriptions
   - Risk: Undetected bugs in critical flows

---

### Phase 2: Workbench UI ✅ **Complete** (~90%)

#### ✅ Implemented
- **ProjectionsPage** (`frontend/src/pages/workbench/ProjectionsPage.tsx`):frontend/src/pages/workbench/ProjectionsPage.tsx:1
  - ✅ List projections for project
  - ✅ Create projection with graph picker and type selector
  - ✅ Open projection (web tab or Tauri window)
  - ✅ Export projection (base64 ZIP download)
  - ✅ Delete projection
  - ✅ Uses correct `/projections/graphql` endpoint
  - ✅ Proper Apollo client configuration

#### ⚠️ Issues

1. **Field Naming Inconsistency**:frontend/src/pages/workbench/ProjectionsPage.tsx:18-26
   - GraphQL query uses snake_case aliases (`projection_type`, `graph_id`, `updated_at`)
   - Plan specified camelCase in GraphQL schema
   - Inconsistent with GraphQL conventions (should be camelCase)

2. **Tauri Window Support**:frontend/src/pages/workbench/ProjectionsPage.tsx:148-165
   - Code exists but untested/unverified
   - No confirmation Tauri window can access projection routes
   - No documentation about Tauri packaging requirements

3. **No Navigation Integration**:
   - Projections page not linked from main Workbench nav
   - Users can't discover the feature easily

---

### Phase 3: Projection Frontend Build 🚧 **Incomplete** (~60%)

#### ✅ Implemented
- **Separate Frontend Build** (`projections-frontend/`):
  - ✅ Separate Vite project with TypeScript
  - ✅ Force-graph dependency installed
  - ✅ Apollo Client with subscriptions
  - ✅ Basic projection viewer component

- **ProjectionViewerPage (Main App)** (`frontend/src/pages/projections/ProjectionViewerPage.tsx`):frontend/src/pages/projections/ProjectionViewerPage.tsx:1
  - ✅ Query projection metadata and graph
  - ✅ Subscribe to graph/state updates
  - ✅ Render force-graph visualization
  - ✅ Save state functionality

- **Projections Frontend App** (`projections-frontend/src/app.tsx`):projections-frontend/src/app.tsx:1
  - ✅ Standalone app with Apollo client
  - ✅ Force-graph rendering
  - ✅ UI controls (node/link color, size, show/hide links)
  - ✅ State persistence

#### ❌ Major Issues

1. **WRONG LIBRARY**: Using 2D `force-graph` instead of 3D `3d-force-graph`
   - **Plan specified**: `force3d` using https://github.com/vasturiano/3d-force-graph
   - **Actually using**: 2D `force-graph` packageprojections-frontend/package.json:17
   - **Impact**: Feature mismatch, not 3D visualization as planned
   - **Fix Required**: Replace with `3d-force-graph` package and update rendering code

2. **No `layer3d` Scaffolding**:
   - Plan required: Stub component and default state for `layer3d` type
   - Currently: Only `force3d` (2D) implementation exists
   - Users can select `layer3d` type but nothing renders

3. **No Advanced Controls**:
   - Current: Basic color/size controls only
   - Plan mentioned: Force params, filters, layer-based coloring
   - Missing: Camera controls, physics parameters, layer filtering

4. **Field Naming Inconsistency**:projections-frontend/src/app.tsx:8-23
   - Uses camelCase (correct) but main app uses snake_case
   - Indicates GraphQL schema may be inconsistent

#### ⚠️ Minor Issues

1. **Duplicate Implementation**:
   - Two separate projection viewers: one in main app (`ProjectionViewerPage.tsx`), one in projections-frontend (`app.tsx`)
   - Similar but not identical functionality
   - Maintenance burden, confusion about which to use

2. **No Routing in Standalone Build**:projections-frontend/src/app.tsx:51-58
   - Parses projection ID from URL manually
   - Should use proper router (React Router)

3. **Missing Build Commands**:
   - Plan mentioned: `npm run projections:dev`, `npm run projections:build`
   - Actual: Only standard Vite commands in package.json
   - No convenience scripts in root workspace

---

### Phase 4: Export Pipeline ⚠️ **Partial** (~70%)

#### ✅ Implemented
- **Export Mutation**:layercake-projections/src/graphql.rs:171-178
  - ✅ GraphQL mutation `exportProjection` returns base64 ZIP
  - ✅ Frontend downloads and saves ZIP file

- **Export Service**:layercake-projections/src/service.rs:373-501
  - ✅ Bundles projection metadata + graph + state into JSON payload
  - ✅ Creates ZIP with `index.html`, `data.js`, `projection.js`
  - ✅ Embeds force-graph library (if found in node_modules)
  - ✅ Standalone-pack pattern implemented (window.PROJECTION_EXPORT)

#### ⚠️ Issues

1. **Fallback Implementation**:layercake-projections/src/service.rs:394-432
   - Prefers built projection assets but falls back to hardcoded template
   - Fallback uses basic HTML/JS, may not match live UI features
   - No warning/logging when fallback is used

2. **Force-Graph Bundle Path Hardcoded**:layercake-projections/src/service.rs:503-520
   - Searches specific node_modules paths
   - May fail in different build environments (Docker, CI)
   - No configuration option

3. **No Export for `layer3d`**:
   - Export only works for `force3d` (using force-graph)
   - `layer3d` exports would fail or render nothing

4. **No Asset Optimization**:
   - Large force-graph.min.js embedded in every export (~500KB)
   - Could use CDN links or smaller bundle

---

### Phase 5: Desktop/Tauri 🚧 **Incomplete** (~40%)

#### ⚠️ Issues

1. **Tauri Window Opening**:frontend/src/pages/workbench/ProjectionsPage.tsx:148-165
   - Code exists but **not tested/verified**
   - No documentation about setup requirements
   - Unknown if projection routes accessible from Tauri windows

2. **Asset Packaging**:
   - Plan: "package projection assets in desktop build"
   - Status: Unknown if `projections-frontend/dist` bundled in Tauri app
   - No documentation about Tauri configuration

3. **WebSocket Support**:
   - Plan: "verify CORS/IPC allowances"
   - Status: Untested, unknown if subscriptions work in Tauri windows

**Recommendation**: Mark as TODO until Tauri build tested.

---

### Phase 6: Hardening ❌ **Not Started** (~5%)

#### ❌ Missing

1. **Auth/Authorization**:
   - No project-level permission checks on projection operations
   - Anyone with endpoint access can query any projection
   - Service uses ensure_graph_in_project but no user context validation

2. **Tests**:
   - ❌ No service tests
   - ❌ No GraphQL schema tests
   - ❌ No UI smoke tests
   - ❌ No export roundtrip tests

3. **Performance**:
   - No lazy loading for large graphs (loads all nodes/edges upfront)
   - No pagination on projections list
   - No caching beyond in-memory state store
   - No backpressure on broadcast channels

4. **Monitoring**:
   - No telemetry/logging for projection operations
   - No metrics for subscription usage
   - No error tracking

5. **Rate Limiting**:
   - No limits on subscription connections
   - No limits on export requests
   - Could be abused for resource exhaustion

---

## Legacy Code and Data Structure Issues

### ✅ No Legacy Table References

**Excellent**: Implementation correctly uses unified `graph_data` model throughout:
- Migration FKs reference `graph_data.id` (not legacy `graphs.id`)layercake-core/src/database/migrations/m20251210_000005_create_projections.rs:52-58
- Entity relations use `graph_data::Entity`layercake-core/src/database/entities/projections.rs:29-34
- Service loads from `graph_data_nodes` and `graph_data_edges`layercake-projections/src/service.rs:233-241
- No references to old `graphs`, `graph_nodes`, `graph_edges` tables ✨

This aligns perfectly with the completed graph_data refactoring!

### ⚠️ Inconsistent Field Naming

**Issue**: Plan uses camelCase, implementation mixes snake_case and camelCase:

| Context | Field Name | Convention |
|---------|-----------|-----------|
| Plan | `projectionType`, `graphId`, `updatedAt` | camelCase |
| DB Schema | `projection_type`, `graph_id`, `updated_at` | snake_case (correct) |
| GraphQL Types (Backend) | `projection_type`, `graph_id`, `updated_at` | snake_case ❌ |
| Frontend Main App | `projection_type`, `graph_id`, `updated_at` | snake_case ❌ |
| Frontend Standalone | `projectionType`, `graphId` | camelCase ✅ |

**Expected**: GraphQL schema should expose camelCase (GraphQL convention), backend uses snake_case internally.

**Recommendation**: Update GraphQL type definitions to use camelCase aliases.

---

## Critical Gaps Summary

### 🔴 Blockers (Must Fix)

1. **Wrong Visualization Library**: Using 2D force-graph instead of 3D 3d-force-graph
   - Plan explicitly specified 3D version
   - Feature mismatch with user expectations
   - **Fix**: Replace package, rewrite renderer

2. **No `layer3d` Implementation**: Type selectable but renders nothing
   - Users can create `layer3d` projections that don't work
   - **Fix**: Add stub component or remove from type picker

### 🟡 Important (Should Fix)

3. **No Tests**: Zero test coverage for projections
   - High risk for regression bugs
   - **Fix**: Add service, GraphQL, and UI tests

4. **No Auth**: Project permissions not enforced
   - Security/privacy risk
   - **Fix**: Add user context validation in service

5. **Export Uses Fallback**: Not using built projection assets
   - Exported UI may differ from live version
   - **Fix**: Ensure projections-frontend/dist is built and available

6. **Tauri Support Unverified**: Code exists but untested
   - Unknown if feature works in desktop app
   - **Fix**: Test Tauri window opening and asset access

### 🟢 Nice to Have

7. **Field Naming Consistency**: GraphQL should use camelCase
   - Minor inconvenience, not breaking
   - **Fix**: Add GraphQL aliases

8. **No Advanced Controls**: Basic vis controls only
   - Plan mentioned force params, filters
   - **Fix**: Add force physics controls, layer filtering

9. **Duplicate Viewer Code**: Two separate projection viewers
   - Maintenance burden
   - **Fix**: Consolidate or remove main app viewer

10. **No Performance Optimizations**: Loads entire graph upfront
    - May fail for very large graphs
    - **Fix**: Add lazy loading, pagination

---

## Incomplete Migrations

### ✅ Migration Complete and Correct

The projections migration is:
- ✅ **Registered** in migrations/mod.rslayercake-core/src/database/migrations/mod.rs:44
- ✅ **Uses graph_data** (not legacy graphs table)
- ✅ **Proper FKs** with CASCADE on delete
- ✅ **Indexed** on project_id and graph_id
- ✅ **Reversible** (down() implementation exists)

**No migration issues found.**

---

## Recommendations

### Immediate Actions

1. **Fix Visualization Library** (1-2 days):
   - Replace `force-graph` with `3d-force-graph` in package.json
   - Update rendering code in both viewers
   - Test 3D controls and camera

2. **Add `layer3d` Stub** (1 hour):
   - Create placeholder component in projections-frontend
   - Show "Coming soon" message for layer3d type
   - Or remove from type picker until implemented

3. **Fix Field Naming** (2 hours):
   - Add camelCase aliases to GraphQL types
   - Update frontend to use camelCase fields
   - Document convention in README

### Short-term (Next Sprint)

4. **Add Tests** (2-3 days):
   - Unit tests for ProjectionService
   - Integration tests for GraphQL mutations
   - E2E test for export roundtrip

5. **Add Auth** (1 day):
   - Inject user context into ProjectionService
   - Validate user has access to project
   - Add authorization tests

6. **Verify Tauri Support** (1 day):
   - Test projection windows in Tauri app
   - Document required Tauri config
   - Fix any asset loading issues

### Medium-term (Next Quarter)

7. **Implement Layer3D** (1-2 weeks):
   - Design layer-based 3D layout
   - Implement renderer
   - Add layer-specific controls

8. **Performance Optimizations** (1 week):
   - Add lazy loading for large graphs
   - Implement pagination for projections list
   - Add caching layer

9. **Advanced Controls** (1 week):
   - Force physics parameters
   - Layer filtering
   - Node/edge search
   - Camera bookmarks

---

## Documentation Gaps

### Missing Documentation

1. **Setup Instructions**: No docs for building/running projections-frontend
2. **API Documentation**: GraphQL schema not documented
3. **Tauri Packaging**: No guide for desktop build
4. **Export Format**: Standalone pack format not documented
5. **Contribution Guide**: No guide for adding new projection types

### Recommended Docs

- `docs/projections-setup.md`: Local development setup
- `docs/projections-api.md`: GraphQL API reference
- `docs/projections-export-format.md`: Export bundle specification
- `docs/projections-tauri.md`: Desktop app integration
- `docs/projections-adding-types.md`: How to add new projection types

---

## Conclusion

### Status Summary

| Phase | Status | Completeness |
|-------|--------|--------------|
| 1. Backend Foundation | ✅ Complete | 95% |
| 2. Workbench UI | ✅ Complete | 90% |
| 3. Projection Frontend | 🚧 Incomplete | 60% |
| 4. Export Pipeline | ⚠️ Partial | 70% |
| 5. Desktop/Tauri | 🚧 Incomplete | 40% |
| 6. Hardening | ❌ Not Started | 5% |
| **Overall** | **🟡 Partial** | **~70%** |

### Key Takeaways

**Strengths**:
- ✅ Solid backend architecture and GraphQL API
- ✅ Correctly uses unified graph_data model (no legacy code!)
- ✅ Working end-to-end flow for basic use cases
- ✅ Export functionality exists and works

**Weaknesses**:
- ❌ Wrong visualization library (2D instead of 3D)
- ❌ No layer3d implementation
- ❌ No tests
- ❌ No auth
- ❌ Tauri support unverified

**Verdict**: The projections feature is **functional for basic use** but **not production-ready**. It requires the critical fixes (3D library, tests, auth) before being promoted to users. The good news is the backend foundation is solid and correctly architected.

---

**Next Steps**: Address the 🔴 Blockers first, then tackle 🟡 Important issues before promoting the feature beyond internal testing.
