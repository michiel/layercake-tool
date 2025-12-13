# De-AI Modularization Plan (Optional GenAI)

## Objectives & Scope
- Preserve AI/agentic capabilities (rig-core, MCP, chat runtime) but relocate them into `layercake-genai` as an optional feature crate.
- Keep `layercake-core` clean/minimal by default; AI is opt-in via feature flags and dependency gating.
- Ensure GraphQL, CLI, and frontend chat flows only activate when the genai feature is enabled.
- Maintain existing non-AI functionality (plans/graphs/projections) unchanged and compilable in both modes.

## Assumptions / Inputs
- Chat history/data model can remain but should be isolated under the optional feature; DB migrations stay intact unless we explicitly disable AI at deploy time.
- Default build path: AI off (`genai` feature disabled), minimizing build time and binary size; CI still runs a `genai` feature matrix job.
- Feature flags live at workspace root and propagate to binaries (`layercake`, Tauri backend, server).

## Backend Plan (Rust workspace)
1) **Feature wiring**
   - Add a `genai` cargo feature at workspace and `layercake-core` level; set `default-features = false` for AI deps.
   - Ensure `layercake-genai` is an optional workspace member; gate its dependency in `layercake-core` behind `features = ["genai"]`.
2) **Dependency relocation**
   - Move `rig` (rig-core + rmcp), provider crates, and MCP deps (`axum-mcp`, provider SDKs) into `layercake-genai/Cargo.toml`.
   - Strip AI deps from `layercake-core`/root `Cargo.toml`; re-export only through `layercake-genai` when the feature is on.
   - Move AI-focused examples/tests (`rig_spike*`, `test_rig_integration`) into `layercake-genai/examples` or guard them with `cfg(feature = "genai")`.
3) **Module separation**
   - Relocate chat runtime (`console/chat`, `graphql/chat_manager`, `services/chat_history_service`, chat config) into `layercake-genai` with public interfaces consumed via traits.
   - Provide shims in `layercake-core` that no-op or return `StructuredError::unavailable` when `genai` is off.
4) **GraphQL surface gating**
   - Move chat/mcp GraphQL modules (`graphql/types/chat.rs`, `mutations/chat.rs`, `mutations/mcp.rs`, subscriptions) into `layercake-genai`.
   - In `layercake-core` schema builders, wrap chat/mcp roots with `#[cfg(feature = "genai")]` and register them conditionally.
   - Adjust `GraphQLContext` to inject `ChatManager` and chat config only when the feature is enabled; provide stubs otherwise.
5) **Server/CLI integration**
   - Gate MCP routers and chat websocket routes in `server/app.rs` behind `genai`.
   - Move CLI chat commands/options into `layercake-genai`; expose them through a feature-guarded extension trait so the base CLI compiles without AI.
6) **Data models & migrations**
   - Keep chat tables/entities in place but guard runtime usage with `genai`. If a deployment wants to drop AI entirely, add an opt-in migration to drop tables (separate from default path).
7) **Docs & samples**
   - Document `--features genai` usage and environment variables needed for providers; ensure samples reference the optional feature.

## Frontend Plan (React)
1) **Feature toggle plumbing**
   - Introduce a build-time flag (e.g., `VITE_ENABLE_GENAI`) that drives route/component registration.
   - Default to `false` in `.env.example`; CI matrix builds with both values.
2) **Routing & providers**
   - Wrap chat routes (`/projects/:id/chat`, `/chat/logs`) and nav items in conditional guards.
   - Mount `ChatProvider` and `useRegisterChatContext` only when the flag is true; no-op fallbacks otherwise.
3) **Pages/components**
   - Move chat pages/components/hooks into a `frontend/src/genai/` folder; export them conditionally.
   - Ensure shared layout gracefully collapses when chat is disabled (no gaps in nav).
4) **GraphQL ops & types**
   - Split chat GraphQL documents/types into a genai-specific bundle; only generate/consume them when `VITE_ENABLE_GENAI` is true.
   - Ensure Apollo/urql caches do not reference chat types in non-genai builds.
5) **Build scripts**
   - Add `frontend:build:genai` (flag on) and keep `frontend:build` as default (flag off).

## Build/Test Pipeline Updates
- Cargo: add matrix jobs `cargo build` (default, no genai) and `cargo build --features genai`; mirror for `cargo test`.
- Frontend: run `npm run frontend:build` (flag off) and `npm run frontend:build:genai` (flag on).
- Tauri: default build without genai; optional `tauri:build:genai` that passes the feature/env flag through.
- Update README/BUILD to explain the optional AI path and how to enable it locally/CI.

## Build-Time Impact (estimate)
- **Default (genai off):** Backend build time should drop ~15–25% and binaries shrink due to skipping rig-core/provider/MCP crates. Frontend bundles slightly smaller (chat UI omitted). Tauri builds benefit similarly (~2–4 min saved on clean builds).
- **Genai on:** Build times remain roughly current; overhead isolated to feature builds only.

## Execution Milestones
1) Add `genai` feature wiring and relocate dependencies to `layercake-genai`; ensure core builds without AI.  
2) Move chat runtime/GraphQL/server/CLI modules into `layercake-genai` with conditional registration.  
3) Add frontend flag + conditional routing/components/GraphQL ops; verify both build modes.  
4) Update CI scripts/docs and validate build-time deltas for on/off matrices.  
