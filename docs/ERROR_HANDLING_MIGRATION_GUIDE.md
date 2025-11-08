# Error Handling Migration Guide

This guide explains how to migrate services from generic `anyhow::Result` to domain-specific typed errors in the Layercake codebase.

## Overview

We've created comprehensive domain-specific error types to improve error handling:
- `GraphError` - Graph operations
- `PlanError` - Plan and DAG operations
- `DataSourceError` - Data source operations
- `AuthError` - Authentication/authorization
- `ImportExportError` - Import/export operations

## Type Aliases

Convenient type aliases are available:
```rust
use crate::errors::{GraphResult, PlanResult, DataSourceResult, AuthResult, ImportExportResult};

// Instead of Result<T, GraphError>
pub fn my_function() -> GraphResult<Graph> { ... }
```

## Migration Pattern

### Before (Generic Error)
```rust
use anyhow::Result;
use sea_orm::{EntityTrait, DatabaseConnection};

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub async fn get_graph(&self, id: i32) -> Result<Graph> {
        let graph = Graphs::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph {} not found", id))?;

        Ok(graph)
    }
}
```

### After (Typed Error)
```rust
use crate::errors::{GraphError, GraphResult};
use sea_orm::{EntityTrait, DatabaseConnection};

pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub async fn get_graph(&self, id: i32) -> GraphResult<Graph> {
        let graph = Graphs::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?  // Convert DB error
            .ok_or(GraphError::NotFound(id))?;  // Use typed error

        Ok(graph)
    }
}
```

## Common Error Conversions

### Database Errors
```rust
// Before
.await?

// After - explicit conversion
.await.map_err(GraphError::Database)?
.await.map_err(PlanError::Database)?
.await.map_err(DataSourceError::Database)?
```

### Not Found Errors
```rust
// Before
.ok_or_else(|| anyhow::anyhow!("Graph {} not found", id))?

// After
.ok_or(GraphError::NotFound(id))?
.ok_or(PlanError::NotFound(id))?
.ok_or(DataSourceError::NotFound(id))?
```

### Validation Errors
```rust
// Before
if invalid {
    return Err(anyhow::anyhow!("Validation failed: {}", reason));
}

// After
if invalid {
    return Err(GraphError::Validation(reason.to_string()));
}
```

### Context Errors
```rust
// Before
.context("Failed to parse config")?

// After
.map_err(|e| PlanError::InvalidConfig(e.to_string()))?
```

## Error Type Selection Guide

### GraphService → GraphError
Methods related to:
- Finding graphs, nodes, edges, layers
- Graph creation, updates, deletion
- Graph validation
- Graph transformations

**Example variants:**
- `GraphError::NotFound(id)` - Graph not found
- `GraphError::InvalidNode(id)` - Invalid node reference
- `GraphError::CycleDetected(msg)` - Cycle in graph
- `GraphError::Database(err)` - Database error

### PlanDagService → PlanError
Methods related to:
- Plan CRUD operations
- DAG node/edge operations
- Plan execution
- DAG validation

**Example variants:**
- `PlanError::NotFound(id)` - Plan not found
- `PlanError::NodeNotFound(id)` - DAG node not found
- `PlanError::ExecutionFailed { node, reason }` - Execution error
- `PlanError::CycleDetected(msg)` - Cycle in DAG

### DataSourceService → DataSourceError
Methods related to:
- Data source upload/import
- File format handling
- Data source processing
- Export operations

**Example variants:**
- `DataSourceError::NotFound(id)` - Data source not found
- `DataSourceError::UnsupportedFormat(fmt)` - Unsupported format
- `DataSourceError::InvalidCsv(msg)` - CSV parsing error
- `DataSourceError::ImportFailed(msg)` - Import error

### AuthService → AuthError
Methods related to:
- User authentication
- Session management
- Permission checking
- Role validation

**Example variants:**
- `AuthError::InvalidCredentials` - Bad login
- `AuthError::SessionExpired` - Expired session
- `AuthError::PermissionDenied(action)` - Insufficient permissions
- `AuthError::InvalidRole(role)` - Invalid role

### ImportService/ExportService → ImportExportError
Methods related to:
- Graph import/export
- Format conversion
- Template rendering
- Data serialization

**Example variants:**
- `ImportExportError::ImportFailed(msg)` - Import error
- `ImportExportError::UnsupportedFormat(fmt)` - Format not supported
- `ImportExportError::TemplateError(msg)` - Template rendering error
- `ImportExportError::SerializationError(err)` - JSON/YAML error

## Service Migration Checklist

For each service file:

- [ ] Import appropriate error type and result alias
  ```rust
  use crate::errors::{GraphError, GraphResult};
  ```

- [ ] Update all function signatures
  ```rust
  // Before: Result<T>
  // After:  GraphResult<T>
  ```

- [ ] Convert database errors
  ```rust
  .await.map_err(GraphError::Database)?
  ```

- [ ] Replace generic errors with typed variants
  ```rust
  // Before: anyhow::anyhow!("...")
  // After:  GraphError::Validation(...)
  ```

- [ ] Update tests to expect typed errors
  ```rust
  assert!(matches!(result, Err(GraphError::NotFound(_))));
  ```

- [ ] Remove unused `anyhow` imports
  ```rust
  // Remove if no longer needed:
  // use anyhow::{Result, Context};
  ```

## GraphQL Integration

After migrating services, update GraphQL mutations:

```rust
use crate::errors::ToGraphQLError;

async fn get_graph(ctx: &Context<'_>, id: i32) -> Result<Graph> {
    let graph = ctx.app.graph_service()
        .get_graph(id)
        .await
        .map_err(|e| e.to_graphql_error())?;  // Convert to GraphQL error

    Ok(graph)
}
```

The `ToGraphQLError` trait automatically:
- Converts error to GraphQL error format
- Adds appropriate error codes
- Preserves error messages
- Includes structured data for clients

## Testing Error Handling

### Unit Tests
```rust
#[tokio::test]
async fn test_graph_not_found() {
    let service = create_test_service().await;

    let result = service.get_graph(9999).await;

    assert!(matches!(result, Err(GraphError::NotFound(9999))));
}

#[tokio::test]
async fn test_validation_error() {
    let service = create_test_service().await;

    let result = service.validate_graph(&invalid_graph).await;

    match result {
        Err(GraphError::Validation(msg)) => {
            assert!(msg.contains("expected"));
        }
        _ => panic!("Expected validation error"),
    }
}
```

### Error Categorization Tests
```rust
#[test]
fn test_error_categorization() {
    let err = GraphError::NotFound(1);
    assert!(err.is_not_found());
    assert!(err.is_client_error());

    let err = GraphError::Database(db_err);
    assert!(!err.is_client_error());
}
```

## Common Pitfalls

### ❌ Don't lose error context
```rust
// Bad - loses original error
.map_err(|_| GraphError::Validation("Invalid".to_string()))?

// Good - preserves context
.map_err(|e| GraphError::Validation(format!("Invalid graph: {}", e)))?
```

### ❌ Don't convert at wrong layer
```rust
// Bad - converting in model layer
impl Model {
    fn validate(&self) -> GraphResult<()> { ... }  // Too specific
}

// Good - service layer handles conversion
impl GraphService {
    fn validate_graph(&self, graph: &Graph) -> GraphResult<()> { ... }
}
```

### ❌ Don't use generic errors for business logic
```rust
// Bad - business logic with generic error
if graph.nodes.is_empty() {
    return Err(anyhow::anyhow!("No nodes"));
}

// Good - specific error
if graph.nodes.is_empty() {
    return Err(GraphError::Validation("Graph must have at least one node".to_string()));
}
```

## Migration Order

Recommended migration order:

1. **High-traffic services first:**
   - graph_service.rs → GraphError
   - plan_dag_service.rs → PlanError
   - data_source_service.rs → DataSourceError

2. **Authentication layer:**
   - auth_service.rs → AuthError
   - authorization.rs → AuthError

3. **Supporting services:**
   - import_service.rs → ImportExportError
   - export_service.rs → ImportExportError
   - graph_edit_service.rs → GraphError
   - graph_analysis_service.rs → GraphError

4. **Specialized services:**
   - All other services based on their domain

5. **GraphQL layer:**
   - Update all mutations to use ToGraphQLError

## Reference Implementation

See `graph_service.rs` for a complete reference implementation showing all patterns.

## Benefits

After migration:
- ✅ Type-safe error handling
- ✅ Clear error messages with context
- ✅ Error codes for API clients
- ✅ Better error categorization
- ✅ Easier debugging
- ✅ Improved testing
- ✅ GraphQL error integration

## Questions?

If you encounter an error scenario not covered here, consider:
1. Is this a new error variant needed? → Add to appropriate error enum
2. Is this a cross-cutting concern? → May need a new error type
3. Is this really an error? → Consider using Option<T> instead
