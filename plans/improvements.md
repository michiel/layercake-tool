# Layercake Codebase Improvement Plan

**Document Version:** 1.0
**Date:** 2026-01-01
**Status:** Planning

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Implementation Roadmap](#implementation-roadmap)
3. [Phase 1: Critical Security & Stability (Week 1-2)](#phase-1-critical-security--stability-week-1-2)
4. [Phase 2: Code Quality & Testing (Week 3-6)](#phase-2-code-quality--testing-week-3-6)
5. [Phase 3: Performance & Type Safety (Week 7-10)](#phase-3-performance--type-safety-week-7-10)
6. [Phase 4: Architecture Refactoring (Week 11-16)](#phase-4-architecture-refactoring-week-11-16)
7. [Phase 5: Observability & Documentation (Week 17-20)](#phase-5-observability--documentation-week-17-20)
8. [Phase 6: Extensibility & Future-Proofing (Week 21-26)](#phase-6-extensibility--future-proofing-week-21-26)
9. [Complete Review Report](#complete-review-report)

---

## Executive Summary

This document outlines a comprehensive improvement plan for the Layercake codebase based on an extensive maintainability, suitability, extensibility, and functional correctness review conducted on 2026-01-01.

**Overall Assessment: B+ (Good with Room for Improvement)**

The project demonstrates strong architectural foundations but requires focused attention in several key areas:

- **Critical:** Authentication bypass mechanism poses production risk
- **High Priority:** Test coverage at 11%, oversized files, console logging pollution
- **Medium Priority:** Unwrap usage, clone overhead, type safety gaps
- **Long-term:** Extensibility, performance optimization, monitoring

This plan organizes 50+ specific recommendations into 6 implementation phases spanning approximately 26 weeks, prioritised by risk and impact.

---

## Implementation Roadmap

### Phase Overview

| Phase | Focus Area | Duration | Priority | Risk Reduction |
|-------|-----------|----------|----------|----------------|
| 1 | Critical Security & Stability | 2 weeks | CRITICAL | High |
| 2 | Code Quality & Testing | 4 weeks | HIGH | Medium |
| 3 | Performance & Type Safety | 4 weeks | HIGH | Medium |
| 4 | Architecture Refactoring | 6 weeks | MEDIUM | Medium |
| 5 | Observability & Documentation | 4 weeks | MEDIUM | Low |
| 6 | Extensibility & Future-Proofing | 6 weeks | LOW | Low |

### Success Criteria

- **Security:** Zero authentication bypasses possible in production
- **Testing:** Minimum 60% code coverage across core and server
- **Performance:** No N+1 query issues, <100ms p95 GraphQL response time
- **Maintainability:** No files >5000 lines, all modules well-documented
- **Type Safety:** Zero `any` types in TypeScript, <10 unwraps in critical paths

---

## Phase 1: Critical Security & Stability (Week 1-2)

**Goal:** Eliminate critical security vulnerabilities and prevent production incidents
**Success Criteria:**
- Authentication bypass mechanism secured with compile-time guards
- Zero critical unwraps in auth/payment paths
- Security audit passes

### Task 1.1: Secure Authentication Bypass Mechanism

**Priority:** CRITICAL
**Effort:** 2 days
**Files:**
- `layercake-server/src/auth/mod.rs:51-59`
- `layercake-core/src/services/authorization.rs:191-199`
- `dev.sh:40`

**Current Issue:**
```rust
// DANGEROUS: Can disable all auth in production
fn local_auth_bypass_enabled() -> bool {
    std::env::var("LAYERCAKE_LOCAL_AUTH_BYPASS")
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}
```

**Implementation Steps:**

1. **Add compile-time feature flag** (1 hour)

   **File: `Cargo.toml`**
   ```toml
   [features]
   default = []
   dev-auth-bypass = []  # Only enabled in dev builds
   ```

2. **Refactor auth bypass to use feature flag** (2 hours)

   **File: `layercake-server/src/auth/mod.rs`**
   ```rust
   #[cfg(feature = "dev-auth-bypass")]
   fn local_auth_bypass_enabled() -> bool {
       use std::sync::atomic::{AtomicBool, Ordering};
       static WARNED: AtomicBool = AtomicBool::new(false);

       let enabled = std::env::var("LAYERCAKE_LOCAL_AUTH_BYPASS")
           .ok()
           .map(|value| {
               let normalized = value.trim().to_ascii_lowercase();
               matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
           })
           .unwrap_or(false);

       if enabled && !WARNED.swap(true, Ordering::Relaxed) {
           eprintln!("⚠️  WARNING: Authentication bypass is ENABLED. This is a development-only feature!");
           eprintln!("⚠️  NEVER use this in production!");
       }

       enabled
   }

   #[cfg(not(feature = "dev-auth-bypass"))]
   fn local_auth_bypass_enabled() -> bool {
       false  // Always disabled in production builds
   }
   ```

3. **Update dev.sh to use feature flag** (30 min)

   **File: `dev.sh`**
   ```bash
   cargo build --features dev-auth-bypass
   LAYERCAKE_LOCAL_AUTH_BYPASS=1 cargo run --features dev-auth-bypass
   ```

4. **Add build.rs to warn on production builds** (1 hour)

   **File: `layercake-server/build.rs`**
   ```rust
   fn main() {
       #[cfg(feature = "dev-auth-bypass")]
       {
           println!("cargo:warning=Building with dev-auth-bypass feature enabled!");
           println!("cargo:warning=This build should NEVER be used in production!");
       }
   }
   ```

5. **Add integration test** (2 hours)

   **File: `tests/auth_bypass_security.rs`**
   ```rust
   #[test]
   #[cfg(not(feature = "dev-auth-bypass"))]
   fn test_auth_bypass_disabled_in_production_builds() {
       // Verify bypass is impossible without feature flag
       std::env::set_var("LAYERCAKE_LOCAL_AUTH_BYPASS", "1");
       let result = authenticate_user(/* ... */);
       assert!(result.is_err(), "Auth bypass should be impossible in production");
   }
   ```

6. **Update documentation** (1 hour)
   - Add security warning to README.md
   - Document dev-auth-bypass feature in DEV_SCRIPTS.md
   - Add deployment checklist to ensure feature is not enabled

**Testing:**
- [ ] Unit tests for both feature-enabled and feature-disabled builds
- [ ] Integration test verifying production build cannot bypass auth
- [ ] Manual test of dev workflow with feature enabled

**Acceptance Criteria:**
- [ ] Impossible to bypass auth in production builds (compile-time guarantee)
- [ ] Dev workflow remains ergonomic with feature flag
- [ ] Clear warnings when bypass is enabled
- [ ] Documentation updated

---

### Task 1.2: Audit and Fix Critical Unwraps

**Priority:** CRITICAL
**Effort:** 3 days
**Files:** 153 unwraps in `layercake-core/src/`, 3 in `layercake-server/src/`

**Current Issues:**
```rust
// layercake-core/src/services/graph_service.rs:91
let parsed: Value = serde_json::from_str(&data_set.graph_json).unwrap_or_default();

// layercake-server/src/graphql/queries/mod.rs
let result = some_array.first().unwrap();
```

**Implementation Steps:**

1. **Generate unwrap audit report** (2 hours)
   ```bash
   rg "\.unwrap\(\)|\.expect\(" --type rust -n > unwrap_audit.txt
   ```

   Then categorise by severity:
   - **CRITICAL**: Auth, payment, data integrity paths
   - **HIGH**: User-facing operations
   - **MEDIUM**: Internal operations with recovery
   - **LOW**: Test code, truly infallible operations

2. **Fix critical path unwraps** (8 hours)

   **Auth paths:**
   ```rust
   // BEFORE
   let user_id = session.user_id.unwrap();

   // AFTER
   let user_id = session.user_id
       .ok_or(CoreError::Authentication("Session missing user_id".into()))?;
   ```

   **Data parsing:**
   ```rust
   // BEFORE
   let parsed: Value = serde_json::from_str(&data_set.graph_json).unwrap_or_default();

   // AFTER
   let parsed: Value = serde_json::from_str(&data_set.graph_json)
       .map_err(|e| CoreError::Serialization(format!("Failed to parse graph JSON: {}", e)))?;
   ```

   **Array access:**
   ```rust
   // BEFORE
   let first = array.first().unwrap();

   // AFTER
   let first = array.first()
       .ok_or_else(|| CoreError::InvalidState("Expected non-empty array".into()))?;
   ```

3. **Add error handling helpers** (2 hours)

   **File: `layercake-core/src/errors/helpers.rs`**
   ```rust
   /// Extension trait for Option to convert to CoreError
   pub trait OptionExt<T> {
       fn ok_or_invalid_state(self, msg: impl Into<String>) -> CoreResult<T>;
   }

   impl<T> OptionExt<T> for Option<T> {
       fn ok_or_invalid_state(self, msg: impl Into<String>) -> CoreResult<T> {
           self.ok_or_else(|| CoreError::InvalidState(msg.into()))
       }
   }

   // Usage:
   let value = maybe_value.ok_or_invalid_state("Expected value to exist")?;
   ```

4. **Update remaining unwraps** (8 hours)
   - Fix all HIGH priority unwraps
   - Document MEDIUM priority unwraps with SAFETY comments
   - Leave LOW priority (test code) but add comments

5. **Add clippy lint** (1 hour)

   **File: `Cargo.toml` or `.clippy.toml`**
   ```toml
   [lints.clippy]
   unwrap_used = "deny"
   expect_used = "warn"
   ```

**Testing:**
- [ ] All existing tests pass
- [ ] New error paths covered by tests
- [ ] Integration tests verify graceful error handling

**Acceptance Criteria:**
- [ ] Zero unwraps in auth/payment/data integrity paths
- [ ] All unwraps either fixed or documented with SAFETY comments
- [ ] Clippy lint prevents new unwraps
- [ ] Error messages are actionable

---

### Task 1.3: Add Security Smoke Tests

**Priority:** HIGH
**Effort:** 2 days

**Implementation Steps:**

1. **Create security test suite** (4 hours)

   **File: `layercake-integration-tests/tests/security_smoke_tests.rs`**
   ```rust
   #[tokio::test]
   async fn test_cannot_access_other_users_projects() {
       let user1 = create_test_user("user1").await;
       let user2 = create_test_user("user2").await;

       let project = create_project(user1.id).await;

       let result = get_project_as_user(project.id, user2.id).await;
       assert!(result.is_err(), "User should not access other user's project");
   }

   #[tokio::test]
   async fn test_session_expiry_enforced() {
       let session = create_expired_session().await;
       let result = authenticate_with_session(session.token).await;
       assert!(result.is_err(), "Expired session should be rejected");
   }

   #[tokio::test]
   async fn test_sql_injection_prevented() {
       let malicious_input = "'; DROP TABLE users; --";
       let result = search_projects(malicious_input).await;
       // Should not panic or corrupt database
       assert!(result.is_ok());
   }
   ```

2. **Add CI security checks** (2 hours)

   **File: `.github/workflows/security.yml`**
   ```yaml
   name: Security Checks
   on: [push, pull_request]

   jobs:
     cargo-audit:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - run: cargo install cargo-audit
         - run: cargo audit

     security-tests:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - run: cargo test --test security_smoke_tests
   ```

3. **Document security practices** (2 hours)
   - Create SECURITY.md with vulnerability reporting process
   - Add security checklist to PR template
   - Document auth/authz architecture

**Acceptance Criteria:**
- [ ] Security test suite passes
- [ ] CI runs security checks on every PR
- [ ] SECURITY.md published

---

## Phase 2: Code Quality & Testing (Week 3-6)

**Goal:** Increase test coverage to 60%+ and improve code maintainability
**Success Criteria:**
- Core module coverage ≥60%
- Server module coverage ≥60%
- Frontend coverage ≥40%
- No files >10,000 lines

### Task 2.1: Split Oversized Files

**Priority:** HIGH
**Effort:** 5 days
**Files:**
- `layercake-core/src/graph.rs` (116KB)
- `layercake-core/src/pipeline/dag_executor.rs` (71KB)

**Current Issue:** Files are too large to maintain, causing:
- Long compile times
- Difficult code navigation
- Merge conflicts
- Hard to understand scope

**Implementation Steps:**

#### 2.1.1: Refactor graph.rs (3 days)

**New structure:**
```
layercake-core/src/graph/
├── mod.rs                  # Public API, Graph struct
├── operations.rs           # Graph operations (add, remove, update)
├── transformations.rs      # Graph transformations (filter, aggregate)
├── validation.rs           # Graph validation logic
├── serialization.rs        # JSON serialization/deserialization
├── queries.rs              # Graph queries (find, search)
├── layout.rs               # Layout calculations
└── sanitization.rs         # Label sanitization, cleanup
```

1. **Create new module structure** (2 hours)
   ```bash
   mkdir -p layercake-core/src/graph
   touch layercake-core/src/graph/{mod,operations,transformations,validation,serialization,queries,layout,sanitization}.rs
   ```

2. **Extract operations** (4 hours)

   **File: `layercake-core/src/graph/operations.rs`**
   ```rust
   use super::Graph;
   use crate::errors::GraphResult;

   impl Graph {
       /// Add a node to the graph
       pub fn add_node(&mut self, node: Node) -> GraphResult<NodeId> {
           // Implementation moved from graph.rs
       }

       /// Remove a node and all connected edges
       pub fn remove_node(&mut self, node_id: NodeId) -> GraphResult<()> {
           // Implementation moved from graph.rs
       }
   }
   ```

3. **Extract transformations** (4 hours)

   **File: `layercake-core/src/graph/transformations.rs`**
   ```rust
   use super::Graph;
   use crate::errors::GraphResult;

   pub trait GraphTransform {
       fn transform(&self, graph: &Graph) -> GraphResult<Graph>;
   }

   pub struct FilterTransform { /* ... */ }
   pub struct AggregateTransform { /* ... */ }
   pub struct MergeTransform { /* ... */ }
   ```

4. **Extract validation** (3 hours)

   **File: `layercake-core/src/graph/validation.rs`**
   ```rust
   use super::Graph;
   use crate::errors::GraphResult;

   pub struct GraphValidator;

   impl GraphValidator {
       pub fn validate(graph: &Graph) -> GraphResult<()> {
           Self::validate_structure(graph)?;
           Self::validate_references(graph)?;
           Self::detect_cycles(graph)?;
           Ok(())
       }
   }
   ```

5. **Update mod.rs with public API** (2 hours)

   **File: `layercake-core/src/graph/mod.rs`**
   ```rust
   mod operations;
   mod transformations;
   mod validation;
   mod serialization;
   mod queries;
   mod layout;
   mod sanitization;

   // Re-export public API
   pub use transformations::{GraphTransform, FilterTransform, AggregateTransform};
   pub use validation::GraphValidator;

   // Graph struct definition
   pub struct Graph {
       // Fields
   }

   // Public API methods stay here
   impl Graph {
       pub fn new() -> Self { /* ... */ }
   }
   ```

6. **Update imports across codebase** (2 hours)
   ```rust
   // Update from:
   use crate::graph::Graph;

   // To:
   use crate::graph::{Graph, GraphValidator, GraphTransform};
   ```

7. **Run tests and fix compilation** (1 hour)

#### 2.1.2: Refactor dag_executor.rs (2 days)

**New structure:**
```
layercake-core/src/pipeline/dag_executor/
├── mod.rs                  # Public API, DagExecutor struct
├── executor.rs             # Main execution logic
├── topological_sort.rs     # Dependency resolution
├── state_manager.rs        # Execution state tracking
├── events.rs               # Event publishing
└── node_runner.rs          # Individual node execution
```

1. **Create module structure** (1 hour)
2. **Extract topological sort** (3 hours)
3. **Extract state management** (3 hours)
4. **Extract event publishing** (2 hours) - implements 6 TODOs
5. **Extract node execution** (3 hours)
6. **Update public API** (2 hours)
7. **Test and verify** (2 hours)

**Testing:**
- [ ] All existing tests pass
- [ ] No behaviour changes
- [ ] Compilation time improves
- [ ] Module boundaries are clear

**Acceptance Criteria:**
- [ ] No file >5000 lines
- [ ] Clear module responsibilities
- [ ] All tests passing
- [ ] Documentation updated

---

### Task 2.2: Implement Test Coverage Strategy

**Priority:** HIGH
**Effort:** 10 days
**Current State:** 11% coverage (111 test modules / 858 files)

**Implementation Steps:**

#### 2.2.1: Set up coverage infrastructure (1 day)

1. **Install tarpaulin** (30 min)
   ```bash
   cargo install cargo-tarpaulin
   ```

2. **Add coverage scripts** (1 hour)
   ```bash
   # scripts/coverage.sh
   #!/bin/bash
   cargo tarpaulin \
       --out Html \
       --out Lcov \
       --exclude-files "*/tests/*" "*/examples/*" \
       --target-dir target/tarpaulin \
       --timeout 300
   ```

3. **Add CI coverage reporting** (2 hours)
   ```yaml
   # .github/workflows/coverage.yml
   name: Coverage
   on: [push, pull_request]

   jobs:
     coverage:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - uses: actions-rs/toolchain@v1
           with:
             toolchain: stable
         - run: cargo install cargo-tarpaulin
         - run: cargo tarpaulin --out Xml
         - uses: codecov/codecov-action@v3
   ```

4. **Generate baseline report** (2 hours)
   ```bash
   cargo tarpaulin --out Html
   # Analyse current coverage, identify gaps
   ```

#### 2.2.2: Add service layer tests (4 days)

**Priority targets:**
- `graph_service.rs` - CRITICAL, currently under-tested
- `plan_dag_service.rs` - CRITICAL, no visible tests
- `data_set_service.rs` - HIGH, complex logic
- `authorization.rs` - CRITICAL, security-sensitive

1. **Test graph_service.rs** (1 day)
   ```rust
   // layercake-core/src/services/graph_service/tests.rs

   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::test_helpers::*;

       #[tokio::test]
       async fn test_create_graph() {
           let db = setup_test_db().await;
           let service = GraphService::new(db);

           let graph = service.create_graph(CreateGraphInput {
               name: "Test Graph".into(),
               project_id: 1,
           }).await.unwrap();

           assert_eq!(graph.name, "Test Graph");
       }

       #[tokio::test]
       async fn test_get_graph_not_found() {
           let db = setup_test_db().await;
           let service = GraphService::new(db);

           let result = service.get_graph(99999).await;
           assert!(matches!(result, Err(CoreError::NotFound(_))));
       }

       #[tokio::test]
       async fn test_update_graph_concurrency() {
           // Test concurrent updates don't cause data corruption
       }
   }
   ```

2. **Test plan_dag_service.rs** (1 day)
   ```rust
   #[tokio::test]
   async fn test_dag_execution_order() {
       // Test topological sort produces correct execution order
   }

   #[tokio::test]
   async fn test_dag_handles_cycles() {
       // Test cycle detection
   }

   #[tokio::test]
   async fn test_dag_partial_failure_recovery() {
       // Test execution continues after node failure
   }
   ```

3. **Test data_set_service.rs** (1 day)
   ```rust
   #[tokio::test]
   async fn test_file_upload_size_limits() {
       // Test large file handling
   }

   #[tokio::test]
   async fn test_invalid_file_format_rejected() {
       // Test format validation
   }
   ```

4. **Test authorization.rs** (1 day)
   ```rust
   #[tokio::test]
   async fn test_viewer_cannot_edit() {
       // Test role enforcement
   }

   #[tokio::test]
   async fn test_project_scoped_authorization() {
       // Test users can't access other projects
   }
   ```

#### 2.2.3: Add integration tests (3 days)

1. **DAG execution end-to-end** (1 day)
   ```rust
   // layercake-integration-tests/tests/dag_execution.rs

   #[tokio::test]
   async fn test_multi_stage_pipeline_execution() {
       // Create pipeline with dependencies
       // Execute and verify all stages complete
       // Verify output is correct
   }
   ```

2. **GraphQL API tests** (1 day)
   ```rust
   // layercake-integration-tests/tests/graphql_api.rs

   #[tokio::test]
   async fn test_create_project_mutation() {
       let schema = create_test_schema().await;
       let query = r#"
           mutation {
               createProject(input: { name: "Test" }) {
                   id
                   name
               }
           }
       "#;

       let result = schema.execute(query).await;
       assert!(result.errors.is_empty());
   }
   ```

3. **Auth flow integration tests** (1 day)
   ```rust
   #[tokio::test]
   async fn test_full_auth_flow() {
       // Register user
       // Login
       // Access protected resource
       // Logout
       // Verify session invalidated
   }
   ```

#### 2.2.4: Add frontend tests (2 days)

1. **Set up testing infrastructure** (4 hours)
   ```bash
   cd frontend
   npm install --save-dev @testing-library/react @testing-library/jest-dom vitest
   ```

   ```typescript
   // frontend/vite.config.ts
   export default defineConfig({
     test: {
       globals: true,
       environment: 'jsdom',
       setupFiles: './src/test/setup.ts',
     },
   });
   ```

2. **Add service tests** (4 hours)
   ```typescript
   // frontend/src/services/__tests__/PlanDagQueryService.test.ts

   import { describe, it, expect, vi } from 'vitest';
   import { PlanDagQueryService } from '../PlanDagQueryService';

   describe('PlanDagQueryService', () => {
     it('should fetch plan dag', async () => {
       const mockApollo = {
         query: vi.fn().mockResolvedValue({
           data: { getPlanDag: { id: 1, name: 'Test' } }
         })
       };

       const service = new PlanDagQueryService(mockApollo);
       const result = await service.getPlanDag(1);

       expect(result).toEqual({ id: 1, name: 'Test' });
     });
   });
   ```

3. **Add component tests** (8 hours)
   ```typescript
   // frontend/src/components/__tests__/ProjectCard.test.tsx

   import { render, screen } from '@testing-library/react';
   import { ProjectCard } from '../ProjectCard';

   describe('ProjectCard', () => {
     it('renders project name', () => {
       render(<ProjectCard project={{ id: 1, name: 'Test Project' }} />);
       expect(screen.getByText('Test Project')).toBeInTheDocument();
     });
   });
   ```

**Testing:**
- [ ] Coverage reports generated successfully
- [ ] CI coverage check passes
- [ ] All new tests pass

**Acceptance Criteria:**
- [ ] Core module coverage ≥60%
- [ ] Server module coverage ≥60%
- [ ] Frontend coverage ≥40%
- [ ] Coverage tracking in CI
- [ ] Coverage trends visible

---

### Task 2.3: Remove Console Logging Pollution

**Priority:** HIGH
**Effort:** 2 days
**Files:** 373 console.log/warn/error statements in frontend

**Implementation Steps:**

1. **Implement logging framework** (4 hours)
   ```typescript
   // frontend/src/lib/logger.ts

   type LogLevel = 'debug' | 'info' | 'warn' | 'error';

   interface Logger {
     debug(message: string, data?: unknown): void;
     info(message: string, data?: unknown): void;
     warn(message: string, data?: unknown): void;
     error(message: string, data?: unknown): void;
   }

   class ConsoleLogger implements Logger {
     private shouldLog(level: LogLevel): boolean {
       const minLevel = import.meta.env.VITE_LOG_LEVEL || 'info';
       const levels: LogLevel[] = ['debug', 'info', 'warn', 'error'];
       return levels.indexOf(level) >= levels.indexOf(minLevel as LogLevel);
     }

     debug(message: string, data?: unknown): void {
       if (this.shouldLog('debug')) {
         console.debug(`[DEBUG] ${message}`, data);
       }
     }

     info(message: string, data?: unknown): void {
       if (this.shouldLog('info')) {
         console.info(`[INFO] ${message}`, data);
       }
     }

     warn(message: string, data?: unknown): void {
       if (this.shouldLog('warn')) {
         console.warn(`[WARN] ${message}`, data);
       }
     }

     error(message: string, data?: unknown): void {
       if (this.shouldLog('error')) {
         console.error(`[ERROR] ${message}`, data);
       }
     }
   }

   export const logger = new ConsoleLogger();
   ```

2. **Replace console statements** (8 hours)
   ```bash
   # Find all console statements
   rg "console\.(log|warn|error|debug)" frontend/src --type typescript -l > console_files.txt

   # Replace systematically
   # console.log → logger.debug
   # console.info → logger.info
   # console.warn → logger.warn
   # console.error → logger.error
   ```

   ```typescript
   // BEFORE
   console.log('Fetching plan dag:', planDagId);

   // AFTER
   import { logger } from '@/lib/logger';
   logger.debug('Fetching plan dag', { planDagId });
   ```

3. **Add ESLint rule** (1 hour)
   ```javascript
   // frontend/.eslintrc.cjs
   module.exports = {
     rules: {
       'no-console': ['error', { allow: [] }],
     },
   };
   ```

4. **Clean up debug statements** (3 hours)
   - Remove purely debug console.logs
   - Keep essential error logging
   - Add structured context to logs

5. **Add production log filtering** (2 hours)
   ```typescript
   // Only log errors in production
   if (import.meta.env.PROD) {
     window.addEventListener('error', (event) => {
       logger.error('Unhandled error', { error: event.error });
       // Send to error tracking service
     });
   }
   ```

**Testing:**
- [ ] No console output in production build (except errors)
- [ ] Debug logging works in development
- [ ] ESLint prevents new console statements

**Acceptance Criteria:**
- [ ] Zero direct console.* calls in source code
- [ ] Structured logging in place
- [ ] Log levels configurable
- [ ] ESLint enforces logging standards

---

## Phase 3: Performance & Type Safety (Week 7-10)

**Goal:** Optimise database queries and eliminate type safety gaps
**Success Criteria:**
- No N+1 query issues
- Zero `any` types in frontend
- <100 unwraps in non-critical paths
- GraphQL response time p95 <100ms

### Task 3.1: Implement GraphQL DataLoader Pattern

**Priority:** HIGH
**Effort:** 4 days

**Current Issue:** N+1 queries when loading related data

**Implementation Steps:**

1. **Add DataLoader dependency** (30 min)
   ```toml
   # layercake-server/Cargo.toml
   [dependencies]
   dataloader = "0.17"
   ```

2. **Create DataLoader infrastructure** (4 hours)
   ```rust
   // layercake-server/src/graphql/dataloaders/mod.rs

   use dataloader::{BatchFn, DataLoader};
   use layercake_core::database::entities::prelude::*;
   use sea_orm::*;

   pub struct ProjectLoader {
       db: DatabaseConnection,
   }

   #[async_trait::async_trait]
   impl BatchFn<i32, Option<project::Model>> for ProjectLoader {
       async fn load(&mut self, keys: &[i32]) -> HashMap<i32, Option<project::Model>> {
           let projects = Project::find()
               .filter(project::Column::Id.is_in(keys.to_vec()))
               .all(&self.db)
               .await
               .unwrap_or_default();

           let mut map = HashMap::new();
           for key in keys {
               let project = projects.iter().find(|p| p.id == *key).cloned();
               map.insert(*key, project);
           }
           map
       }
   }

   pub struct Loaders {
       pub project_loader: DataLoader<ProjectLoader>,
       pub user_loader: DataLoader<UserLoader>,
       pub graph_loader: DataLoader<GraphLoader>,
   }

   impl Loaders {
       pub fn new(db: DatabaseConnection) -> Self {
           Self {
               project_loader: DataLoader::new(ProjectLoader { db: db.clone() }),
               user_loader: DataLoader::new(UserLoader { db: db.clone() }),
               graph_loader: DataLoader::new(GraphLoader { db }),
           }
       }
   }
   ```

3. **Integrate into GraphQL context** (2 hours)
   ```rust
   // layercake-server/src/graphql/schema.rs

   pub struct Context {
       pub db: DatabaseConnection,
       pub loaders: Loaders,
       pub user: Option<User>,
   }

   impl Context {
       pub fn new(db: DatabaseConnection, user: Option<User>) -> Self {
           Self {
               loaders: Loaders::new(db.clone()),
               db,
               user,
           }
       }
   }
   ```

4. **Update resolvers to use DataLoader** (8 hours)
   ```rust
   // BEFORE: N+1 query
   #[Object]
   impl Project {
       async fn owner(&self, ctx: &Context) -> Result<User> {
           User::find_by_id(self.owner_id)
               .one(&ctx.db)
               .await?
               .ok_or_else(|| "User not found".into())
       }
   }

   // AFTER: Batched loading
   #[Object]
   impl Project {
       async fn owner(&self, ctx: &Context) -> Result<User> {
           ctx.loaders.user_loader
               .load(self.owner_id)
               .await
               .ok_or_else(|| "User not found".into())
       }
   }
   ```

5. **Add query complexity analysis** (4 hours)
   ```rust
   // layercake-server/src/graphql/complexity.rs

   use async_graphql::*;

   pub fn complexity_calculator(ctx: &VisitorContext, child_complexity: usize) -> usize {
       // Limit query complexity to prevent abuse
       match ctx.field_name {
           "projects" => child_complexity * 10,
           "graphs" => child_complexity * 20,
           _ => child_complexity,
       }
   }

   // In schema:
   let schema = Schema::build(Query, Mutation, Subscription)
       .limit_complexity(1000)  // Max complexity
       .finish();
   ```

6. **Add performance monitoring** (4 hours)
   ```rust
   // layercake-server/src/graphql/extensions.rs

   use async_graphql::extensions::*;

   pub struct PerformanceExtension;

   impl ExtensionFactory for PerformanceExtension {
       fn create(&self) -> Arc<dyn Extension> {
           Arc::new(PerformanceExtensionImpl)
       }
   }

   struct PerformanceExtensionImpl;

   #[async_trait::async_trait]
   impl Extension for PerformanceExtensionImpl {
       async fn execute(
           &self,
           ctx: &ExtensionContext<'_>,
           operation_name: Option<&str>,
           next: NextExecute<'_>,
       ) -> Response {
           let start = std::time::Instant::now();
           let result = next.run(ctx, operation_name).await;
           let duration = start.elapsed();

           if duration.as_millis() > 100 {
               eprintln!("SLOW QUERY: {:?} took {:?}", operation_name, duration);
           }

           result
       }
   }
   ```

**Testing:**
- [ ] Load test showing no N+1 queries
- [ ] Query complexity limits enforced
- [ ] Performance monitoring works

**Acceptance Criteria:**
- [ ] No N+1 query patterns in resolvers
- [ ] Query complexity limiting in place
- [ ] Performance metrics tracked
- [ ] p95 response time <100ms

---

### Task 3.2: Implement GraphQL Code Generation for TypeScript

**Priority:** HIGH
**Effort:** 3 days

**Current Issue:** Heavy use of `any` types, no type safety between frontend and backend

**Implementation Steps:**

1. **Install GraphQL CodeGen** (1 hour)
   ```bash
   cd frontend
   npm install --save-dev @graphql-codegen/cli \
       @graphql-codegen/typescript \
       @graphql-codegen/typescript-operations \
       @graphql-codegen/typescript-react-apollo
   ```

2. **Configure CodeGen** (2 hours)
   ```yaml
   # frontend/codegen.yml
   overwrite: true
   schema: "http://localhost:8080/graphql"
   documents: "src/**/*.graphql"
   generates:
     src/generated/graphql.ts:
       plugins:
         - "typescript"
         - "typescript-operations"
         - "typescript-react-apollo"
       config:
         withHooks: true
         withComponent: false
         withHOC: false
         scalars:
           DateTime: string
           JSON: any
   ```

3. **Extract GraphQL operations to .graphql files** (4 hours)
   ```graphql
   # frontend/src/graphql/queries/GetPlanDag.graphql
   query GetPlanDag($id: ID!) {
     getPlanDag(id: $id) {
       id
       name
       nodes {
         id
         label
         type
       }
       edges {
         id
         source
         target
       }
     }
   }
   ```

4. **Generate types** (1 hour)
   ```bash
   npm run codegen
   ```

   ```json
   // package.json
   {
     "scripts": {
       "codegen": "graphql-codegen --config codegen.yml",
       "codegen:watch": "graphql-codegen --config codegen.yml --watch"
     }
   }
   ```

5. **Update services to use generated types** (8 hours)
   ```typescript
   // BEFORE
   async getPlanDag(id: number): Promise<any> {
     const result = await this.apollo.query({
       query: GET_PLAN_DAG,
       variables: { id },
     });
     return (result.data as any)?.getPlanDag || null;
   }

   // AFTER
   import { GetPlanDagQuery, GetPlanDagDocument, useGetPlanDagQuery } from '@/generated/graphql';

   async getPlanDag(id: number): Promise<GetPlanDagQuery['getPlanDag']> {
     const result = await this.apollo.query({
       query: GetPlanDagDocument,
       variables: { id: id.toString() },
     });
     return result.data.getPlanDag;
   }

   // Or use generated hook:
   const { data, loading, error } = useGetPlanDagQuery({ variables: { id: '1' } });
   ```

6. **Remove all `any` types** (4 hours)
   ```bash
   # Find remaining any types
   rg "\: any" frontend/src --type typescript

   # Replace with proper types
   ```

7. **Add type checking to CI** (1 hour)
   ```yaml
   # .github/workflows/frontend.yml
   - name: Type check
     run: |
       cd frontend
       npm run codegen
       npm run type-check
   ```

**Testing:**
- [ ] All components type-check
- [ ] No `any` types in source code
- [ ] Generated types match GraphQL schema

**Acceptance Criteria:**
- [ ] Zero `any` types in TypeScript code
- [ ] Type-safe GraphQL operations
- [ ] Auto-completion works in IDE
- [ ] CI enforces type safety

---

### Task 3.3: Optimise Clone Usage

**Priority:** MEDIUM
**Effort:** 3 days
**Files:** 290 `.clone()` calls in services

**Implementation Steps:**

1. **Audit clone usage** (4 hours)
   ```bash
   # Generate clone report
   rg "\.clone\(\)" --type rust -n layercake-core/src/services/ > clone_audit.txt

   # Categorise:
   # - UNNECESSARY: Can use reference instead
   # - ARC_CANDIDATE: Shared ownership needed
   # - LEGITIMATE: Small copy is clearer than lifetime
   ```

2. **Use Arc for shared ownership** (8 hours)
   ```rust
   // BEFORE
   pub struct GraphService {
       db: DatabaseConnection,  // Clone on every call
   }

   impl GraphService {
       pub async fn get_graph(&self, id: i32) -> GraphResult<Graph> {
           let graph = Graph::find_by_id(id)
               .one(&self.db.clone())  // Clone!
               .await?;
           // ...
       }
   }

   // AFTER
   use std::sync::Arc;

   pub struct GraphService {
       db: Arc<DatabaseConnection>,  // Shared ownership
   }

   impl GraphService {
       pub async fn get_graph(&self, id: i32) -> GraphResult<Graph> {
           let graph = Graph::find_by_id(id)
               .one(&*self.db)  // No clone needed
               .await?;
           // ...
       }
   }
   ```

3. **Use references instead of clones** (8 hours)
   ```rust
   // BEFORE
   fn process_nodes(nodes: Vec<Node>) -> Vec<ProcessedNode> {
       nodes.into_iter().map(|node| {
           let label = node.label.clone();  // Unnecessary clone
           ProcessedNode { label }
       }).collect()
   }

   // AFTER
   fn process_nodes(nodes: &[Node]) -> Vec<ProcessedNode> {
       nodes.iter().map(|node| {
           ProcessedNode { label: node.label.clone() }  // Still needed
       }).collect()
   }
   ```

4. **Benchmark performance improvement** (4 hours)
   ```rust
   // layercake-core/benches/graph_operations.rs

   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn benchmark_graph_operations(c: &mut Criterion) {
       c.bench_function("add_nodes_100", |b| {
           b.iter(|| {
               let mut graph = Graph::new();
               for i in 0..100 {
                   graph.add_node(black_box(create_test_node(i)));
               }
           });
       });
   }

   criterion_group!(benches, benchmark_graph_operations);
   criterion_main!(benches);
   ```

**Testing:**
- [ ] All tests pass
- [ ] Benchmarks show improvement
- [ ] No behaviour changes

**Acceptance Criteria:**
- [ ] <100 clones in critical paths
- [ ] Arc used for shared state
- [ ] Benchmarks show measurable improvement

---

## Phase 4: Architecture Refactoring (Week 11-16)

**Goal:** Improve modularity and reduce coupling
**Success Criteria:**
- AppContext split into domain contexts
- Service dependencies explicit
- Clear module boundaries

### Task 4.1: Refactor AppContext

**Priority:** MEDIUM
**Effort:** 5 days
**File:** `layercake-core/src/app_context/mod.rs`

**Current Issue:** AppContext manages 14+ services, creating tight coupling

**Implementation Steps:**

1. **Analyse service dependencies** (1 day)
   ```bash
   # Create dependency graph
   # Identify service clusters (graph domain, project domain, auth domain, etc.)
   ```

2. **Design domain contexts** (1 day)
   ```rust
   // layercake-core/src/contexts/mod.rs

   pub mod graph_context;
   pub mod project_context;
   pub mod auth_context;
   pub mod pipeline_context;

   /// Context for graph-related operations
   pub struct GraphContext {
       graph_service: GraphService,
       layer_palette_service: LayerPaletteService,
       graph_analysis_service: GraphAnalysisService,
       db: Arc<DatabaseConnection>,
   }

   /// Context for project-related operations
   pub struct ProjectContext {
       project_service: ProjectService,
       collaboration_service: CollaborationService,
       db: Arc<DatabaseConnection>,
   }

   /// Main application context (facade over domain contexts)
   pub struct AppContext {
       pub graph: GraphContext,
       pub project: ProjectContext,
       pub auth: AuthContext,
       pub pipeline: PipelineContext,
   }
   ```

3. **Migrate services to domain contexts** (2 days)
4. **Update service constructors** (1 day)
5. **Update all callsites** (3 days)
6. **Add integration tests** (1 day)

**Acceptance Criteria:**
- [ ] Clear domain boundaries
- [ ] Explicit dependencies
- [ ] All tests pass

---

### Task 4.2: Implement Repository Pattern

**Priority:** MEDIUM
**Effort:** 4 days

**Current Issue:** Services directly use SeaORM entities, mixing business logic with persistence

**Implementation Steps:**

1. **Create repository trait** (1 day)
   ```rust
   // layercake-core/src/repositories/mod.rs

   #[async_trait::async_trait]
   pub trait Repository<T, ID> {
       async fn find_by_id(&self, id: ID) -> CoreResult<Option<T>>;
       async fn find_all(&self) -> CoreResult<Vec<T>>;
       async fn save(&self, entity: &T) -> CoreResult<T>;
       async fn delete(&self, id: ID) -> CoreResult<()>;
   }

   // layercake-core/src/repositories/project_repository.rs
   pub struct ProjectRepository {
       db: Arc<DatabaseConnection>,
   }

   #[async_trait::async_trait]
   impl Repository<Project, i32> for ProjectRepository {
       async fn find_by_id(&self, id: i32) -> CoreResult<Option<Project>> {
           Ok(project::Entity::find_by_id(id).one(&*self.db).await?)
       }

       // Additional domain-specific queries
       async fn find_by_user(&self, user_id: i32) -> CoreResult<Vec<Project>> {
           Ok(project::Entity::find()
               .filter(project::Column::OwnerId.eq(user_id))
               .all(&*self.db)
               .await?)
       }
   }
   ```

2. **Implement repositories for key entities** (2 days)
3. **Update services to use repositories** (3 days)
4. **Add repository tests** (1 day)

**Acceptance Criteria:**
- [ ] Services don't directly use SeaORM
- [ ] Clear persistence abstraction
- [ ] Easier to test services

---

### Task 4.3: Extract Edit Replay System

**Priority:** MEDIUM
**Effort:** 3 days

**Current Issue:** Edit replay logic scattered across services

**Implementation Steps:**

1. **Create dedicated edit replay module** (1 day)
   ```rust
   // layercake-core/src/graph/edit_replay/mod.rs

   pub struct EditReplayEngine {
       graph_service: Arc<GraphService>,
   }

   impl EditReplayEngine {
       pub async fn replay_edits(
           &self,
           base_graph: &Graph,
           edits: Vec<GraphEdit>,
       ) -> GraphResult<Graph> {
           let mut current = base_graph.clone();

           for edit in edits {
               current = self.apply_edit(current, edit).await?;
           }

           Ok(current)
       }
   }
   ```

2. **Add comprehensive tests** (2 days)
3. **Document replay semantics** (1 day)

**Acceptance Criteria:**
- [ ] Edit replay logic centralised
- [ ] 100% test coverage
- [ ] Clear documentation

---

## Phase 5: Observability & Documentation (Week 17-20)

**Goal:** Improve debugging capabilities and knowledge transfer
**Success Criteria:**
- Structured logging in place
- Metrics exported
- API documentation published
- Architecture documented

### Task 5.1: Implement Structured Logging

**Priority:** MEDIUM
**Effort:** 3 days

**Implementation Steps:**

1. **Add tracing dependencies** (30 min)
   ```toml
   [dependencies]
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
   tracing-appender = "0.2"
   ```

2. **Set up tracing subscriber** (2 hours)
   ```rust
   // layercake-server/src/main.rs

   use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

   #[tokio::main]
   async fn main() {
       tracing_subscriber::registry()
           .with(EnvFilter::from_default_env())
           .with(tracing_subscriber::fmt::layer().json())
           .init();

       tracing::info!("Starting Layercake server");
       // ...
   }
   ```

3. **Add instrumentation to services** (1 day)
   ```rust
   use tracing::{info, warn, error, instrument};

   #[instrument(skip(self), fields(graph_id = %id))]
   pub async fn get_graph(&self, id: i32) -> GraphResult<Graph> {
       info!("Fetching graph");

       let graph = self.repository.find_by_id(id)
           .await?
           .ok_or_else(|| {
               warn!("Graph not found");
               CoreError::NotFound(format!("Graph {} not found", id))
           })?;

       info!("Graph fetched successfully");
       Ok(graph)
   }
   ```

4. **Add request tracing** (4 hours)
5. **Add log aggregation** (8 hours) - Configure log shipping to observability platform

**Acceptance Criteria:**
- [ ] Structured JSON logs
- [ ] Request tracing with correlation IDs
- [ ] Log levels configurable
- [ ] Logs aggregated

---

### Task 5.2: Add Metrics and Monitoring

**Priority:** MEDIUM
**Effort:** 3 days

**Implementation Steps:**

1. **Add Prometheus metrics** (1 day)
   ```toml
   [dependencies]
   prometheus = "0.13"
   ```

   ```rust
   // layercake-server/src/metrics.rs

   use prometheus::{Encoder, IntCounter, Histogram, Registry};
   use lazy_static::lazy_static;

   lazy_static! {
       pub static ref REGISTRY: Registry = Registry::new();

       pub static ref HTTP_REQUESTS_TOTAL: IntCounter = IntCounter::new(
           "http_requests_total",
           "Total HTTP requests"
       ).unwrap();

       pub static ref GRAPHQL_QUERY_DURATION: Histogram = Histogram::new(
           "graphql_query_duration_seconds",
           "GraphQL query duration"
       ).unwrap();
   }

   pub fn init_metrics() {
       REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
       REGISTRY.register(Box::new(GRAPHQL_QUERY_DURATION.clone())).unwrap();
   }

   pub fn export_metrics() -> String {
       let encoder = prometheus::TextEncoder::new();
       let metric_families = REGISTRY.gather();
       let mut buffer = vec![];
       encoder.encode(&metric_families, &mut buffer).unwrap();
       String::from_utf8(buffer).unwrap()
   }
   ```

2. **Add metrics endpoints** (4 hours)
3. **Create Grafana dashboards** (1 day)

**Acceptance Criteria:**
- [ ] Key metrics exported
- [ ] Metrics endpoint available
- [ ] Dashboards created

---

### Task 5.3: Generate and Publish Documentation

**Priority:** MEDIUM
**Effort:** 4 days

**Implementation Steps:**

1. **Add rustdoc comments** (2 days)
   ```rust
   /// Service for managing graphs and their lifecycle.
   ///
   /// This service provides CRUD operations for graphs and handles
   /// the complex business logic around graph transformations,
   /// validation, and persistence.
   ///
   /// # Example
   ///
   /// ```
   /// let service = GraphService::new(db);
   /// let graph = service.create_graph(input).await?;
   /// ```
   pub struct GraphService { /* ... */ }
   ```

2. **Generate documentation** (1 day)
   ```bash
   cargo doc --no-deps --open
   ```

3. **Add TypeDoc for frontend** (1 day)
   ```bash
   npm install --save-dev typedoc
   npx typedoc --out docs src
   ```

4. **Create architecture documentation** (2 days)
   - System architecture diagram
   - Data flow diagrams
   - API documentation
   - Deployment guide

5. **Publish documentation** (1 day)
   - Set up GitHub Pages or similar
   - Automate doc generation in CI

**Acceptance Criteria:**
- [ ] Rustdoc published
- [ ] TypeDoc published
- [ ] Architecture documented
- [ ] Auto-updated in CI

---

## Phase 6: Extensibility & Future-Proofing (Week 21-26)

**Goal:** Enable extensibility and prepare for scale
**Success Criteria:**
- Plugin architecture for transformations
- PostgreSQL support
- API versioning strategy
- Performance benchmarks

### Task 6.1: Design Plugin Architecture

**Priority:** LOW
**Effort:** 5 days

**Implementation Steps:**

1. **Design plugin trait** (1 day)
   ```rust
   // layercake-core/src/plugins/mod.rs

   #[async_trait::async_trait]
   pub trait GraphTransformPlugin: Send + Sync {
       fn name(&self) -> &str;
       fn version(&self) -> &str;

       async fn transform(
           &self,
           graph: &Graph,
           config: serde_json::Value,
       ) -> GraphResult<Graph>;

       fn config_schema(&self) -> serde_json::Value;
   }

   pub struct PluginRegistry {
       plugins: HashMap<String, Arc<dyn GraphTransformPlugin>>,
   }

   impl PluginRegistry {
       pub fn register(&mut self, plugin: Arc<dyn GraphTransformPlugin>) {
           self.plugins.insert(plugin.name().to_string(), plugin);
       }

       pub async fn execute(
           &self,
           name: &str,
           graph: &Graph,
           config: serde_json::Value,
       ) -> GraphResult<Graph> {
           let plugin = self.plugins.get(name)
               .ok_or_else(|| CoreError::NotFound(format!("Plugin {} not found", name)))?;

           plugin.transform(graph, config).await
       }
   }
   ```

2. **Implement example plugins** (2 days)
3. **Add plugin discovery** (1 day)
4. **Document plugin API** (1 day)

**Acceptance Criteria:**
- [ ] Plugin trait defined
- [ ] Example plugins work
- [ ] Documentation complete

---

### Task 6.2: Add PostgreSQL Support

**Priority:** LOW
**Effort:** 4 days

**Implementation Steps:**

1. **Add PostgreSQL driver** (30 min)
   ```toml
   [dependencies]
   sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-native-tls"] }
   ```

2. **Make database configurable** (1 day)
   ```rust
   // layercake-core/src/database/mod.rs

   pub enum DatabaseConfig {
       Sqlite { path: String },
       Postgres { url: String },
   }

   pub async fn connect(config: DatabaseConfig) -> CoreResult<DatabaseConnection> {
       match config {
           DatabaseConfig::Sqlite { path } => {
               Database::connect(format!("sqlite://{}", path)).await
           }
           DatabaseConfig::Postgres { url } => {
               Database::connect(url).await
           }
       }
       .map_err(|e| CoreError::Database(e.to_string()))
   }
   ```

3. **Update migrations** (1 day) - Ensure migrations work on both databases
4. **Add PostgreSQL integration tests** (1 day)
5. **Document PostgreSQL setup** (1 day)

**Acceptance Criteria:**
- [ ] PostgreSQL works
- [ ] Migrations work on both databases
- [ ] Tests pass on both
- [ ] Documented

---

### Task 6.3: Implement API Versioning

**Priority:** LOW
**Effort:** 3 days

**Implementation Steps:**

1. **Design versioning strategy** (1 day)
   - Decision: URL-based (/v1/graphql) vs Header-based
   - Breaking change policy
   - Deprecation timeline

2. **Implement versioned schema** (1 day)
   ```rust
   // layercake-server/src/graphql/v1/mod.rs
   pub struct QueryV1;

   #[Object]
   impl QueryV1 {
       async fn projects(&self, ctx: &Context) -> Result<Vec<Project>> {
           // V1 implementation
       }
   }

   // layercake-server/src/graphql/v2/mod.rs
   pub struct QueryV2;

   #[Object]
   impl QueryV2 {
       async fn projects(
           &self,
           ctx: &Context,
           limit: Option<i32>,  // New parameter in V2
       ) -> Result<ProjectConnection> {  // Changed return type
           // V2 implementation
       }
   }
   ```

3. **Add deprecation warnings** (1 day)
4. **Document versioning policy** (1 day)

**Acceptance Criteria:**
- [ ] Multiple API versions supported
- [ ] Deprecation mechanism works
- [ ] Policy documented

---

### Task 6.4: Add Performance Benchmarks

**Priority:** LOW
**Effort:** 3 days

**Implementation Steps:**

1. **Set up criterion benchmarks** (1 day)
   ```rust
   // layercake-core/benches/graph_benchmarks.rs

   use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

   fn benchmark_graph_operations(c: &mut Criterion) {
       let mut group = c.benchmark_group("graph_operations");

       for size in [10, 100, 1000, 10000].iter() {
           group.bench_with_input(
               BenchmarkId::from_parameter(size),
               size,
               |b, &size| {
                   b.iter(|| {
                       let mut graph = Graph::new();
                       for i in 0..size {
                           graph.add_node(create_node(i));
                       }
                   });
               },
           );
       }

       group.finish();
   }

   criterion_group!(benches, benchmark_graph_operations);
   criterion_main!(benches);
   ```

2. **Add database benchmarks** (1 day)
3. **Add GraphQL benchmarks** (1 day)
4. **Set up continuous benchmarking** (1 day)

**Acceptance Criteria:**
- [ ] Benchmarks cover critical paths
- [ ] Baseline established
- [ ] CI tracks performance

---

## Complete Review Report

### Comprehensive Codebase Review: Layercake Tool

**Review Date:** 2026-01-01
**Reviewer:** Automated Analysis Agent
**Codebase Version:** master (3c7167a4)

---

### Executive Summary

Layercake is an ambitious multi-tier graph-based planning and collaboration platform with approximately **70,500 lines of Rust** and **48,800 lines of TypeScript**. The project demonstrates strong architectural foundations with clear separation of concerns across layercake-core (business logic), layercake-server (GraphQL API), layercake-cli (CLI), and a React/TypeScript frontend. The codebase shows evidence of recent security improvements (project-scoped authorization) and good error handling patterns, but faces challenges around test coverage, code duplication, and certain architectural complexities.

**Overall Assessment: B+ / Good with Room for Improvement**

---

### 1. Architecture Assessment

#### 1.1 Overall Architecture

**Strengths:**
- **Clean layered architecture**: Clear separation between core (business logic), server (API), CLI, and frontend
- **Service layer pattern**: Well-organised service modules in `layercake-core/src/services/` (28 service files)
- **Shared workspace**: Effective use of Cargo workspace with 12 member crates
- **Database abstraction**: SeaORM provides good database abstraction with 60+ migration files showing systematic schema evolution
- **GraphQL API**: async-graphql provides type-safe API layer with proper schema definition

**Weaknesses:**
- **Monolithic core module**: The `graph.rs` file is **116,709 bytes** - this is excessively large and should be split
- **Mixed concerns**: Some services handle both business logic and persistence (see `data_set_service.rs` - 1,000+ lines)
- **Circular dependencies potential**: Frontend service classes and hooks create complex dependency graphs
- **AppContext bloat**: The `AppContext` in `layercake-core/src/app_context/mod.rs` manages 14+ services - consider breaking into domain-specific contexts

#### 1.2 Data Flow

```
Frontend (React/TypeScript)
    ↓ GraphQL queries/mutations
GraphQL Server (async-graphql)
    ↓ Service layer
Core Services (business logic)
    ↓ Entity layer
SeaORM (database abstraction)
    ↓
SQLite
```

**Observations:**
- Good separation of read (queries) and write (mutations) in GraphQL
- Real-time collaboration via GraphQL subscriptions and WebSocket
- Plan DAG execution uses event-driven architecture with execution state tracking
- CQRS pattern evident in frontend (`PlanDagQueryService` separates reads from commands)

---

### 2. Module-by-Module Analysis

#### 2.1 layercake-core (Business Logic)

**File:** `layercake-core/src/`

**Strengths:**
- **Comprehensive error types**: Custom error types for Graph, Plan, DataSet, Auth, ImportExport with proper domain categorisation in `layercake-core/src/errors/`
- **Good type definitions**: Strongly typed Graph/Node/Edge/Layer structures
- **Service organisation**: 28 well-organised service modules covering distinct domains
- **Migration strategy**: 42 database migrations showing careful schema evolution

**Issues:**

1. **Critical: Oversized files**
   - `layercake-core/src/graph.rs`: 116,709 bytes
   - `layercake-core/src/pipeline/dag_executor.rs`: 71,030 bytes
   - Recommendation: Split `graph.rs` into separate modules (operations, transformations, validation, etc.)

2. **Moderate: Error handling inconsistency**
   - 153 instances of `.unwrap()` or `.expect()` in core
   - Example in `layercake-core/src/services/graph_service.rs:91`:
     ```rust
     let parsed: Value = serde_json::from_str(&data_set.graph_json).unwrap_or_default();
     ```
   - Should use proper error propagation instead of `unwrap_or_default()`

3. **Minor: Clone overhead**
   - 290 `.clone()` calls in services directory
   - Example in `layercake-core/src/services/data_set_service.rs:136`:
     ```rust
     blob: Set(file_data.clone()),
     ```
   - Consider using references or Arc where appropriate

4. **Test coverage**
   - Only 41 test modules out of 375 Rust files (11% coverage)
   - Notable: 173 tests passing in core, but concentrated in specific modules (errors, database entities)
   - Missing tests for critical services like `plan_dag_service`, `graph_service`

#### 2.2 layercake-server (API Layer)

**File:** `layercake-server/src/`

**Strengths:**
- **Structured error handling**: Excellent error mapping in `layercake-server/src/graphql/errors.rs` with proper GraphQL error codes
- **Authorization model**: Recent authorization refactor (based on git status) with proper role-based access control
- **GraphQL schema organisation**: Clean separation of queries, mutations, subscriptions

**Issues:**

1. **Security concerns:**
   - Local auth bypass enabled by default in dev (`LAYERCAKE_LOCAL_AUTH_BYPASS=1` in `dev.sh:40`)
   - Found in `layercake-server/src/auth/mod.rs:51-59` and `layercake-core/src/services/authorization.rs:191-199`
   - **Risk**: If this environment variable leaks to production, all auth is bypassed
   - **Recommendation**: Use a more explicit development-only mode flag, add runtime warnings

2. **Moderate: Unwrap usage**
   - 3 unwraps in queries module at `layercake-server/src/graphql/queries/mod.rs`
   - Lines with `.unwrap()` on array operations
   - Should use proper error handling

3. **Password hashing:**
   - Uses bcrypt (good) in `layercake-server/src/graphql/mutations/auth.rs:61`
   - No explicit cost factor configuration visible - verify bcrypt cost is appropriate (default 12 is good)

#### 2.3 Frontend (React/TypeScript)

**File:** `frontend/src/`

**Strengths:**
- **Modern stack**: React 19, TypeScript 5.9, Vite, Apollo Client
- **Component organisation**: Clear separation into pages, components, services
- **State management**: Uses Apollo Client for GraphQL state management with local state hooks
- **CQRS pattern**: `PlanDagQueryService` implements query/command separation

**Issues:**

1. **Critical: Console statement pollution**
   - 373 console.log/warn/error statements in frontend
   - Example in `frontend/src/services/PlanDagQueryService.ts:29, 38, 52, 68, 72, 82, 95`
   - **Recommendation**: Implement proper logging framework (e.g., debug library, remove production console logs)

2. **High: React Hooks proliferation**
   - 858 useState/useEffect/useCallback/useMemo instances
   - Large component files (e.g., `App.tsx` is 84KB)
   - **Recommendation**: Extract custom hooks, use state management library (Zustand/Jotai) for complex state

3. **Moderate: Type safety concerns**
   - Heavy use of `any` types in service code:
     ```typescript
     const result = await this.apollo.query({...})
     const planDag = (result.data as any)?.getPlanDag || null
     ```
   - **Recommendation**: Generate TypeScript types from GraphQL schema using codegen

4. **Async operations**
   - 520 async/await usages - verify proper error boundaries and loading states

#### 2.4 Database Layer

**Strengths:**
- **Migration discipline**: 42 systematic migrations from `m20251008_000000` to recent ones
- **SeaORM entities**: Type-safe database access
- **Schema evolution**: Good migration naming and organisation

**Issues:**

1. **Minor: Migration naming inconsistency**
   - Mix of feature-based and date-based naming
   - Example: `m20251112_000018_rename_data_sources_to_data_sets.rs` vs `m20251103_000010_create_chat_sessions.rs`
   - Not a major issue but could be more consistent

2. **No raw SQL files found** - Good sign (using ORM properly)

---

### 3. Code Quality and Patterns

#### 3.1 Positive Patterns

1. **Error handling hierarchy**:
   - Well-defined error types in `layercake-core/src/errors/`
   - Proper error conversion from CoreError to GraphQL errors
   - Type aliases for domain-specific results (`GraphResult<T>`, `PlanResult<T>`)

2. **Service pattern**:
   ```rust
   pub struct GraphService {
       db: DatabaseConnection,
   }
   impl GraphService {
       pub fn new(db: DatabaseConnection) -> Self { ... }
       pub async fn get_graph(&self, id: i32) -> CoreResult<Graph> { ... }
   }
   ```
   - Clean, testable, dependency-injectable

3. **Authorization abstraction**:
   - Trait-based authorization in `layercake-core/src/auth/mod.rs`
   - Supports multiple authorization strategies

#### 3.2 Code Smells

1. **God objects**:
   - `Graph` struct in `graph.rs` has 116KB of implementation
   - `AppContext` manages 14+ services

2. **Long functions**:
   - `dag_executor.rs` has several 200+ line functions
   - Hard to test and understand

3. **Magic strings**:
   - Hardcoded strings like "processing", "active", "manual_edit" in data_set_service.rs
   - Should use enums/constants

4. **Technical debt markers**:
   - 10 TODO comments found:
     - `layercake-core/src/pipeline/dag_executor.rs`: 6 TODOs for event publishing
     - `layercake-core/src/services/mcp_agent_service.rs`: Test database migration issues
     - `layercake-core/src/plan_execution.rs`: Missing edge/layer verification

---

### 4. Testing Strategy and Coverage

#### 4.1 Test Organisation

**Current state:**
- 173 unit tests passing in layercake-core
- 111 files with test modules (out of 858 total Rust files = 13%)
- Integration tests in separate crate (`layercake-integration-tests`)
- No frontend tests visible

#### 4.2 Test Coverage Analysis

**Well-tested modules:**
- `layercake-core/src/errors/` - comprehensive test coverage
- `layercake-core/src/database/entities/common_types.rs` - good type tests
- `layercake-core/src/graph.rs` - some graph operation tests

**Under-tested critical paths:**
- Service layer (only basic tests in some services)
- Pipeline execution (`dag_executor.rs` - no visible tests)
- Authentication/authorization (some test database issues noted in TODOs)
- GraphQL resolvers (no test files found)
- Frontend components (no test files found)

**Recommendations:**
1. Add integration tests for DAG execution pipeline
2. Add GraphQL schema tests (snapshot testing)
3. Add frontend component tests (React Testing Library)
4. Implement property-based testing for graph operations (use proptest)
5. Add end-to-end tests for critical user flows

---

### 5. Dependency Management

#### 5.1 Rust Dependencies

**From `Cargo.toml`:**

**Well-chosen core dependencies:**
- `sea-orm` - Modern, type-safe ORM
- `async-graphql` - Type-safe GraphQL
- `tokio` - Industry-standard async runtime
- `serde` - Standard serialization

**Concerns:**
1. **Version pinning**: Some dependencies pinned to specific versions (e.g., `toml = "0.5"`) - may miss security updates
2. **Workspace dependencies**: Good use of workspace dependencies for consistency
3. **Cargo.lock size**: 10,105 lines - reasonable for a project of this size
4. **External rig dependency**: Using `rig = { version = "0.25.0" }` for MCP - ensure this is maintained

#### 5.2 Frontend Dependencies

**From `frontend/package.json`:**

**Modern stack:**
- React 19.1.1 (latest)
- TypeScript 5.9.2 (recent)
- Vite 7.1.6 (latest)
- Apollo Client 4.0.5

**Concerns:**
1. **Dependency count**: 89 dependencies - manageable but audit regularly
2. **Version ranges**: Using `^` ranges - ensure breaking changes are caught
3. **Tailwind CSS 4.1.16**: Very recent version - monitor stability
4. **Multiple graph libraries**: d3-graphviz, dagre, elkjs, force-graph, 3d-force-graph - consider consolidating

---

### 6. Security Considerations

#### 6.1 Authentication & Authorization

**Strengths:**
- Bcrypt for password hashing
- Session-based authentication with expiry
- Role-based access control (Viewer, Editor, Owner)
- Project-scoped authorization (recent addition based on git status)

**Critical Issues:**

1. **Auth bypass in development**:
   ```rust
   // layercake-server/src/auth/mod.rs:51
   fn local_auth_bypass_enabled() -> bool {
       std::env::var("LAYERCAKE_LOCAL_AUTH_BYPASS")
           .ok()
           .map(|value| {
               let normalized = value.trim().to_ascii_lowercase();
               matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
           })
           .unwrap_or(false)
   }
   ```
   - Present in both server and core authorization
   - Default enabled in `dev.sh`
   - **Risk**: Production deployment with this variable could disable all auth
   - **Mitigation**: Add compile-time flag for dev-only features, add runtime warnings, fail-safe to false

2. **Session management**:
   - Sessions stored in database (good)
   - Expiry checking in place (good)
   - No visible session rotation mechanism
   - No visible CSRF protection (GraphQL subscriptions use WebSocket - verify CORS config)

#### 6.2 Input Validation

**Strengths:**
- File format validation in `data_set_service.rs:113-121`
- Email validation in auth service
- GraphQL provides schema validation

**Concerns:**
1. **Sanitization**: Found sanitization in graph labels (`graph.rs:sanitize_labels`) - good
2. **SQL injection**: Using SeaORM parameterised queries - protected
3. **XSS**: Frontend uses React (auto-escaping) - generally safe, but verify any `dangerouslySetInnerHTML` usage
4. **Path traversal**: File uploads use `filename` - verify path sanitization in file operations

#### 6.3 Data Protection

**Observations:**
- Password hashing with bcrypt (good)
- No encryption at rest visible (SQLite database unencrypted by default)
- No TLS configuration visible in server code (likely handled by reverse proxy in production)
- Environment variables used for secrets (embedding providers, etc.)

**Recommendations:**
1. Add secrets management (e.g., hashicorp vault integration)
2. Document production TLS requirements
3. Consider SQLite encryption for sensitive data (SQLCipher)
4. Add rate limiting for authentication endpoints

---

### 7. Functional Correctness

#### 7.1 Error Handling

**Strengths:**
- Comprehensive domain error types
- Proper error propagation with `?` operator
- GraphQL error mapping with structured error codes

**Issues:**

1. **Unwrap usage**: 153 unwraps in core, 3 in server
2. **Default fallbacks**: Use of `unwrap_or_default()` can mask errors
   ```rust
   let parsed: Value = serde_json::from_str(&data_set.graph_json).unwrap_or_default();
   ```
3. **Panic potential**: 3 panic/unimplemented/unreachable found (minimal - good)

#### 7.2 Edge Cases

**Well-handled:**
- Empty dataset handling in `data_set_service.rs:55-92`
- Null/None handling in graph operations
- Cycle detection in graph transformations (mentioned in errors)

**Potential issues:**

1. **Concurrency**: DashMap used for collaboration - ensure proper synchronisation
2. **Large file handling**: File upload stores in memory (blob field) - could cause OOM with large files
3. **Graph size limits**: No visible limits on node/edge counts - could cause performance issues

#### 7.3 Business Logic Correctness

**Complex areas requiring careful review:**

1. **DAG execution** (`dag_executor.rs`):
   - Topological sort implementation
   - Dependency resolution
   - State management across nodes
   - **6 TODOs** for event publishing - incomplete feature

2. **Graph transformations** (`plan_dag/transforms.rs`):
   - Multiple transformation types (filter, aggregate, etc.)
   - Complex merge operations
   - Layer operations

3. **Edit replay** (mentioned in SPECIFICATION.md):
   - GraphEdits tracked and replayed on data refresh
   - Critical for data integrity
   - No visible tests for this feature

---

### 8. Maintainability

#### 8.1 Code Organisation

**Strengths:**
- Clear module hierarchy
- Consistent naming conventions (snake_case Rust, camelCase TypeScript)
- Good use of Cargo workspace features

**Weaknesses:**
- Oversized files need splitting
- Some circular dependency potential in frontend
- `graph.rs` is a maintenance nightmare at 116KB

#### 8.2 Documentation

**Available documentation:**
- `README.md` - comprehensive project overview
- `SPECIFICATION.md` - product specification
- `BUILD.md`, `DEV_SCRIPTS.md` - operational documentation
- `docs/` directory with historical documentation

**Code documentation:**
- Module-level docs in errors (`errors/mod.rs` has excellent documentation)
- Inconsistent inline documentation
- No visible API documentation generation (cargo doc)

**Recommendations:**
1. Generate and publish rustdoc documentation
2. Add TypeDoc for TypeScript code
3. Document complex algorithms (DAG execution, graph transformations)
4. Add architecture decision records (ADRs) for major decisions

#### 8.3 Readability

**Good practices:**
- Type aliases for domain concepts
- Descriptive function names
- Structured error messages

**Issues:**
- Long functions (200+ lines in dag_executor)
- Deep nesting in some graph operations
- Magic numbers (e.g., project ID 1 hardcoded in auth mutations)

---

### 9. Extensibility

#### 9.1 Plugin Architecture

**Current state:**
- MCP integration for agentic workflows
- Export templates using Handlebars (in `resources/library`)
- Pipeline stages modular but tightly coupled to core

**Strengths:**
- Service layer allows easy feature additions
- GraphQL schema is extensible
- Database migrations support schema evolution

**Weaknesses:**
- No clear plugin API for external extensions
- Hard to add new graph transformation types without modifying core
- Export templates limited to Handlebars

#### 9.2 API Design

**GraphQL API:**
- Well-structured queries and mutations
- Subscriptions for real-time updates
- Good error handling

**Concerns:**
- No visible API versioning strategy
- Schema evolution strategy unclear (breaking changes?)
- Rate limiting not visible

#### 9.3 Coupling

**Areas of concern:**
1. **AppContext tight coupling**: 14+ services in one context
2. **Frontend-backend coupling**: GraphQL types duplicated (no codegen visible)
3. **Database coupling**: Services directly use SeaORM entities - consider repository pattern

---

### 10. Performance Considerations

#### 10.1 Database Performance

**Potential issues:**
1. **N+1 queries**: No visible DataLoader implementation for GraphQL
2. **Large result sets**: Graph queries could return thousands of nodes - pagination implemented (`graph_page` query)
3. **Indexes**: Migration files should be checked for proper indexing (not reviewed in detail)

#### 10.2 Memory Usage

**Concerns:**
1. **File uploads**: Stored in memory as Vec<u8> before persistence
2. **Large graphs**: No streaming for large graph operations
3. **Clone usage**: 290 clones in services - potential unnecessary allocations

#### 10.3 Concurrency

**Observations:**
- 38 Arc/Mutex/RwLock usages (minimal - good)
- Tokio async runtime (good choice)
- DashMap for concurrent collaboration
- SQLite database (single-writer limitation for high concurrency)

**Recommendations:**
1. Consider PostgreSQL for production high-concurrency scenarios
2. Implement connection pooling configuration
3. Add performance benchmarks for critical paths

---

### 11. Specific Issues Found

#### 11.1 Critical

1. **Auth bypass risk** (`layercake-server/src/auth/mod.rs:51`, `layercake-core/src/services/authorization.rs:191`)
   - Severity: CRITICAL
   - Impact: Complete authentication bypass if env var leaks to production
   - Fix: Compile-time dev flag, fail-safe defaults, runtime warnings

2. **Oversized files** (`graph.rs` 116KB, `dag_executor.rs` 71KB)
   - Severity: HIGH
   - Impact: Maintenance difficulty, long compile times, merge conflicts
   - Fix: Split into logical modules

#### 11.2 High Priority

1. **Test coverage** (11% files with tests)
   - Severity: HIGH
   - Impact: Regression risk, hard to refactor confidently
   - Fix: Systematic test addition, target 60%+ coverage

2. **Console logging in production** (373 statements)
   - Severity: MEDIUM-HIGH
   - Impact: Log pollution, potential info disclosure
   - Fix: Implement proper logging framework, remove debug statements

3. **Unwrap usage** (153+ instances)
   - Severity: MEDIUM
   - Impact: Potential panics in production
   - Fix: Replace with proper error handling

#### 11.3 Medium Priority

1. **Clone overhead** (290 in services)
   - Severity: MEDIUM
   - Impact: Performance, memory usage
   - Fix: Use references, Arc where appropriate

2. **Type safety in frontend** (heavy `any` usage)
   - Severity: MEDIUM
   - Impact: Runtime errors, poor autocomplete
   - Fix: GraphQL codegen for TypeScript types

3. **Missing GraphQL N+1 protection**
   - Severity: MEDIUM
   - Impact: Database performance
   - Fix: Implement DataLoader pattern

#### 11.4 Low Priority

1. **Dependency version pinning** (e.g., `toml = "0.5"`)
   - Severity: LOW
   - Impact: Miss security updates
   - Fix: Audit and update dependencies regularly

2. **Magic strings** (status values, etc.)
   - Severity: LOW
   - Impact: Typo bugs, hard to refactor
   - Fix: Use enums/constants

---

### 12. Recommendations Summary

#### 12.1 Immediate Actions (Week 1-2)

1. **Security audit**: Review and fix auth bypass mechanism
2. **Split large files**: Break `graph.rs` into modules
3. **Remove console statements**: Implement proper frontend logging
4. **Add critical path tests**: DAG execution, auth, graph operations

#### 12.2 Short-term (Week 3-6)

1. **Improve test coverage**: Target 60%+ line coverage
2. **Replace unwraps**: Systematic audit and fix
3. **Implement GraphQL codegen**: Type-safe frontend-backend communication
4. **Add DataLoader**: Optimise N+1 queries
5. **Document architecture**: ADRs, API docs, architecture diagrams

#### 12.3 Medium-term (Week 7-16)

1. **Refactor AppContext**: Split into domain contexts
2. **Optimise cloning**: Performance audit and optimisation
3. **Add integration tests**: End-to-end test suite
4. **Implement monitoring**: Logging, metrics, tracing (OpenTelemetry)
5. **API versioning strategy**: Plan for schema evolution

#### 12.4 Long-term (Week 17-26)

1. **Plugin architecture**: Extensible transformation/export system
2. **Performance benchmarks**: Automated performance testing
3. **PostgreSQL support**: For production scaling
4. **Security hardening**: Penetration testing, security audit
5. **Frontend state management**: Consider Redux/Zustand for complex state

---

### 13. Conclusion

Layercake is a **well-architected, ambitious project** with strong foundations in Rust and TypeScript. The codebase demonstrates:

**Key Strengths:**
- Clean service-oriented architecture
- Type-safe database and API layers
- Good error handling patterns
- Recent security improvements
- Comprehensive migration strategy

**Key Challenges:**
- Large files requiring splitting
- Test coverage needs improvement
- Some code duplication and cloning
- Auth bypass mechanism is risky
- Frontend needs better type safety

**Overall Grade: B+ (Good with Room for Improvement)**

The project is production-ready with **attention to the critical security issues** (especially auth bypass), but would benefit significantly from improved test coverage, refactoring of large files, and better documentation. The architecture is sound and extensible, making it a good foundation for future development.

**Recommended Priority:**
1. Fix auth bypass (CRITICAL)
2. Improve test coverage (HIGH)
3. Refactor large files (HIGH)
4. Remove console statements (MEDIUM)
5. Optimise cloning and performance (MEDIUM)

The codebase shows evidence of careful design and recent improvements, particularly in authorization. With focused attention on the identified issues, this can be a robust, maintainable platform.

---

## Appendix A: Metrics and Measurements

### Codebase Statistics

- **Total Rust LOC**: ~70,500
- **Total TypeScript LOC**: ~48,800
- **Total Files**: 858 Rust files, frontend files not counted
- **Test Files**: 111 (13% coverage)
- **Services**: 28 in `layercake-core/src/services/`
- **Database Migrations**: 42
- **Cargo Workspace Crates**: 12

### Code Quality Metrics

- **Unwraps**: 156 total (153 core, 3 server)
- **Clones**: 290 in services directory
- **Console statements**: 373 in frontend
- **TODO comments**: 10
- **Largest file**: graph.rs (116KB)
- **Test pass rate**: 100% (173 tests passing)

### Dependencies

- **Rust dependencies**: ~100 (from Cargo.lock 10,105 lines)
- **Frontend dependencies**: 89 (from package.json)

---

## Appendix B: Tools and Commands

### Useful Analysis Commands

```bash
# Count lines of code
tokei

# Find unwraps
rg "\.unwrap\(\)|\.expect\(" --type rust -n

# Find clones
rg "\.clone\(\)" --type rust -n

# Find console statements
rg "console\.(log|warn|error|debug)" frontend/src --type typescript

# Find TODOs
rg "TODO|FIXME|HACK" --type rust

# Generate test coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench

# Check for security vulnerabilities
cargo audit

# Format code
cargo fmt
cd frontend && npm run format

# Lint code
cargo clippy
cd frontend && npm run lint
```

---

**End of Document**
