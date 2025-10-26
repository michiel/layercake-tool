# Mutation Module Refactoring Plan

**Date**: 2025-10-26
**Status**: In Progress (Phase 2.1)
**Priority**: High

## Problem Statement

Current `mutations/mod.rs` file is 3,253 lines with 60+ mutation functions, violating single responsibility principle and making maintenance difficult.

## Current State

```
layercake-core/src/graphql/mutations/
├── mod.rs (3,253 lines - TOO LARGE!)
│   ├── Project mutations (5 functions)
│   ├── Plan mutations (4 functions)
│   ├── Plan DAG mutations (15+ functions)
│   ├── Authentication mutations (5 functions)
│   ├── Collaboration mutations (7 functions)
│   ├── Data Source mutations (10+ functions)
│   ├── Graph mutations (10+ functions)
│   └── Library Source mutations (5+ functions)
└── plan_dag_delta.rs (helper module)
```

## Target State

```
layercake-core/src/graphql/mutations/
├── mod.rs (< 200 lines - CLEAN FACADE)
├── project.rs (Project domain mutations)
├── plan.rs (Plan domain mutations)
├── plan_dag.rs (Plan DAG domain mutations)
├── auth.rs (Authentication mutations)
├── collaboration.rs (Collaboration mutations)
├── data_source.rs (Data Source mutations)
├── graph.rs (Graph mutations)
├── library_source.rs (Library Source mutations)
├── plan_dag_delta.rs (existing helper)
└── helpers.rs (shared utility functions)
```

## Architecture Pattern

### Option 1: Nested Field Resolvers (RECOMMENDED)

Expose domain mutations through nested field resolvers on root `Mutation` object.

**GraphQL Schema:**
```graphql
type Mutation {
  # Domain-specific mutation namespaces
  project: ProjectMutations!
  plan: PlanMutations!
  planDag: PlanDagMutations!
  auth: AuthMutations!
  # ... etc
}

type ProjectMutations {
  create(input: CreateProjectInput!): Project!
  update(id: Int!, input: UpdateProjectInput!): Project!
  delete(id: Int!): Boolean!
  createFromSample(sampleKey: String!): Project!
}
```

**Usage:**
```graphql
mutation {
  project {
    create(input: { name: "My Project", description: "..." }) {
      id
      name
    }
  }
}
```

**Implementation:**
```rust
// mutations/mod.rs
pub mod project;
pub mod plan;
// ... other modules

pub struct Mutation;

#[Object]
impl Mutation {
    /// Project management mutations
    async fn project(&self) -> project::ProjectMutations {
        project::ProjectMutations
    }

    /// Plan management mutations
    async fn plan(&self) -> plan::PlanMutations {
        plan::PlanMutations
    }

    // ... other domains
}

// mutations/project.rs
pub struct ProjectMutations;

#[Object]
impl ProjectMutations {
    /// Create a new project
    async fn create(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        // Implementation
    }

    /// Update an existing project
    async fn update(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        // Implementation
    }

    // ... etc
}
```

**Pros:**
- Clear domain separation
- Self-documenting schema
- Easy to discover related mutations
- No breaking changes for new namespaces
- Can migrate incrementally

**Cons:**
- Changes GraphQL API (breaking for existing clients)
- Frontend needs updates
- More verbose mutation calls

### Option 2: Flat with Module Organization (NON-BREAKING)

Keep flat mutation structure but organize code in modules.

**GraphQL Schema (UNCHANGED):**
```graphql
type Mutation {
  createProject(input: CreateProjectInput!): Project!
  updateProject(id: Int!, input: UpdateProjectInput!): Project!
  deleteProject(id: Int!): Boolean!
  # ... all mutations at root level
}
```

**Implementation:**
```rust
// mutations/mod.rs
mod project;
mod plan;
// ... other modules

pub struct Mutation;

#[Object]
impl Mutation {
    // Delegate to modules but keep flat API
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        project::create_project(ctx, input).await
    }

    async fn update_project(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        project::update_project(ctx, id, input).await
    }

    // ... all mutations delegated to modules
}

// mutations/project.rs
pub async fn create_project(
    ctx: &Context<'_>,
    input: CreateProjectInput,
) -> Result<Project> {
    let context = ctx.data::<GraphQLContext>()?;
    // Implementation
}
```

**Pros:**
- No breaking changes
- Can migrate incrementally
- Code organization improves immediately

**Cons:**
- Still flat GraphQL API
- Delegation layer adds boilerplate
- Less clear domain boundaries in API

### Option 3: Hybrid Approach

Use Option 2 (non-breaking) for migration, then gradually introduce Option 1 namespaces.

**Phase 1:** Organize existing mutations (non-breaking)
**Phase 2:** Add new namespaced mutations alongside old ones
**Phase 3:** Deprecate old flat mutations
**Phase 4:** Remove old mutations after migration period

## Implementation Strategy

### Recommended: Start with Option 1 (Nested Resolvers)

**Week 1: Foundation**
1. Create `mutations/project.rs` with `ProjectMutations` struct
2. Create `mutations/plan.rs` with `PlanMutations` struct
3. Update `mutations/mod.rs` to expose these
4. Keep old mutations for backward compatibility (deprecated)

**Week 2: Migration**
5. Create remaining domain modules (auth, data_source, graph, etc.)
6. Test all new mutations work correctly
7. Update frontend to use new namespaced mutations
8. Add deprecation notices to old mutations

**Week 3: Cleanup**
9. Monitor usage of old vs new mutations
10. Remove old mutations after migration period
11. Update documentation

## Module Structure Template

Each domain module should follow this pattern:

```rust
// mutations/{domain}.rs

use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::database::entities::{/* domain entities */};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{/* domain types */};

pub struct {Domain}Mutations;

#[Object]
impl {Domain}Mutations {
    /// Create a new {entity}
    async fn create(
        &self,
        ctx: &Context<'_>,
        input: Create{Entity}Input,
    ) -> Result<{Entity}> {
        let context = ctx.data::<GraphQLContext>()?;
        // Implementation
    }

    /// Update an existing {entity}
    async fn update(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: Update{Entity}Input,
    ) -> Result<{Entity}> {
        let context = ctx.data::<GraphQLContext>()?;
        // Implementation
    }

    /// Delete a {entity}
    async fn delete(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        // Implementation
    }

    // ... domain-specific mutations
}
```

## Domain Breakdown

### 1. Project Mutations (mutations/project.rs)
- `create` - Create new project
- `update` - Update existing project
- `delete` - Delete project
- `createFromSample` - Create from sample template

**Lines**: ~80 lines
**Dependencies**: `projects` entity, `SampleProjectService`

### 2. Plan Mutations (mutations/plan.rs)
- `create` - Create new plan
- `update` - Update existing plan
- `delete` - Delete plan
- `execute` - Execute plan DAG

**Lines**: ~150 lines
**Dependencies**: `plans` entity, `DagExecutor`

### 3. Plan DAG Mutations (mutations/plan_dag.rs)
- `update` - Bulk update (deprecated)
- `addNode` - Add single node
- `updateNode` - Update single node
- `deleteNode` - Delete single node
- `addEdge` - Add single edge
- `deleteEdge` - Delete single edge
- `updateEdge` - Update single edge
- `moveNode` - Move single node
- `batchMoveNodes` - Move multiple nodes

**Lines**: ~800 lines (largest module)
**Dependencies**: `plan_dag_nodes`, `plan_dag_edges`, `plan_dag_delta`

### 4. Authentication Mutations (mutations/auth.rs)
- `register` - Register new user
- `login` - Authenticate user
- `logout` - End user session
- `updateUser` - Update user profile

**Lines**: ~200 lines
**Dependencies**: `users`, `user_sessions`, `AuthService`

### 5. Collaboration Mutations (mutations/collaboration.rs)
- `invite` - Invite collaborator to project
- `accept` - Accept collaboration invite
- `decline` - Decline collaboration invite
- `updateRole` - Update collaborator role
- `remove` - Remove collaborator
- `join` - Join project collaboration
- `leave` - Leave project collaboration

**Lines**: ~250 lines
**Dependencies**: `project_collaborators`

### 6. Data Source Mutations (mutations/data_source.rs)
- `create` - Create data source
- `createEmpty` - Create empty data source
- `createFromFile` - Create from file upload
- `bulkUpload` - Upload multiple data sources
- `update` - Update data source
- `delete` - Delete data source
- `import` - Import data sources
- `export` - Export data sources

**Lines**: ~400 lines
**Dependencies**: `data_sources`, `DataSourceService`, `DataSourceBulkService`

### 7. Graph Mutations (mutations/graph.rs)
- `create` - Create graph
- `update` - Update graph
- `delete` - Delete graph
- `createLayer` - Add layer to graph
- `deleteLayer` - Remove layer from graph
- `createEdit` - Create graph edit
- `deleteEdit` - Delete graph edit
- `applyEdits` - Apply edits to graph
- `replayEdits` - Replay edit history

**Lines**: ~500 lines
**Dependencies**: `graphs`, `graph_layers`, `graph_edits`, `GraphService`, `GraphEditService`

### 8. Library Source Mutations (mutations/library_source.rs)
- `create` - Create library source
- `update` - Update library source
- `delete` - Delete library source
- `import` - Import library sources

**Lines**: ~150 lines
**Dependencies**: `library_sources`, `LibrarySourceService`

## Shared Utilities (mutations/helpers.rs)

Extract common helper functions:

```rust
// mutations/helpers.rs

use crate::graphql::types::{PlanDagNode, PlanDagNodeType};

/// Generate a unique node ID based on node type and existing nodes
pub fn generate_node_id(
    node_type: &PlanDagNodeType,
    existing_nodes: &[PlanDagNode],
) -> String {
    generate_node_id_from_ids(
        node_type,
        &existing_nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
    )
}

/// Generate a unique node ID based on node type and existing node IDs
pub fn generate_node_id_from_ids(
    node_type: &PlanDagNodeType,
    existing_node_ids: &[&str],
) -> String {
    // Implementation...
}

/// Generate a unique edge ID based on source and target
pub fn generate_edge_id(source: &str, target: &str) -> String {
    use uuid::Uuid;
    format!("edge-{}-{}-{}", source, target, Uuid::new_v4().simple())
}
```

## Testing Strategy

### Unit Tests

Each domain module should have its own test module:

```rust
// mutations/project.rs

#[cfg(test)]
mod tests {
    use super::*;
    // Test create_project
    // Test update_project
    // Test delete_project
}
```

### Integration Tests

Test cross-domain workflows:

```rust
// tests/integration/mutations.rs

#[tokio::test]
async fn test_create_project_and_plan() {
    // Create project via project mutations
    // Create plan via plan mutations
    // Verify relationships
}
```

## Migration Checklist

### Backend
- [ ] Create domain module files
- [ ] Move mutation functions to modules
- [ ] Update mod.rs with domain structs
- [ ] Move helper functions to helpers.rs
- [ ] Add deprecation notices to old mutations
- [ ] Update tests
- [ ] Verify all mutations work
- [ ] Run `cargo check` and `cargo test`

### Frontend
- [ ] Regenerate GraphQL types
- [ ] Update mutation calls to use namespaces
- [ ] Update Apollo cache logic
- [ ] Test all mutation flows
- [ ] Update documentation

### Documentation
- [ ] Update API documentation
- [ ] Update migration guide
- [ ] Add code examples
- [ ] Update GraphQL schema docs

## Success Criteria

- [ ] mod.rs file < 200 lines
- [ ] Each domain module < 500 lines
- [ ] All mutations work correctly
- [ ] No breaking changes (old mutations deprecated but functional)
- [ ] Test coverage > 80%
- [ ] Documentation updated
- [ ] Frontend migrated to new namespaces

## Timeline

| Week | Tasks | Deliverables |
|------|-------|--------------|
| 1 | Create modules, move code | Domain modules created |
| 2 | Frontend migration, testing | Frontend updated |
| 3 | Deprecation, documentation | Old mutations deprecated |
| 4 | Cleanup, remove old code | Refactoring complete |

## Example: Project Mutations

See `layercake-core/src/graphql/mutations/project.rs` for complete implementation example.

---

**Status**: Template created, project and plan modules implemented as examples
**Next**: Complete remaining domain modules and integrate into mod.rs
