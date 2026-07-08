# Horizon 1 — Stabilise: Implementation Plan

**Source review:** `reviews/2026-07-09-robustness-suitability-roadmap.md` (§3 Horizon 1)
**Branch:** `feature/horizon-1-stabilise`
**Started:** 2026-07-09
**Goal:** No silent data loss, no fail-open auth, no stranded backend state. This is the prerequisite for any shared/client use and for all later horizons.

## Guiding constraints

- Incremental: every stage compiles and passes tests before moving on.
- Test-driven where the surface allows (backend integrity + security). Frontend fixes get behaviour tests where a harness exists; otherwise manual verification notes.
- Match existing patterns (`context.app.*` authorized path; SeaORM `TransactionTrait`; client-scoped subscription filtering).
- Do not touch the pre-existing `external-modules/axum-mcp` submodule deletion; keep it out of commits.

## Status legend

`Not Started` · `In Progress` · `Complete` · `Blocked`

---

## Stage 1: Security — close the fail-open gaps (R1, R5)

**Goal:** Authorization cannot be bypassed without an explicit opt-in, and DAG-execution mutations are authorized.

**Why first:** Highest severity, smallest change, independently shippable. Unshippable to any shared instance until fixed.

**Tasks**
- [ ] 1.1 (R1) Flip `layercake-core/src/services/authorization.rs:197` `local_auth_bypass_enabled()` default from `true` → `false` (unset env var = bypass OFF). Align semantics with the server helper (`layercake-server/src/auth/mod.rs:51`), which already defaults to `false`.
- [ ] 1.2 (R1) Centralise the two duplicate `local_auth_bypass_enabled()` helpers into one canonical function in `layercake-core` and have the server re-use it, so the two can never diverge again.
- [ ] 1.3 (R1) Ensure `dev.sh` / dev tooling still sets `LAYERCAKE_LOCAL_AUTH_BYPASS=true` explicitly for local convenience (CHANGES.md shows dev flows rely on it). Confirm `.env`/`dev.sh` already do this.
- [ ] 1.4 (R5) Add a `pub async fn authorize_project_write_access(&self, actor, project_id)` (and read variant if needed) to `AppContext` (wraps the existing private `authorize_project_write`).
- [ ] 1.5 (R5) In `layercake-server/src/graphql/mutations/plan.rs::execute_plan` and `plan_dag_nodes.rs::execute_node`: build `actor = context.actor_for_request(ctx).await` and call `context.app.authorize_project_write_access(&actor, project_id)` before running the executor.

**Success criteria**
- With no env var set, an unauthenticated/unauthorised actor is denied on `check_project_access` and on `execute_plan`/`execute_node`.
- With `LAYERCAKE_LOCAL_AUTH_BYPASS=true`, local dev flows still work unrestricted.
- Only one `local_auth_bypass_enabled` implementation exists.

**Tests**
- Unit: `local_auth_bypass_enabled()` returns `false` when unset, `true` when `=true`.
- Integration: `execute_plan`/`execute_node` denied for an actor without project write (bypass off); allowed with editor role.

**Status:** Not Started

---

## Stage 2: Backend transactional integrity (R2, R8)

**Goal:** DAG execution and edit replay never leave persistent state half-written or stranded in `Processing`.

**Tasks**
- [ ] 2.1 (R2) Add `GraphDataService::replace_contents(graph_data_id, nodes, edges)` that performs the node-delete/insert + edge-delete/insert + count updates inside a **single** transaction (refactor `replace_nodes`/`replace_edges` to share an inner `_in_txn` helper). Update `DagExecutor::persist_graph_contents` to call it.
- [ ] 2.2 (R2) Wrap each DAG node's execute body so that on any error it calls `mark_error(graph_data_id, err)` (currently `mark_error` has zero callers). Cover `execute_dataset_reference_node`, `persist_transformed_graph`, and the MergeNode path.
- [ ] 2.3 (R2) Add a startup reconciliation: on server boot, any `graph_data` row left in `Processing` is transitioned to `Error` ("interrupted") so it is visibly recoverable rather than silently stuck. Wire into `layercake-server` startup.
- [ ] 2.4 (R8) Wrap the graph-edit replay batch (`graph_edit_service.rs:352-416`) so edit application + `mark_edit_applied` for the batch is atomic (single transaction), with rollback on failure. Preserve the existing "discard non-matching edit" semantics.

**Success criteria**
- Killing execution mid-node never leaves `edges=0`-with-`nodes>0` committed and never leaves a row stuck in `Processing` after restart.
- Replay is all-or-nothing per invocation.

**Tests**
- `replace_contents` atomicity: inject a failure between node and edge write → row unchanged (both old or both new, never mixed).
- `mark_error` called on executor failure (mock/inject).
- Startup reconciliation flips a seeded `Processing` row to `Error`.
- Replay batch: a failing edit mid-batch rolls back the whole batch.

**Status:** Not Started

---

## Stage 3: Frontend — kill wall-clock coordination (R3, R12)

**Goal:** Remote deltas are never dropped; drag/refresh no longer relies on `setTimeout` guesses.

**Tasks**
- [ ] 3.1 (R3) Remove the 500 ms `MUTATION_ECHO_WINDOW_MS` echo suppression in `PlanDagQueryService.ts:120-133` (and the coordinated `getCommandTimestamp`/`markMutationOccurred` timestamp path in `PlanDagCommandService.ts`/`PlanDagCQRSService.ts`). Rely solely on the existing per-`clientId` filter (`PlanDagQueryService.ts:67`), which is correct and sufficient.
- [ ] 3.2 (R12) Replace the fixed `setTimeout(100)` drag re-enable in `PlanVisualEditor.tsx:439-505` and the 200 ms refresh delay in `usePlanDagCQRS.ts:570-577` with completion-driven gating (re-enable external sync when the move mutation resolves, not after a fixed delay).

**Success criteria**
- Two clients editing different nodes simultaneously both see each other's changes (no dropped deltas).
- A node dragged while a slow mutation is in flight keeps its new position (not overwritten by a stale sync).

**Tests / verification**
- Existing frontend test suite passes (`npm run frontend:build` typecheck at minimum).
- Manual two-client verification note recorded in this file.

**Status:** Not Started

---

## Stage 4: Frontend — durable editing (R4, R6, R7, R10, R11)

**Goal:** No editor silently loses or fakes a save.

**Tasks**
- [ ] 4.1 (R4) Flush pending debounced saves on unmount in `useUnifiedUpdateManager.ts:236-241` (call `flushOperations()` before `clearTimers()`/queue clear).
- [ ] 4.2 (R6) Guard the `useEffect(…, [graphData])` reset in `GraphSpreadsheetEditor.tsx:102-124` behind a dirty check so an incoming prop object does not wipe unsaved local edits.
- [ ] 4.3 (R7) `NodePropertiesForm.tsx:32-55`: drive the "Saved" indicator from the actual mutation result (await/inspect `onUpdate`), not a fixed `setTimeout`; surface failure. Also persist on unmount/close, not blur-only.
- [ ] 4.4 (R11) Surface autosave failures in `usePlanDagCQRS.ts:147-160` / `useUnifiedUpdateManager.ts:106-123`: restore `isDirty` on failure and notify the user instead of swallowing to `console.error`.
- [ ] 4.5 (R10) Add a `beforeunload` guard (hook) that warns when any editor holds unsaved/dirty state, wired to the dirty-state introduced above.

**Success criteria**
- Typing in the spreadsheet then triggering a background refetch does not lose the edits.
- A failed save is visibly reported and leaves the editor dirty (retryable), never shows "Saved".
- Closing the tab with unsaved edits prompts.

**Tests / verification**
- Typecheck passes. Manual verification notes recorded here for each editor.

**Status:** Not Started

---

## Stage 5: Concurrency correctness + tests for the untested paths (R9 + coverage)

**Goal:** Concurrent edits can't collide on sequence numbers; the previously untested critical paths gain tests.

**Tasks**
- [ ] 5.1 (R9) Add a unique constraint on `(graph_id, sequence_number)` via a new migration; allocate sequence numbers atomically (inside the create transaction / via `INSERT … RETURNING` or a retry-on-conflict).
- [ ] 5.2 Add tests for `graph_data_edit_applicator` ordering + idempotence (the current unified path — legacy applicator is already tested).
- [ ] 5.3 Add at least basic collaboration coordinator/actor concurrency tests (currently zero).
- [ ] 5.4 Add DAG failure-path tests (partial write, mark_error, reconciliation) — overlaps Stage 2 tests; consolidate.

**Success criteria**
- Concurrent `create_edit` on one graph never yields duplicate `sequence_number`.
- New tests are green and cover ordering/idempotence/failure.

**Status:** Not Started

---

## Exit criteria (whole horizon)

- [ ] A scripted concurrent-edit + kill-mid-execution scenario shows zero lost edits and zero stranded `Processing` rows.
- [ ] Auth cannot be bypassed without an explicit env flag; execute mutations are authorized.
- [ ] `cargo test -p layercake-core -p layercake-server` and frontend typecheck pass.
- [ ] `cargo fmt` + `cargo clippy` clean for changed crates.

---

## Progress log

- 2026-07-09: Plan created on `feature/horizon-1-stabilise`. Fix sites confirmed against current code. Beginning Stage 1.
