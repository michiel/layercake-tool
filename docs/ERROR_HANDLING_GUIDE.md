# GraphQL Error Handling Guide

**Date**: 2025-10-26
**Status**: Implemented (Phase 2.3)

## Overview

Structured error handling system for consistent, machine-readable errors across the GraphQL API.

## Error Module

Location: `layercake-core/src/graphql/errors.rs`

### Error Codes

All errors include a machine-readable error code:

```rust
pub enum ErrorCode {
    NotFound,           // Resource not found (404)
    Unauthorized,       // Not authenticated (401)
    Forbidden,          // Not authorized (403)
    ValidationFailed,   // Input validation failed (400)
    DatabaseError,      // Database operation failed
    ServiceError,       // External service error
    InternalError,      // Internal server error (500)
    Conflict,           // Resource conflict (409)
    BadRequest,         // Bad request (400)
}
```

### StructuredError Helper

Provides consistent error creation methods:

```rust
use crate::graphql::errors::StructuredError;

// Not Found errors
StructuredError::not_found("Project", 42)
// → "Project with id '42' not found" + code: "NOT_FOUND"

StructuredError::not_found_msg("Custom not found message")
// → "Custom not found message" + code: "NOT_FOUND"

// Unauthorized errors
StructuredError::unauthorized("Login required")
// → "Login required" + code: "UNAUTHORIZED"

// Forbidden errors
StructuredError::forbidden("Insufficient permissions")
// → "Insufficient permissions" + code: "FORBIDDEN"

// Validation errors
StructuredError::validation("email", "Invalid format")
// → "Validation failed for 'email': Invalid format" + code: "VALIDATION_FAILED"

// Database errors
StructuredError::database("insert", db_error)
// → "Database error during insert: {cause}" + code: "DATABASE_ERROR"

// Service errors
StructuredError::service("EmailService", service_error)
// → "Service 'EmailService' error: {cause}" + code: "SERVICE_ERROR"

// Internal errors
StructuredError::internal("Unexpected state")
// → "Unexpected state" + code: "INTERNAL_ERROR"

// Conflict errors
StructuredError::conflict("Project", "Name already exists")
// → "Project: Name already exists" + code: "CONFLICT"

// Bad request errors
StructuredError::bad_request("Invalid parameter")
// → "Invalid parameter" + code: "BAD_REQUEST"
```

## Usage Examples

### Basic Error Handling

**Before (inconsistent):**
```rust
async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Project>> {
    let context = ctx.data::<GraphQLContext>()?;
    let project = projects::Entity::find_by_id(id)
        .one(&context.db)
        .await?
        .ok_or_else(|| Error::new("Project not found"))?;  // ❌ No error code

    Ok(Some(Project::from(project)))
}
```

**After (structured):**
```rust
use crate::graphql::errors::StructuredError;

async fn project(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Project>> {
    let context = ctx.data::<GraphQLContext>()?;
    let project = projects::Entity::find_by_id(id)
        .one(&context.db)
        .await?
        .ok_or_else(|| StructuredError::not_found("Project", id))?;  // ✅ Structured

    Ok(Some(Project::from(project)))
}
```

### Mutation Error Handling

```rust
use crate::graphql::errors::StructuredError;

async fn create_project(
    &self,
    ctx: &Context<'_>,
    input: CreateProjectInput,
) -> Result<Project> {
    let context = ctx.data::<GraphQLContext>()?;

    // Validation
    if input.name.is_empty() {
        return Err(StructuredError::validation("name", "Cannot be empty"));
    }

    // Database operation with error handling
    let mut project = projects::ActiveModel::new();
    project.name = Set(input.name);
    project.description = Set(input.description);

    let project = project
        .insert(&context.db)
        .await
        .map_err(|e| StructuredError::database("create project", e))?;

    Ok(Project::from(project))
}
```

### Service Error Handling

```rust
use crate::graphql::errors::StructuredError;

async fn import_data_sources(
    &self,
    ctx: &Context<'_>,
    input: ImportDataSourcesInput,
) -> Result<ImportDataSourcesResult> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = DataSourceService::new(context.db.clone());

    let result = service
        .import_from_file(&input.file_path)
        .await
        .map_err(|e| StructuredError::service("DataSourceService", e))?;

    Ok(result.into())
}
```

### Authorization Error Handling

```rust
use crate::graphql::errors::StructuredError;

async fn delete_project(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
    let context = ctx.data::<GraphQLContext>()?;

    // Check if user is authenticated
    let user = context.current_user.as_ref()
        .ok_or_else(|| StructuredError::unauthorized("Login required"))?;

    // Check if user has permission
    let has_permission = check_project_permission(user.id, id).await?;
    if !has_permission {
        return Err(StructuredError::forbidden("You don't have permission to delete this project"));
    }

    // Proceed with deletion
    projects::Entity::delete_by_id(id)
        .exec(&context.db)
        .await
        .map_err(|e| StructuredError::database("delete project", e))?;

    Ok(true)
}
```

### Using ResultExt for Context

```rust
use crate::graphql::errors::{StructuredError, ResultExt};

async fn complex_operation(&self, ctx: &Context<'_>) -> Result<Data> {
    let context = ctx.data::<GraphQLContext>()?;

    // Add context to errors
    let data = fetch_data()
        .await
        .context("Failed to fetch data from external API")?;

    // Dynamic context
    let processed = process_data(data)
        .with_context(|| format!("Failed to process {} records", data.len()))?;

    Ok(processed)
}
```

## Frontend Handling

### GraphQL Error Response

Errors include extensions with error codes:

```json
{
  "errors": [
    {
      "message": "Project with id '42' not found",
      "extensions": {
        "code": "NOT_FOUND",
        "resource": "Project"
      },
      "path": ["project"]
    }
  ]
}
```

### React/TypeScript Example

```typescript
import { useQuery } from '@apollo/client';

function ProjectView({ id }: { id: number }) {
  const { data, error } = useQuery(GET_PROJECT, {
    variables: { id }
  });

  if (error) {
    // Check error code
    const errorCode = error.graphQLErrors[0]?.extensions?.code;

    switch (errorCode) {
      case 'NOT_FOUND':
        return <NotFoundPage resource="Project" />;
      case 'UNAUTHORIZED':
        return <Redirect to="/login" />;
      case 'FORBIDDEN':
        return <ForbiddenPage />;
      default:
        return <ErrorPage message={error.message} />;
    }
  }

  return <ProjectDetails project={data.project} />;
}
```

### Apollo Error Link

```typescript
import { onError } from '@apollo/client/link/error';

const errorLink = onError(({ graphQLErrors, networkError }) => {
  if (graphQLErrors) {
    graphQLErrors.forEach(({ message, extensions, path }) => {
      const code = extensions?.code;

      switch (code) {
        case 'UNAUTHORIZED':
          // Redirect to login
          window.location.href = '/login';
          break;
        case 'FORBIDDEN':
          // Show permission denied message
          toast.error('Permission denied');
          break;
        case 'VALIDATION_FAILED':
          // Show validation error
          const field = extensions?.field;
          toast.error(`Validation error in ${field}: ${message}`);
          break;
        default:
          console.error(`[GraphQL error]: ${message}`, {
            code,
            path,
            extensions
          });
      }
    });
  }

  if (networkError) {
    console.error(`[Network error]: ${networkError}`);
    toast.error('Network error. Please check your connection.');
  }
});
```

## Migration Guide

### Step 1: Update Imports

Add to your mutation/query files:

```rust
use crate::graphql::errors::StructuredError;
```

### Step 2: Replace Error::new()

**Find:**
```rust
.ok_or_else(|| Error::new("Project not found"))?
```

**Replace with:**
```rust
.ok_or_else(|| StructuredError::not_found("Project", id))?
```

### Step 3: Add Context to Service Errors

**Find:**
```rust
.map_err(|e| Error::new(format!("Failed to import: {}", e)))?
```

**Replace with:**
```rust
.map_err(|e| StructuredError::service("ImportService", e))?
```

### Step 4: Standardize Validation Errors

**Find:**
```rust
if input.name.is_empty() {
    return Err(Error::new("Name cannot be empty"));
}
```

**Replace with:**
```rust
if input.name.is_empty() {
    return Err(StructuredError::validation("name", "Cannot be empty"));
}
```

## Best Practices

### 1. Use Specific Error Types

```rust
// ❌ Too generic
Err(Error::new("Something went wrong"))

// ✅ Specific error type
Err(StructuredError::database("save user", db_error))
```

### 2. Include Context

```rust
// ❌ No context
.map_err(|e| Error::new(e.to_string()))?

// ✅ With context
.map_err(|e| StructuredError::service("EmailService", e))?
```

### 3. Consistent Resource Names

```rust
// ❌ Inconsistent
StructuredError::not_found("project", id)
StructuredError::not_found("Projects", id)

// ✅ Consistent (singular, PascalCase)
StructuredError::not_found("Project", id)
```

### 4. Meaningful Messages

```rust
// ❌ Vague
StructuredError::validation("data", "Invalid")

// ✅ Specific
StructuredError::validation("email", "Must be a valid email address")
```

### 5. Don't Leak Internal Details

```rust
// ❌ Exposes internal details
StructuredError::internal(format!("SQL error: {}", raw_sql_error))

// ✅ Generic message for internal errors
StructuredError::internal("Failed to process request")
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphql::errors::StructuredError;

    #[test]
    fn test_not_found_error_message() {
        let error = StructuredError::not_found("Project", 42);
        assert!(error.message.contains("Project"));
        assert!(error.message.contains("42"));
        assert!(error.message.contains("not found"));
    }

    #[test]
    fn test_validation_error_extensions() {
        let error = StructuredError::validation("email", "Invalid format");
        // Extensions are tested at integration level
        assert!(error.message.contains("email"));
        assert!(error.message.contains("Invalid format"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_project_not_found_error() {
    let result = schema.execute(
        r#"query { project(id: 999999) { id name } }"#
    ).await;

    assert!(result.is_err());
    let error = &result.errors[0];
    assert_eq!(error.extensions.get("code"), Some(&"NOT_FOUND"));
}
```

## Performance Considerations

- Error creation is lightweight (no allocations beyond the message string)
- Extensions are added via async-graphql's efficient extension system
- No runtime overhead for successful paths

## Future Enhancements

1. **Trace IDs**: Add request trace IDs to errors for debugging
2. **Localization**: Support multiple languages for error messages
3. **Error Analytics**: Track error frequencies and patterns
4. **Retry Hints**: Include retry-ability hints in error extensions

---

**Status**: Implemented and ready for use
**Module**: `layercake-core/src/graphql/errors.rs`
**Next Steps**: Gradually migrate existing error handling to use StructuredError
