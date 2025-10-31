# MCP Interface Parity Plan

## Current State Overview

- **GraphQL / Frontend**  
  - Projects: full CRUD, sample seeding, collaborator management, auth, plan DAG orchestration, data source ingestion, graph editing, exports, and chat sessions (`layercake-core/src/graphql/mutations/mod.rs`, `layercake-core/src/graphql/queries/mod.rs`).  
  - Web UI consumes these mutations/queries for plan editing, graph visualization, data source previews, library sources, and real-time chat (`frontend/src/pages/GraphEditorPage.tsx`, `frontend/src/pages/ProjectChatPage.tsx`).
- **MCP Server**  
  - Exposes limited project, plan, and graph helpers plus stubbed analysis tools (`layercake-core/src/mcp/tools/{projects,plans,graph_data,analysis}.rs`).  
  - Tool registry lists only CRUD-lite operations; authentication helpers exist but aren’t registered, and resources/prompts return placeholder data (`layercake-core/src/mcp/server.rs`, `layercake-core/src/mcp/resources.rs`).  
  - Analysis utilities and resources for graphs are unimplemented; no coverage for plan DAG nodes/edges, data sources, library sources, or collaboration flows.

## Guiding Principles

- **Unified surfaces**: Every capability must be implemented once and surfaced everywhere. GraphQL resolvers, MCP tools, and console commands should call the same services/DTOs rather than duplicating logic.  
- **Primary audience**: Favour agent-facing scenarios (plan execution, graph automation) while still ensuring the console and UI benefit from the shared implementation; defer collaboration/chat extras to Phase 7.  
- **Security**: MCP tooling is available both inside the built-in chat UI and over HTTP. Require API-key authentication for HTTP exposure only (see `layercake-core/src/mcp/server.rs`), while allowing trusted, in-process chat usage without keys. Document `Authorization: Bearer …` expectations for external clients.  
- **Realtime scope**: Subscription-style updates are not a near-term requirement. Prefer polling/batch responses for execution status until priorities shift.  
- **Serialization**: Treat the GraphQL schema as the leading contract. Serialize once via shared GraphQL DTOs so MCP responses and console output reuse the exact same field names (PascalCase types, camelCase fields).

## Gaps & Discrepancies

1. **Project Lifecycle** – GraphQL supports update, collaboration, samples, and metadata; MCP only lists/create/get/delete and the resource endpoint returns mock data.  
2. **Authentication & Sessions** – MCP has helper functions but the tool registry never exposes them; no parity with GraphQL’s login/logout/session queries.  
3. **Plan Management** – GraphQL offers create/update/delete/execute plus full DAG editing (nodes, edges, moves, validations). MCP offers only create (overwrites existing), execute, status—no deletes, updates, or DAG operations.  
4. **Graph Editing & Execution** – Frontend mutates nodes/edges/layers, replays edits, exports previews; MCP can import/export CSV or dump graph data but can’t edit, replay, or preview.  
5. **Data Sources & Library Sources** – Extensive GraphQL coverage (upload, reprocess, import/export, seed). MCP lacks any tooling for these domains.  
6. **Collaboration & Access Control** – GraphQL manages invitations, roles, sessions; MCP has no equivalents.  
7. **Chat & Tool Awareness** – GraphQL chat integrates with MCP tools dynamically; MCP server cannot start chat sessions or enumerate tool capabilities per provider context.  
8. **Resources/Prompts** – Resource registry responds with static placeholders and skips key formats (graph exports, analysis). No prompt parity with frontend flows.  
9. **Telemetry & Streaming** – GraphQL subscriptions stream chat and execution events; MCP lacks structured streaming/batch endpoints for comparable visibility.

## Extension Roadmap

### Phase 1 – Foundation & Parity Audit
- Consolidate project/plan/graph services into shared modules that are imported by GraphQL resolvers, MCP tools, and console commands alike (single source of truth for business logic).  
- Replace placeholder resource responses with real DB queries via those shared services; implement missing formats for `get_graph_resource` so all surfaces return identical payloads.

### Phase 2 – Project & Plan Feature Parity
- Extend shared services/DTOs so project update/archive logic is callable from GraphQL, MCP, and console contexts without divergence.  
- Add plan DAG tooling (list/add/update/delete/move/validate) in the shared layer, exposing thin adapters for GraphQL mutations and MCP tools with identical payloads.  
- Provide plan deletion/cancellation and execution monitoring commands that call into `PlanDagService`/`PlanExecution` through the same façade used by the console `plan` commands.

### Phase 3 – Graph Editing & Analysis
- Centralise node/edge/layer CRUD, bulk updates, graph edit playback/clear, and export preview logic in shared helpers backed by `GraphService`/`GraphEditService`; wire adapters for GraphQL mutations, MCP tools, and console commands.  
- Finish the analysis module (`analyze_connectivity`, `find_paths`, etc.) within the shared layer so all interfaces return the same analytics (degree stats, layering checks).  
- Prepare optional streaming/batch wrappers for long-running exports once axum-mcp supports it—GraphQL and console can reuse the same progress polling utilities.

### Phase 4 – Data Source & Library Workflows
- Move data source and library source lifecycle logic (create/upload/reprocess/delete/bulk import/export) into shared services used by GraphQL mutations today; expose equivalent MCP/console entry points by reusing those helpers.  
- Support raw/JSON download via shared resource adapters so `download_data_source_*` (GraphQL), MCP resources, and console tooling all return identical payloads.

### Phase 5 – Telemetry & Execution Visibility
- Build shared telemetry helpers that extract execution state, graph edit counters, and plan progress; surface the same JSON via GraphQL queries, MCP resources, and console reporting commands.  
- Evaluate streaming or incremental responses for long-running tasks once axum-mcp batch APIs mature, ensuring any implementation can be wrapped for GraphQL subscriptions in the future.

### Phase 6 – Validation, UX, & Documentation
- Build integration tests that exercise shared services via GraphQL resolvers, MCP handlers, and console commands to guarantee identical behaviour.  
- Update developer docs to show how each interface layers atop the shared modules, including example payloads reused across GraphQL/MCP/console.  
- Ensure UI + MCP share schemas by deriving everything from GraphQL types and generating console output from the same serde structures.

### Phase 7 – Optional: Authentication, Collaboration, & Chat Parity
- Register authentication tools (`register_user`, `login_user`, `logout_user`) and align API-key/session policies between GraphQL and MCP.  
- Expose collaboration management (invites, role updates, session listings) for agents that need shared project workflows.  
- Provide MCP tasks to start chat sessions, relay messages, and enumerate providers/models using the existing chat manager.  
- Synchronize prompt registry entries with frontend templates (plan creation, graph diagnostics) and document when/why to use them.

## Open Questions / Dependencies

- Confirm final API-key distribution/rotation process for third-party agents.  
- Clarify whether agents require export artifacts pushed to remote storage (vs returned inline) for large graphs.  
- Identify existing console commands that bypass GraphQL services so they can be migrated to the shared layer.  
- Track axum-mcp batch API maturity to reassess streaming support when needed.

## Technical Implementation Details

### Phase 1 – Shared Services & Resource Parity
- Introduce a shared application layer (e.g., `layercake-core/src/app`) that wraps reusable helpers currently wired through `graphql/context.rs`, and export them for GraphQL, MCP, and console consumers.  
- Refactor MCP tool implementations and console commands to accept the same lightweight context (DB + shared services) instead of reimplementing queries.  
- Replace placeholder resource payloads in `layercake-core/src/mcp/resources.rs` with real data fetched via the shared services; continue serialising through GraphQL DTO structs (`layercake-core/src/graphql/types`) to maintain identical shapes.  
- Harden API-key enforcement on HTTP entry points by documenting required headers (`Authorization: Bearer ...`) and ensuring external requests pass through `LayercakeAuth::authenticate`, while privileged chat invocations reuse the trusted security context.

### Phase 2 – Project & Plan Enhancements
- Map GraphQL mutation inputs (e.g., `UpdateProjectInput`, `PlanDagNodeInput`) into serde structs shared with MCP and console. Consider a new crate module `shared::dto` consumed by all front-ends.  
- Implement thin adapters for project updates/archival that reuse the shared `ProjectService` operations and return GraphQL-shaped responses.  
- Build plan DAG adapters that call `PlanDagService` methods (`add_node`, `update_node`, `add_edge`, etc.) through the shared façade, allowing GraphQL/MCP/console to reuse validation logic.  
- Expose plan cancellation/monitoring by calling `plan_execution::stop_plan_execution` and reading execution state tables via the shared layer.

### Phase 3 – Graph Editing & Analysis
- Wrap GraphQL graph mutations (`update_graph_node`, `bulk_update_graph_data`, `replay_graph_edits`) into shared service functions invoked by MCP tools and console commands, keeping audit trails intact.  
- Finalise analysis routines: move TODO logic from `mcp/tools/analysis.rs` into reusable functions that consume `GraphService::build_graph_from_dag_graph` outputs; add path-finding utilities under `services` for all front-ends.  
- Provide export preview helpers that reuse `ExportService::export_to_string`, with optional pagination limits configurable per consumer (GraphQL, MCP, console).

### Phase 4 – Data Source & Library Integration
- Extract shared upload/reprocess logic from `graphql/mutations/mod.rs` into `services::data_source_service`/`library_source_service` helpers that accept raw content so MCP tools and console scripts can call them directly.  
- Implement resource URIs for data source downloads pointing to `layercake://datasources/{id}/raw|json`, reusing the same serializer as GraphQL download queries.  
- Document expected MIME types and size limits for agent uploads to avoid blocking the MCP runtime, and reference the same guidance in console docs.

### Phase 5 – Telemetry & Execution Visibility
- Leverage shared helpers over the `execution_state` tables so plan progress is exposed via GraphQL queries, MCP resources, and console reports alike.  
- Add reporting utilities that aggregate graph edit counts using `GraphEditService::count_unapplied_edits`, with lightweight wrappers for each interface.  
- Provide optional polling scripts in docs demonstrating how agents or console users can query status periodically instead of relying on subscriptions.

### Phase 6 – Validation & Documentation
- Build integration tests that spin up GraphQL, MCP, and console entry points against a SQLite test DB, invoking the shared services from each vector to compare their payloads.  
- Add contract tests ensuring serialisation matches GraphQL JSON (e.g., using snapshot tests referencing `frontend/src/graphql` expectations) and asserting MCP/console outputs match the same snapshots.  
- Update developer docs with example MCP/GraphQL/console requests, API-key configuration (for HTTP), and troubleshooting notes for shared service errors.

## Incremental Delivery Checklist

- [ ] **Baseline sanity**  
  - [x] Confirm the current repo builds (`cargo test -p layercake-core`, `npm run build`) and document the versions used; this becomes the starting point for subsequent diffs.  
  - [ ] Tag or branch the baseline so regressions can be bisected quickly during the refactor.  

- [ ] **Bootstrap shared foundation**  
  - [x] Create a lightweight `AppContext` (DB + core services) and wire it into the GraphQL server, MCP server, and console bridge without changing existing behaviour.  
  - [x] Add regression checks (manual or automated) proving project list/create still work via GraphQL after the refactor.  

- [ ] **Project parity**  
  - [ ] Refactor GraphQL project CRUD mutations to call `AppContext`; update MCP project tools to reuse the same helpers.  
  - [ ] Update `layercake://projects/...` resource responses to return live data through the shared DTOs.  
  - [ ] Verify parity across GraphQL + MCP project operations.

- [ ] **Plan summary parity**  
  - [ ] Introduce shared helpers for plan create/update/get/delete and migrate GraphQL mutations.  
  - [ ] Rework MCP plan tools to delegate to the shared helpers.  
  - [ ] Ensure plan resources emit real JSON snapshots identical to GraphQL outputs.

- [x] **Plan DAG read path**  
  - [x] Add an `AppContext::load_plan_dag` helper returning nodes/edges.  
  - [x] Wire GraphQL `getPlanDag` and a new MCP `get_plan_dag` tool to the helper; confirm serialized shapes match.

- [ ] **Plan DAG mutations**  
  - [ ] Wrap node/edge create/update/delete/move logic in shared functions (leveraging `PlanDagService`).  
  - [ ] Migrate GraphQL DAG mutations and expose matching MCP tools; smoke-test both surfaces.

- [ ] **Prepare for Phase 3+**  
  - [ ] Document remaining gaps (graph editing, data sources, telemetry).  
  - [ ] Update test/CI strategy to exercise GraphQL and MCP endpoints in parallel before advancing to later phases.

### Phase 7 – Optional Features (Auth/Collaboration/Chat)
- When prioritised, expose auth tools by wrapping `AuthService` flows and persisting sessions for MCP clients.  
- Mirror collaboration GraphQL mutations using `CollaborationService`, ensuring role validation matches existing resolver checks.  
- Add chat orchestration tools that delegate to `ChatManager`, including commands to list providers and send/receive messages for agent-driven conversations.  
- Align prompt registry definitions with the frontend by loading shared templates from `resources/prompts`.
