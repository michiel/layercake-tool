# Layer Explorer Feature - Projection Fixes

## Issues
1. The graph dropdown in the projection creation form was empty, even though graphs were visible at `/projects/14/graphs`
2. After creating a projection, it was not visible in the projections list

## Root Cause
The system underwent a data migration where:
- **Old system**: Separate `data_sets` and `graphs` tables
- **New system**: Unified `graph_data` table (source_type='dataset' or 'computed')

The migration (`m20251210_000004_migrate_existing_graph_data.rs`):
1. Migrates `data_sets` to `graph_data` with **same IDs**
2. Migrates `graphs` to `graph_data` with **IDs offset by +1,000,000**

Example: `graphs.id = 14` → `graph_data.id = 1,000,014`

The issues:

**Issue 1 - Empty dropdown and validation error:**
- Frontend was querying the old `graphs` table which contains unmigrated data
- Frontend was adding a +1,000,000 offset to convert old IDs to new IDs
- Backend validation checks that `graph_data.id` exists, but the migration hadn't been run
- Error: `RecordNotFound Error: graph_data 1000425`

**Issue 2 - Projections not visible after creation:**
- Backend GraphQL schema was updated to use camelCase (`#[graphql(rename_fields = "camelCase")]`)
- Frontend queries were using snake_case field names with aliases (e.g., `projectionType: projection_type`)
- This caused GraphQL field resolution to fail silently, returning empty results
- Input types also needed camelCase renaming for consistency

## Solution

**frontend/src/pages/workbench/ProjectionsPage.tsx:**

1. **Query legacy `graphs` table directly** instead of new `graph_data` table
   - This allows the UI to work before migration completes
   - Uses graph IDs as-is (no offset needed)
   - Compatible with pre-migration databases

2. **Updated all GraphQL queries to use camelCase** field names:
   - `LIST_PROJECTIONS`: Changed `projection_type` → `projectionType`, `graph_id` → `graphId`, `updated_at` → `updatedAt`
   - `LIST_GRAPHS`: Queries old `graphs` table for dropdown options
   - `CREATE_PROJECTION`: Uses camelCase field names

3. **Updated mutation input** to use camelCase: `projectId`, `graphId`, `projectionType`

**layercake-projections/src/graphql.rs:**

4. **Added camelCase renaming to input types** for consistency:
   - `CreateProjectionInput`: Added `#[graphql(rename_fields = "camelCase")]`
   - `UpdateProjectionInput`: Added `#[graphql(rename_fields = "camelCase")]`

**layercake-projections/src/service.rs:**

5. **Added fallback validation in `ensure_graph_in_project`** (lines 564-604):
   - First checks new unified `graph_data` table
   - Falls back to legacy `graphs` table if not found
   - Validates project_id matches in either case
   - Allows projections to work before and after migration

**layercake-projections/src/entities/graphs.rs (NEW FILE):**

6. **Created legacy `graphs` entity** for backward compatibility:
   - SeaORM entity definition for old `graphs` table
   - Allows projections service to query legacy data
   - Added to module exports in `entities/mod.rs`

## Important Note

**This fix works both before and after the data migration runs**. The fallback validation allows projections to work with the legacy `graphs` table, so the system is functional immediately.

The migration (`m20251210_000004_migrate_existing_graph_data.rs`) copies data from the old `graphs` and `data_sets` tables into the new unified `graph_data` table. Once the migration completes:
- The validation will find graphs in `graph_data` first
- The legacy `graphs` table fallback will no longer be needed
- Both paths continue to work during the transition period

If the dropdown is empty after this fix:
1. There are no graphs in the legacy `graphs` table
2. Check that graphs exist at `/projects/{id}/graphs`

To verify migration status, check the `graph_data_migration_validation` table for migration statistics.

## Future Work
Eventually:
1. Update `/projects/14/graphs` page to query `graph_data` table instead of old `graphs` table
2. Complete migration of all legacy data
3. Remove dependency on old `graphs` and `data_sets` tables
4. Update all UI to use unified `graph_data` system consistently

## Related Files
- `frontend/src/pages/workbench/ProjectionsPage.tsx` - Projection management UI (queries and mutations)
- `layercake-projections/src/graphql.rs` - GraphQL schema with camelCase field renaming
- `layercake-projections/src/service.rs` - Projection service with fallback validation (lines 564-604)
- `layercake-projections/src/entities/graphs.rs` - Legacy graphs entity for backward compatibility
- `layercake-projections/src/entities/mod.rs` - Entity module exports
- `layercake-core/src/database/migrations/m20251210_000004_migrate_existing_graph_data.rs` - Migration logic
- `layercake-core/src/database/entities/graph_data.rs` - Unified graph entity
- `layercake-projections/src/entities/projections.rs` - Projection entity (references graph_data.id)
