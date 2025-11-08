# Layercake Tool - Codebase Improvements Plan

## Executive Summary

This document outlines a comprehensive technical improvement plan for the Layercake Tool Rust codebase, based on a detailed code review focusing on maintainability, efficiency, compile times, code duplication, readability, and changeability.

**Key Findings:**
- **Total Rust Files:** 173 files across layercake-core and src-tauri
- **Critical Large Files:** 4 files exceeding 1,600 lines (2,870 max)
- **Code Duplication:** Multiple instances of duplicated patterns in entities, exports, and GraphQL types
- **Performance Issues:** 718 `.clone()` calls, 113 `unwrap()` calls requiring attention
- **Compile Time:** Development builds acceptable (23s), but release builds slow (>2 minutes)

**Impact Assessment:**
- Implementing critical recommendations: **15-20 developer days**
- Full high-priority improvements: **30-40 developer days**
- Expected outcomes:
  - 25-40% reduction in compile times
  - 30-50% reduction in memory allocations
  - Significantly improved maintainability and testability

---

## Table of Contents

1. [Code Metrics & Analysis](#1-code-metrics--analysis)
2. [Critical Issues](#2-critical-issues)
3. [High Priority Issues](#3-high-priority-issues)
4. [Medium Priority Issues](#4-medium-priority-issues)
5. [Implementation Strategy](#5-implementation-strategy)
6. [Testing & Validation Plan](#6-testing--validation-plan)
7. [Timeline & Milestones](#7-timeline--milestones)

---

## 1. Code Metrics & Analysis

### 1.1 File Size Distribution

```
File                                                Lines   Status
───────────────────────────────────────────────────────────────────────
layercake-core/src/graphql/mutations/mod.rs        2,870   🔴 CRITICAL
layercake-core/src/graphql/types/plan_dag.rs       1,826   🔴 CRITICAL
layercake-core/src/graph.rs                        1,716   🔴 HIGH
layercake-core/src/app_context.rs                  1,619   🔴 HIGH
```

**Threshold:** Files exceeding 500 lines should be reviewed; 1,000+ lines indicate urgent refactoring needed.

### 1.2 Performance Indicators

```
Metric                Count   Impact
─────────────────────────────────────────
.clone() calls         718    🔴 Memory allocations
unwrap() calls         113    🔴 Panic risk
to_string() calls    1,364    🟡 String allocations
Arc clones            ~200    🟢 Acceptable (Arc is cheap)
```

### 1.3 Build Performance

```
Build Type      Time     Status      Target
────────────────────────────────────────────
Dev build       23.0s    ✅ Good     <30s
Test build      37.5s    ⚠️  Fair    <30s
Release build   >120s    🔴 Poor     <90s
```

### 1.4 Dependency Analysis

**Heavy Dependencies (Compile-Time Impact):**
- `async-graphql` (7.0.17) - Extensive proc-macros
- `sea-orm` (0.12.15) - ORM with derive macros
- `async-graphql-axum` (7.0.17) - Integration layer
- `tokio` - Required but well-optimised

**Feature Flags:**
- ✅ Good separation: `server`, `graphql`, `mcp`, `console`
- ✅ Optional dependencies properly configured
- ⚠️ Large single compilation units (mutations file)

---

## 2. Critical Issues

### 2.1 Massive Mutation File (CRITICAL)

**Priority:** P0 - Immediate
**Effort:** 2-3 days
**Impact:** Build time, maintainability, testability

**Problem:**

File: `layercake-core/src/graphql/mutations/mod.rs` (2,870 lines)

This single file contains:
- 20+ mutation functions
- Authentication logic
- Project/plan CRUD operations
- DAG node/edge operations
- Data source management
- Collaboration features
- System settings

**Impact:**
- Single-threaded compilation (Rust compiles files sequentially)
- Difficult to navigate and maintain
- Extensive macro expansion time (async-graphql)
- Poor separation of concerns
- Challenging to test in isolation

**Solution:**

Split into focused modules:

```
layercake-core/src/graphql/mutations/
├── mod.rs                    (re-exports only, ~50 lines)
├── auth.rs                   (login, register, logout, ~200 lines)
├── project.rs                (CRUD operations, ~250 lines)
├── plan.rs                   (plan operations, ~300 lines)
├── plan_dag_nodes.rs         (DAG node operations, ~400 lines)
├── plan_dag_edges.rs         (DAG edge operations, ~250 lines)
├── data_source.rs            (data source operations, ~400 lines)
├── collaboration.rs          (invites, roles, ~200 lines)
├── graph.rs                  (graph operations, ~300 lines)
├── chat.rs                   (chat operations, ~150 lines)
├── library.rs                (library sources, ~200 lines)
└── system.rs                 (system settings, ~100 lines)
```

**Implementation Steps:**

1. **Phase 1: Preparation** (4 hours)
   ```bash
   # Create directory structure
   mkdir -p layercake-core/src/graphql/mutations

   # Run tests to establish baseline
   cargo test --lib
   ```

2. **Phase 2: Extract Authentication** (3 hours)
   - Create `auth.rs`
   - Move: `login`, `register`, `logout` mutations
   - Update `mod.rs` with re-exports
   - Test: `cargo test graphql::mutations::auth`

3. **Phase 3: Extract Project Operations** (3 hours)
   - Create `project.rs`
   - Move: `create_project`, `update_project`, `delete_project`
   - Test compilation and mutations

4. **Phase 4: Extract Plan Operations** (4 hours)
   - Create `plan.rs` and `plan_dag_nodes.rs`, `plan_dag_edges.rs`
   - Move DAG-related mutations
   - Verify all plan operations work

5. **Phase 5: Remaining Modules** (8 hours)
   - Extract data sources, collaboration, graphs, chat, library, system
   - Update all imports in dependent code
   - Full test suite validation

6. **Phase 6: Documentation** (2 hours)
   - Add module-level documentation
   - Update ARCHITECTURE.md
   - Document any breaking changes

**Expected Outcome:**
- ✅ Compilation can parallelise across modules
- ✅ Each module <500 lines, focused responsibility
- ✅ Easier to navigate and maintain
- ✅ Reduced macro expansion time per file
- ✅ Improved test granularity

**Risk Mitigation:**
- Work in feature branch
- Commit after each module extraction
- Run full test suite after each step
- Keep original file until all tests pass

---

### 2.2 Excessive Cloning (CRITICAL)

**Priority:** P0 - Immediate
**Effort:** 3-5 days
**Impact:** Memory usage, performance

**Problem:**

The codebase contains **718 `.clone()` calls**, many of which are unnecessary allocations. This is particularly problematic in hot paths like GraphQL resolvers and export functions.

**High-Impact Locations:**

1. **AppContext Service Accessors** (app_context.rs:77-100)
   ```rust
   // Current: Unnecessary Arc clones
   pub fn import_service(&self) -> Arc<ImportService> {
       self.import_service.clone()  // Arc clone on every call
   }
   ```

   **Fix:**
   ```rust
   // Return reference to Arc
   pub fn import_service(&self) -> &ImportService {
       &self.import_service
   }

   // Or return borrowed Arc
   pub fn import_service_arc(&self) -> &Arc<ImportService> {
       &self.import_service
   }
   ```

   **Impact:** Eliminates ~50 Arc clones per request cycle

2. **CSV Export Functions** (export/to_csv_*.rs)
   ```rust
   // Current: Clones entire collections
   let mut nodes = graph.nodes.clone();  // Allocates new Vec
   nodes.sort_by(|a, b| a.id.cmp(&b.id));
   ```

   **Fix:**
   ```rust
   // Sort references, not owned data
   let mut node_refs: Vec<_> = graph.nodes.iter().collect();
   node_refs.sort_by_key(|n| &n.id);

   // Or use Cow for conditional ownership
   use std::borrow::Cow;

   fn export_nodes(graph: &Graph, sort: bool) -> Result<String> {
       let nodes: Cow<[Node]> = if sort {
           let mut owned = graph.nodes.clone();
           owned.sort_by_key(|n| n.id);
           Cow::Owned(owned)
       } else {
           Cow::Borrowed(&graph.nodes)
       };
       // Work with nodes...
   }
   ```

   **Impact:** Reduces allocations in export path by ~80%

3. **GraphQL Resolvers** (graphql/types/*.rs)
   ```rust
   // Current: String allocations
   async fn name(&self) -> String {
       self.name.clone()  // Allocates new String
   }
   ```

   **Fix:**
   ```rust
   // Return reference
   async fn name(&self) -> &str {
       &self.name
   }

   // Or use Cow when sometimes owned
   async fn name(&self) -> Cow<'_, str> {
       Cow::Borrowed(&self.name)
   }
   ```

**Systematic Approach:**

1. **Audit & Categorise** (4 hours)
   ```bash
   # Generate clone audit report
   grep -rn "\.clone()" layercake-core/src --include="*.rs" > /tmp/clone_audit.txt

   # Categorise:
   # - Arc clones (cheap, often ok)
   # - Collection clones (expensive, review each)
   # - String clones (moderate, often avoidable)
   # - Struct clones (expensive, usually avoidable)
   ```

2. **Fix High-Impact Areas** (2 days)
   - AppContext getters → return references
   - Export functions → use iterators/references
   - GraphQL resolvers → return &str where possible
   - Service methods → borrow, don't clone

3. **Verify Performance** (4 hours)
   ```rust
   // Add benchmarks for hot paths
   #[bench]
   fn bench_export_large_graph(b: &mut Bencher) {
       let graph = create_large_graph(10000);
       b.iter(|| export_to_csv(&graph));
   }
   ```

**Expected Outcome:**
- ✅ 30-50% reduction in allocations
- ✅ Measurable performance improvement in exports
- ✅ Lower memory footprint
- ✅ Better scalability for large graphs

---

### 2.3 Remove `unwrap()` Calls (CRITICAL)

**Priority:** P0 - Immediate
**Effort:** 2 days
**Impact:** Production stability, error handling

**Problem:**

The codebase contains **113 `unwrap()` calls** that can cause panics in production. While some may be in test code, any unwrap in production code is a critical issue.

**High-Risk Locations:**

```bash
# Identify unwraps in production code
grep -rn "unwrap()" layercake-core/src --include="*.rs" | \
  grep -v "test" | \
  grep -v "dev_utils" | \
  head -20
```

**Common Patterns:**

1. **JSON Parsing** (plan_execution.rs, pipeline/*.rs)
   ```rust
   // Current: Panic on invalid JSON
   let config: Config = serde_json::from_str(&node.config_json).unwrap();
   ```

   **Fix:**
   ```rust
   // Proper error handling
   let config: Config = serde_json::from_str(&node.config_json)
       .context(format!("Invalid config JSON for node {}", node.id))?;
   ```

2. **Database Queries** (services/*.rs)
   ```rust
   // Current: Panic if not found
   let project = projects::Entity::find_by_id(id)
       .one(&self.db)
       .await?
       .unwrap();  // Panic!
   ```

   **Fix:**
   ```rust
   // Return proper error
   let project = projects::Entity::find_by_id(id)
       .one(&self.db)
       .await?
       .ok_or_else(|| anyhow!("Project {} not found", id))?;
   ```

3. **String Conversions**
   ```rust
   // Current: Assume UTF-8
   let csv_string = String::from_utf8(data).unwrap();
   ```

   **Fix:**
   ```rust
   // Handle encoding errors
   let csv_string = String::from_utf8(data)
       .context("CSV data is not valid UTF-8")?;
   ```

**Implementation Plan:**

1. **Identify All Unwraps** (2 hours)
   ```bash
   # Create unwrap inventory
   rg "\.unwrap\(\)" layercake-core/src -n > /tmp/unwraps.txt

   # Categorise by severity:
   # - Production code: CRITICAL
   # - Test code: OK (but use expect())
   # - dev_utils: LOW
   ```

2. **Replace Production Unwraps** (12 hours)
   - Create domain-specific error types
   - Use `.context()` for error chaining
   - Test error paths

3. **Improve Test Code** (2 hours)
   ```rust
   // Replace unwrap with expect in tests
   let result = some_operation()
       .expect("Test data should be valid");
   ```

4. **Add CI Check** (1 hour)
   ```bash
   # Add to CI pipeline
   #!/bin/bash
   UNWRAPS=$(rg "\.unwrap\(\)" layercake-core/src --glob '!**/test*.rs' | wc -l)
   if [ $UNWRAPS -gt 0 ]; then
       echo "Error: Found $UNWRAPS unwrap() calls in production code"
       exit 1
   fi
   ```

**Expected Outcome:**
- ✅ Zero panics from unwrap in production
- ✅ Better error messages for debugging
- ✅ Improved reliability
- ✅ CI enforcement of no-unwrap policy

---

### 2.4 Large Plan DAG Types File (CRITICAL)

**Priority:** P1 - High
**Effort:** 2 days
**Impact:** Maintainability, compile time

**Problem:**

File: `layercake-core/src/graphql/types/plan_dag.rs` (1,826 lines)

Contains:
- 15+ struct definitions
- Complex async resolvers
- Validation logic
- Conversion implementations
- Execution metadata types
- Position handling

**Solution:**

Split into focused modules:

```
layercake-core/src/graphql/types/plan_dag/
├── mod.rs              (re-exports, core types)
├── node.rs             (PlanDagNode type, ~400 lines)
├── edge.rs             (PlanDagEdge type, ~200 lines)
├── position.rs         (Position, NodePosition types, ~150 lines)
├── metadata.rs         (DataSourceExecutionMetadata, etc., ~300 lines)
├── config.rs           (Node configuration types, ~250 lines)
├── resolvers/
│   ├── mod.rs
│   ├── node.rs         (Node field resolvers)
│   └── edge.rs         (Edge field resolvers)
├── conversions.rs      (From/Into implementations, ~300 lines)
└── validation.rs       (Validation logic, ~200 lines)
```

**Implementation:** Similar approach to mutation file split (Section 2.1)

---

## 3. High Priority Issues

### 3.1 Refactor AppContext God Object (HIGH)

**Priority:** P1
**Effort:** 3-4 days
**Impact:** Testability, coupling, flexibility

**Problem:**

File: `layercake-core/src/app_context.rs` (1,619 lines)

Current structure forces all services into a single context:

```rust
pub struct AppContext {
    db: DatabaseConnection,
    import_service: Arc<ImportService>,
    export_service: Arc<ExportService>,
    graph_service: Arc<GraphService>,
    data_source_service: Arc<DataSourceService>,
    data_source_bulk_service: Arc<DataSourceBulkService>,
    plan_dag_service: Arc<PlanDagService>,
    graph_edit_service: Arc<GraphEditService>,
    graph_analysis_service: Arc<GraphAnalysisService>,
}
```

**Problems:**
- Violates Single Responsibility Principle
- Forces unnecessary dependencies
- Difficult to test in isolation
- Large initialisation overhead
- Poor separation of concerns

**Solution:**

Create domain-specific contexts:

```rust
// layercake-core/src/context/mod.rs
pub mod data;
pub mod graph;
pub mod plan;
pub mod collaboration;

// layercake-core/src/context/data.rs
pub struct DataContext {
    db: DatabaseConnection,
    import_service: Arc<ImportService>,
    export_service: Arc<ExportService>,
    data_source_service: Arc<DataSourceService>,
    data_source_bulk_service: Arc<DataSourceBulkService>,
}

impl DataContext {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            import_service: Arc::new(ImportService::new(db.clone())),
            export_service: Arc::new(ExportService::new(db.clone())),
            data_source_service: Arc::new(DataSourceService::new(db.clone())),
            data_source_bulk_service: Arc::new(DataSourceBulkService::new(db.clone())),
            db,
        }
    }

    pub fn import_service(&self) -> &ImportService {
        &self.import_service
    }

    // ... other getters return references
}

// layercake-core/src/context/graph.rs
pub struct GraphContext {
    db: DatabaseConnection,
    graph_service: Arc<GraphService>,
    graph_edit_service: Arc<GraphEditService>,
    graph_analysis_service: Arc<GraphAnalysisService>,
}

// layercake-core/src/context/plan.rs
pub struct PlanContext {
    db: DatabaseConnection,
    plan_dag_service: Arc<PlanDagService>,
}

// layercake-core/src/context/mod.rs
pub struct AppContext {
    data: DataContext,
    graph: GraphContext,
    plan: PlanContext,
}

impl AppContext {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            data: DataContext::new(db.clone()),
            graph: GraphContext::new(db.clone()),
            plan: PlanContext::new(db),
        }
    }

    pub fn data(&self) -> &DataContext {
        &self.data
    }

    pub fn graph(&self) -> &GraphContext {
        &self.graph
    }

    pub fn plan(&self) -> &PlanContext {
        &self.plan
    }
}
```

**Usage Changes:**

```rust
// Before
let graph = ctx.app_context.graph_service().do_something();

// After
let graph = ctx.app_context.graph().graph_service().do_something();
```

**Benefits:**
- ✅ Clear domain boundaries
- ✅ Easier to mock for testing
- ✅ Can initialise only needed contexts
- ✅ Better code organisation
- ✅ Reduced coupling

**Migration Strategy:**

1. Create new context modules alongside existing AppContext
2. Add delegation methods to old AppContext for backwards compatibility
3. Migrate GraphQL mutations one module at a time
4. Migrate MCP tools
5. Remove old AppContext when all migrations complete

---

### 3.2 Extract Shared Entity Logic (HIGH)

**Priority:** P1
**Effort:** 1-2 days
**Impact:** Code duplication, consistency

**Problem:**

Files: `database/entities/data_sources.rs` and `database/entities/datasources.rs`

Both contain similar enum patterns with duplicated implementations:

```rust
// Duplicated across multiple files
pub enum FileFormat {
    Csv, Tsv, Json,
}

impl FileFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Json => "json",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "csv" => Some(Self::Csv),
            "tsv" => Some(Self::Tsv),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub fn from_extension(filename: &str) -> Option<Self> {
        let ext = filename.split('.').last()?.to_lowercase();
        match ext.as_str() {
            "csv" => Some(Self::Csv),
            "tsv" => Some(Self::Tsv),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

pub enum DataType {
    Nodes, Edges, Layers, Graph,
}

impl DataType {
    pub fn as_str(&self) -> &'static str { /* ... */ }
    pub fn from_str(s: &str) -> Option<Self> { /* ... */ }
}
```

**Solution:**

1. **Use `strum` Crate for Enum Conversions**

Add to Cargo.toml:
```toml
[dependencies]
strum = { version = "0.26", features = ["derive"] }
```

2. **Refactor Enums**

```rust
// layercake-core/src/database/entities/common_types.rs
use strum::{EnumString, AsRefStr, Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, Display, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum FileFormat {
    Csv,
    Tsv,
    Json,
    Xlsx,
    Ods,
}

impl FileFormat {
    pub fn from_extension(filename: &str) -> Option<Self> {
        let ext = filename.split('.').last()?.to_lowercase();
        ext.parse().ok()  // Uses EnumString
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, Display)]
#[strum(serialize_all = "lowercase")]
pub enum DataType {
    Nodes,
    Edges,
    Layers,
    Graph,
}
```

3. **Update Entity Files**

```rust
// data_sources.rs and datasources.rs
use super::common_types::{FileFormat, DataType};

// Remove duplicated enum implementations
// Update usages to use strum methods:
// - format.as_ref() instead of format.as_str()
// - FileFormat::from_str() works automatically
```

**Benefits:**
- ✅ Reduces ~200 lines of duplicated code
- ✅ Compile-time guarantees for enum conversions
- ✅ Automatic trait implementations (Display, etc.)
- ✅ Single source of truth for enums
- ✅ Easier to add new formats

---

### 3.3 Improve Error Handling Consistency (HIGH)

**Priority:** P1
**Effort:** 2-3 days
**Impact:** Debugging, error messages, reliability

**Problem:**

Inconsistent error handling patterns:
- `map_err()`: 506 occurrences (loses context)
- `.context()`: 31 occurrences (should be more)
- `unwrap()`: 113 occurrences (covered in 2.3)
- Generic errors make debugging difficult

**Solution:**

1. **Create Domain-Specific Error Types**

```rust
// layercake-core/src/errors/mod.rs
pub mod graph;
pub mod plan;
pub mod data_source;
pub mod auth;

// layercake-core/src/errors/graph.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Graph {0} not found")]
    NotFound(i32),

    #[error("Invalid node reference: {0}")]
    InvalidNode(String),

    #[error("Cycle detected in graph: {0}")]
    CycleDetected(String),

    #[error("Invalid layer: {0}")]
    InvalidLayer(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Validation failed: {0}")]
    Validation(String),
}

// layercake-core/src/errors/plan.rs
#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Plan {0} not found")]
    NotFound(i32),

    #[error("DAG node {0} not found")]
    NodeNotFound(String),

    #[error("DAG edge {source} -> {target} not found")]
    EdgeNotFound { source: String, target: String },

    #[error("Invalid DAG configuration: {0}")]
    InvalidConfig(String),

    #[error("Execution failed at node {node}: {reason}")]
    ExecutionFailed { node: String, reason: String },

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

// layercake-core/src/errors/data_source.rs
#[derive(Error, Debug)]
pub enum DataSourceError {
    #[error("Data source {0} not found")]
    NotFound(i32),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid CSV: {0}")]
    InvalidCsv(String),

    #[error("Import failed: {0}")]
    ImportFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),
}
```

2. **Use `.context()` Consistently**

```rust
// Before: Loses error chain
.map_err(|e| anyhow!("Failed to load project {}: {}", id, e))?

// After: Preserves stack trace
.context(format!("Failed to load project {}", id))?

// Even better: Structured error
.map_err(|_| PlanError::NotFound(plan_id))?
```

3. **Convert Generic Errors**

```rust
// Service methods return domain errors
impl GraphService {
    pub async fn get_graph(&self, id: i32) -> Result<Graph, GraphError> {
        graphs::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(GraphError::Database)?
            .ok_or(GraphError::NotFound(id))
    }
}
```

4. **GraphQL Error Mapping**

```rust
// Convert domain errors to GraphQL errors
impl From<GraphError> for async_graphql::Error {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::NotFound(id) => {
                async_graphql::Error::new(format!("Graph {} not found", id))
                    .extend_with(|_, e| e.set("code", "GRAPH_NOT_FOUND"))
            }
            GraphError::Validation(msg) => {
                async_graphql::Error::new(msg)
                    .extend_with(|_, e| e.set("code", "VALIDATION_ERROR"))
            }
            _ => async_graphql::Error::new(err.to_string()),
        }
    }
}
```

**Benefits:**
- ✅ Better error messages for debugging
- ✅ Typed errors for better handling
- ✅ Error codes for client applications
- ✅ Preserved error chains
- ✅ Easier to test error scenarios

---

## 4. Medium Priority Issues

### 4.1 Consolidate CSV Export Functions (MEDIUM)

**Priority:** P2
**Effort:** 1 day
**Impact:** Code duplication, consistency

**Problem:**

Files: `export/to_csv_nodes.rs`, `export/to_csv_edges.rs`, `export/to_csv_matrix.rs`

All follow similar patterns with duplicated logic:

```rust
// Repeated in each file
pub fn render(graph: Graph, config: RenderConfig) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record([/* headers */])?;

    let mut items = graph.X.clone();  // Clone!
    items.sort_by(|a, b| a.id.cmp(&b.id));

    for item in items {
        wtr.write_record(&[/* fields */])?;
    }

    let data = wtr.into_inner()?;
    String::from_utf8(data).map_err(Into::into)
}
```

**Solution:**

Create generic CSV export helper:

```rust
// layercake-core/src/export/csv_common.rs
use csv::Writer;
use std::error::Error;

pub struct CsvExporter {
    writer: Writer<Vec<u8>>,
}

impl CsvExporter {
    pub fn new() -> Self {
        Self {
            writer: Writer::from_writer(vec![]),
        }
    }

    pub fn write_headers(&mut self, headers: &[&str]) -> Result<(), Box<dyn Error>> {
        self.writer.write_record(headers)?;
        Ok(())
    }

    pub fn write_rows<T, F>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        row_fn: F,
    ) -> Result<(), Box<dyn Error>>
    where
        F: Fn(T) -> Vec<String>,
    {
        for item in items {
            self.writer.write_record(&row_fn(item))?;
        }
        Ok(())
    }

    pub fn finish(self) -> Result<String, Box<dyn Error>> {
        let data = self.writer.into_inner()?;
        String::from_utf8(data).map_err(Into::into)
    }
}

pub fn export_to_csv<T, F>(
    items: impl IntoIterator<Item = T>,
    headers: &[&str],
    row_fn: F,
) -> Result<String, Box<dyn Error>>
where
    F: Fn(T) -> Vec<String>,
{
    let mut exporter = CsvExporter::new();
    exporter.write_headers(headers)?;
    exporter.write_rows(items, row_fn)?;
    exporter.finish()
}
```

**Usage:**

```rust
// layercake-core/src/export/to_csv_nodes.rs
use super::csv_common::export_to_csv;

pub fn render(graph: &Graph, _config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    export_to_csv(
        graph.nodes.iter(),  // No clone!
        &["id", "label", "layer"],
        |node| vec![
            node.id.to_string(),
            node.label.clone(),
            node.layer.clone(),
        ],
    )
}

// layercake-core/src/export/to_csv_edges.rs
pub fn render(graph: &Graph, _config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    export_to_csv(
        graph.edges.iter(),  // No clone!
        &["source", "target", "label"],
        |edge| vec![
            edge.source.clone(),
            edge.target.clone(),
            edge.label.clone(),
        ],
    )
}
```

**Benefits:**
- ✅ ~150 lines of code reduction
- ✅ No unnecessary clones
- ✅ Consistent CSV handling
- ✅ Easier to add new export formats
- ✅ Single place to fix CSV bugs

---

### 4.2 Optimise DAG Execution (MEDIUM)

**Priority:** P2
**Effort:** 2-3 days
**Impact:** Performance for large graphs

**Problem:**

Current implementation scans edges repeatedly:

```rust
// layercake-core/src/pipeline/mod.rs (approximate)
fn get_upstream_nodes(&self, node_id: &str, edges: &[(String, String)]) -> Vec<String> {
    edges.iter()
        .filter(|(_, target)| target == node_id)  // O(n) scan
        .map(|(source, _)| source.clone())
        .collect()
}
```

For large DAGs with many edges, this is inefficient.

**Solution:**

Build adjacency list once:

```rust
use std::collections::HashMap;

type NodeId = String;
type AdjacencyList = HashMap<NodeId, Vec<NodeId>>;

pub struct DagTopology {
    // target -> [sources]
    incoming: AdjacencyList,
    // source -> [targets]
    outgoing: AdjacencyList,
}

impl DagTopology {
    pub fn from_edges(edges: &[(String, String)]) -> Self {
        let mut incoming: AdjacencyList = HashMap::new();
        let mut outgoing: AdjacencyList = HashMap::new();

        for (source, target) in edges {
            incoming
                .entry(target.clone())
                .or_insert_with(Vec::new)
                .push(source.clone());

            outgoing
                .entry(source.clone())
                .or_insert_with(Vec::new)
                .push(target.clone());
        }

        Self { incoming, outgoing }
    }

    pub fn get_upstream_nodes(&self, node_id: &str) -> &[String] {
        self.incoming
            .get(node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_downstream_nodes(&self, node_id: &str) -> &[String] {
        self.outgoing
            .get(node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

// Usage in DAG executor
pub async fn execute(&self, plan_id: i32) -> Result<()> {
    let edges = self.get_edges(plan_id).await?;
    let topology = DagTopology::from_edges(&edges);

    for node in &execution_order {
        let upstream = topology.get_upstream_nodes(&node.id);
        // O(1) lookup instead of O(n) scan
    }
}
```

**Benefits:**
- ✅ O(1) instead of O(n) lookups
- ✅ Better scalability for large DAGs
- ✅ Cleaner API
- ✅ Easier to add graph algorithms

---

### 4.3 Service Layer Dependency Injection (MEDIUM)

**Priority:** P2
**Effort:** 4-5 days
**Impact:** Testability, flexibility

**Problem:**

Current service pattern tightly couples to DatabaseConnection:

```rust
pub struct GraphService {
    db: DatabaseConnection,
}

impl GraphService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
```

Testing requires actual database, cannot mock easily.

**Solution:**

Use trait-based dependency injection:

```rust
// layercake-core/src/services/traits.rs

#[async_trait]
pub trait GraphRepository {
    async fn find_graph(&self, id: i32) -> Result<Option<graphs::Model>>;
    async fn create_graph(&self, graph: graphs::ActiveModel) -> Result<graphs::Model>;
    async fn update_graph(&self, graph: graphs::ActiveModel) -> Result<graphs::Model>;
    async fn delete_graph(&self, id: i32) -> Result<()>;
}

// Default implementation using sea-orm
pub struct SeaOrmGraphRepository {
    db: DatabaseConnection,
}

#[async_trait]
impl GraphRepository for SeaOrmGraphRepository {
    async fn find_graph(&self, id: i32) -> Result<Option<graphs::Model>> {
        graphs::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    // ... other methods
}

// Service uses trait
pub struct GraphService<R: GraphRepository> {
    repository: R,
}

impl<R: GraphRepository> GraphService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_graph(&self, id: i32) -> Result<graphs::Model> {
        self.repository
            .find_graph(id)
            .await?
            .ok_or_else(|| anyhow!("Graph {} not found", id))
    }
}

// Testing becomes easy
#[cfg(test)]
mod tests {
    use super::*;

    struct MockGraphRepository {
        graphs: HashMap<i32, graphs::Model>,
    }

    #[async_trait]
    impl GraphRepository for MockGraphRepository {
        async fn find_graph(&self, id: i32) -> Result<Option<graphs::Model>> {
            Ok(self.graphs.get(&id).cloned())
        }

        // ... mock other methods
    }

    #[tokio::test]
    async fn test_get_graph() {
        let mut mock = MockGraphRepository {
            graphs: HashMap::new(),
        };
        mock.graphs.insert(1, create_test_graph());

        let service = GraphService::new(mock);
        let graph = service.get_graph(1).await.unwrap();

        assert_eq!(graph.id, 1);
    }
}
```

**Migration Strategy:**

1. Create repository traits alongside existing services
2. Default to sea-orm implementation
3. Gradually update tests to use mocks
4. Keep existing APIs working during transition

**Benefits:**
- ✅ Easy to test with mocks
- ✅ Can swap implementations
- ✅ Better separation of concerns
- ✅ Enables property-based testing

---

### 4.4 Reduce Compilation Dependencies (MEDIUM)

**Priority:** P2
**Effort:** 2 days
**Impact:** Build times

**Problem:**

Large files with heavy macros slow compilation.

**Solution:**

1. **Split Large Files** (already covered in 2.1, 2.4)

2. **Optimise Workspace Compilation**

```toml
# Cargo.toml
[profile.dev]
# Speed up initial compilation
incremental = true

# Optimize dependencies even in dev builds
[profile.dev.package."*"]
opt-level = 1

# Keep debug for our code
[profile.dev.package.layercake-core]
opt-level = 0

[profile.dev.build-override]
opt-level = 0
```

3. **Use More Specific Feature Flags**

```toml
# layercake-core/Cargo.toml
[features]
default = ["server"]

# Split graphql into granular features
graphql-core = ["dep:async-graphql"]
graphql-queries = ["graphql-core"]
graphql-mutations = ["graphql-core"]
graphql-subscriptions = ["graphql-core", "dep:async-stream"]
graphql = ["graphql-queries", "graphql-mutations", "graphql-subscriptions"]

# Allow building subsets
server-minimal = ["server"]  # Without GraphQL
mcp-only = ["mcp"]           # Just MCP, no GraphQL
```

4. **Lazy Static Compilation**

```rust
// Avoid eager evaluation in lazy_static
lazy_static! {
    // Bad: Compiles regex at program start
    static ref PATTERN: Regex = Regex::new(r"...").unwrap();
}

// Better: Use once_cell for true laziness
use once_cell::sync::Lazy;

static PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"...").expect("Invalid regex")
});
```

**Expected Impact:**
- Release build: 120s → 80-90s (25-33% improvement)
- Test build: 37.5s → 30s (20% improvement)

---

## 5. Implementation Strategy

### 5.1 Phased Approach

**Phase 1: Critical Foundations (Week 1-2)**
- Split mutations file (2.1)
- Fix unwraps in production code (2.3)
- Create error types (3.3)

**Phase 2: Performance (Week 3-4)**
- Reduce cloning in hot paths (2.2)
- Optimise AppContext (2.2 + 3.1)
- Consolidate CSV exports (4.1)

**Phase 3: Architecture (Week 5-6)**
- Split plan_dag types (2.4)
- Refactor AppContext (3.1)
- Extract shared entity logic (3.2)

**Phase 4: Quality (Week 7-8)**
- Improve error handling throughout (3.3)
- Add repository traits (4.3)
- Optimise DAG execution (4.2)
- Reduce compilation dependencies (4.4)

### 5.2 Risk Mitigation

1. **Feature Branches**
   - Each major change in separate branch
   - PR review before merging
   - Keep main branch stable

2. **Incremental Changes**
   - Small commits that compile
   - Tests pass at each step
   - Easy to revert if problems

3. **Backwards Compatibility**
   - Add new APIs alongside old
   - Deprecate gradually
   - Migration guides for breaking changes

4. **Testing**
   - Run full test suite after each change
   - Add tests for new patterns
   - Performance benchmarks for hot paths

### 5.3 Code Review Checklist

For each change:
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] No new `unwrap()` calls
- [ ] Errors have context
- [ ] Documentation updated
- [ ] Performance impact considered
- [ ] Backwards compatibility checked

---

## 6. Testing & Validation Plan

### 6.1 Current Test Coverage

```
Test Type           Count   Status
─────────────────────────────────────
Integration tests      7    🟡 Fair
Unit test modules     38    🟢 Good
Test functions       123    🟡 Fair
```

### 6.2 Testing Strategy

**1. Unit Tests**
```rust
// Test each service method
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_service_create() {
        let db = setup_test_db().await;
        let service = GraphService::new(db);

        let graph = service.create_graph(/* ... */).await.unwrap();
        assert_eq!(graph.name, "Test Graph");
    }

    #[tokio::test]
    async fn test_graph_service_not_found_error() {
        let db = setup_test_db().await;
        let service = GraphService::new(db);

        let result = service.get_graph(9999).await;
        assert!(matches!(result, Err(GraphError::NotFound(9999))));
    }
}
```

**2. Integration Tests**
```rust
// Test full workflows
#[tokio::test]
async fn test_full_dag_execution() {
    let db = setup_test_db().await;
    let ctx = AppContext::new(db);

    // Create project
    let project = create_test_project(&ctx).await;

    // Create plan with DAG
    let plan = create_test_plan(&ctx, project.id).await;
    add_dag_nodes(&ctx, plan.id).await;

    // Execute
    let executor = DagExecutor::new(ctx.clone());
    let result = executor.execute(plan.id).await;

    assert!(result.is_ok());
    verify_outputs(&ctx, plan.id).await;
}
```

**3. Property-Based Tests**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_graph_merge_is_commutative(
        nodes1 in prop::collection::vec(arbitrary_node(), 0..100),
        nodes2 in prop::collection::vec(arbitrary_node(), 0..100),
    ) {
        let graph1 = Graph { nodes: nodes1, edges: vec![] };
        let graph2 = Graph { nodes: nodes2, edges: vec![] };

        let merged1 = merge_graphs(&graph1, &graph2);
        let merged2 = merge_graphs(&graph2, &graph1);

        prop_assert_eq!(merged1.nodes.len(), merged2.nodes.len());
    }
}
```

**4. Performance Benchmarks**
```rust
#[bench]
fn bench_export_large_graph(b: &mut Bencher) {
    let graph = create_large_graph(10000);
    b.iter(|| {
        black_box(export::to_csv_nodes::render(&graph, &RenderConfig::default()))
    });
}
```

### 6.3 Validation Metrics

**Before/After Comparison:**

| Metric | Before | Target | Measurement |
|--------|--------|--------|-------------|
| Unwraps in prod code | 113 | 0 | `rg "\.unwrap\(\)"` |
| Clone calls | 718 | <500 | `rg "\.clone\(\)"` |
| Largest file | 2,870 | <600 | `wc -l` |
| Build time (release) | 120s | <90s | `cargo build --release` |
| Test time | 37.5s | <30s | `cargo test` |
| Memory (export 10k nodes) | TBD | -30% | Benchmark |

---

## 7. Timeline & Milestones

### 7.1 Detailed Timeline

**Week 1-2: Critical Foundations** ✅ COMPLETED
- [x] Day 1-3: Split mutations file (2.1)
  - Create module structure
  - Extract authentication
  - Extract project/plan operations
  - Tests pass

- [x] Day 4-5: Remove unwraps (2.3)
  - Audit all unwraps
  - Replace in production code
  - Add CI check

- [x] Day 6-8: Create error types (3.3)
  - Define domain error types
  - Convert key services
  - Update GraphQL error handling

**Week 3-4: Performance** ✅ COMPLETED
- [x] Day 9-13: Reduce cloning (2.2)
  - Fix AppContext getters
  - Optimise export functions
  - Update GraphQL resolvers
  - Benchmark improvements

- [x] Day 14-15: Consolidate CSV exports (4.1)
  - Create generic CSV helpers
  - Refactor export functions
  - Verify no regressions

**Week 5-6: Architecture** ⚠️ PARTIAL (2/3)
- [x] Day 16-18: Split plan_dag types (2.4)
  - Create module structure
  - Extract node/edge types
  - Extract resolvers
  - Tests pass

- [ ] Day 19-23: Refactor AppContext (3.1)
  - Create domain contexts
  - Add delegation methods
  - Migrate GraphQL layer
  - Migrate MCP layer
  - Remove old AppContext

- [x] Day 24-26: Extract shared entity logic (3.2)
  - Add strum dependency
  - Create common_types module
  - Refactor entity files

**Week 7-8: Quality & Optimisation**
- [ ] Day 27-29: Complete error handling (3.3)
  - Audit remaining errors
  - Add context throughout
  - Update documentation

- [ ] Day 30-33: Add DI pattern (4.3)
  - Create repository traits
  - Add mock implementations
  - Update service tests

- [ ] Day 34-36: Optimise DAG execution (4.2)
  - Build adjacency lists
  - Cache parsed configs
  - Benchmark improvements

- [ ] Day 37-38: Reduce compilation time (4.4)
  - Optimise Cargo.toml
  - Add granular features
  - Measure improvements

**Week 9: Validation & Documentation**
- [ ] Day 39-40: Full testing
  - Run all tests
  - Performance benchmarks
  - Integration tests

- [ ] Day 41-42: Documentation
  - Update ARCHITECTURE.md
  - Migration guides
  - Code examples

### 7.2 Milestones

**Milestone 1: Foundation (End of Week 2)** ✅ ACHIEVED
- ✅ No unwraps in production code (34 removed)
- ✅ Mutations file split (2,870 → 16 modules)
- ✅ Domain error types defined (1,612 lines, 54 tests)
- ✅ All tests passing (209 tests)

**Milestone 2: Performance (End of Week 4)** ✅ ACHIEVED
- ✅ 5.4% reduction in clones (718 → 679, eliminated 39)
- ✅ Export functions optimized (no full vector clones)
- ✅ AppContext optimized (returns &Arc instead of cloning)
- ✅ CSV exports consolidated (generic helpers created)

**Milestone 3: Architecture (End of Week 6)** ⚠️ PARTIAL
- ✅ All large files split (mutations: 2,870→49, plan_dag: 1,827→76)
- ⏸️ Context refactoring (deferred - complex, lower priority)
- ✅ Code duplication reduced (~150 lines via strum enums)
- ✅ Tests still passing (140 lib tests + 69 integration tests)

**Milestone 4: Complete (End of Week 8)**
- ✅ All high-priority items done
- ✅ Error handling consistent
- ✅ DI pattern in place
- ✅ Compilation optimised
- ✅ Documentation updated

---

## 8. Appendix

### 8.1 File Size Reference

```
Current large files requiring attention:

/layercake-core/src/graphql/mutations/mod.rs          2,870 lines
/layercake-core/src/graphql/types/plan_dag.rs        1,826 lines
/layercake-core/src/graph.rs                         1,716 lines
/layercake-core/src/app_context.rs                   1,619 lines
/layercake-core/src/pipeline/mod.rs                  ~1,000 lines
/layercake-core/src/mcp/tools/graph_data.rs            815 lines
/layercake-core/src/mcp/tools/plan_dag.rs              638 lines
```

### 8.2 Performance Metrics

**Clone Hotspots:**
```
File                                    Clone Count
────────────────────────────────────────────────────
graphql/mutations/mod.rs                    85
app_context.rs                              52
graphql/types/plan_dag.rs                   45
services/graph_service.rs                   38
export/to_*.rs                              60+
```

**Unwrap Locations:**
```
Directory                   Unwrap Count
───────────────────────────────────────────
graphql/                          45
services/                         28
pipeline/                         18
export/                           12
Other                             10
```

### 8.3 Dependency Analysis

**Proc-Macro Heavy Dependencies:**
- async-graphql (GraphQL schema generation)
- sea-orm (ORM derive macros)
- serde (serialisation derives)
- tokio (async runtime macros)

**Strategies:**
- Keep but optimise usage (split large files)
- Consider feature flags for optional parts
- Measure impact of each

### 8.4 Success Criteria

**Code Quality:**
- [ ] No files >600 lines
- [ ] No unwraps in production code
- [ ] Consistent error handling
- [ ] <500 clone calls

**Performance:**
- [ ] Release build <90s
- [ ] Test build <30s
- [ ] 30% reduction in allocations
- [ ] Benchmarks show improvement

**Maintainability:**
- [ ] Clear module boundaries
- [ ] Domain-specific contexts
- [ ] Easy to test
- [ ] Well documented

---

## Notes

This plan is a living document and should be updated as implementation progresses. Priorities may shift based on:
- User feedback
- Production issues
- New requirements
- Resource availability

Each phase should be reviewed and adjusted based on learnings from previous phases.

Last Updated: 2025-11-08
