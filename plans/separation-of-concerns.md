# Separation of Concerns: Layercake Architecture Refactoring

## Executive Summary

This document outlines a comprehensive plan to restructure the Layercake project from a monolithic `layercake-core` crate into three distinct, purpose-built crates, while keeping database ownership in core for now:

- **layercake-core**: Pure business logic, data models, DAG execution, and service layer
- **layercake-server**: HTTP/GraphQL API, WebSocket collaboration, session management, and ephemeral state
- **layercake-cli**: Command-line interface, interactive console, and user-facing tooling

This separation will improve modularity, testability, and maintainability while enabling independent deployment scenarios (headless server, CLI-only, library usage).

---

## Current State Analysis

### Existing Structure

The current `layercake-core` crate contains:
- **Core business logic**: Graph operations, plan execution, data model
- **Database layer**: SeaORM entities, migrations, connection management
- **Service layer**: 20+ services exposing domain operations
- **HTTP/GraphQL server**: Axum app, GraphQL schema, WebSocket handlers
- **CLI**: Clap-based command parser, subcommands for run/init/serve/console
- **Interactive console**: REPL with chat integration and MCP bridge
- **MCP server**: Model Context Protocol integration for AI agents
- **Collaboration**: Real-time WebSocket coordination, presence tracking

All functionality is controlled via Cargo features (`server`, `graphql`, `mcp`, `console`), resulting in:
- **Single binary** with optional features
- **Tight coupling** between layers
- **Difficult testing** due to mixed concerns
- **Deployment inflexibility** (cannot run server-only without CLI dependencies)

### Problems to Solve

1. **Boundary violations**: CLI commands directly import server internals; GraphQL mutations access database directly
2. **Dependency bloat**: CLI builds pull in Axum/GraphQL; server builds pull in REPL/chat libraries
3. **Testing complexity**: Unit tests for services require HTTP test harness
4. **Deployment constraints**: Cannot ship server-only Docker image without CLI bloat
5. **Feature flag confusion**: 5+ features control overlapping functionality
6. **Code organisation**: 25+ modules in single `src/` directory
7. **Reusability**: Cannot use core logic as library without pulling server dependencies

---

## Target Architecture

### Crate Responsibilities

```
layercake-workspace/
├── layercake-core/          ← Pure business logic library
├── layercake-exports/       ← Import/export tooling (templates + file formats)
├── layercake-server/        ← HTTP/GraphQL/WebSocket server
├── layercake-cli/           ← CLI binary + interactive console
├── layercake-code-analysis/ ← Existing (unchanged)
├── layercake-genai/         ← Existing (unchanged)
├── layercake-projections/   ← Existing (unchanged)
└── layercake-integration-tests/
```

---

## 1. layercake-core: Business Logic Library

### Purpose
Pure Rust library crate containing all domain logic, data models, and stateless services. Zero HTTP/CLI dependencies. Can be used as a library by other Rust projects. Database access remains in core for now, but is explicitly isolated to enable future extraction.

### Responsibilities

#### Core Domain Models
- **Graph types**: `Graph`, `Node`, `Edge`, `Layer`, validation logic
- **Plan types**: `Plan`, `PlanStep`, `Dependency`, execution state machines
- **Project types**: `Project`, `DataSet`, `PlanDag`, aggregates
- **Common types**: `FileFormat`, `DataType`, result types, error enums

#### DAG Execution Engine
- **Plan executor**: Parse YAML plans, resolve dependencies, execute steps
- **Pipeline system**: Data transformation pipeline, step chaining
- **Export orchestration**: Delegates template rendering and file format handling to `layercake-exports`
- **Validation**: Graph validation, plan validation, schema checks

#### Data Access Layer
- **Database connection**: SeaORM connection pool management, URL configuration
- **Entities**: All SeaORM entity models (projects, graphs, nodes, edges, etc.)
- **Migrations**: All SeaORM migrations for schema evolution
- **Repositories** (new): Abstraction over entity queries (optional, for cleaner separation)
- **Isolation plan**: Keep DB access concentrated in `database/` and repository modules, and route all service access through those boundaries to enable later decoupling if needed.

#### Service Layer (Core Services)
All services become pure, database-driven business logic without HTTP concerns:

- `GraphService`: CRUD for graphs, nodes, edges, layers
- `GraphEditService`: Edit application, history tracking
- `GraphAnalysisService`: Connectivity analysis, pathfinding, cycles
- `PlanService`: Plan CRUD, validation
- `PlanDagService`: DAG node/edge CRUD, snapshot management
- `DataSetService`: DataSet CRUD, graph JSON operations
- `DataSetBulkService`: Bulk upload processing
- `ProjectService`: Project CRUD, archiving
- `ExportService`: Graph export to various formats (delegates to `layercake-exports`)
- `ImportService`: Graph import from files (delegates to `layercake-exports`)
- `LibraryItemService`: Library/palette management
- `CodeAnalysisService`: Code graph operations
- `ValidationService`: Cross-entity validation
- `AuthService`: Password hashing, user CRUD (no session management)

#### Application Context
- **`AppContext`**: Central service registry, dependency injection container
- Constructed with `DatabaseConnection`, owns all service instances
- Provides accessor methods for each service
- No HTTP/session/WebSocket state (moves to server)

#### Utilities
- **File utilities**: Read/write YAML/JSON for plans/config (export/import format handling lives in `layercake-exports`)
- **Graph algorithms**: Traversal, cycle detection, pathfinding
- **Update system**: Binary update checker/downloader (moved to CLI; core should not own distribution concerns)

### Dependencies
**Include:**
- `sea-orm`, `sea-orm-migration`: Database ORM
- `serde`, `serde_json`, `serde_yaml`: Serialisation
- `anyhow`, `thiserror`: Error handling
- `chrono`, `uuid`: Data types
- `indexmap`: Ordered collections
- `layercake-exports`: Import/export support
- `layercake-genai`, `layercake-code-analysis`, `layercake-projections`: Domain integrations

**Exclude:**
- ❌ `axum`, `tower`, `tower-http`: HTTP server
- ❌ `async-graphql`, `async-graphql-axum`: GraphQL
- ❌ `tokio-tungstenite`: WebSockets
- ❌ `clap`, `clap-repl`: CLI parsing
- ❌ `rig`: Chat/agent integration
- ❌ `dashmap`: Concurrent session storage

### Public API Surface
```rust
// Re-export from lib.rs
pub mod graph;
pub mod plan;
pub mod pipeline;
pub mod export;
pub mod database;
pub mod services;
pub mod app_context;
pub mod errors;
pub mod common;

pub use app_context::AppContext;
pub use database::connection::{establish_connection, get_database_url};
pub use database::migrations::Migrator;
```

---

## 2. layercake-exports: Import/Export Library

### Purpose
Dedicated library crate for import/export functionality (XLSX, ODS, CSV) and Handlebars-based rendering. Depends on `layercake-core` types and keeps format-specific dependencies out of core while letting core services delegate to this crate.

### Responsibilities
- **Template rendering**: Handlebars templates, helpers, and template loading
- **Export formats**: CSV, XLSX, ODS writers; format selection and options
- **Import formats**: CSV/XLSX/ODS readers and schema mapping
- **File adapters**: Read/write helpers for export/import (isolated from core plan/config I/O)

### Dependencies
**Include:**
- `layercake-core`: Domain types used in import/export
- `handlebars`, `regex`: Template rendering
- `csv`, `calamine`, `rust_xlsxwriter`, `spreadsheet-ods`: File format support
- `serde`, `serde_json`: Data marshaling
- `anyhow`, `thiserror`: Error handling

**Exclude:**
- ❌ `sea-orm`, `sea-orm-migration`: Database access
- ❌ `axum`, `tower`, `async-graphql`: Server stack
- ❌ `clap`, `clap-repl`: CLI parsing

### Public API Surface
```rust
pub mod export;
pub mod import;
pub mod templates;

pub use export::{export_graph, ExportFormat, ExportOptions};
pub use import::{import_graph, ImportFormat, ImportOptions};
```

---

## 3. layercake-server: HTTP/GraphQL/WebSocket Server

### Purpose
HTTP server binary providing GraphQL API, REST endpoints, WebSocket collaboration, and session/presence management. Depends on `layercake-core` for business logic.

### Responsibilities

#### HTTP Server Infrastructure
- **Axum app setup**: Route registration, middleware stack, CORS configuration
- **Health endpoints**: `/health` for load balancers
- **Static file serving**: Serve frontend assets (optional)
- **Graceful shutdown**: Signal handling, connection draining

#### GraphQL API
- **Schema definition**: Queries, Mutations, Subscriptions using `async-graphql`
- **Resolvers**: Thin wrappers calling `AppContext` services
- **Error mapping**: Convert core errors to GraphQL errors with stable codes and user-safe messages (see Error Mapping Contract below)
- **Context injection**: `GraphQLContext` with database, services, session

**Queries:**
- `projects`, `project(id)`, `graphs`, `dataSets`, `plans`, `planDag`, etc.
- Fetch operations only, delegate to core services

**Mutations:**
- `createProject`, `updateGraph`, `applyGraphEdit`, `createPlanDagNode`, etc.
- State changes, delegate to core services

**Subscriptions:**
- `graphEdits(projectId)`, `collaborationUpdates(projectId)`
- Real-time change streams using `async-stream`

#### WebSocket Collaboration
- **Connection management**: Session lifecycle, authentication
- **Presence tracking**: User cursors, active editors, online status
- **Edit broadcasting**: Distribute graph edits to connected clients
- **Coordination**: `CollaborationCoordinator` actor system (from current code)
- **Ephemeral state**: In-memory `DashMap` for session data (not persisted)

#### Session Management
- **User sessions**: Session tokens, expiry, CRUD
- **Authentication middleware**: Token validation, user context injection
- **Authorization**: Role-based access control (see Authorization Boundaries below)

#### MCP Server Integration
- **Deprecated**: MCP transport and tool definitions will be removed during the migration.
- **Replacement**: Add tools directly to Rig agents case by case (see Rig Tools Plan below).

#### Chat Management
- **Chat history service**: Store/retrieve chat messages via `ChatManager`
- **Session coordination**: Link chat sessions to projects
- **Message broadcasting**: Distribute chat updates via WebSocket

#### Configuration
- **Server config**: Port, database path, CORS origins
- **Settings service integration**: Load runtime config from `SystemSettingsService`
- **Environment loading**: `.env` file parsing for API keys, provider settings

### Dependencies
**Include:**
- `layercake-core`: Business logic library
- `axum`, `tower`, `tower-http`: HTTP server
- `async-graphql`, `async-graphql-axum`: GraphQL
- `tokio`, `tokio-tungstenite`: Async runtime, WebSockets
- `dashmap`: Concurrent session storage
- `layercake-projections`: Projections GraphQL schema merge

**Exclude:**
- ❌ `clap`, `clap-repl`: CLI (not needed for server)
- ❌ `rig`: Console chat (not needed for HTTP server)

### Entry Point
```rust
// src/main.rs
use layercake_core::{AppContext, establish_connection, get_database_url, Migrator};
use layercake_server::{create_app, ServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::from_env()?;

    let db = establish_connection(&get_database_url(config.database_path.as_deref())).await?;
    Migrator::up(&db, None).await?;

    let app_context = AppContext::new(db);
    let app = create_app(app_context, &config).await?;

    let listener = tokio::net::TcpListener::bind(config.bind_addr()).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Public API
```rust
// lib.rs exports for testing/embedding
pub mod app;
pub mod graphql;
pub mod websocket;
pub mod session;
pub mod config;

pub use app::create_app;
pub use config::ServerConfig;
```

---

## Error Mapping Contract (Server)

### Goals
- Keep error shapes stable across the refactor to avoid breaking GraphQL clients.
- Separate internal diagnostics from user-facing messages.

### Contract
- Core services return a typed error enum `CoreError` (or structured error) with:
  - `kind`: stable machine-readable category (`NotFound`, `Validation`, `Conflict`, `Unauthorized`, `Forbidden`, `Internal`, `Unavailable`)
  - `message`: safe human-readable message
  - `fields`: optional map of field-level validation errors
  - `source`: internal error chain for logging
- GraphQL resolver mapping:
  - `NotFound` → GraphQL error with `extensions.code = "NOT_FOUND"` and `status = 404`
  - `Validation` → `extensions.code = "VALIDATION"` and `status = 400`, include `extensions.fields`
  - `Conflict` → `extensions.code = "CONFLICT"` and `status = 409`
  - `Unauthorized` → `extensions.code = "UNAUTHORIZED"` and `status = 401`
  - `Forbidden` → `extensions.code = "FORBIDDEN"` and `status = 403`
  - `Unavailable` → `extensions.code = "UNAVAILABLE"` and `status = 503`
  - `Internal` (and unknown) → `extensions.code = "INTERNAL"` and `status = 500`, message defaults to "Internal server error"
- Logging:
  - Log full error chains server-side, never expose internal context to GraphQL responses.

### Implementation Details
- Introduce `core::errors::CoreError` and `CoreErrorKind`.
- Provide `impl From<CoreError> for async_graphql::Error` in server crate.
- Add unit tests in `layercake-server/tests/` for resolver error mapping.

### Validation
- Capture a baseline of current GraphQL error shapes for representative queries/mutations.
- Add golden tests that assert `extensions.code`, `status`, and optional `extensions.fields`.
- Compare baseline vs. new mapping before removing legacy error paths.
- Sample baseline set:
  - `project(id: "missing")` → NotFound
  - `createProject(name: "")` → Validation
  - `applyGraphEdit` with stale version → Conflict
  - `graph(id: "no-access")` as non-member → Forbidden
  - `projects` with invalid/expired session → Unauthorized

---

## Authorization Boundaries (Server)

### Goals
- Ensure authorization decisions are consistent, centralized, and testable.
- Prevent resolvers or services from bypassing access control.

### Boundaries
- **Core services**:
  - Take explicit `Actor`/`UserContext` input for any operation that requires authorization.
  - Enforce access checks inside services (not only at GraphQL layer).
  - Provide a `core::auth::Authorizer` trait to decouple policy from service logic.
- **Server**:
  - Builds `Actor` from session/auth middleware.
  - Injects `Actor` into GraphQL context.
  - Does not perform business logic authorization checks in resolvers beyond basic authentication.

### Implementation Details
- Add `Actor` (id, roles, scopes) in core.
- Add `Authorizer` trait in core with methods like `can_read_project`, `can_edit_graph`, `can_manage_users`.
- Add default implementation in server that uses existing roles/permissions.
- Add tests in `layercake-core/tests/` for authorization paths.

---

## 4. layercake-cli: Command-Line Interface

### Purpose
User-facing CLI binary providing subcommands for plan execution, database management, project generation, and interactive console. Depends on `layercake-core` for domain logic.

### Responsibilities

#### Command-Line Interface
- **Argument parsing**: Clap-based CLI with subcommands
- **Subcommands**:
  - `run`: Execute plan YAML files (with `--watch` for file monitoring)
  - `init`: Generate empty plan YAML template
  - `generate`: Create sample projects, templates
  - `db`: Database initialisation, migrations
  - `update`: Self-update binary from GitHub releases
  - `console`: Launch interactive REPL
  - `chat-credentials`: Manage API keys for chat providers
  - `code-analysis`: Trigger code analysis jobs (delegates to `layercake-code-analysis`)

#### Plan Execution
- **File watching**: Monitor YAML plan files for changes, re-execute
- **Progress reporting**: Terminal output, status updates, error formatting
- **Export triggering**: Invoke core export service, write output files

#### Database Management
- **Migration runner**: CLI wrapper for `Migrator::up/down/fresh`
- **Initialisation**: Create database file, run initial migrations
- **Connection helper**: Accept `--database` flag, pass to core

#### Interactive Console
- **REPL loop**: `clap-repl` integration for command/chat mode
- **Chat integration**: `rig` + direct tool bindings for LLM interactions
- **Command handlers**: `/help`, `/projects`, `/graphs`, etc.
- **Output formatting**: `nu-ansi-term` for colored terminal output

#### Project Generation
- **Sample project scaffolding**: Copy template directories
- **Template rendering**: Generate starter YAML files

#### Self-Update
- **Version checking**: Query GitHub releases API
- **Binary download**: Fetch new version, verify checksum
- **Installation**: Replace current binary, optional backup/rollback

### Dependencies
**Include:**
- `layercake-core`: Business logic library
- `clap`: CLI parsing
- `clap-repl`: Interactive REPL
- `rig`: Chat agent integration (for console)
- `nu-ansi-term`, `colored`: Terminal formatting
- `tokio`: Async runtime (for console WebSocket client)
- `notify`: File watching (for `run --watch`)
- `reqwest`: HTTP client (for update, console remote calls)

**Exclude:**
- ❌ `axum`, `tower`: HTTP server (CLI may call server via HTTP client, but doesn't host)
- ❌ `async-graphql`: GraphQL schema (consumes via HTTP, doesn't define)
- ❌ `dashmap`: Session storage (no sessions in CLI)

### Entry Point
```rust
// src/main.rs
use clap::{Parser, Subcommand};
use layercake_core::{AppContext, establish_connection, get_database_url, Migrator};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { plan: String, watch: bool },
    Init { plan: String },
    Serve { /* removed - now in layercake-server binary */ },
    Db { /* ... */ },
    Console { database: Option<String> },
    Update { /* ... */ },
    // ...
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { plan, watch } => {
            // Load plan, execute via core::plan_execution
        }
        Commands::Console { database } => {
            // Launch REPL, establish DB connection, run console loop
            let db = establish_connection(&get_database_url(database.as_deref())).await?;
            let app_context = AppContext::new(db);
            console::run_console(app_context).await?;
        }
        Commands::Db { /* ... */ } => {
            // Run migrations via Migrator
        }
        // ...
    }

    Ok(())
}
```

### Console Architecture
The interactive console becomes a CLI-only feature:
- **Database access**: Direct connection to local SQLite (same as server)
- **Service calls**: Use `AppContext` services directly (no HTTP)
- **Remote mode** (optional): HTTP client to call `layercake-server` GraphQL API
- **Chat agent**: `rig` integration with direct tools, can call local or remote server

---

## Migration Concerns & Challenges

### Dependency Management

**Challenge**: Circular dependencies between crates
- `layercake-core` must not depend on `layercake-server` or `layercake-cli`
- `layercake-server` and `layercake-cli` both depend on `layercake-core`
- Workspace dependencies must be carefully version-locked

**Solution**:
- Use `workspace = true` inheritance for shared dependencies
- Extract common types to `layercake-core::common` module
- Define clear public API surface in `layercake-core/src/lib.rs`

### Feature Flags vs Crates

**Challenge**: Current feature flags (`server`, `graphql`, `mcp`, `console`) control large chunks of functionality
- Removing features may break existing builds
- Feature combinations are tested in CI

**Solution**:
- **Phase 1**: Keep features during migration, mark deprecated
- **Phase 2**: Remove features once crates are established
- Update CI to build/test each crate independently

### Code Relocation

**Challenge**: Moving modules between crates requires fixing imports project-wide
- 100+ files with `use crate::server`, `use crate::console`, `use crate::graphql`
- External crates (`src-tauri`, tests) import `layercake::server::*`

**Solution**:
- Use `cargo fix` and IDE refactoring tools
- Create compatibility re-exports during transition (`pub use layercake_server as server;`)
- Update imports incrementally per module

### Database Migrations

**Challenge**: Migrations live in `layercake-core`, but server needs to run them
- Both CLI and server must run migrations on startup
- Migration runner is `sea-orm-migration` tied to entities

**Solution**:
- **Keep migrations in `layercake-core`**: Database schema is core domain logic
- Export `Migrator` from `layercake-core::database::migrations`
- Both server and CLI import and run migrations on their connection
- **Isolation rule**: all DB bootstrapping happens in `layercake-core::database`, other modules do not create connections directly

### Testing Strategy

**Challenge**: Integration tests currently mix CLI, server, and core logic
- `tests/integration_test.rs` uses server endpoints and database

**Solution**:
- **Core tests**: Move to `layercake-core/tests/` (pure service/domain logic)
- **Server tests**: Move to `layercake-server/tests/` (HTTP/GraphQL/WebSocket)
- **CLI tests**: Move to `layercake-cli/tests/` (command invocation, REPL)
- **Integration tests**: Keep in `layercake-integration-tests/` (cross-crate scenarios)

### Tauri Integration

**Challenge**: `src-tauri` currently depends on `layercake-core` with `server` feature
- Tauri app embeds HTTP server, needs `start_server` function
- Desktop app is effectively "server + CLI in one binary"

**Solution**:
- **Option A**: Tauri depends on both `layercake-server` and `layercake-cli`
  - Embeds server for HTTP API
  - Uses CLI for console commands (if exposed)
- **Option B**: Tauri depends only on `layercake-server`
  - Remove console from desktop app (simplify UX)
  - Focus on web UI via embedded server
- **Recommended**: Option A (preserve feature parity), but monitor binary size and startup time; if >20% regression, switch to Option B and expose CLI features via server APIs.

### AppContext Split

**Challenge**: `AppContext` currently mixes core services with server-specific state
- Holds `DatabaseConnection`, service instances (core)
- May hold session managers, WebSocket state (server)

**Solution**:
- **Core `AppContext`**: Only database + core services (in `layercake-core`)
- **Server `AppState`**: Extends with GraphQL schema, coordinator handle, projections (in `layercake-server`)
- Server wraps core context: `struct AppState { core: AppContext, ... }`

### Collaboration Coordinator

**Challenge**: `CollaborationCoordinator` is stateful, actor-based, holds WebSocket connections
- Lives in `layercake-core/src/collaboration/`
- Tightly coupled to WebSocket handler

**Solution**:
- **Move to `layercake-server`**: Collaboration is server-specific ephemeral state
- Keep `collaboration::types` in core (edit events, messages) for reuse
- Server owns `CoordinatorHandle`, spawns actor task

### MCP Server

**Challenge**: MCP server spans protocol + transport and duplicates logic with console tools.

**Solution**:
- **Remove MCP**: delete MCP transport and tool definitions as part of the refactor.
- **Rig Tools Plan**:
  - Replace MCP tools with direct Rig tool bindings for each capability.
  - Initial minimal tool list: `list_projects`, `get_project`, `list_graphs`.
  - Expand the tool list after the refactor is complete and agent development resumes.
  - Keep tool definitions near the CLI console modules, backed by `AppContext` services.
  - For server use, expose equivalent GraphQL/REST endpoints; Rig tools can target these if remote mode is enabled.
  - Maintain a short tool registry list in `layercake-cli` documenting available tools and their backing services.

### Console Chat Integration

**Challenge**: Console chat needs consistent tool definitions and a clean local/remote split
- Can call local functions or remote HTTP server
- Needs access to both database (local) and server API (remote)

**Solution**:
- **Console modes**:
  - **Local mode**: Direct `AppContext` + database connection (default)
  - **Remote mode**: HTTP client to `layercake-server` (optional flag `--remote-url`)
- REPL commands route to appropriate backend
- Rig tools live in CLI and map directly to core services; remote mode uses HTTP-backed adapters.

### Deployment Scenarios

After refactoring, support multiple deployment modes:

| Scenario | Crates Used | Binary |
|----------|-------------|--------|
| **Headless server** | `layercake-core` + `layercake-server` | `layercake-server` |
| **CLI-only** | `layercake-core` + `layercake-cli` | `layercake` |
| **Desktop app** | All three (via Tauri) | `Layercake.app` |
| **Library usage** | `layercake-core` + `layercake-exports` | (imported as dep) |
| **Docker server** | `layercake-server` only | `FROM scratch` + binary |

---

## Implementation Plan

### Stage 1: Preparation & Core Extraction (Week 1-2)

**Goal**: Extract pure business logic into clean `layercake-core` library without breaking existing builds.

#### Tasks

1. **Audit current modules** (1 day)
   - Categorise each `src/` module as: Core / Server / CLI / Shared
   - Document import dependencies between modules
   - Identify circular dependencies to break

2. **Create new crate structure** (1 day)
   ```bash
   mkdir -p layercake-server/src layercake-cli/src layercake-exports/src
   # Copy Cargo.toml templates
   ```

3. **Extract core modules** (3 days)
   - Move pure domain logic to `layercake-core/src/`:
     - `graph.rs`, `plan.rs`, `plan_execution.rs`, `pipeline/`
     - `database/` (entities, migrations, connection)
     - `services/` (all service files)
     - `app_context/` (core context only)
     - `common/`, `errors/`, `utils/`
   - Remove server/CLI dependencies from these files
   - Fix imports to use `crate::` (not `super::server`)

4. **Create `layercake-exports` crate** (2 days)
   - Move import/export modules from core:
     - `export/`, template helpers, format adapters
     - XLSX/ODS/CSV readers and writers
   - Introduce `layercake-exports` public API (`export_*`, `import_*`)
   - Replace core `ExportService`/`ImportService` internals to call `layercake-exports`

5. **Define `layercake-core` public API** (2 days)
   - Update `layercake-core/src/lib.rs`:
     ```rust
     pub mod graph;
     pub mod plan;
     pub mod pipeline;
     pub mod database;
     pub mod services;
     pub mod app_context;
     pub mod common;
     pub mod errors;

     pub use app_context::AppContext;
     pub use database::connection::{establish_connection, get_database_url};
     pub use database::migrations::Migrator;
     ```
   - Mark internal modules as `pub(crate)` where appropriate
   - Document expected usage patterns

6. **Update core `Cargo.toml`** (1 day)
   - Remove server dependencies (`axum`, `tower`, `async-graphql`)
   - Remove CLI dependencies (`clap`, `clap-repl`, `rig`)
   - Keep only: `sea-orm`, `serde`, `anyhow`, domain integrations
   - Keep feature flags during migration; remove only in Stage 5

7. **Verify core builds in isolation** (1 day)
   ```bash
   cd layercake-core && cargo build --lib
   cargo test --lib
   ```
   - Fix any compilation errors
   - Ensure no `axum`/`clap` imports remain

**Success Criteria**:
- `layercake-core` builds as standalone library
- `layercake-exports` builds as standalone library
- Zero HTTP/CLI dependencies in core
- All services callable via `AppContext`

---

### Stage 2: Server Crate Extraction (Week 2-3)

**Goal**: Move HTTP/GraphQL/WebSocket infrastructure to `layercake-server` crate.

#### Tasks

1. **Create `layercake-server` skeleton** (1 day)
   - Set up `Cargo.toml` with dependencies:
     ```toml
     [dependencies]
     layercake-core = { path = "../layercake-core" }
     axum = { workspace = true }
     async-graphql = { workspace = true }
     # ... etc
     ```
   - Create `src/main.rs` (server entry point)
   - Create `src/lib.rs` (for testing)

2. **Move server modules** (3 days)
   - Relocate from `layercake-core/src/` to `layercake-server/src/`:
     - `server/app.rs` → `app.rs`
     - `server/handlers/` → `handlers/`
     - `server/middleware/` → `middleware/`
     - `server/websocket/` → `websocket/`
   - Update imports: `use layercake_core::services::*;`

3. **Move GraphQL schema** (3 days)
   - Relocate `graphql/` module to `layercake-server/src/graphql/`
   - Update resolvers to call `AppContext` from core:
     ```rust
     use layercake_core::AppContext;

     async fn projects(ctx: &Context<'_>) -> Vec<Project> {
         let app_ctx = ctx.data_unchecked::<AppContext>();
         app_ctx.project_operations().list_projects().await
     }
     ```
   - Remove business logic from resolvers (thin wrappers only)
   - Implement error mapping contract and add tests
   - Validate error mapping against baseline GraphQL responses
   - Wire `Actor` into context and enforce authorization boundaries

4. **Move collaboration coordinator** (2 days)
   - Move `collaboration/coordinator.rs` to `layercake-server/src/collaboration/`
   - Keep `collaboration/types.rs` in core (shared types)
   - Update WebSocket handler to use server-side coordinator

5. **Remove MCP transport** (2 days)
   - Delete MCP-related routes and remove `axum-mcp` dependency
   - Remove MCP tool definitions from core
   - Update any callers to use Rig tools or GraphQL/REST endpoints

6. **Implement server `main.rs`** (1 day)
   - Parse CLI args (port, database, cors-origin)
   - Establish database connection via core
   - Create `AppContext` from core
   - Build Axum app, start server

7. **Test server binary** (1 day)
   ```bash
   cd layercake-server
   cargo run -- --port 3001 --database ../layercake.db
   ```
   - Verify GraphQL playground at `/graphql`
   - Test WebSocket collaboration endpoint
   - Validate error mapping and authorization behavior

**Success Criteria**:
- `layercake-server` binary runs standalone
- GraphQL queries/mutations work
- WebSocket collaboration functional

---

### Stage 3: CLI Crate Extraction (Week 3-4)

**Goal**: Move CLI commands and interactive console to `layercake-cli` crate.

#### Tasks

1. **Create `layercake-cli` skeleton** (1 day)
   - Set up `Cargo.toml`:
     ```toml
     [dependencies]
     layercake-core = { path = "../layercake-core" }
     clap = { workspace = true }
     tokio = { workspace = true }
     # ... etc
     ```
   - Create `src/main.rs` (CLI entry point)

2. **Move CLI command definitions** (2 days)
   - Copy command structs from `layercake-core/src/main.rs`:
     - `Commands`, `DbCommands`, `GenerateCommands`
   - Implement command handlers calling core services

3. **Move console module** (3 days)
   - Relocate `console/` to `layercake-cli/src/console/`
   - Update to use `AppContext` from core:
     ```rust
     use layercake_core::AppContext;

   pub async fn run_console(app_ctx: AppContext) -> Result<()> {
        // REPL loop
    }
    ```
   - Keep chat integration (`rig`)
   - Implement Rig tools case by case and document a tool registry

4. **Move update command** (1 day)
   - Relocate `update/` to `layercake-cli/src/update/`
   - Self-update targets `layercake-cli` binary

5. **Move plan execution command** (2 days)
   - Keep `plan_execution.rs` in core (domain logic)
   - CLI wraps with argument parsing, file watching, output formatting

6. **Move chat credentials CLI** (1 day)
   - Relocate `chat_credentials_cli/` to `layercake-cli/src/`

7. **Implement CLI `main.rs`** (1 day)
   ```rust
   match cli.command {
       Commands::Run { plan, watch } => {
           layercake_core::plan_execution::execute_plan(plan, watch)?;
       }
       Commands::Console { database } => {
           let db = establish_connection(&get_database_url(database.as_deref())).await?;
           let app_ctx = AppContext::new(db);
           console::run_console(app_ctx).await?;
       }
       // ...
   }
   ```

8. **Test CLI binary** (1 day)
   ```bash
   cd layercake-cli
   cargo run -- run --plan ../resources/sample-v1/attack_tree/plan.yaml
   cargo run -- console --database ../layercake.db
   cargo run -- db init
   ```

**Success Criteria**:
- `layercake-cli` binary runs all commands
- Console REPL works with local database
- Plan execution works
- Database migrations work

---

### Stage 4: Integration & Testing (Week 4-5)

**Goal**: Update dependent crates, fix integration tests, verify all deployment scenarios.

#### Tasks

1. **Update Tauri app** (2 days)
   - Update `src-tauri/Cargo.toml`:
     ```toml
     [dependencies]
     layercake-core = { path = "../layercake-core" }
     layercake-server = { path = "../layercake-server" }
     # Optional: layercake-cli if console is exposed
     ```
   - Update `src-tauri/src/server.rs` to use `layercake_server::create_app`
   - Test desktop app launches, server starts

2. **Update integration tests** (2 days)
   - Move tests to appropriate crates:
     - `integration_test.rs` → `layercake-core/tests/`
   - Create cross-crate tests in `layercake-integration-tests/`

3. **Update CI/CD** (1 day)
   - Add build jobs for each crate:
     ```yaml
     - cargo build -p layercake-core
     - cargo build -p layercake-exports
     - cargo build -p layercake-server
     - cargo build -p layercake-cli
     - cargo test --workspace
     ```
   - Remove feature flag matrix (no longer needed)

4. **Update documentation** (2 days)
   - Update `README.md` with new build instructions
   - Document deployment scenarios
   - Update `BUILD.md` with crate architecture

5. **Compatibility testing** (2 days)
   - Test server-only Docker build
   - Test CLI-only installation
   - Test Tauri desktop app
   - Test library usage (create example project importing `layercake-core`)

**Success Criteria**:
- All crates build and test pass
- Tauri app works
- CI passes
- Documentation updated

---

### Stage 5: Cleanup & Optimisation (Week 5-6)

**Goal**: Remove deprecated code, optimise builds, finalise public APIs.

#### Tasks

1. **Remove old feature flags** (1 day)
   - Delete `[features]` section from `layercake-core/Cargo.toml`
   - Remove `#[cfg(feature = "...")]` blocks (now handled by crate separation)

2. **Finalise public APIs** (2 days)
   - Mark internal modules as `pub(crate)` in core
   - Add `#![deny(missing_docs)]` to core library
   - Write doc comments for public types/functions

3. **Optimise dependencies** (1 day)
   - Run `cargo tree` for each crate, remove unused deps
   - Enable `lto = true` for release builds
   - Add `strip = true` to reduce binary size

4. **Benchmark impact** (1 day)
   - Measure build times before/after refactoring
   - Measure binary sizes (server, CLI, Tauri)
   - Verify no performance regressions

5. **Migration guide** (1 day)
   - Document breaking changes for external users
   - Provide import path migration table
   - Tag release with migration notes

**Success Criteria**:
- No deprecated code remains
- Public APIs documented
- Release-ready binaries

---

## Success Metrics

### Build Metrics
- **Core library build time**: <30s (down from 60s monolith)
- **Server binary size**: <40MB (down from 70MB with CLI deps)
- **CLI binary size**: <30MB (down from 70MB with server deps)

### Code Quality
- **Crate independence**: Core builds without server/CLI deps
- **Test coverage**: Maintain >70% coverage per crate
- **Public API surface**: <50 public types in core (focused)

### Deployment Flexibility
- ✅ Server-only Docker image (~100MB)
- ✅ CLI-only binary distribution
- ✅ Core as reusable library (publish optional)
- ✅ Tauri desktop app (all features)

---

## Risk Mitigation

### Risk: Breaking Tauri Integration
**Probability**: Medium
**Impact**: High (desktop app broken)
**Mitigation**:
- Test Tauri build after each stage
- Keep compatibility layer if needed
- Maintain feature parity throughout migration

### Risk: Performance Regression
**Probability**: Low
**Impact**: Medium
**Mitigation**:
- Benchmark GraphQL query latency before/after
- Profile WebSocket message throughput
- Optimise hot paths if degradation detected

### Risk: Import Hell During Migration
**Probability**: High
**Impact**: Medium (time cost)
**Mitigation**:
- Use IDE refactoring tools (rust-analyzer "Move module")
- Create compatibility re-exports during transition
- Migrate one module at a time, test incrementally

### Risk: Dependency Version Conflicts
**Probability**: Medium
**Impact**: Low
**Mitigation**:
- Lock all workspace dependencies to same versions
- Use `workspace = true` inheritance
- Run `cargo update` sparingly, test thoroughly

### Risk: GraphQL Behavior Regression
**Probability**: Medium
**Impact**: Medium
**Mitigation**:
- Lock error mapping contract before refactor
- Add resolver behavior tests for error shapes and status codes
- Add authorization tests at core service boundaries

---

## Workspace Dependency Policy

- Use `[workspace.dependencies]` as the single source of truth for versions.
- Disallow crate-local versions unless explicitly documented.
- Add a `cargo tree` check in CI to flag divergent versions.

---

## Test Fixture Strategy

- Keep shared fixtures (CSV/TSV/YAML) in `sample/` and reference from each crate test.
- Provide a shared `test_utils` crate or module for DB setup and temp directories.
- Document golden file ownership per crate to avoid duplication.

---

## Alternative Approaches Considered

### Alternative 1: Keep Monolith with Better Feature Flags
**Pros**: Less upfront work, no import changes
**Cons**: Doesn't solve dependency bloat, testing remains complex
**Decision**: Rejected - feature flags can't enforce compile-time separation

### Alternative 2: Keep MCP as Separate Crate
**Pros**: MCP as standalone library, reusable by other projects
**Cons**: Adds complexity, duplicates tooling with Rig, and keeps transport coupling
**Decision**: Rejected - MCP is removed in favor of direct Rig tools

### Alternative 3: Two Crates (Core + Unified Binary)
**Pros**: Simpler, fewer crates to manage
**Cons**: Server/CLI still bundled, doesn't enable headless server deployment
**Decision**: Rejected - doesn't achieve deployment flexibility goal

---

## Open Questions

1. **Should console support remote mode (`--remote-url` to call server API)?**
   - **Pro**: Enables remote project management via CLI
   - **Con**: Adds HTTP client complexity to console
   - **Recommendation**: Yes, add as optional feature (local by default)

2. **Should core publish to crates.io?**
   - **Pro**: Enables third-party Rust projects to use Layercake as library
   - **Con**: Requires API stability commitment, versioning discipline
   - **Recommendation**: Not initially, defer until API stabilises

3. **How to handle breaking changes during migration?**
   - **Option A**: Major version bump (0.3.x → 0.4.0)
   - **Option B**: Keep version, document in CHANGELOG
   - **Recommendation**: Option A (signals breaking change clearly)

4. **Should Tauri app include CLI console, or only web UI?**
   - **Option A**: Include console (full feature parity)
   - **Option B**: Remove console (focus on visual UI)
   - **Recommendation**: Option A (users may want CLI fallback)

5. **How should service-level transaction boundaries be expressed post-split?**
   - **Option A**: Expose explicit transaction handles in service methods
   - **Option B**: Centralize transactions in repository layer and keep services transactional by default
   - **Recommendation**: Option B (minimizes API churn and keeps services consistent)

6. **Should core become database-agnostic long term?**
   - **Option A**: Keep SeaORM ownership in core indefinitely
   - **Option B**: Extract DB interfaces to traits, keep SeaORM adapters in a separate crate
   - **Recommendation**: Option A for now, revisit after split stabilizes and service boundaries are clean

---

## Appendix: File Relocation Map

### layercake-core
**Keep (domain logic)**:
- `src/graph.rs`
- `src/plan.rs`, `src/plan_execution.rs`
- `src/pipeline/`
- `src/database/` (entities, migrations, connection)
- `src/services/` (all service files)
- `src/app_context/` (core only)
- `src/common/`, `src/errors/`, `src/utils/`
- `src/code_analysis_*.rs`
- `src/infra_graph.rs`
- `src/sequence_context.rs`
- `src/data_loader.rs`

**Move out**:
- `src/server/` → `layercake-server/src/`
- `src/graphql/` → `layercake-server/src/graphql/`
- `src/collaboration/coordinator.rs` → `layercake-server/src/collaboration/`
- `src/mcp/` → removed (replaced by Rig tools)
- `src/export/` → `layercake-exports/src/export/`
- `src/import/` → `layercake-exports/src/import/`
- `src/templates/` → `layercake-exports/src/templates/`
- `src/console/` → `layercake-cli/src/console/`
- `src/update/` → `layercake-cli/src/update/`
- `src/chat_credentials_cli/` → `layercake-cli/src/`
- `src/main.rs` (CLI commands) → `layercake-cli/src/main.rs`

### layercake-server
**New files**:
- `src/main.rs` (server entry point)
- `src/lib.rs` (for testing)
- `src/app.rs` (from core `server/app.rs`)
- `src/graphql/` (from core)
- `src/handlers/` (from core `server/handlers/`)
- `src/middleware/` (from core `server/middleware/`)
- `src/websocket/` (from core `server/websocket/`)
- `src/collaboration/` (coordinator from core)
- `src/session/` (new module for session management)
- `src/config.rs` (new, server configuration)

### layercake-exports
**New files**:
- `src/export/` (format writers, export orchestration)
- `src/import/` (format readers, schema mapping)
- `src/templates/` (Handlebars templates and helpers)
- `src/lib.rs` (public API)

### layercake-cli
**New files**:
- `src/main.rs` (CLI entry point)
- `src/console/` (from core)
- `src/update/` (from core)
- `src/chat_credentials_cli/` (from core)
- `src/commands/` (new, command implementations)

---

## Timeline Summary

| Stage | Duration | Key Deliverables |
|-------|----------|------------------|
| **Stage 1**: Core Extraction | 1-2 weeks | `layercake-core` library builds standalone |
| **Stage 2**: Server Extraction | 1-2 weeks | `layercake-server` binary runs, GraphQL works |
| **Stage 3**: CLI Extraction | 1 week | `layercake-cli` binary runs, console works |
| **Stage 4**: Integration & Testing | 1 week | Tauri app works, CI passes, docs updated |
| **Stage 5**: Cleanup & Optimisation | 1 week | APIs finalised, release-ready |
| **Total** | **5-6 weeks** | Three independent crates, all scenarios working |

---

## Conclusion

This refactoring will transform Layercake from a feature-flag-driven monolith into a modular, purpose-built architecture. The separation enables:

- **Independent deployment**: Headless server, CLI-only, library usage
- **Faster iteration**: Change server without rebuilding CLI, vice versa
- **Cleaner testing**: Unit tests for core, integration tests for server/CLI
- **Better documentation**: Public API surface is explicit and minimal
- **Third-party integration**: Core can be used as library by other Rust projects

By following the staged migration plan, we minimise risk and maintain working software throughout the transition. Each stage delivers testable, incremental value, allowing course correction if issues arise.

The resulting crates will be easier to maintain, test, and deploy—setting Layercake up for long-term success as the architecture scales.
