# Technical Plan: Multiple Plans per Project

## Overview

Currently, a project has a 1:1 relationship with a plan. This change allows projects to have multiple plans, where each plan has its own name, description, and tags. All plans within a project share the same datasets.

## Current State

### Database Schema

- `projects` table with `has_one` relationship to `plans`
- `plans` table: `id`, `project_id`, `name`, `yaml_content`, `dependencies`, `status`, `version`, `created_at`, `updated_at`
- `plan_dag_nodes` table references `plan_id`
- `plan_dag_edges` table references `plan_id`

### GraphQL API

- `getPlanDag(projectId)` - assumes one plan per project
- `updatePlanDag(projectId, planDag)` - same assumption
- Most mutations use `projectId` to find the plan

### Frontend

- Routes: `/projects/:projectId/plan` - no plan selection
- Queries assume single plan per project

## Target State

### Database Schema Changes

1. **Update `plans` table** - add `description` and `tags` columns:
   ```sql
   ALTER TABLE plans ADD COLUMN description TEXT;
   ALTER TABLE plans ADD COLUMN tags TEXT DEFAULT '[]';
   ```

2. **Update `projects` entity** - change relationship from `has_one` to `has_many`

### Data Migration

1. Existing projects keep their current plan as the default
2. No data loss - existing plan_dag_nodes and plan_dag_edges remain linked to their plan_id

---

## Implementation Stages

### Stage 1: Database Migration
**Goal**: Add new columns and update relationships
**Success Criteria**: Migration runs without errors, existing data preserved
**Tests**: Migration up/down tests

1. Create migration `m20251122_000001_add_plan_description_tags.rs`:
   - Add `description TEXT` column to `plans`
   - Add `tags TEXT DEFAULT '[]'` column to `plans`

2. Update `layercake-core/src/database/entities/plans.rs`:
   - Add `description: Option<String>` field
   - Add `tags: String` field (JSON array as string)

3. Update `layercake-core/src/database/entities/projects.rs`:
   - Change `has_one = "super::plans::Entity"` to `has_many = "super::plans::Entity"`

**Status**: Not Started

---

### Stage 2: Backend GraphQL Updates
**Goal**: Support multiple plans in GraphQL API
**Success Criteria**: Can query plans by project, create new plans, query plan by ID
**Tests**: GraphQL query/mutation tests

1. Update `layercake-core/src/graphql/types/plan.rs`:
   - Add `description` and `tags` fields to `Plan` type
   - Update `CreatePlanInput` to include `description` and `tags`
   - Add `UpdatePlanInput` to include `description` and `tags`

2. Add new queries in `layercake-core/src/graphql/queries/`:
   - `plans(projectId: Int!) -> [Plan!]!` - list all plans for a project
   - `plan(id: Int!) -> Plan` - get single plan by ID

3. Update existing queries:
   - `getPlanDag(projectId: Int!, planId: Int)` – `planId` optional; when omitted resolve server-side to the project's default plan for backwards compatibility.

4. Add new mutations in `layercake-core/src/graphql/mutations/plan.rs`:
   - `createPlan(input: CreatePlanInput!) -> Plan`
   - `updatePlan(id: Int!, input: UpdatePlanInput!) -> Plan`
   - `deletePlan(id: Int!) -> Boolean`
   - `duplicatePlan(id: Int!, name: String!) -> Plan` - copy plan with nodes/edges

5. Update plan DAG mutations to accept `planId` (with optional `projectId` for a deprecation period):
   - `updatePlanDag(planId: Int!, planDag: PlanDagInput!, projectId: Int)` – `projectId` optional, used only for auth/backwards compatibility, and logged as deprecated.
   - `updatePlanDagNode(planId: Int!, ...)` and `updatePlanDagEdge(planId: Int!, ...)` follow the same signature.
   - Legacy callers supplying only `projectId` will have their plan resolved server-side (default plan), but responses will include the actual `planId` so clients can migrate.

6. Update `layercake-core/src/services/plan_dag_service.rs`:
   - Update methods to accept `plan_id` directly instead of finding by `project_id`
   - Add method to get default plan for project (for backwards compatibility)

**Status**: Not Started

---

### Stage 3: Backend Plan Service
**Goal**: Full plan management service
**Success Criteria**: Can create, update, delete, duplicate plans
**Tests**: Service unit tests

1. Create/update `layercake-core/src/services/plan_service.rs`:
   - `list_plans(project_id: i32) -> Vec<Plan>`
   - `get_plan(id: i32) -> Option<Plan>`
   - `get_default_plan(project_id: i32) -> Option<Plan>` - first plan or most recently updated
   - `create_plan(input: CreatePlanInput) -> Plan`
   - `update_plan(id: i32, input: UpdatePlanInput) -> Plan`
   - `delete_plan(id: i32) -> Result<()>` - cascade delete nodes/edges
   - `duplicate_plan(id: i32, new_name: String) -> Plan` - copy plan with all nodes/edges

2. Update project creation to auto-create default plan (existing behaviour)

**Status**: Not Started

---

### Stage 4: Frontend GraphQL Updates
**Goal**: Update frontend GraphQL types and queries
**Success Criteria**: TypeScript compiles, queries match backend
**Tests**: Type checking

1. Update `frontend/src/graphql/plan-dag.ts`:
   - Add `GET_PLANS` query
   - Add `GET_PLAN` query
   - Add `CREATE_PLAN` mutation
   - Add `UPDATE_PLAN` mutation
   - Add `DELETE_PLAN` mutation
   - Add `DUPLICATE_PLAN` mutation
   - Update `GET_PLAN_DAG` to accept optional `planId`
   - Update all mutations to use `planId`

2. Create `frontend/src/graphql/plans.ts`:
   - Plan type definition with `id`, `name`, `description`, `tags`, `projectId`, `createdAt`, `updatedAt`
   - All plan-related queries/mutations

**Status**: Not Started

---

### Stage 5: Frontend Plan List UI
**Goal**: UI for listing and managing plans
**Success Criteria**: Can see all plans, create new plan, select plan to edit
**Tests**: Manual testing

1. Create `frontend/src/components/plans/PlansPage.tsx`:
   - List all plans for project
   - Show plan name, description, tags, last modified
   - Create new plan button
   - Edit plan metadata (name, description, tags)
   - Delete plan (with confirmation)
   - Duplicate plan

2. Create `frontend/src/components/plans/CreatePlanModal.tsx`:
   - Form for name, description, tags
   - Option to start from blank or duplicate existing

3. Create `frontend/src/components/plans/EditPlanModal.tsx`:
   - Edit name, description, tags

4. Add routes `/projects/:projectId/plans` (list) and `/projects/:projectId/plans/:planId` (editor) to App.tsx.

5. Update sidebar navigation to include "Plans" link

**Status**: Not Started

---

### Stage 6: Frontend Plan Editor Integration
**Goal**: Update plan editor to work with selected plan
**Success Criteria**: Can edit any plan, switching plans works correctly
**Tests**: Manual testing

1. Update routes in `App.tsx`:
   - Change `/projects/:projectId/plan` to `/projects/:projectId/plans/:planId`
   - Add redirect from old route to new route (using default plan)

2. Update `frontend/src/pages/WorkbenchPage.tsx`:
   - Add plan selector or link to plans list
   - Show current plan info

3. Update `frontend/src/components/editors/PlanVisualEditor/`:
   - Accept `planId` prop instead of deriving from project
   - Update all CQRS hooks to use `planId`

4. Update services:
   - `PlanDagQueryService.ts` - use `planId`
   - `PlanDagCommandService.ts` - use `planId`
   - `PlanDagCQRSService.ts` - use `planId`

5. Update all components using plan DAG:
   - `GraphsPage.tsx`
   - `PlanNodesPage.tsx`
   - `ProjectArtefactsPage.tsx`

**Status**: Not Started

---

### Stage 7: Project Creation Update
**Goal**: Ensure new projects start with a default plan
**Success Criteria**: New projects have one plan named "Main Plan"
**Tests**: Project creation test

1. Update `CreateProjectModal.tsx`:
   - After project creation, auto-create default plan
   - Or ensure backend creates default plan

2. Update backend project creation:
   - Auto-create plan named "Main Plan" when project is created
   - This is likely already happening, verify and document

**Status**: Not Started

---

## API Changes Summary

### New Queries
- `plans(projectId: Int!): [Plan!]!`
- `plan(id: Int!): Plan`

### Updated Queries
- `getPlanDag(projectId: Int!, planId: Int): PlanDag` - planId optional for backwards compat

### New Mutations
- `createPlan(input: CreatePlanInput!): Plan`
- `updatePlan(id: Int!, input: UpdatePlanInput!): Plan`
- `deletePlan(id: Int!): Boolean`
- `duplicatePlan(id: Int!, name: String!): Plan`

### Updated Mutations
All plan DAG mutations updated to use `planId` instead of `projectId`:
- `updatePlanDag(planId: Int!, planDag: PlanDagInput!): PlanDag`
- `addPlanDagNode(planId: Int!, ...): PlanDagNode`
- `updatePlanDagNode(planId: Int!, ...): PlanDagNode`
- `deletePlanDagNode(planId: Int!, nodeId: String!): Boolean`
- etc.

## Routes Summary

### New Routes
- `/projects/:projectId/plans` - Plan list page
- `/projects/:projectId/plans/:planId` - Plan editor (workbench)
- `/projects/:projectId/plans/:planId/artefacts` - Artefacts for plan

### Deprecated Routes (redirect to default plan)
- `/projects/:projectId/plan` -> `/projects/:projectId/plans/:defaultPlanId`

## Migration Path

1. Deploy database migration (no breaking changes)
2. Deploy backend with backwards-compatible queries
3. Deploy frontend with new UI
4. Old API calls continue to work (use default plan)
5. Eventually deprecate projectId-based queries

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing API consumers | Keep `projectId` optional in DAG APIs, resolve default plan ID when omitted, and log deprecation warnings so clients migrate; update docs/examples |
| Data loss on plan deletion | Plan service deletes DAG nodes/edges inside a transaction, UI double-confirms destructive actions |
| Confusion about which plan is active | Workbench header shows selected plan name/description; plan selector (or link back to plan list) always visible |
| Performance with many plans | Add pagination/`ORDER BY` to `plans(projectId)` and indexes on `(project_id, updated_at)` |
| Legacy routes/bookmarks | Frontend redirect `/projects/:projectId/plan → /projects/:projectId/plans/:defaultPlanId`; optionally add backend redirects for REST entry points |

## Testing Strategy

1. **Unit tests**: Plan service methods
2. **Integration tests**: GraphQL queries/mutations
3. **E2E tests**: Create project -> create plans -> edit plan -> delete plan
4. **Migration tests**: Up/down migration with existing data
