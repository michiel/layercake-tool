# docs-tool/ + Agent-Facing API Access

**Date:** 2026-07-15
**Branch:** `feat/docs-tool-and-api-access`
**Goal:** Give agents first-class ways to drive Layercake against a running instance — via HTTP/GraphQL, schema introspection, direct DB access, and the CLI — backed by embedded, discoverable docs surfaced through a new `layercake doc` command. Agents should be able to appear as distinct collaborators in the multi-user UI (presence + cursors), just like a human browser tab. Identify and fix structural gaps that make agent use harder than it should be.

## Multi-user: agents as collaborators (key finding)

The collaboration protocol is **fully client-driven identity**. A client connects to `GET /ws/collaboration?project_id=N` and sends `JoinSession { user_id, user_name, avatar_color, document_id? }` (`websocket/types.rs:56,63-70`). The server broadcasts `UserPresence`/`BulkPresence` to every other connected client (`types.rs:98-115`). There is **no server-side user registry** gating who may join or what name/id they use.

**Implication:** an agent can appear as another user in the UI **with the existing protocol** — no server change required for basic presence. The gap is purely that there is no supported client or docs for an agent to (1) open the WS, (2) announce presence, (3) optionally move a cursor. This becomes a first-class deliverable (Part E).

**CONFIRMED SCOPE (user):** presence + cursors **and visual distinction** (agents render with a bot icon + "Agent" badge, distinct from humans). Mutation-attribution unification was NOT selected → deferred to a non-blocking investigation (Stage F3).

Visual distinction requires a new `is_agent` (bool) field threaded end-to-end:
- Server: `JoinSessionData` + `UserPresenceData` (`websocket/types.rs:63-70,106-115`), the collaboration `types.rs`/`project_actor.rs` presence structs, and the join handler.
- Frontend: `types/websocket.ts` `UserPresenceData`, `UserPresenceIndicator.tsx` (bot icon + Agent badge), `CollaborativeCursor.tsx` (distinct cursor style).
- Backward-compatible: field defaults to `false` (humans unaffected) via `#[serde(default)]`.

## Code-analysis removal (CONFIRMED: remove the whole command)

Clarified: "remove code-analysis from the REPL" → **remove the entire `code-analysis` feature** (it was never in the REPL; it's a top-level command + server/core/frontend stack). Well-isolated: ~15 files delete outright, ~6 careful registration edits.

**Delete outright:** the `layercake-code-analysis/` crate; server `graphql/{mutations,types}/code_analysis.rs`; core `services/code_analysis_service.rs`, `code_analysis_graph.rs`, `code_analysis_solution_graph.rs`, `code_analysis_enhanced_solution_graph.rs`, `infra_graph.rs`, `database/entities/code_analysis_profiles.rs`; frontend `pages/CodeAnalysis{Page,DetailPage}.tsx`, `components/code-analysis/`.

**Careful registration edits:** workspace `Cargo.toml:4`; `layercake-cli/Cargo.toml:37`, `layercake-core/Cargo.toml:63`; `cli/main.rs:3,68-69,187-189`; server `graphql/mutations/mod.rs:11,38`, `queries/mod.rs:9,71-102`, `types/mod.rs:8,25`; core `lib.rs:2-4,11`, `services/mod.rs:6`, `app_context/mod.rs:14,41,62,76,218-220`, `database/entities/mod.rs:1`; frontend `App.tsx:16-17,302-307,2199-2208`.

**⚠️ DB migration — DO NOT REMOVE.** Keep `migrations/m20251208_000003_create_code_analysis_profiles.rs` and its registration (`migrations/mod.rs:37,92`). It's a historical migration already applied to existing DBs; removing it breaks SeaORM migration validation. The orphaned `code_analysis_profiles` table is harmless. (Optional follow-up: a NEW forward `DROP TABLE` migration — deferred, not in this change.)

**Docs:** references in `docs/obsolete/`, `plans/`, `reviews/` are historical — left as-is (no build impact).

---

### (Historical note) The original REPL premise

Code-analysis was NOT wired into the REPL (`repl.rs`) or console. It existed only as:
- the top-level `layercake code-analysis` (alias `ca`) command (`main.rs:68-69,187-189`),
- server GraphQL + core service layers.

(Resolved above: remove the whole command.)

## What we're building

### Part A — `docs-tool/` content tree
```
docs-tool/
  command/     # one .md per agent-facing command topic
  workflow/    # one .md per end-to-end task an agent performs
```
Each file maps to `layercake doc <type> <name>` (name = filename without `.md`).
Example: `docs-tool/workflow/edit-a-plan.md` → `layercake doc workflow edit-a-plan`.

Initial files:
- `workflow/edit-a-plan.md` — drive plan DAG edits end-to-end against a running instance.
- `workflow/drive-via-api.md` — connect to `layercake serve`, run queries/mutations over `/graphql`.
- `workflow/inspect-database.md` — read the SQLite file directly (read-only) for inspection.
- `workflow/run-a-plan-headless.md` — execute a plan with the CLI, no server.
- `workflow/join-as-collaborator.md` — appear as a user in the multi-user UI (presence/cursors).
- `workflow/develop-a-story.md` — **(user-flagged, essential)** agents constructing "stories": stories build sequence diagrams from nodes/edges and add commentary, letting us create stories of sequences based on datasets. Must document the story GraphQL ops + how an agent assembles a sequence + commentary. (Investigate the Story node / sequence GraphQL surface first — `export_sequence_artefact_node`, StoryNode config.)
- `command/serve.md` — start the API/UI server (host/port/open, loopback default).
- `command/query.md` — the `layercake query` one-shot CLI interface.
- `command/schema.md` — dump the GraphQL SDL (new command, below).
- `command/doc.md` — the doc command itself (self-describing / discovery).
- `command/api.md` — the `layercake api` helper: info / call / join (new command, below).
- `command/db.md` — `layercake db info` and existing db commands.

### Part B — `layercake doc` command (embedded, discoverable)
Mirror the existing `Guide`/`GuideCommands` pattern but back it with `include_dir!("docs-tool")` so it scales:
- `layercake doc list` — enumerate all workflows + commands (for agent discovery).
- `layercake doc workflow <name>` — print `docs-tool/workflow/<name>.md`.
- `layercake doc command <name>` — print `docs-tool/command/<name>.md`.
- Unknown name → error listing available names.

### Part C — New agent-facing commands (the "what else do we need")

Structural gaps found and the commands that close them:

1. **`layercake schema dump [--json]`** — print the GraphQL SDL.
   - *Gap:* agents can't use the API without knowing its shape; today they'd have to run a server and introspect. `async_graphql`'s `Schema::sdl()` works WITHOUT a DB/context, so we can build the schema standalone and print SDL offline. `--json` emits the introspection JSON.
   - Low risk, high value, no running server needed.

2. **`layercake api <info|url|call>`** — a thin agent helper for the running instance.
   - `layercake api info` — print resolved endpoint(s) for a host/port (`/graphql`, `/graphql/ws`, `/health`), the session-header name (`x-layercake-session`), and DB path. Machine-readable (`--json`).
   - `layercake api call --query '...' [--variables '{...}'] [--url http://127.0.0.1:3000]` — POST a GraphQL op to a running instance and print the JSON response. Saves agents from hand-rolling curl + headers; reuses the existing HTTP client.
   - *Gap:* there is no CLI path to talk to a running server today (`query` goes straight to the DB, bypassing the server). This gives agents a supported, discoverable HTTP path.

3. **`layercake db info [--database path] [--json]`** — DB location + basic stats.
   - Prints resolved DB path, existence, size, and (optionally) row counts for key tables. Read-only.
   - *Gap:* the Tauri-only DatabaseSettings gave this in the UI (removed in #71, tracked as gh-72). A CLI equivalent is the natural web-first replacement and directly serves "the database file" access mode.

### Part E — Agent presence in the multi-user UI
4. **`layercake api join`** — connect to the collaboration WS and hold a presence session.
   - `layercake api join --project N --name "Claude agent" [--id <uuid>] [--color "#7c3aed"] [--url ...]` — opens `/ws/collaboration?project_id=N`, sends `JoinSession`, keeps the connection alive (heartbeat `Ping`) until Ctrl-C, so the agent shows up as an online collaborator in every open browser.
   - Optional `--cursor x,y` / stdin-driven cursor updates (later; basic presence first).
   - *Gap:* no supported client exists for an agent to announce presence. This is the direct answer to "agents showing up as other users."
   - Reuses the existing `ClientMessage`/`ServerMessage` types (could depend on `layercake-server` types or re-declare a minimal JSON client). Verify a browser sees the agent join/leave.

### Part D — Structural fixes / friction reduction
- **Health/info endpoint parity:** confirm `/health` returns enough for an agent to detect readiness + version (it returns `{service,status,version}` — good; document it).
- **Session header discoverability:** the API expects `x-layercake-session`; document it in `command/api.md` and expose via `layercake api info`.
- **`query` vs server ambiguity:** docs must be explicit that `layercake query` operates directly on the DB file (no server), while `layercake api call` targets a running server. This is the single biggest agent-confusion risk.
- **Schema availability offline:** `schema dump` removes the need to boot a server just to learn the API.

## Stages

Ordered so each stage compiles + is independently verifiable and committable. Group **C** (code-analysis removal) is independent of the rest and can land first or last; I'll do it after the additive commands so a build break there can't mask new-feature bugs.

### Stage 1 — `layercake doc` command + `docs-tool/` skeleton
Add `Doc`/`DocCommands` (`list` / `workflow <name>` / `command <name>`) via `include_dir!("$CARGO_MANIFEST_DIR/../docs-tool")`. Create the dir with `workflow/edit-a-plan.md` + a couple more. `doc list` enumerates files; unknown name errors with the list.
**Verify:** `layercake doc list`, `layercake doc workflow edit-a-plan` print; bad name errors.
**Status:** Complete

**Notes:** Added `layercake-cli/src/doc.rs` (`include_dir!("../docs-tool")` — 0.6.2 literal path, `.files().iter()`); `Doc`/`DocCommands` in main.rs; docs-tool/{command,workflow}/ with `edit-a-plan.md`, `drive-via-api.md`, `command/doc.md`. Added `include_dir` to cli Cargo.toml. Verified list/print/error paths. NOTE: cli crate currently still depends on `layercake-code-analysis` (removed in Stage 7).

### Stage 2 — `layercake schema dump [--json]`
Build the GraphQL schema standalone (no DB context) in the CLI (depends on `layercake-server` types) and print `.sdl()`; `--json` → introspection query result.
**Verify:** SDL contains `type Query`, `type Mutation`, `Project`; runs with no server/DB.
**Status:** Complete

**Notes:** Added `sdl()` / `introspection_json()` / `build_schema_for_introspection()` to `layercake-server/src/graphql/schema.rs` (keeps async-graphql encapsulated in the server crate — cli has no async_graphql dep). cli `schema_dump.rs` delegates to those. `Schema` command with `Dump { --json }`. Verified SDL has core types, `--json` is valid introspection JSON, and NO db file is created (truly offline).

### Stage 3 — `layercake db info [--database] [--json]`
Resolve DB path; print path/existence/size (+ optional key-table row counts) as text or JSON.
**Verify:** against a scratch DB and a missing path.
**Status:** Complete

**Notes:** `db_info.rs` reports path/absolute/exists/size, filesystem-only (safe while server holds the DB). Added `DbCommands::Info { database, json }`. `docs-tool/command/db.md`. Verified text + `--json` + missing-file. Deferred `--stats` (table row counts, needs a read-only connection) as not needed now.

### Stage 4 — `layercake api info` + `layercake api call`
`api info [--host --port] [--json]` prints endpoints (`/graphql`, `/graphql/ws`, `/health`), the `x-layercake-session` header name, resolved DB path. `api call --query [--variables] [--url]` POSTs a GraphQL op to a running server, prints JSON. Reuse `reqwest` (already a dep).
**Verify:** `api call '{ __typename }'` against a live `layercake serve` returns data; `api info --json` parses.
**Status:** Complete

**Notes:** Added `reqwest` to cli. `api.rs`: `info` (endpoints/headers/db, text+json) and `call` (POST GraphQL, `--variables` inline or `@file`, `--url` or `--host/--port`, `--session`, pretty JSON, clear connection-refused error). `Api`/`ApiCommands`. `docs-tool/command/api.md` with the `api call` vs `query` table. Verified against a live server: `__typename`, `projects`, variables transmitted, and no-server error path.

### Stage 5 — `layercake api join` (agent presence) + visual distinction
- CLI: `api join --project N --name --id? --color? --agent --url?` opens `/ws/collaboration?project_id=N`, sends `JoinSession` (with new `is_agent`), heartbeats until Ctrl-C.
- Protocol: add `is_agent: bool` (`#[serde(default)]`) to `JoinSessionData` + `UserPresenceData` and the collaboration presence structs; thread through `project_actor`/handler.
- Frontend: add `isAgent` to `types/websocket.ts`; `UserPresenceIndicator` shows a bot icon + "Agent" badge; `CollaborativeCursor` uses a distinct style for agents.
**Verify:** a second WS client (or browser) sees the agent join with `is_agent=true`; humans still render normally (default false); `tsc --noEmit` clean.
**Status:** Complete

**Notes:** Threaded `is_agent: bool` end-to-end server-side (mirroring `avatar_color`): `JoinSessionData`/`UserPresenceData`/`UserPresence` (websocket/types.rs), `CoordinatorCommand::JoinProject`/`ProjectCommand::Join` (collaboration/types.rs), `join_project`/`join` fns (coordinator.rs, project_actor.rs), `UserData` + 3 `UserPresenceData` build sites, handler passes `data.is_agent`. All `#[serde(default)]` → backward-compatible (humans omit it → false). CLI `api join` (tokio-tungstenite): connects `/ws/collaboration?project_id=N`, sends `join_session` with `isAgent`, heartbeats, prints presence, clean leave on Ctrl-C. Frontend: `isAgent?` on `UserPresenceData`, `UserPresenceIndicator` (robot icon + Agent badge), `CollaborativeCursor(s)` (robot cursor + 🤖 label). **Verified E2E:** observer WS client received `user_presence` with `"isAgent":true`; server+cli build; `tsc --noEmit` clean. `docs-tool/workflow/join-as-collaborator.md`.

### Stage 6 — Fill out `docs-tool/` + discovery + README
Complete all workflow/command md files; cross-reference; document the `query`-vs-`api call` distinction prominently; README "Agents / API access" pointer.
**Verify:** every command has a doc; `doc list` shows all.
**Status:** Complete

**Notes:** Wrote all remaining docs: workflows `develop-a-story` (grounded in the real Story/Sequence model — createStory → createSequence(edgeOrder w/ note+notePosition) → updatePlanDag(StoryNode→SequenceArtefactNode) → exportNodeOutput; showNotes caveat), `inspect-database`, `run-a-plan-headless`; commands `serve`, `query`. README "Agents / API access" section. `doc list` shows 6 workflows + 6 commands. **Robustness fix:** added `layercake-cli/build.rs` with `rerun-if-changed=../docs-tool` — `include_dir!` alone doesn't re-embed newly-added files (would be a CI trap); verified a new doc appears without touching any .rs.

### Stage 7 (group C) — Remove the `code-analysis` feature
**On its own branch/PR** (`feat/remove-code-analysis`, off master) per user decision — not on this branch.
Delete the crate + server/core/frontend files; apply the careful registration edits (see the code-analysis section). **Keep the historical DB migration.** Remove frontend routes/nav/imports.
**Verify:** `cargo build --workspace` + `cargo tree | grep code.analysis` empty (except kept migration); `tsc --noEmit`; server tests pass; migrations still run against an existing DB.
**Status:** Not Started

### Stage F3 (investigate, non-blocking, may defer) — unify agent identity across HTTP mutations + WS presence
Decide whether agent GraphQL mutations should be attributed to the same identity shown in presence, and whether it needs a server change. Report; implement only if cheap and you approve.
**Status:** Not Started

## Decisions locked
- Doc command: **embed + list + print** (`include_dir!`).
- API access modes: **HTTP/GraphQL, SDL introspection, direct DB, CLI/query** + gap-fixes.
- Command set: **doc + schema + api (info/call/join) + db info**.
- Agent presence: **presence + cursors + visual distinction** (`is_agent` field end-to-end).
- Code-analysis: **remove the whole feature**; keep the historical migration.
- Mutation-attribution unification: **deferred** (Stage F3, non-blocking).
