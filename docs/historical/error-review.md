# Error Handling Review - MCP and GraphQL API Surfaces

**Date**: 2025-10-27
**Reviewer**: Automated Error Analysis
**Scope**: Error handling patterns across MCP and GraphQL API surfaces
**Files Analyzed**: 33 files (26 GraphQL, 7 MCP)

---

## Executive Summary

The Layercake codebase implements two distinct API surfaces (GraphQL and MCP) with **parallel but inconsistent error handling patterns**. Despite having well-designed error infrastructure (`StructuredError` for GraphQL, `McpError` for MCP), the codebase shows **0% adoption** of GraphQL's structured errors and **mixed patterns** in MCP tools.

### Critical Findings

| Metric | Value | Status |
|--------|-------|--------|
| **GraphQL `Error::new()` calls** | 137 | ❌ Should use StructuredError |
| **GraphQL StructuredError usage** | 0 | ❌ Zero adoption |
| **MCP error helper availability** | 1 file only | ⚠️ Limited reuse |
| **Shared error code** | 0 lines | ❌ Complete duplication |
| **Error code compliance** | 0% | ❌ No error codes set |
| **Lost error context** | 80+ cases | ❌ High |

### Impact Assessment

- **User Experience**: Frontend cannot reliably distinguish error types
- **Debugging**: Lost error context makes troubleshooting difficult
- **Maintainability**: Duplicate error handling patterns across two systems
- **Reliability**: Inconsistent error messages confuse clients
- **Security**: Auth/permission errors not properly categorized

---

## Part 1: GraphQL Error Handling Analysis

### 1.1 Current Infrastructure

#### Existing Error Types

**File**: `layercake-core/src/graphql/errors.rs` (153 lines)

```rust
pub enum ErrorCode {
    NotFound,           // 404-equivalent
    Unauthorized,       // 401-equivalent
    Forbidden,          // 403-equivalent
    ValidationFailed,   // 400-equivalent
    DatabaseError,      // Database failures
    ServiceError,       // Service call failures
    InternalError,      // Internal errors
    Conflict,           // 409-equivalent
    BadRequest,         // 400-equivalent
}

pub struct StructuredError;

impl StructuredError {
    pub fn not_found(resource: &str, id: impl Display) -> Error;
    pub fn not_found_msg(message: impl Into<String>) -> Error;
    pub fn unauthorized(message: impl Into<String>) -> Error;
    pub fn forbidden(message: impl Into<String>) -> Error;
    pub fn validation(field: &str, message: impl Into<String>) -> Error;
    pub fn database(operation: &str, cause: impl Display) -> Error;
    pub fn service(service: &str, cause: impl Display) -> Error;
    pub fn internal(message: impl Into<String>) -> Error;
    pub fn conflict(resource: &str, message: impl Into<String>) -> Error;
    pub fn bad_request(message: impl Into<String>) -> Error;
}

pub trait ResultExt<T> {
    fn context(self, message: impl Into<String>) -> Result<T>;
    fn with_context<F>(self, f: F) -> Result<T>;
}
```

**Status**: ✅ Well-designed, comprehensive
**Problem**: ❌ Not used anywhere in codebase

---

### 1.2 Actual Error Patterns Found

#### Pattern Distribution

| Pattern | Count | Files | Compliance |
|---------|-------|-------|------------|
| `Error::new()` direct | 137 | 7 | ❌ Non-compliant |
| `StructuredError::*` | 0 | 0 | ❌ Zero usage |
| `.ok_or_else(Error::new)` | 46 | Multiple | ❌ Missing codes |
| `.map_err(Error::new)` | 67 | Multiple | ❌ Missing codes |
| `.await?` (lost context) | 80+ | Multiple | ❌ Lost context |

#### Breakdown by File

**layercake-core/src/graphql/mutations/mod.rs** (3,253 lines):
- 88+ `Error::new()` calls
- 0 `StructuredError` calls
- Highest concentration of errors

**layercake-core/src/graphql/queries/mod.rs** (822 lines):
- 14 `Error::new()` calls
- 0 `StructuredError` calls

**Other GraphQL files**:
- `types/graph.rs`: 4 instances
- `types/data_source.rs`: 4 instances
- `mutations/plan.rs`: 4 instances
- `mutations/project.rs`: 3 instances

---

### 1.3 Common Error Patterns (Anti-Patterns)

#### Anti-Pattern 1: Not Found Without Context

**Current code** (46 instances):
```rust
// mutations/mod.rs:151, 211, 234, 252, 315, 428, etc.
.ok_or_else(|| Error::new("Project not found"))?
.ok_or_else(|| Error::new("Plan not found"))?
.ok_or_else(|| Error::new("Node not found"))?
```

**Problems**:
- No error code
- No entity ID in error
- Not programmatically distinguishable

**Should be**:
```rust
.ok_or_else(|| StructuredError::not_found("Project", id))?
```

**Result**:
```json
{
  "message": "Project with id '42' not found",
  "extensions": {
    "code": "NOT_FOUND",
    "resource": "Project"
  }
}
```

---

#### Anti-Pattern 2: Database Errors Without Context

**Current code** (80+ instances):
```rust
// mutations/mod.rs:150, 172, 321, etc.
let project = projects::Entity::find_by_id(id)
    .one(&context.db)
    .await?  // <-- Error context lost!
    .ok_or_else(|| Error::new("Project not found"))?;
```

**Problems**:
- Database error propagated without operation context
- Generic "database error" message
- No indication of what operation failed

**Should be**:
```rust
let project = projects::Entity::find_by_id(id)
    .one(&context.db)
    .await
    .map_err(|e| StructuredError::database("find project by id", e))?
    .ok_or_else(|| StructuredError::not_found("Project", id))?;
```

**Result**:
```json
{
  "message": "Database error during find project by id: connection timeout",
  "extensions": {
    "code": "DATABASE_ERROR",
    "operation": "find project by id"
  }
}
```

---

#### Anti-Pattern 3: Validation Without Field Metadata

**Current code** (12+ instances):
```rust
// mutations/mod.rs:1074-1078
.map_err(|e| Error::new(format!("Email validation failed: {}", e)))?
.map_err(|e| Error::new(format!("Username validation failed: {}", e)))?
.map_err(|_| Error::new("Invalid role"))?
```

**Problems**:
- No error code
- Field name in message but not in extensions
- Frontend cannot highlight specific field

**Should be**:
```rust
.map_err(|e| StructuredError::validation("email", e))?
.map_err(|e| StructuredError::validation("username", e))?
.map_err(|_| StructuredError::validation("role", "Invalid role value"))?
```

**Result**:
```json
{
  "message": "Validation failed for 'email': invalid format",
  "extensions": {
    "code": "VALIDATION_FAILED",
    "field": "email"
  }
}
```

---

#### Anti-Pattern 4: Service Errors Without Categorization

**Current code** (30+ instances):
```rust
// mutations/mod.rs:1484, 1852, 2148, etc.
.map_err(|e| Error::new(format!("Failed to create DataSource: {}", e)))?
.map_err(|e| Error::new(format!("Failed to delete DataSource: {}", e)))?
.map_err(|e| Error::new(format!("Failed to create Graph: {}", e)))?
```

**Problems**:
- No error code
- Service name not structured
- Cannot retry based on error type

**Should be**:
```rust
.map_err(|e| StructuredError::service("DataSourceService", e))?
.map_err(|e| StructuredError::service("DataSourceService", e))?
.map_err(|e| StructuredError::service("GraphService", e))?
```

---

#### Anti-Pattern 5: Auth/Permission Errors

**Current code** (8+ instances):
```rust
// mutations/mod.rs:1133, 1145
.ok_or_else(|| Error::new("Invalid email or password"))?
return Err(Error::new("Account is deactivated"));
```

**Problems**:
- No distinction between UNAUTHORIZED (401) and FORBIDDEN (403)
- Cannot implement appropriate client-side handling
- Security risk: reveals account existence

**Should be**:
```rust
.ok_or_else(|| StructuredError::unauthorized("Invalid email or password"))?
return Err(StructuredError::forbidden("Account is deactivated"));
```

---

### 1.4 High-Frequency Error Messages

These messages appear repeatedly and should be standardized:

| Message | Count | Should Use |
|---------|-------|------------|
| "Project not found" | 12 | `StructuredError::not_found("Project", id)` |
| "Plan not found for project" | 8 | `StructuredError::not_found_msg(...)` |
| "Plan not found" | 4 | `StructuredError::not_found("Plan", id)` |
| "Collaboration not found" | 4 | `StructuredError::not_found("Collaboration", id)` |
| "Node not found" | 3 | `StructuredError::not_found("Node", id)` |
| "DataSource not found" | 3 | `StructuredError::not_found("DataSource", id)` |
| "Invalid role" | 2 | `StructuredError::validation("role", ...)` |
| "Invalid email or password" | 2 | `StructuredError::unauthorized(...)` |
| "Failed to decode base64" | 4 | `StructuredError::bad_request(...)` |
| "Failed to update layer" | 2 | `StructuredError::service(...)` |

---

### 1.5 Unused Infrastructure

#### ResultExt Trait

**Defined but barely used** (1 usage out of 67 opportunities):

```rust
// File: errors.rs
pub trait ResultExt<T> {
    fn context(self, message: impl Into<String>) -> Result<T>;
}

// Single usage in types/plan_dag.rs:842:
.context("Invalid query builder configuration")?
```

**67 locations** that should use `.context()` but use `.map_err()` instead.

---

## Part 2: MCP Error Handling Analysis

### 2.1 MCP Error Infrastructure

#### Error Type: McpError

**File**: `external-modules/axum-mcp/src/error.rs`

The MCP implementation uses a comprehensive error enum:

```rust
pub enum McpError {
    Transport { message: String },
    Protocol { message: String },
    Authentication { message: String },
    Authorization { message: String },
    ToolNotFound { tool: String },
    ToolExecution { tool: String, message: String },
    ResourceNotFound { resource: String },
    InvalidResource { message: String },
    Validation { message: String },
    Internal { message: String },
    RateLimit { message: String },
    RateLimitExceeded { message: String },
    ServerTimeout { message: String },
    ClientTimeout { message: String },
    ConnectionTimeout { message: String },
    Connection { message: String },
    ConnectionFailed { message: String },
    Session { message: String },
    Configuration { message: String },
    Serialization { message: String },
    Io { source: std::io::Error },
    Network { message: String },
}
```

**Features**:
- ✅ HTTP status codes via `status_code()`
- ✅ JSON-RPC error codes via `error_code()`
- ✅ Client-safe messages via `client_message()`
- ✅ Automatic conversions from std errors

---

### 2.2 MCP Error Patterns

#### Pattern 1: Parameter Validation (Good)

**File**: `mcp/tools/projects.rs`

```rust
let name = get_required_param(&arguments, "name")?
    .as_str()
    .ok_or_else(|| McpError::Validation {
        message: "Project name must be a string".to_string(),
    })?;
```

✅ **Good**: Uses specific `Validation` variant
✅ **Good**: Clear error message

---

#### Pattern 2: Database Errors (Inconsistent)

**File**: `mcp/tools/projects.rs`

```rust
// Pattern A: Generic Internal error
let projects = projects::Entity::find()
    .all(db)
    .await
    .map_err(|e| McpError::Internal {
        message: format!("Database error: {}", e),
    })?;

// Pattern B: Tool-specific error
let project = projects::Entity::find_by_id(project_id)
    .one(db)
    .await
    .map_err(|e| McpError::Internal {
        message: format!("Database error: {}", e),
    })?
    .ok_or_else(|| McpError::ToolExecution {
        tool: "get_project".to_string(),
        message: format!("Project with ID {} not found", project_id),
    })?;
```

⚠️ **Inconsistent**: Uses `Internal` for database errors but `ToolExecution` for not found
❌ **Problem**: Should use `ResourceNotFound` variant

---

#### Pattern 3: Helper Function (Best Practice)

**File**: `mcp/tools/graph_data.rs`

```rust
fn internal_error(context: &str, error: impl std::fmt::Display) -> McpError {
    McpError::Internal {
        message: format!("{context}: {error}"),
    }
}

// Usage:
.map_err(|e| internal_error("Failed to import nodes CSV", e))?
```

✅ **Good**: Helper reduces boilerplate
✅ **Good**: Consistent error formatting
❌ **Limited**: Only available in one file, not shared

---

#### Pattern 4: Multi-Stage Validation (Good)

**File**: `mcp/tools/auth.rs`

```rust
// Validate input
AuthService::validate_email(&email).map_err(|e| McpError::Validation {
    message: e.to_string(),
})?;

// Check business rules
if existing_user.is_some() {
    return Err(McpError::Validation {
        message: "User with this email already exists".to_string(),
    });
}
```

✅ **Good**: Uses `Validation` variant
✅ **Good**: Clear validation messages

---

### 2.3 MCP Error Usage Statistics

**Analysis of 7 MCP tool files**:

| Error Variant | Usage Count | Files |
|---------------|-------------|-------|
| `McpError::Internal` | 35+ | All tools |
| `McpError::Validation` | 12 | 4 files |
| `McpError::ToolExecution` | 8 | 3 files |
| `McpError::ResourceNotFound` | 0 | 0 files (❌ unused) |
| Helper functions | 1 | 1 file only |

**Findings**:
- ❌ `ResourceNotFound` never used (use `ToolExecution` instead)
- ❌ `InvalidResource` never used
- ⚠️ Over-reliance on `Internal` variant
- ✅ Good use of `Validation` variant
- ❌ Helper functions not shared across tools

---

## Part 3: Cross-System Comparison

### 3.1 Conceptual Alignment

Both systems have similar error categories:

| GraphQL ErrorCode | MCP McpError | HTTP Status |
|-------------------|--------------|-------------|
| `NotFound` | `ResourceNotFound` / `ToolNotFound` | 404 |
| `Unauthorized` | `Authentication` | 401 |
| `Forbidden` | `Authorization` | 403 |
| `ValidationFailed` | `Validation` | 400 |
| `BadRequest` | `InvalidResource` / `Validation` | 400 |
| `DatabaseError` | `Internal` (misused) | 500 |
| `ServiceError` | `ToolExecution` | 500 |
| `InternalError` | `Internal` | 500 |
| `Conflict` | *(no equivalent)* | 409 |

**Observation**: The error categories align well conceptually but are expressed differently.

---

### 3.2 Duplicated Error Handling Patterns

#### Duplication 1: Database Error Formatting

**GraphQL**:
```rust
.map_err(|e| Error::new(format!("Database error: {}", e)))?
```

**MCP**:
```rust
.map_err(|e| McpError::Internal {
    message: format!("Database error: {}", e),
})?
```

**Shared logic**: Both format database errors the same way
**Opportunity**: Create shared `db_error_msg(operation, error)` helper

---

#### Duplication 2: Not Found Handling

**GraphQL**:
```rust
.ok_or_else(|| Error::new("Project not found"))?
// Or better:
.ok_or_else(|| StructuredError::not_found("Project", id))?
```

**MCP**:
```rust
.ok_or_else(|| McpError::ToolExecution {
    tool: "get_project".to_string(),
    message: format!("Project with ID {} not found", project_id),
})?
```

**Shared logic**: Both construct "X not found" messages
**Opportunity**: Create shared `not_found_msg(resource, id)` helper

---

#### Duplication 3: Service Call Wrapping

**GraphQL**:
```rust
service.create_sample_project(&sample_key)
    .await
    .map_err(|e| Error::new(format!("Failed to create sample project: {}", e)))?
```

**MCP**:
```rust
service.import_layers_from_csv(graph.id, layers_csv)
    .await
    .map_err(|e| internal_error("Failed to import graph_layers CSV", e))?
```

**Shared logic**: Both wrap service errors with context
**Opportunity**: Create shared `service_error_msg(context, error)` helper

---

#### Duplication 4: Validation Messages

**GraphQL**:
```rust
.map_err(|e| Error::new(format!("Email validation failed: {}", e)))?
```

**MCP**:
```rust
AuthService::validate_email(&email).map_err(|e| McpError::Validation {
    message: e.to_string(),
})?
```

**Shared logic**: Both validate inputs and format errors
**Opportunity**: Create shared validation message helpers

---

### 3.3 Infrastructure Gaps

#### Gap 1: No Shared Error Message Builders

**Problem**: Every file reimplements error message formatting

**Impact**:
- Inconsistent error messages across APIs
- Duplicate code (50-60 lines across both systems)
- Harder to maintain consistency

**Solution**: Create `layercake-core/src/common/error_helpers.rs`

---

#### Gap 2: No Error Categorization Helpers

**Problem**: No shared logic for categorizing database errors

**Example use case**:
```rust
match db_error_kind(&err) {
    DbErrorKind::NotFound => // Return 404
    DbErrorKind::UniqueViolation => // Return 409 Conflict
    DbErrorKind::ForeignKeyViolation => // Return 400 Bad Request
    DbErrorKind::ConnectionError => // Return 503 Service Unavailable
    _ => // Return 500 Internal Error
}
```

---

#### Gap 3: No Shared Validation Logic

**Problem**: Validation messages inconsistent

**Examples**:
- GraphQL: "Email validation failed: invalid format"
- MCP: "Project name must be a string"
- Different phrasing, different structure

**Solution**: Shared validation message builders

---

## Part 4: Proposed Unified Error System

### 4.1 Shared Error Helpers Module

**Create**: `layercake-core/src/common/error_helpers.rs`

```rust
//! Shared error message builders for GraphQL and MCP APIs
//!
//! This module provides consistent error message formatting across
//! both API surfaces without forcing them to share error types.

/// Create contextualized error message
pub fn context_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("{}: {}", context, error)
}

/// Database error message
pub fn db_error_msg(operation: &str, error: impl std::fmt::Display) -> String {
    format!("Database error during {}: {}", operation, error)
}

/// Service error message
pub fn service_error_msg(service: &str, error: impl std::fmt::Display) -> String {
    format!("Service '{}' failed: {}", service, error)
}

/// Not found message with ID
pub fn not_found_msg(resource: &str, id: impl std::fmt::Display) -> String {
    format!("{} with id '{}' not found", resource, id)
}

/// Not found message without ID
pub fn not_found_simple(resource: &str) -> String {
    format!("{} not found", resource)
}

/// Validation message builders
pub mod validation {
    pub fn required_field(field: &str) -> String {
        format!("Missing required parameter: {}", field)
    }

    pub fn invalid_type(field: &str, expected: &str) -> String {
        format!("{} must be a {}", field, expected)
    }

    pub fn invalid_format(field: &str, message: &str) -> String {
        format!("Invalid format for '{}': {}", field, message)
    }

    pub fn already_exists(resource: &str, identifier: &str) -> String {
        format!("{} '{}' already exists", resource, identifier)
    }

    pub fn out_of_range(field: &str, min: impl std::fmt::Display, max: impl std::fmt::Display) -> String {
        format!("{} must be between {} and {}", field, min, max)
    }
}

/// Authentication/Authorization message builders
pub mod auth {
    pub fn invalid_credentials() -> String {
        "Invalid email or password".to_string()
    }

    pub fn account_disabled() -> String {
        "Account is deactivated".to_string()
    }

    pub fn session_expired() -> String {
        "Session has expired".to_string()
    }

    pub fn insufficient_permissions(action: &str) -> String {
        format!("Insufficient permissions to {}", action)
    }
}
```

---

### 4.2 Database Error Categorization

**Create**: `layercake-core/src/common/db_errors.rs`

```rust
//! Database error categorization and message formatting

use sea_orm::DbErr;

/// Categories of database errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbErrorKind {
    /// Record not found (query returned no results)
    NotFound,
    /// Unique constraint violation
    UniqueViolation,
    /// Foreign key constraint violation
    ForeignKeyViolation,
    /// Database connection error
    ConnectionError,
    /// Query timeout
    Timeout,
    /// Transaction deadlock
    Deadlock,
    /// Unknown/other database error
    Unknown,
}

impl DbErrorKind {
    /// Categorize a sea_orm database error
    pub fn from_db_err(err: &DbErr) -> Self {
        use sea_orm::DbErr;

        match err {
            DbErr::RecordNotFound(_) => Self::NotFound,
            DbErr::Conn(msg) if msg.contains("timeout") => Self::Timeout,
            DbErr::Conn(_) => Self::ConnectionError,
            DbErr::Exec(msg) | DbErr::Query(msg) => {
                let msg_lower = msg.to_lowercase();
                if msg_lower.contains("unique") || msg_lower.contains("duplicate") {
                    Self::UniqueViolation
                } else if msg_lower.contains("foreign key") {
                    Self::ForeignKeyViolation
                } else if msg_lower.contains("deadlock") {
                    Self::Deadlock
                } else {
                    Self::Unknown
                }
            }
            _ => Self::Unknown,
        }
    }

    /// Get appropriate HTTP status code for this error kind
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::UniqueViolation => 409, // Conflict
            Self::ForeignKeyViolation => 400, // Bad Request
            Self::ConnectionError => 503, // Service Unavailable
            Self::Timeout => 504, // Gateway Timeout
            Self::Deadlock => 503, // Service Unavailable (retry)
            Self::Unknown => 500, // Internal Server Error
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::ConnectionError | Self::Timeout | Self::Deadlock)
    }
}

/// Format database error with operation context
pub fn format_db_error(operation: &str, err: &DbErr) -> (DbErrorKind, String) {
    let kind = DbErrorKind::from_db_err(err);

    let message = match kind {
        DbErrorKind::NotFound => format!("{}: record not found", operation),
        DbErrorKind::UniqueViolation => format!("{}: duplicate key violation", operation),
        DbErrorKind::ForeignKeyViolation => format!("{}: foreign key constraint violation", operation),
        DbErrorKind::ConnectionError => format!("{}: database connection failed", operation),
        DbErrorKind::Timeout => format!("{}: query timeout", operation),
        DbErrorKind::Deadlock => format!("{}: transaction deadlock", operation),
        DbErrorKind::Unknown => format!("{}: database error - {}", operation, err),
    };

    (kind, message)
}
```

---

### 4.3 Usage Examples

#### GraphQL Usage

```rust
use crate::common::error_helpers::*;
use crate::common::db_errors::*;

// Not found errors
.ok_or_else(|| StructuredError::not_found_msg(
    &not_found_msg("Project", id)
))?

// Database errors with categorization
let project = projects::Entity::find_by_id(id)
    .one(&context.db)
    .await
    .map_err(|e| {
        let (kind, message) = format_db_error("find project by id", &e);
        match kind {
            DbErrorKind::NotFound => StructuredError::not_found("Project", id),
            _ => StructuredError::database("find project", e),
        }
    })?;

// Service errors
.map_err(|e| StructuredError::service_msg(
    &service_error_msg("DataSourceService", e)
))?

// Validation errors
.map_err(|e| StructuredError::validation("email",
    &validation::invalid_format("email", &e.to_string())
))?
```

#### MCP Usage

```rust
use crate::common::error_helpers::*;
use crate::common::db_errors::*;

// Not found errors
.ok_or_else(|| McpError::ResourceNotFound {
    resource: not_found_msg("Project", id),
})?

// Database errors with categorization
let project = projects::Entity::find_by_id(project_id)
    .one(db)
    .await
    .map_err(|e| {
        let (kind, message) = format_db_error("find project", &e);
        match kind {
            DbErrorKind::NotFound => McpError::ResourceNotFound {
                resource: format!("Project {}", project_id),
            },
            _ => McpError::Internal { message },
        }
    })?;

// Service errors
.map_err(|e| McpError::ToolExecution {
    tool: "import_graph".to_string(),
    message: service_error_msg("GraphService", e),
})?

// Validation errors
.map_err(|_| McpError::Validation {
    message: validation::invalid_type("name", "string"),
})?
```

---

## Part 5: Implementation Roadmap

### Phase 1: Foundation (Week 1) - IMMEDIATE PRIORITY

**Goal**: Create shared error infrastructure
**Effort**: 4-6 hours
**Risk**: Low

#### Tasks:

1. **Create error helpers module**
   - File: `layercake-core/src/common/error_helpers.rs`
   - Implement message builders
   - Add comprehensive doc comments
   - Write unit tests for message formatting

2. **Create database error module**
   - File: `layercake-core/src/common/db_errors.rs`
   - Implement `DbErrorKind` enum
   - Add error categorization logic
   - Test with real database errors

3. **Create common module**
   - File: `layercake-core/src/common/mod.rs`
   - Export error_helpers and db_errors
   - Document usage patterns

4. **Update lib.rs**
   - Add `pub mod common;`
   - Make modules accessible

**Deliverables**:
- ✅ Shared error message builders
- ✅ Database error categorization
- ✅ Comprehensive tests
- ✅ Usage documentation

**Success Criteria**:
- All tests pass
- No regressions in existing error handling
- Documentation complete

---

### Phase 2: GraphQL Migration (Weeks 2-3) - HIGH PRIORITY

**Goal**: Migrate GraphQL to use StructuredError
**Effort**: 12-16 hours
**Risk**: Medium (changes 137 error sites)

#### Stage 1: High-Value Files (Week 2)

**Target files**:
1. `graphql/mutations/mod.rs` (88+ errors)
2. `graphql/queries/mod.rs` (14 errors)

**Approach**:
- Replace error patterns systematically:
  1. Not found errors → `StructuredError::not_found*`
  2. Validation errors → `StructuredError::validation`
  3. Database errors → `StructuredError::database`
  4. Service errors → `StructuredError::service`
  5. Auth errors → `StructuredError::unauthorized/forbidden`

**Testing**:
- Run full test suite after each file
- Test error responses with GraphQL playground
- Verify error codes in responses

#### Stage 2: Remaining Files (Week 3)

**Target files**:
3. `graphql/types/graph.rs` (4 errors)
4. `graphql/types/data_source.rs` (4 errors)
5. `graphql/mutations/plan.rs` (4 errors)
6. `graphql/mutations/project.rs` (3 errors)

**Deliverables**:
- ✅ 100% StructuredError adoption in GraphQL
- ✅ Consistent error codes on all errors
- ✅ Updated integration tests
- ✅ Error handling documentation

**Success Criteria**:
- Zero direct `Error::new()` calls in GraphQL
- All errors have error codes
- All tests pass
- No regression in error quality

---

### Phase 3: MCP Enhancement (Week 4) - MEDIUM PRIORITY

**Goal**: Improve MCP error consistency
**Effort**: 6-8 hours
**Risk**: Low

#### Tasks:

1. **Create MCP error helpers**
   - File: `mcp/tools/error_utils.rs`
   - Wrap common error_helpers for MCP usage
   - Make available to all tool files

2. **Standardize database error handling**
   - Use `db_errors::format_db_error()`
   - Map to appropriate MCP error variants
   - Use `ResourceNotFound` instead of `ToolExecution` for missing records

3. **Fix MCP error variant usage**
   - Replace `Internal` with `ResourceNotFound` for missing records
   - Use `InvalidResource` for malformed inputs
   - Consistent use of `Validation` variant

4. **Update all MCP tools**
   - projects.rs
   - plans.rs
   - graph_data.rs
   - auth.rs
   - analysis.rs (stub file)

**Deliverables**:
- ✅ Shared error helpers in MCP
- ✅ Consistent error categorization
- ✅ All tools use appropriate variants
- ✅ Documentation

**Success Criteria**:
- Consistent error messages across MCP tools
- Proper use of error variants
- All tests pass
- Error responses improved

---

### Phase 4: Advanced Features (Week 5-6) - LOW PRIORITY

**Goal**: Add error telemetry and advanced features
**Effort**: 8-10 hours
**Risk**: Low

#### Tasks:

1. **Error telemetry**
   - Add tracing for errors by type
   - Track error frequency metrics
   - Monitor error code distribution

2. **Error testing utilities**
   - Create fixtures for common errors
   - Add error response validators
   - Build error scenario tests

3. **Error documentation**
   - Document all error codes
   - Create error handling guide
   - Add examples to API docs

4. **Lint rules**
   - Add clippy lint to prevent `Error::new()` in GraphQL
   - Enforce StructuredError usage
   - Add CI checks

**Deliverables**:
- ✅ Error telemetry system
- ✅ Comprehensive error tests
- ✅ Complete documentation
- ✅ CI enforcement

**Success Criteria**:
- Errors tracked in metrics
- Test coverage >90% for error paths
- Documentation complete
- CI prevents regressions

---

## Part 6: Migration Strategy

### 6.1 File-by-File Migration Approach

**Order of migration** (by impact):

1. ✅ **mutations/mod.rs** (88+ errors) - Highest impact
2. ✅ **queries/mod.rs** (14 errors) - High impact
3. ✅ **types/graph.rs** (4 errors) - Medium impact
4. ✅ **types/data_source.rs** (4 errors) - Medium impact
5. ✅ **mutations/plan.rs** (4 errors) - Medium impact
6. ✅ **mutations/project.rs** (3 errors) - Medium impact

### 6.2 Error Pattern Replacement Guide

#### Pattern 1: Simple Not Found

**Before**:
```rust
.ok_or_else(|| Error::new("Project not found"))?
```

**After**:
```rust
.ok_or_else(|| StructuredError::not_found_msg("Project not found"))?
```

**Better** (with ID):
```rust
.ok_or_else(|| StructuredError::not_found("Project", id))?
```

---

#### Pattern 2: Database Operations

**Before**:
```rust
let project = projects::Entity::find_by_id(id)
    .one(&context.db)
    .await?
    .ok_or_else(|| Error::new("Project not found"))?;
```

**After**:
```rust
use crate::common::db_errors::*;

let project = projects::Entity::find_by_id(id)
    .one(&context.db)
    .await
    .map_err(|e| StructuredError::database("find project", e))?
    .ok_or_else(|| StructuredError::not_found("Project", id))?;
```

**Advanced** (with categorization):
```rust
let project = projects::Entity::find_by_id(id)
    .one(&context.db)
    .await
    .map_err(|e| {
        let (kind, message) = format_db_error("find project", &e);
        match kind {
            DbErrorKind::NotFound => StructuredError::not_found("Project", id),
            _ => StructuredError::database("find project", e),
        }
    })?;
```

---

#### Pattern 3: Validation Errors

**Before**:
```rust
.map_err(|e| Error::new(format!("Email validation failed: {}", e)))?
```

**After**:
```rust
.map_err(|e| StructuredError::validation("email", e))?
```

---

#### Pattern 4: Service Calls

**Before**:
```rust
.map_err(|e| Error::new(format!("Failed to create DataSource: {}", e)))?
```

**After**:
```rust
.map_err(|e| StructuredError::service("DataSourceService", e))?
```

---

### 6.3 Testing Strategy

#### Unit Tests

For each migrated file:
1. Test error messages are formatted correctly
2. Test error codes are set
3. Test error extensions include expected fields

**Example test**:
```rust
#[tokio::test]
async fn test_project_not_found_error() {
    let result = some_query(999); // Non-existent ID

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Check message
    assert!(err.message.contains("Project"));
    assert!(err.message.contains("999"));

    // Check error code extension
    assert_eq!(err.extensions.get("code"), Some("NOT_FOUND"));
    assert_eq!(err.extensions.get("resource"), Some("Project"));
}
```

#### Integration Tests

Test complete request/response cycles:
```rust
#[tokio::test]
async fn test_graphql_error_response() {
    let query = r#"
        query {
            project(id: 999) {
                id
                name
            }
        }
    "#;

    let response = execute_query(query).await;

    assert!(response.errors.is_some());
    let error = &response.errors.unwrap()[0];

    assert_eq!(error.extensions.get("code"), Some("NOT_FOUND"));
}
```

---

## Part 7: Risk Assessment & Mitigation

### 7.1 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Breaking API contract** | Low | High | Thorough testing, staged rollout |
| **Regression in error quality** | Medium | Medium | Comprehensive test coverage |
| **Performance degradation** | Very Low | Low | Error construction is not hot path |
| **Incomplete migration** | Medium | Medium | Linting rules, CI checks |
| **Message format changes** | Low | Medium | Document changes, version API |

### 7.2 Mitigation Strategies

#### Strategy 1: Staged Rollout

1. **Phase 1**: Internal testing with shared helpers
2. **Phase 2**: Migrate one file at a time with tests
3. **Phase 3**: Monitor error rates in staging
4. **Phase 4**: Roll out to production gradually

#### Strategy 2: Backwards Compatibility

Ensure error messages remain understandable:
- Keep core message format similar
- Add error codes as extensions (non-breaking)
- Document error code meanings

#### Strategy 3: Comprehensive Testing

- Unit tests for error construction
- Integration tests for API responses
- Manual testing in GraphQL playground
- Automated error response validation

#### Strategy 4: Rollback Plan

- Keep git history clean (one file per commit)
- Can revert individual file migrations
- Monitor error rates after deployment
- Quick rollback capability if issues detected

---

## Part 8: Success Metrics

### 8.1 Quantitative Metrics

**Before Migration**:
- StructuredError usage: 0%
- Error codes set: 0%
- Shared error code: 0 lines
- Direct Error::new() calls: 137

**Target After Migration**:
- StructuredError usage: 100%
- Error codes set: 100%
- Shared error code: ~200 lines
- Direct Error::new() calls: 0
- Code deduplication: 50-60 lines removed

### 8.2 Qualitative Improvements

**Developer Experience**:
- ✅ Consistent error handling patterns
- ✅ Clear documentation
- ✅ Reduced boilerplate code
- ✅ Better error messages

**User Experience**:
- ✅ Consistent error messages across APIs
- ✅ Programmatically distinguishable errors
- ✅ Better error information for debugging
- ✅ Appropriate HTTP status codes

**Maintenance**:
- ✅ Single source of truth for error messages
- ✅ Easier to update error patterns
- ✅ Reduced code duplication
- ✅ Better test coverage

---

## Part 9: Recommendations Summary

### Immediate Actions (This Week)

1. ✅ **Create shared error helpers** (`common/error_helpers.rs`)
2. ✅ **Create database error module** (`common/db_errors.rs`)
3. ✅ **Start GraphQL migration** with mutations/mod.rs
4. ✅ **Add error code tests**

### Short-Term (2-3 Weeks)

5. ✅ **Complete GraphQL migration** (all 137 error sites)
6. ✅ **Update GraphQL tests** for new error format
7. ✅ **Improve MCP error consistency**
8. ✅ **Document error handling patterns**

### Medium-Term (4-6 Weeks)

9. ✅ **Add error telemetry**
10. ✅ **Create error testing utilities**
11. ✅ **Build error documentation**
12. ✅ **Implement lint rules**

### Long-Term (Continuous)

13. ✅ **Monitor error rates** and quality
14. ✅ **Refine error messages** based on feedback
15. ✅ **Maintain shared error infrastructure**
16. ✅ **Keep error patterns consistent**

---

## Conclusion

The Layercake codebase has **well-designed error infrastructure that is not being used**. Despite having `StructuredError` and comprehensive error codes, the GraphQL layer uses direct `Error::new()` calls 137 times with zero structured error usage.

### Key Findings:

1. ❌ **0% adoption** of StructuredError in GraphQL
2. ❌ **137 error sites** without error codes
3. ❌ **80+ cases** of lost error context
4. ⚠️ **Mixed patterns** in MCP tools
5. ❌ **No shared code** between GraphQL and MCP error handling

### Recommended Approach:

**Phase 1** (Week 1): Build shared error infrastructure
- Create `common/error_helpers.rs`
- Create `common/db_errors.rs`
- Establish foundation for both systems

**Phase 2** (Weeks 2-3): Migrate GraphQL to StructuredError
- Replace all 137 `Error::new()` calls
- Add error codes to all errors
- Improve error context preservation

**Phase 3** (Week 4): Enhance MCP consistency
- Standardize MCP tool error handling
- Use shared error helpers
- Fix error variant usage

**Phase 4** (Weeks 5-6): Add advanced features
- Error telemetry
- Comprehensive testing
- Documentation and CI enforcement

### Expected Impact:

- **Developer productivity**: 30-40% reduction in error handling boilerplate
- **Error quality**: 100% of errors will have codes and proper categorization
- **Debugging**: Significantly improved error context preservation
- **Consistency**: Unified error messages across both API surfaces
- **Maintenance**: Single source of truth for error patterns

**Total Effort**: 30-40 developer-hours over 6 weeks
**Risk**: Low-Medium (mitigated by staged approach and comprehensive testing)
**ROI**: High (improves UX, debugging, and maintainability significantly)

---

**Review Status**: ✅ Complete
**Implementation Status**: 🔄 In Progress
**Maintainer**: Development Team

---

## Implementation Progress

### Phase 1: Foundation - ✅ COMPLETED (2025-10-27)

**Status**: Complete
**Effort**: 4 hours
**Commit**: 027252fa

#### Deliverables:

1. ✅ **common/error_helpers.rs** (370 lines)
   - Message builders for all error types
   - Validation helpers
   - Auth/permission message builders
   - 16 comprehensive tests

2. ✅ **common/db_errors.rs** (390 lines)
   - `DbErrorKind` enum with 7 categories
   - Database error categorization from sea_orm::DbErr
   - HTTP status code mapping
   - Retryable/client/server error detection
   - 12 comprehensive tests

3. ✅ **common/mod.rs**
   - Module structure and re-exports
   - Convenient API access

#### Test Results:
- ✅ All 28 tests passing
- ✅ Zero compilation errors
- ✅ 100% API coverage

#### Impact:
- Created foundation for consistent error handling
- Established shared error vocabulary
- Ready for GraphQL and MCP adoption

**Next**: Begin Phase 2 - GraphQL migration

---

### Phase 2: GraphQL Migration - 🔄 IN PROGRESS

**Status**: Not started
**Target**: Migrate 137 error sites to StructuredError
**Priority**: High
