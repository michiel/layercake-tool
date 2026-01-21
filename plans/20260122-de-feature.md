## Goal
Eliminate the under-developed RAG/chat/genAI surface and the MCP infrastructure so the product no longer ships or maintains unused components.

## Scope
- Remove RAG/chat-related backend services (data models, config, GraphQL schema, handlers, and history persistence).
- Purge front-end UI/pages, hooks, and GraphQL fragments/queries tied to chat/RAG/collaboration.
- Drop MCP-related crates/services/integration wiring from backend, CLI, and supporting scripts/config.
- Clean up shared configuration, env vars, documentation, and build tasks referencing the deprecated features.

## Constraints
- Preserve existing plan-editing, projection, dataset, graph, and CLI functionality.
- Keep workspace buildable: update Cargo/npm manifests after removing crates and dependencies.
- Maintain roll-forward traceability (plan file is implementation record, not directly runnable).

## Strategy
1. **Inventory & dependency graph**
   - Map chat/RAG/MCP GraphQL schema entries and resolvers (`layercake-core/src/graphql`, `layercake-server/src/graphql`, `frontend/src/graphql`).
   - List backend services and crates specific to OpenAI/MCP/agentic workflows (`layercake-core/src/services/chat_*`, `layercake-server/src/mcp_*`, `layercake-cli/src/agent`, etc.).
   - Identify front-end pages/components using chat, history, or MCP data (`frontend/src/pages/Chat*`, `components/chat`, `hooks/useChat`, etc.).
   - Record config/env settings to drop (`layercake-server/tauri.conf`, `.env.sample`, `AppConfig` sections, etc.).

2. **Backend cleanup**
   - Remove chat/RAG domain models, enums, tables, migrations, and services. Update `schema.rs`, `db` modules, and any persistence layers.
   - Delete GraphQL schema types/inputs/mutations/queries for chat, RAG, and MCP. Adjust Apollo resolvers accordingly.
   - Strip MCP client/server crates and any bridging utilities (including `projection-mcp`, `axum-mcp`, or `layercake-mcp` services).
   - Reconcile Cargo workspace `Cargo.toml`, `package.json`, and `tauri.conf` to drop dependencies (OpenAI SDK, `graphql-ws` chat subscriptions, etc.).
   - Ensure CLI commands referencing chat/MCP are removed so `layercake` binary builds clean.

3. **Frontend removal**
   - Delete chat-specific pages, routes, components, and styles.
   - Remove GraphQL queries/mutations/fragments for chats/RAG and MCP telemetry from `frontend/src/graphql`.
   - Cleanup hooks, stores, and providers that manage chat history, streaming connections, or MCP subscriptions.
   - Update navigation, routes, and dashboards to omit now-missing screens (chat, shared workspaces, RAG status).
   - Remove assets (icons, images) dedicated to these features.

4. **Configuration and docs**
   - Remove chat/RAG/MCP entries from default config files, `README`, `AGENTS.md`, `docs/` guides, and environment templates.
   - Update deployment scripts (e.g., Dockerfiles, scripts in `dev.sh`) so they no longer expect chatbot or MCP services.
   - Ensure `plans/README` or release notes mention the de-feature change for traceability.

5. **Validation**
   - Run `cargo check` for all crates and `npm run frontend:build` / `npm run projections:build` to confirm no references remain.
   - Sweep GraphQL schema to ensure only required types remain.
   - Run existing tests that rely on chat/MCP? Remove or replace any impacted tests accordingly.

## Deliverables
- Clean backend codebase with no RAG/chat/MCP services.
- Minimal frontend without chat-related UI/GraphQL.
- Updated configs/docs describing the new feature set.
- This plan file stored at `plans/20260122-de-feature.md` documents the phased implementation steps.

## Progress log
- **2026-01-22**: Captured the de-feature strategy, identified key areas (schema, services, UI, MCP) to remove. Document now reflects the phased approach and scope boundaries.

## Immediate next steps
1. Execute the inventory step by listing chat/RAG/MCP schema nodes, services, and UI entry points so we can target removals safely.
2. Prioritize backend schema/service deletions (models, migrations, GraphQL resolvers) that are currently unused.
3. Once backend removals finish, revisit the frontend routes/queries to drop references to the removed APIs.
