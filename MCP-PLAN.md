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

- **Primary audience**: MCP endpoints target external/agent integrations. Prioritise plan execution and graph manipulation features agents require; defer UI-centric collaboration/chat features to Phase 7.  
- **Security**: MCP tooling is available both inside the built-in chat UI and over HTTP. Require API-key authentication for HTTP exposure only (see `layercake-core/src/mcp/server.rs`), while allowing trusted, in-process chat usage without keys. Document `Authorization: Bearer …` expectations for external clients.  
- **Realtime scope**: Subscription-style updates are not a near-term requirement. Prefer polling/batch responses for execution status until priorities shift.  
- **Serialization**: Treat GraphQL schemas as the canonical contract. MCP tool responses should match GraphQL field names (PascalCase types, camelCase fields) by reusing shared serializers wherever possible.

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
- Refactor shared service wrappers so MCP tools reuse the same business logic invoked by GraphQL resolvers (projects, plans, graph services).  
- Replace placeholder resource responses with real DB queries via `ProjectService`, `PlanService`, and `GraphService`; implement missing formats for `get_graph_resource`.

### Phase 2 – Project & Plan Feature Parity
- Implement MCP tools for `update_project`, metadata edits, and project archiving to mirror GraphQL mutations.  
- Add plan DAG tooling: list DAG, add/update/delete nodes & edges, move/batch move, validate DAG, aligning payload shapes with GraphQL types.  
- Provide plan deletion/cancellation and execution monitoring commands that call into `PlanDagService` and `PlanExecution`.

### Phase 3 – Graph Editing & Analysis
- Expose node/edge/layer CRUD, bulk updates, graph edit playback/clear, and export preview tools using existing services (`GraphService`, `GraphEditService`).  
- Finish the analysis module: implement `analyze_connectivity` / `find_paths` using actual graph data and surface additional analytics consumed by the UI (degree stats, layering checks).  
- Add streaming/batch endpoints for long-running exports or analysis results using MCP batch API once supported.

### Phase 4 – Data Source & Library Workflows
- Mirror GraphQL mutations for data sources (create from file, update metadata, reprocess, delete, bulk import/export) and library sources (seed, update, reprocess).  
- Support raw/JSON download via MCP resources to keep parity with `download_data_source_*` queries.

### Phase 5 – Telemetry & Execution Visibility
- Surface execution status, graph edit counters, and plan progress through MCP resources or batch operations.  
- Evaluate streaming or incremental responses for long-running tasks once axum-mcp batch APIs mature.

### Phase 6 – Validation, UX, & Documentation
- Build integration tests that exercise MCP tools against a seeded database, comparing results with GraphQL resolver outputs.  
- Update developer docs to describe MCP endpoints and example payloads.  
- Ensure UI + MCP share schemas (e.g., reuse async-graphql input objects via serde to avoid drift).

### Phase 7 – Optional: Authentication, Collaboration, & Chat Parity
- Register authentication tools (`register_user`, `login_user`, `logout_user`) and align API-key/session policies between GraphQL and MCP.  
- Expose collaboration management (invites, role updates, session listings) for agents that need shared project workflows.  
- Provide MCP tasks to start chat sessions, relay messages, and enumerate providers/models using the existing chat manager.  
- Synchronize prompt registry entries with frontend templates (plan creation, graph diagnostics) and document when/why to use them.

## Open Questions / Dependencies

- Confirm final API-key distribution/rotation process for third-party agents.  
- Clarify whether agents require export artifacts pushed to remote storage (vs returned inline) for large graphs.  
- Track axum-mcp batch API maturity to reassess streaming support when needed.

## Technical Implementation Details

### Phase 1 – Shared Services & Resource Parity
- Introduce a `SharedGraphqlBridge` module that exposes reusable helpers from `layercake-core/src/graphql/context.rs` (project, plan, graph services).  
- Refactor MCP tool implementations to accept a lightweight context (DB + services) instead of reimplementing queries.  
- Replace placeholder resource payloads in `layercake-core/src/mcp/resources.rs` with real data fetched via `ProjectService`, `PlanDagService`, and `GraphService::build_graph_from_dag_graph`. Ensure responses serialise to camelCase by leveraging GraphQL DTO structs (`layercake-core/src/graphql/types`).  
- Harden API-key enforcement on HTTP entry points by documenting required headers (`Authorization: Bearer ...`) and ensuring external requests pass through `LayercakeAuth::authenticate`, while privileged chat invocations reuse the trusted security context.

### Phase 2 – Project & Plan Enhancements
- Map GraphQL mutation inputs (e.g., `UpdateProjectInput`, `PlanDagNodeInput`) into serde structs shared with MCP. Consider a new crate module `shared::dto` consumed by both resolvers and tools.  
- Implement MCP handlers for project updates/archival by reusing `ProjectService` operations. Return responses using the same GraphQL shapes (`Project` type conversions).  
- Build plan DAG MCP endpoints that call `PlanDagService` methods (`add_node`, `update_node`, `add_edge`, etc.) and mirror validation logic found in `graphql/mutations/mod.rs`.  
- Expose plan cancellation/monitoring by calling `plan_execution::stop_plan_execution` and reading execution state tables.

### Phase 3 – Graph Editing & Analysis
- Wrap GraphQL graph mutations (`update_graph_node`, `bulk_update_graph_data`, `replay_graph_edits`) into service functions that MCP tools can invoke. Ensure audit trails remain intact.  
- Finalise analysis routines: move TODO logic from `mcp/tools/analysis.rs` into reusable functions that consume `GraphService::build_graph_from_dag_graph` outputs; add path-finding utilities under `services`.  
- Provide export preview endpoints that reuse `ExportService::export_to_string`, controlling response sizes for agents (e.g., optional pagination for node/edge datasets).

### Phase 4 – Data Source & Library Integration
- Extract shared upload/reprocess logic from `graphql/mutations/mod.rs` into `services::data_source_service`/`library_source_service` helpers that accept raw content so MCP tools can call them directly.  
- Implement resource URIs for data source downloads pointing to `layercake://datasources/{id}/raw|json`, mirroring GraphQL download queries.  
- Document expected MIME types and size limits for agent uploads to avoid blocking the MCP runtime.

### Phase 5 – Telemetry & Execution Visibility
- Leverage `execution_state` tables to expose plan progress via tools/resources; reuse GraphQL `planExecutionStatus` resolver logic.  
- Add MCP report commands that aggregate graph edit counts using `GraphEditService::count_unapplied_edits`.  
- Provide optional polling scripts in docs demonstrating how agents can query status periodically instead of relying on subscriptions.

### Phase 6 – Validation & Documentation
- Build integration tests that spin up the MCP server against a SQLite test DB, invoke each tool via HTTP, and compare payloads to GraphQL query outputs.  
- Add contract tests ensuring serialisation matches GraphQL JSON (e.g., using snapshot tests referencing `frontend/src/graphql` expectations).  
- Update developer docs with example MCP requests, API-key configuration, and troubleshooting notes for tool execution errors.

### Phase 7 – Optional Features (Auth/Collaboration/Chat)
- When prioritised, expose auth tools by wrapping `AuthService` flows and persisting sessions for MCP clients.  
- Mirror collaboration GraphQL mutations using `CollaborationService`, ensuring role validation matches existing resolver checks.  
- Add chat orchestration tools that delegate to `ChatManager`, including commands to list providers and send/receive messages for agent-driven conversations.  
- Align prompt registry definitions with the frontend by loading shared templates from `resources/prompts`.
