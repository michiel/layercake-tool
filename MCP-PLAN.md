# MCP Interface Parity Plan

## Current State Overview

- **GraphQL / Frontend**  
  - Projects: full CRUD, sample seeding, collaborator management, auth, plan DAG orchestration, data source ingestion, graph editing, exports, and chat sessions (`layercake-core/src/graphql/mutations/mod.rs`, `layercake-core/src/graphql/queries/mod.rs`).  
  - Web UI consumes these mutations/queries for plan editing, graph visualization, data source previews, library sources, and real-time chat (`frontend/src/pages/GraphEditorPage.tsx`, `frontend/src/pages/ProjectChatPage.tsx`).
- **MCP Server**  
  - Exposes limited project, plan, and graph helpers plus stubbed analysis tools (`layercake-core/src/mcp/tools/{projects,plans,graph_data,analysis}.rs`).  
  - Tool registry lists only CRUD-lite operations; authentication helpers exist but aren’t registered, and resources/prompts return placeholder data (`layercake-core/src/mcp/server.rs`, `layercake-core/src/mcp/resources.rs`).  
  - Analysis utilities and resources for graphs are unimplemented; no coverage for plan DAG nodes/edges, data sources, library sources, or collaboration flows.

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
- Register existing auth helpers (`register_user`, `login_user`, `logout_user`) in the tool registry and expose secure API-key gating.  
- Replace placeholder resource responses with real DB queries via `ProjectService`, `PlanService`, and `GraphService`; implement missing formats for `get_graph_resource`.

### Phase 2 – Project & Plan Feature Parity
- Implement MCP tools for `update_project`, `list_collaborators`, `invite_collaborator`, etc., mirroring GraphQL mutations.  
- Add plan DAG tooling: list DAG, add/update/delete nodes & edges, move/batch move, validate DAG, aligning payload shapes with GraphQL types.  
- Provide plan deletion/cancellation and execution monitoring commands that call into `PlanDagService` and `PlanExecution`.

### Phase 3 – Graph Editing & Analysis
- Expose node/edge/layer CRUD, bulk updates, graph edit playback/clear, and export preview tools using existing services (`GraphService`, `GraphEditService`).  
- Finish the analysis module: implement `analyze_connectivity` / `find_paths` using actual graph data and surface additional analytics consumed by the UI (degree stats, layering checks).  
- Add streaming/batch endpoints for long-running exports or analysis results using MCP batch API once supported.

### Phase 4 – Data Source & Library Workflows
- Mirror GraphQL mutations for data sources (create from file, update metadata, reprocess, delete, bulk import/export) and library sources (seed, update, reprocess).  
- Support raw/JSON download via MCP resources to keep parity with `download_data_source_*` queries.

### Phase 5 – Collaboration, Chat, & Prompts
- Surface collaboration tools (invites, accept/decline, role updates) and session listings to MCP clients.  
- Provide MCP tasks to start chat sessions, relay messages, and list available providers/models; reuse GraphQL chat manager for consistency.  
- Synchronize prompt registry with common frontend flows (plan creation templates, graph diagnostics) and document usage.

### Phase 6 – Validation, UX, & Documentation
- Build integration tests that exercise MCP tools against a seeded database, comparing results with GraphQL resolver outputs.  
+- Update developer docs to describe MCP endpoints, authentication, and example payloads.  
- Ensure UI + MCP share schemas (e.g., reuse async-graphql input objects via serde to avoid drift).

## Open Questions / Dependencies

- Confirm desired security posture for MCP (API keys vs. session reuse) before exposing collaborator tools.  
- Determine if MCP clients need subscription-style updates; axum-mcp batch/streaming support may be required.  
- Align serialization formats (camelCase vs snake_case) between GraphQL responses and MCP tool outputs for predictable consumer experience.  
- Identify priority order with stakeholders—if MCP is mainly for agent integrations, focus first on plan/graph tooling before collaboration features.
