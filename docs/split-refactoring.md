# Layercake Rust Workspace Split – Compile-Time Refactor Plan

## Current Pain
- `layercake-core` re‑exports almost everything (`src/lib.rs`) and enables `server`, `mcp`, `graphql`, `console`, `rmcp` by default (`Cargo.toml`). Even `cargo test --lib` builds SeaORM, async-graphql, axum, tokio, Tauri helpers, etc.
- `AppContext` drags in SeaORM entities, every service, exporter logic, GraphQL DTOs, and the data-acquisition crate unconditionally (`src/app_context.rs`). Optional deps never compile out.
- Exporters + Handlebars helpers live in-core (`src/export`, `src/common/handlebars.rs`) and bring `handlebars`, `zip`, `rust_xlsxwriter`, etc. into every build.
- A single binary target mixes synchronous CLI commands with async server startup (`src/main.rs`), forcing tokio/axum builds even for CLI runs.
- `layercake-data-acquisition` is an unconditional dependency, so reqwest/tokio/embedding providers compile for every target.

## Proposed Workspace Layout
```
layercake-core (new lib with CLI-facing primitives only)
 ├─ layercake-graph (graph, plan, pipeline, plan_execution, utils)
 ├─ layercake-export (template helpers, renderers, to_mermaid/* etc.)
 ├─ layercake-services (traits + SeaORM-free service interfaces)
 ├─ layercake-cli (binary; depends on graph + export + optional features)
 ├─ layercake-server (Axum + database app + AppContext impl)
 ├─ layercake-graphql (schema + resolvers; depends on server/services)
 ├─ layercake-mcp (MCP server, tokio-tungstenite, axum-mcp)
 └─ layercake-console (REPL + rig integration)
```
External crates (`layercake-data-acquisition`, `axum-mcp`, etc.) remain workspace members but become optional deps of the modules that actually use them.

## Refactor Steps
1. **Split `layercake-core`:**
   - Move `graph`, `plan`, `pipeline`, `plan_execution`, `sequence_context`, and shared utils into `layercake-graph`.
   - Move exporter modules + Handlebars helpers + `.hbs` assets into `layercake-export` using `include_str!` for templates so Cargo only rebuilds when needed.
2. **Service Traits & Context:**
   - Create `layercake-services` that holds `PlanService`, `GraphService`, `ExportService`, etc. as traits with thin SeaORM adapters (`layercake-server` implements them).
   - Replace the monolithic `AppContext` with feature-specific contexts (server, GraphQL, MCP) that depend on `layercake-services` traits; gate GraphQL imports behind the `graphql` crate.
3. **Binary Split:**
   - Add `layercake-cli` bin that depends only on `layercake-graph` + `layercake-export`. CLI subcommands call into crates instead of `mod` referencing.
   - Move server entry points into `layercake-server` bin; other binaries (GraphQL, MCP, console) can wrap it or enable features as needed.
4. **Feature Cleanup:**
   - Default workspace features: minimal CLI (no server/GraphQL/MCP). Provide `--features server`, `graphql`, `mcp`, `console`.
   - Make `layercake-data-acquisition` gated behind `data-acquisition` feature; only server/GraphQL builds enable it.
5. **Documentation & Tooling:**
   - Update `BUILD.md`, `README.md`, and npm scripts to target the new binaries.
   - Document new features/flags and add guidance for developers on when to run `cargo test -p layercake-server`, etc.

## Expected Compile-Time Impact
- CLI iterations rebuild only small crates (`layercake-graph`, `layercake-export`) instead of the full axum/SeaORM stack.
- Export changes stay isolated; template edits no longer invalidate graph logic and vice versa.
- Optional features now truly optional thanks to per-crate contexts and feature gating.
- Cargo gains more parallelism because crates depend on smaller DAGs, leveraging incremental compilation caches better.
