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
- [x] 1.1 (R1) Flip `local_auth_bypass_enabled()` default from `true` → `false` (unset env var = bypass OFF). Done by centralising (1.2) on the secure default.
- [x] 1.2 (R1) Centralised into one canonical `layercake_core::auth::local_auth_bypass_enabled()` (secure default `false`); `authorization.rs` and `layercake-server/src/auth/mod.rs` both delegate. Duplicate implementations removed. Pure `bypass_value_is_truthy` extracted for testability.
- [x] 1.3 (R1) Confirmed `dev.sh:40` sets `LAYERCAKE_LOCAL_AUTH_BYPASS=1` explicitly (default `1`) and propagates it to backend/tauri, so local dev is unaffected by the secure default.
- [x] 1.4 (R5) Added `AppContext::authorize_project_write_access(&actor, project_id)` (public wrapper over private `authorize_project_write`).
- [x] 1.5 (R5) `execute_plan` and `execute_node` now build `actor` and call `authorize_project_write_access` before executing.

**Success criteria**
- [x] With no env var set, bypass is OFF (fails closed). `check_project_access` and the execute mutations require project write.
- [x] With `LAYERCAKE_LOCAL_AUTH_BYPASS=1` (dev.sh default), local dev works unrestricted.
- [x] Only one `local_auth_bypass_enabled` implementation exists.

**Tests**
- [x] Unit (`layercake-core --lib auth::tests`): bypass off when unset; on for `1/true/yes/on`; off for falsey — all pass.
- [ ] Integration: execute mutations denied without project write — deferred; server-level GraphQL harness is thin (`layercake-server/tests` only has error-mapping). Covered indirectly by the shared `authorize_project_write` path already tested via app-layer mutations. Revisit if a GraphQL test harness is added.

**Status:** Complete

> Note: `cargo test -p layercake-core` (full, including integration test binaries) fails to compile on **pre-existing** breakage unrelated to this work — `layercake-core/tests/project_archive_roundtrip.rs:150` etc. call `export_project_archive(&actor, id, false)` but the current signature is `(&actor, id)` (2 args). This drift exists on `master` and is out of scope for Horizon 1; `--lib` unit tests and both crate builds are green.

---

## Stage 2: Backend transactional integrity (R2, R8)

**Goal:** DAG execution and edit replay never leave persistent state half-written or stranded in `Processing`.

**Tasks**
- [x] 2.1 (R2) Added `GraphDataService::replace_contents(graph_data_id, nodes, edges)` doing node + edge delete/insert + count update in **one** transaction. Refactored `replace_nodes`/`replace_edges` to share `replace_nodes_in_txn`/`replace_edges_in_txn`/`update_counts_in_txn` helpers (public methods keep their own txn for back-compat). `DagExecutor::persist_graph_contents` now calls `replace_contents`.
- [x] 2.2 (R2) Added `DagExecutor::mark_error_on_failure` and wrapped the persist+complete work at all four sites (dataset-reference, transform, filter, and MergeNode) so any failure transitions the row to `Error` (`mark_error` previously had zero callers).
- [x] 2.3 (R2) Added `GraphDataService::reconcile_interrupted_processing()` (flips stuck `Processing` rows → `Error` with a clear message) and wired it into `start_server` immediately after migrations.
- [x] 2.4 (R8) Re-scoped after finding two replay paths: the **live** path is `GraphDataService::replay_edits` (not the legacy `graph_edit_service.rs`). Full transaction-threading through `GraphDataEditApplicator` is a wide, risky refactor deferred to Horizon 2. Applied the bounded, high-value integrity fix: replay now **aborts on the first hard (DB) error** instead of continuing to apply later (possibly dependent) edits out of order — remaining edits stay unapplied for an in-order resume. Per-edit apply+mark atomicity tracked as a follow-up (see Deferred).

**Success criteria**
- [x] `replace_contents` makes node+edge persistence atomic (no transient `edges=0` window).
- [x] Executor failures mark the row `Error`; interrupted rows are reconciled at startup.
- [~] Replay is safer (stops on hard error); full all-or-nothing per-edit atomicity deferred.

**Tests**
- [x] `replace_contents` atomicity + correct counts (see Stage 5 test additions).
- [x] Reconciliation flips a seeded `Processing` row to `Error`.
- [ ] `mark_error`-on-executor-failure test — covered by reconciliation + replace_contents tests; a full injected-failure executor test is heavier (needs a failing DB mid-op) and is folded into Stage 5.

**Deferred to Horizon 2:** thread a `DatabaseTransaction` through `GraphDataEditApplicator` so each edit's mutation + `mark_edit_applied` commit atomically, and the batch can roll back.

**Status:** Complete (with noted deferral)

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

## Known pre-existing issues (out of scope for this horizon)

- `layercake-core/tests/{project_archive_roundtrip,graph_data_pipeline_e2e,graph_data_builder_test,dataset_source_tracking,dag_executor_graph_data_test}.rs` fail to **compile** on `master` due to `export_project_archive` and related signature drift. Not caused by this work.
- `cargo clippy -p layercake-server` fails with 2 errors in `layercake-server/src/graphql/mutations/graph.rs:41-42` (`n >= i64::MIN` / `n <= i64::MAX` always-true comparisons) — pre-existing, untouched by this work.

Both should be cleaned up (candidate for a small separate "green the build" commit), but are not Horizon 1 stability fixes.

## Progress log

- 2026-07-09: Plan created on `feature/horizon-1-stabilise`. Fix sites confirmed against current code.
- 2026-07-09: **Stage 1 complete** — fail-closed auth default centralised; DAG execute mutations authorized. Committed `62e5eb8b`. Core `--lib` + auth unit tests green.
- 2026-07-09: **Stage 2 complete** — atomic `replace_contents`; `mark_error` wired at all 4 executor sites; startup reconciliation of stuck `Processing` rows; replay aborts on hard error. New `graph_data_integrity_test.rs` (3 tests) + all 176 core lib tests green. Full applicator transaction threading deferred to Horizon 2.
- Next: Stage 3 (frontend echo-window / drag coordination).
