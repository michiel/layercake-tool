# GraphQL Naming Standards

**Date**: 2025-10-26
**Status**: Standardized (Phase 3.4)

## Overview

This document defines the naming conventions for the Layercake GraphQL schema to ensure consistency, predictability, and adherence to GraphQL best practices.

## Core Principle

**GraphQL API uses camelCase, Rust code uses snake_case.**

- GraphQL fields, arguments, and enum values: `camelCase`
- Rust struct fields, variables, functions: `snake_case`
- Rust types, enums, traits: `PascalCase`

Use `#[graphql(...)]` attributes to bridge the naming gap.

## Field Naming

### SimpleObject and InputObject Fields

**Rule**: All GraphQL-exposed fields must use camelCase.

**Implementation**:
1. Rust field names use snake_case (per Rust conventions)
2. Use `#[graphql(name = "...")]` to expose as camelCase in GraphQL

**Example**:
```rust
#[derive(SimpleObject)]
pub struct DataSource {
    pub id: i32,

    #[graphql(name = "projectId")]
    pub project_id: i32,

    #[graphql(name = "fileName")]
    pub file_name: String,

    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}
```

**Result in GraphQL**:
```graphql
type DataSource {
  id: Int!
  projectId: Int!
  fileName: String!
  createdAt: DateTime!
  updatedAt: DateTime!
}
```

### Fields Already in camelCase

If a Rust field name is already valid camelCase, no directive is needed:

```rust
#[derive(SimpleObject)]
pub struct Position {
    pub x: f64,  // ✅ Already camelCase, no directive needed
    pub y: f64,  // ✅ Already camelCase, no directive needed
}
```

### Common Patterns

| Rust Field | GraphQL Field | Directive |
|------------|---------------|-----------|
| `id` | `id` | None (already camelCase) |
| `name` | `name` | None (already camelCase) |
| `project_id` | `projectId` | `#[graphql(name = "projectId")]` |
| `created_at` | `createdAt` | `#[graphql(name = "createdAt")]` |
| `updated_at` | `updatedAt` | `#[graphql(name = "updatedAt")]` |
| `is_active` | `isActive` | `#[graphql(name = "isActive")]` |
| `node_type` | `nodeType` | `#[graphql(name = "nodeType")]` |
| `file_path` | `filePath` | `#[graphql(name = "filePath")]` |

## Type Naming

### Object Types

**Rule**: Use PascalCase for all GraphQL type names.

```rust
// ✅ Good
pub struct DataSource { }
pub struct PlanDagNode { }
pub struct UserSession { }

// ❌ Bad
pub struct data_source { }
pub struct plan_dag_node { }
```

### Input Types

**Rule**: Suffix input types with "Input" and use PascalCase.

```rust
#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateUserInput {
    #[graphql(name = "displayName")]
    pub display_name: Option<String>,
}
```

### Dual-Purpose Types

When a type is both SimpleObject and InputObject, specify the input name:

```rust
#[derive(SimpleObject, InputObject, Clone, Debug)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
```

**Result**:
```graphql
type Position {
  x: Float!
  y: Float!
}

input PositionInput {
  x: Float!
  y: Float!
}
```

## Enum Naming

### Enum Types

**Rule**: Enum type names use PascalCase, variant names use PascalCase.

```rust
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ProjectRole {
    Owner,
    Editor,
    Viewer,
}
```

### Enum Values in GraphQL

**Rule**: Use SCREAMING_SNAKE_CASE or PascalCase for GraphQL enum values (both are acceptable).

**Option 1: SCREAMING_SNAKE_CASE (Recommended for constants)**
```rust
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum NodeStatus {
    #[graphql(name = "NOT_STARTED")]
    NotStarted,

    #[graphql(name = "IN_PROGRESS")]
    InProgress,

    #[graphql(name = "COMPLETED")]
    Completed,
}
```

**Option 2: PascalCase (Recommended for types)**
```rust
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ProjectRole {
    Owner,     // Exposed as "Owner" in GraphQL
    Editor,    // Exposed as "Editor" in GraphQL
    Viewer,    // Exposed as "Viewer" in GraphQL
}
```

**Current Practice**: The codebase uses PascalCase for most enums. Be consistent within the same enum.

## Query and Mutation Naming

### Queries

**Rule**: Use camelCase for query names. Start with a verb when appropriate.

```rust
async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>>
async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Project>>
async fn find_user(&self, ctx: &Context<'_>, filter: UserFilter) -> Result<Option<User>>
```

**GraphQL**:
```graphql
type Query {
  projects: [Project!]!
  project(id: Int!): Project
  findUser(filter: UserFilter!): User
}
```

### Mutations

**Rule**: Use camelCase for mutation names. Start with a verb.

```rust
async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project>
async fn update_project(&self, ctx: &Context<'_>, id: i32, input: UpdateProjectInput) -> Result<Project>
async fn delete_project(&self, ctx: &Context<'_>, id: i32) -> Result<bool>
```

**GraphQL**:
```graphql
type Mutation {
  createProject(input: CreateProjectInput!): Project!
  updateProject(id: Int!, input: UpdateProjectInput!): Project!
  deleteProject(id: Int!): Boolean!
}
```

### Subscription Naming

**Rule**: Use camelCase for subscription names. Use past tense or present continuous.

```rust
async fn plan_dag_updated(&self, ctx: &Context<'_>, plan_id: String)
async fn collaboration_events(&self, ctx: &Context<'_>, plan_id: String)
async fn node_execution_status_changed(&self, ctx: &Context<'_>, project_id: i32)
```

**GraphQL**:
```graphql
type Subscription {
  planDagUpdated(planId: String!): PlanDagUpdateEvent!
  collaborationEvents(planId: String!): CollaborationEvent!
  nodeExecutionStatusChanged(projectId: Int!): NodeExecutionStatusEvent!
}
```

## Arguments and Variables

**Rule**: All arguments use camelCase.

```rust
async fn datasource_preview(
    &self,
    ctx: &Context<'_>,
    #[graphql(name = "dataSourceId")] data_source_id: i32,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<DataSourcePreview>
```

**GraphQL**:
```graphql
type Query {
  datasourcePreview(
    dataSourceId: Int!
    offset: Int
    limit: Int
  ): DataSourcePreview!
}
```

## Serde Integration

### JSON Serialization

When types are serialized to JSON (for database storage, APIs, etc.), use camelCase:

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilterConfig {
    pub targets: Vec<QueryFilterTarget>,
    pub mode: QueryFilterMode,
    pub rule_group: serde_json::Value,
}
```

### Database Models

SeaORM entities typically use snake_case (matching database columns):

```rust
// SeaORM entity (database layer)
pub mod users {
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    pub struct Model {
        pub id: i32,
        pub email: String,
        pub created_at: DateTime<Utc>,
    }
}

// GraphQL type (API layer)
#[derive(SimpleObject)]
pub struct User {
    pub id: i32,
    pub email: String,

    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
}
```

## Validation Checklist

When adding new GraphQL types, ensure:

- [ ] Type names are PascalCase
- [ ] Field names are camelCase in GraphQL (use `#[graphql(name = "...")]` if needed)
- [ ] Input types are suffixed with "Input"
- [ ] Enum variants follow consistent casing (PascalCase or SCREAMING_SNAKE_CASE)
- [ ] Query/Mutation names are camelCase and start with verbs
- [ ] Arguments use camelCase
- [ ] Serde serialization uses `#[serde(rename_all = "camelCase")]` where applicable

## Migration Path

For existing types with inconsistent naming:

1. **Non-Breaking**: Add `#[graphql(name = "...")]` attributes without changing Rust code
2. **Breaking**: Only change GraphQL-exposed names during major version bumps
3. **Documentation**: Update this guide when new patterns emerge

## Examples from Codebase

### ✅ Good Examples

**User Type** (layercake-core/src/graphql/types/user.rs):
```rust
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_color: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}
```

Note: This type should add `#[graphql(name = "...")]` to fields like `display_name`, `avatar_color`, etc.

**Position Type** (layercake-core/src/graphql/types/plan_dag.rs):
```rust
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,  // ✅ Already camelCase
    pub y: f64,  // ✅ Already camelCase
}
```

**UserFilter Input** (layercake-core/src/graphql/types/user.rs):
```rust
#[derive(InputObject)]
pub struct UserFilter {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub session_id: Option<String>,  // ❌ Should have #[graphql(name = "sessionId")]
}
```

## Future Improvements

1. **Linting Rule**: Create custom clippy lint to enforce `#[graphql(name = "...")]` on snake_case fields
2. **Code Generation**: Auto-generate GraphQL types from database schemas with correct naming
3. **Documentation**: Generate naming examples in API documentation
4. **Testing**: Add schema validation tests to ensure naming consistency

---

**Status**: Standardized and documented
**Phase**: 3.4 Complete
**Next Steps**: Apply naming directives to existing types during regular maintenance
