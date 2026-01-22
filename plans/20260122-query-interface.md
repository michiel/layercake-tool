## Goal
Deliver a JSON-first CLI/REPL query surface that reuses the existing GraphQL services so humans and TUI agents can manage datasets, plans, plan elements, and render downloads without depending on the web UI. Ensure every key artifact (dataset, plan, DAG node/edge) carries a deterministic identifier (`dataset:PROJECT_ID:DATASET_ID`, `plan:PROJECT_ID:PLAN_ID`, `plannode:PROJECT_ID:PLAN_ID:NODE_ID`) and surface copyable UI controls so these IDs can be retrieved without ambiguity while agents operate over CLI/REPL.

## Scope
- Reuse the current service layer behind `layercake-server`/`layercake-core` so the CLI operates against the canonical implementations of dataset/plan/DAG CRUD and graph rendering. Refactor helpers as needed so both GraphQL resolvers and CLI commands share code rather than duplicating logic.
- Add `layercake query --project=<id> ...` commands that map to GraphQL operations for listing/creating/updating/deleting datasets, plans, nodes, edges, metadata, and exports. Inputs and outputs must be JSON (inline strings or files, structured responses including error codes) to suit agent automation.
- Implement `layercake repl` (and `layercake repl; set project=<id>`) to maintain project context, accept textual commands (`list nodes`, `create node ...`, `download graph node <nodeId>`), and delegate to the same GraphQL helper layer. The REPL should prefer JSON responses so TUI agents can parse results; add optional conveniences like simple history or command aliases.
- Ensure payloads include project/plan context so new nodes automatically inherit the correct identifiers and render/download actions have the necessary scope. Surface copy buttons in the web UI (e.g., on plan DAG nodes, dataset/plan lists) so users can grab the canonical IDs for conversations and command-line invocation. Provide download paths or streamed blobs back to the caller.
- Document the new CLI/REPL surface and the JSON contract in `README.md`, `CHANGES.md`, and the plan file itself.

## Constraints
- Maintain existing `layercake` CLI command structure; add `query`/`repl` subcommands without breaking prior flags.
- Avoid reimplementing services: the CLI must call the same Rust modules used by the GraphQL resolvers (AppContext, PlanDagService, GraphService, etc.).
- Emit structured JSON for every command (success, data, or error) while allowing optional human-friendly formatting when `--pretty` is requested.
- Respect authorization: CLI/REPL should honor the same user/session context as GraphQL (pass `LAYERCAKE_LOCAL_AUTH_BYPASS`, session headers, etc.) to prevent permission errors.

## Strategy
1. **GraphQL-backed helpers**
   - Identify the GraphQL queries/mutations used by the frontend for datasets, plans, plan nodes/edges, metadata, and graph exports. Refactor shared logic into helper modules under `layercake-core`/`layercake-server` so both GraphQL and CLI code share the same authenticator and service wiring.
   - Wrap these helpers with functions that accept JSON-serializable structs, convert CLI flags/JSON blobs into GraphQL variables, and return serializable results (including GraphQL errors/messages).
2. **JSON-first CLI**
   - Extend `layercake-cli` with a `query` subcommand whose arguments specify the entity (`datasets`, `plans`, `nodes`, `edges`, `exports`), the action (`list`, `get`, `create`, `update`, `delete`, `download`), and any JSON payloads/filters. Accept `--input-json`/`--input-file` for request bodies and `--output-json` (default) to emit responses; optionally allow `--pretty` for human-friendly output when interactive.
   - Ensure render/download commands can write binary exports to disk but still return metadata as JSON so agents know where the file landed.
3. **Agent-focused REPL**
   - Add a `layercake repl` mode backed by a minimal readline loop (or `clap-repl`) that keeps `projectId`/`planId` in session state, allows `set project=<id>` and `set plan=<id>`, and maps short commands (`list nodes`, `create node { ... }`, `download graph-node <id>`) to the same `query` helpers.
   - Provide JSON output for every REPL action and allow piping commands via stdin for TUI agents, plus simple command history for interactive tinkering.
4. **Auth & config**
   - Ensure the CLI includes session IDs/headers (mirroring `x-layercake-session`) so requests see the same actor as the frontend; support `--token` or `--session=...` for agent tooling.
   - Document how to run commands locally (e.g., `LAYERCAKE_LOCAL_AUTH_BYPASS=1 layercake query ...`) and mention the upcoming JSON interface in the README/CHANGES/TODO.

## Deliverables
- `layercake query` command covering dataset/plan/node/edge CRUD, metadata inspection, and graph downloads with JSON I/O.
- `layercake repl` interactive shell with project/plan context, command shortcuts, JSON responses, and optional history.
- Shared GraphQL helper modules enabling CLI/REPL to reuse the service layer behind the GraphQL API.
- UI affordances (copy-ID buttons, identifier display) so the canonical `dataset:...`, `plan:...`, and `plannode:...` strings can be captured easily from the frontend.
- Documentation describing the CLI/REPL usage, JSON contract, and the agent-focused story (TUI or scripted automation).

## Next steps
- Implement shared Rust helpers that wrap GraphQL queries/mutations and return JSON-ready structs.
- Build the CLI argument parsing and REPL scaffolding, wiring them to the helper layer.
- Update README and docs with examples of `layercake query`/`repl` usage and JSON responses so agents can onboard quickly.

## Implementation roadmap
1. **Shared GraphQL helpers (Stage 1)** – Refactor the underlying GraphQL services into reusable helpers (datasets, plans, nodes, edges, exports) that accept JSON-friendly structs. Ensure identifiers follow the canonical `dataset:<project>:<dataset>`, `plan:<project>:<plan>`, and `plannode:<project>:<plan>:<node>` formats. Implementation steps:
   * Extract helper functions from `layercake-server/src/graphql/mutations/*.rs` and `queries/mod.rs` (e.g., `add_plan_dag_node`, `get_plan_dag`, `export_node_output`) into `layercake-core/src/services/cli_graphql_helpers.rs` that accept a `CliContext { project_id: i32, plan_id: Option<i32>, session_id: Option<String> }`.
   * Ensure every helper emits structured results (Rust structs deriving `Serialize`) and errors mapped through `core_error_to_graphql_error`. Example helper signature:
     ```rust
     pub async fn cli_create_plan_node(
         ctx: &CliContext,
         node_input: PlanDagNodeInput,
     ) -> CoreResult<PlanDagNodeResult> { ... }
     ```
   * Normalize identifiers inside these helpers before routing to the GraphQL layer:
     ```rust
     fn canonical_plan_id(project_id: i32, plan_id: i32) -> String {
         format!("plan:{project_id}:{plan_id}")
     }
     ```
2. **CLI query command (Stage 2)** – Build `layercake query` via Clap with JSON I/O flags. Detailed instructions:
   * Extend `layercake-cli/src/commands.rs` with a `Query` subcommand that accepts `--entity`, `--action`, `--project`, `--plan`, `--payload-file`, `--payload-json`, and `--output-file` flags.
   * Map actions to helper invocations, e.g., `layercake query --entity node --action create --payload-json '{"nodeType":"GraphNode",...}'`.
   * Output JSON using `serde_json::to_string_pretty`; include the canonical ID in the `data` object:
     ```rust
     println!(
         "{}",
         serde_json::json!({
             "status": "ok",
             "result": created_node,
             "canonicalId": format!("plannode:{project}:{plan}:{node_id}")
         })
     );
     ```
   * When downloading graph renders, write the binary to `--output-file` while returning JSON metadata about the file path.
3. **Interactive REPL (Stage 3)** – Provide `layercake repl` hooking into the query helpers:
   * Reuse `clap_repl` or a simple `rustyline` loop; maintain `SessionState { project_id: Option<i32>, plan_id: Option<i32> }`.
   * Commands should parse JSON arguments and call the same CLI helpers so the underlying wiring stays shared.
   * Ensure each REPL response is a JSON object printed on a single line for easy parsing by agents. Example REPL command output:
     ```json
     {
       "command": "list nodes",
       "status": "ok",
       "result": [ ... ],
       "project": 24,
       "plan": 31
     }
     ```
4. **UI ID affordances (Stage 4)** – Add copy buttons to frontend components:
   * Update `frontend/src/components/datasets/DatasetsPage.tsx`, `PlansPage`, and `PlanVisualEditor/nodes/BaseNode.tsx` to show a small copy icon next to each canonical ID (constructed via the `projectId` context and the data ID). Example:
     ```tsx
     const canonicalId = `dataset:${projectId}:${dataset.id}`;
     return (
       <Button onClick={() => copyToClipboard(canonicalId)}>Copy ID</Button>
     );
     ```
   * Ensure plan nodes render the identifier in the toolbar or context menu so users can share it with agents instantly.
5. **Documentation & rollout (Stage 5)** – Document everything:
   * Add a README section “Command-line query interface” with examples for `layercake query --entity nodes --action list`.
   * Update `CHANGES.md` noting JSON I/O and canonical IDs.
 * Mention the agent/TUI scenario in this plan, referencing the stages above.

## Status
- **Stage 1** – GraphQL helper wiring: ✅ extracted shared CLI helpers that reuse `AppContext` services, return canonical IDs (`dataset:*`, `plan:*`, `plannode:*`, `edge:*`), and emit JSON-ready structs from `CliContext`.
- **Stage 2** – CLI query command: ✅ added `layercake query` with Clap flags, JSON payload handling, and hooks into the shared helpers for datasets, plans, nodes, edges, and exports (download previews + file writes).
- **Stage 3** – REPL surface: ✅ added `layercake repl` with project/plan context, JSON responses, command parsing (`set`, `list`, `create/update/delete/move nodes & edges`, `download export`), interactive prompt, and stdin piping so agents can reuse the same helper layer.
- **Stage 4** – UI affordances: pending; add copy-ID controls across dataset/plan/node lists so canonical identifiers are easily accessible.
- **Stage 5** – Documentation: pending; document the query/repl surface and canonical ID strategy in README/CHANGES.
