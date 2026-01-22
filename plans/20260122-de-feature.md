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
- **2026-01-23**: Completed the inventory pass. Chat/RAG/MCP touches span database entities/migrations (`chat_sessions`, `chat_messages`, `chat_credentials`), `chat_history_service`, `layercake-server/src/chat` modules (config, providers, session, GraphQL manager), GraphQL schema/resolvers/mutations/subscriptions (`chat_manager`, `chat` queries/mutations/subscriptions), and frontend artifacts (`chat.ts` GraphQL, pages `ProjectChatPage`, `ChatLogsPage`, hooks `useChatSession`, components `ChatModal`, `config/chat.ts`). Additional dependencies include MCP agents and forceful GraphQL subscription wiring.
- **2026-01-24**: Removed server-side GraphQL chat/mcp roots, chat history service code, chat database entities/migrations, and CLI console chat commands; the GraphQL context now only wires the remaining services and no longer imports chat-related types.
- **2026-01-25**: Logged the deletion of chat/RAG migrations (`m20251030_000008_create_chat_credentials.rs`, `m20251030_000009_seed_chat_credentials.rs`, `m20251103_000010_create_chat_sessions.rs`, `m20251103_000011_create_chat_messages.rs`, `m20251112_000022_add_rag_to_chat_sessions.rs`), pruned frontend chat components, stores, and utilities, refreshed UI text/navigation/docs (README, dataset origins), and verified the build with `npm run frontend:build` and `cargo check`.
- **2026-01-26**: Completed the Rust workspace audit for lingering `mcp`/genAI keywords, confirmed neither `Cargo.toml` manifests nor GraphQL schema files still reference `layercake-genai`, and dropped the crate plus its SeaORM models/config wiring so the workspace now builds without the genAI surface. 
- **2026-01-27**: Reconciled `GraphQLContext::new` tests with the slimmed context signature (no `ChatManager`), refreshed the console command docs, trimmed `.env.example` so the example environment no longer advertises the removed chat/MCP stack, and validated the narrowed workspace with `cargo check` and `npm run frontend:build` (both pass with pre-existing warnings about unused imports/dynamic imports).
- **2026-01-28**: Updated `README.md` and `AGENTS.md` to describe the reduced surface, linked back to this plan as the authoritative record, and confirmed there are no further high-level docs that still advertise chat/RAG/MCP functionality.
- **2026-01-29**: Added placeholder migrations for the deleted chat/RAG tables so the migration log remains consistent when the database applies previously-run versions, and reran `cargo build` to verify the workspace compiles clean despite lingering unused-import/dead-code warnings.
- **2026-01-30**: Updated `dev.sh` to export `LAYERCAKE_LOCAL_AUTH_BYPASS` when launching the backend so local plan edits remain authorized even though the `.env` sample no longer lists that variable.

## Immediate next steps
- None; the de-feature work is fully documented and complete.
