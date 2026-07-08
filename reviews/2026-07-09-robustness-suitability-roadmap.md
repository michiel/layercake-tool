# Layercake — Robustness Review, Functional Suitability & Product Roadmap

**Date:** 2026-07-09
**Scope:** Full-stack review of `layercake-core`, `layercake-server`, and `frontend/`, plus a functional-fit assessment against the stated product use case and a forward roadmap.
**Reviewers:** Code audit (three parallel deep-dives: frontend state/race conditions, backend integrity, domain suitability), grounded against `SPECIFICATION.md`, `plans/improvements.md`, `plans/20260128-fixes.md`, and the running sample models.

---

## 0. Executive summary

Layercake is a genuinely well-engineered **layered graph ETL + diagram-generation pipeline**. The Rust backend is disciplined (small panic surface, clean actor model, versioned migrations), the two-level model (Plan DAG → Graph data with replayable edits) is a strong idea, and the diagram/export breadth is real. The *mapping and rendering* third of the stated workflow works today.

However, two conclusions dominate this review:

1. **Robustness — the product is not yet trustworthy with a user's data.** The concerns you raised are real and reproducible in the code. The frontend coordinates collaborative state with `setTimeout`/timestamp windows rather than causal ordering, which drops and overwrites edits under normal editing cadence; several editors hold edits in local state that is silently wiped or never flushed; and the backend performs multi-write DAG execution and edit replay **without transactions**, so a crash leaves permanently inconsistent state. On top of this sits a **fail-open authorization default** and two DAG mutations that skip authz entirely.

2. **Functional suitability — the headline verbs of the consulting pitch do not exist yet.** The use case is *discovery, mapping, **simulation**, **projection*** of client environments with **risk/controls overlays**. In the code today: "simulation" does not exist (the 11 transforms are all structural/cosmetic), "projection" means a saved 3D-viewer preset (not a forecast or target-state derivation), and there is no structured risk/controls model or any quantitative rollup. Layercake is currently a capable tool for *drawing* the picture, not *reasoning about* it.

The rest of this document backs both claims with file-level evidence, then lays out a roadmap that sequences **stabilise → deepen the model → build the consulting layer**.

A prior engineering review already exists at `plans/improvements.md` (2026-01-01, rated "B+"). This review updates and re-prioritises its highest-severity items with fresh evidence, and adds the functional/domain and product-roadmap dimensions that document does not cover.

---

## 1. Robustness review

### 1.1 Severity ledger (most severe first)

| # | Severity | Area | Issue | Evidence |
|---|----------|------|-------|----------|
| R1 | **Critical** | Security | Authorization fails **open** by default | `layercake-core/src/services/authorization.rs:197` |
| R2 | **Critical** | Integrity | DAG execution not transactional; crash leaves rows stuck `Processing`, edges transiently deleted; `mark_error` never called | `pipeline/dag_executor.rs`, `services/graph_data_service.rs:87,149,339` |
| R3 | **Critical** | Data loss | 500 ms timestamp "echo window" blanket-mutes the subscription — genuine remote deltas silently dropped | `services/PlanDagQueryService.ts:120-133`, `PlanDagCommandService.ts:20-27` |
| R4 | **High** | Data loss | Debounced saves dropped on unmount (timer cancelled, queue cleared, never flushed) | `hooks/useUnifiedUpdateManager.ts:236-241` |
| R5 | **High** | Security | `execute_plan` / `execute_node` GraphQL mutations skip authorization entirely | `graphql/mutations/plan.rs:90`, `plan_dag_nodes.rs:183` |
| R6 | **High** | Data loss | Editors reset local state + dirty flag on any parent prop-reference change — wipes unsaved edits | `GraphSpreadsheetEditor.tsx:102-124` |
| R7 | **High** | Data loss / trust | `NodePropertiesForm` saves only on blur and shows "Saved" from a fixed `setTimeout`, regardless of whether the mutation succeeded | `components/graphs/NodePropertiesForm.tsx:32-55` |
| R8 | **High** | Integrity | Graph-edit replay is not atomic; partial apply on crash, no rollback, failed edit skipped mid-sequence | `services/graph_edit_service.rs:352-416` |
| R9 | **Medium** | Integrity | Edit sequence numbers via `max()+1` with no transaction/unique constraint — concurrent collaborators collide | `graph_edit_service.rs:81-90` |
| R10 | **Medium** | Data loss | No `beforeunload` guard anywhere; combined with local-only editor state, closing the tab always discards unsaved work | codebase-wide (grep: zero handlers) |
| R11 | **Medium** | Fragility | Autosave failures swallowed to `console.error`, op dropped, `isDirty` already cleared — failed saves look identical to successful ones | `usePlanDagCQRS.ts:147-160`, `useUnifiedUpdateManager.ts:106-123` |
| R12 | **Medium** | Fragility | Drag / refresh coordination uses fixed `setTimeout(100/200ms)` guesses; slow mutation → stale position overwrites the move | `PlanVisualEditor.tsx:439-505`, `usePlanDagCQRS.ts:570-577` |
| R13 | **Low-Med** | Availability | Collaboration coordinator processes all projects on one task with awaited oneshot round-trips → head-of-line blocking across projects | `collaboration/coordinator.rs` |
| R14 | **Low-Med** | Latent panic | `panic!("Unsupported file format…")` in a `From` impl; fires the moment more formats are wired | `graphql/types/data_set.rs:326` |

### 1.2 Root cause: coordination by wall-clock, not by causality (frontend)

The single biggest source of the "full of race conditions" symptom is that the collaborative editor synchronises state using **time windows and `setTimeout` delays** instead of causal ordering (versions / client-scoped echo suppression). Concretely:

- **500 ms echo mute (R3).** After any local mutation, `markMutationOccurred()` stamps `Date.now()`, and the delta subscription then ignores *all* incoming deltas for `MUTATION_ECHO_WINDOW_MS = 500`. This is not scoped to the specific mutation or client — it is a blanket mute. Because a user editing continuously re-opens the window constantly, a large fraction of *remote* deltas (another collaborator, a batch job, execution status) are discarded and never re-requested. A `clientId` check already exists at `PlanDagQueryService.ts:67` and is the correct, sufficient mechanism; the timestamp window layered on top of it is pure lossy race.
- **100 ms drag re-enable (R12).** `handleNodeDragStop` re-enables external sync after a fixed 100 ms; if `moveNode` takes longer, sync fires first and overwrites the just-dragged position with the pre-move server value. `plans/20260128-fixes.md` already documents this as "Scenario A: Node Position Never Syncs".
- **200 ms refresh delay, 100 ms fake-save indicator (R7, R11).** Same pattern in three more places.

The fix is architectural, not local: replace wall-clock coordination with **per-operation, client-scoped acknowledgement** (the mutation carries a client op-id; the subscription echoes it back; the client suppresses exactly that op and nothing else). The existing `plans/20260128-fixes.md` reaches the same conclusion — this review confirms it and rates it Critical.

### 1.3 Root cause: local editor state that is authoritative but non-durable (frontend)

`GraphSpreadsheetEditor`, `NodePropertiesForm`, `AttributesEditor`, and node-config dialogs hold edits in React local state and persist only on **blur** or **manual Save**. Three failure modes compound (R4, R6, R7, R10):

- A background query/subscription produces a new prop object, and an unconditional `useEffect(…, [graphData])` calls `setLocal…(…)` + `setHasChanges(false)`, wiping in-flight edits and greying out Save (R6).
- Navigating or closing before blur/Save, or within the 500 ms debounce, discards the edit — the cleanup effect cancels the timer and clears the queue *without flushing* (R4), and there is no `beforeunload` prompt (R10).
- When a save *does* fire and fails, the failure is swallowed and the UI still says "Saved" (R7, R11).

This is precisely the "data that is not saved" symptom, and it is made worse by *false* save confirmations.

### 1.4 Root cause: non-transactional multi-write operations (backend)

Two critical paths mutate persistent state across multiple independent DB writes with no enclosing transaction:

- **DAG execution (R2).** Each node runs `mark_processing → replace_nodes (own txn) → replace_edges (own txn) → mark_complete`. Between the two `replace_*` calls the graph is transiently `edges=0`; a crash there leaves nodes with all edges deleted. Worse, `mark_error` exists (`graph_data_service.rs:339`) but has **zero callers in the pipeline** — any crash after `mark_processing` strands the row in `Processing` forever, with no recovery path or resumability.
- **Edit replay (R8).** The replay loop applies each edit and marks it `applied` per-iteration with no batch transaction; a mid-loop crash persists a non-atomic prefix. Applicators are only partly idempotent, so re-running can diverge.

Both need a single transaction spanning the operation, a call to `mark_error` on failure, and a startup reconciliation that resolves stranded `Processing` rows.

### 1.5 Authorization (backend)

- **R1 — fail-open default.** There are two `local_auth_bypass_enabled()` functions with opposite defaults. The server's scope check (`layercake-server/src/auth/mod.rs:58`) correctly defaults to `false`. But the core project-access check (`layercake-core/src/services/authorization.rs:197`) defaults to **`true`** when the env var is unset — so `check_project_access` grants `Owner` to everyone by default. This is the exact risk `plans/improvements.md` flagged, but the live default is worse than that doc assumed: bypass is **on unless explicitly disabled**. `CHANGES.md` records this was made deliberately for local dev convenience; it must not survive into any server/multi-user build. Fix: default `false`, make bypass an explicit opt-in env var, and assert it is off in release/server builds.
- **R5 — unauthorized execution.** `execute_plan` and `execute_node` read `context.db` directly and run `DagExecutor` without `actor_for_request` / `authorize_project_write`. Every other mutation goes through the authorized `context.app.*` path; these two go around it. Any caller who can name a `project_id`/`plan_id` can trigger execution and overwrite graph data.

### 1.6 What is *good* (so the roadmap doesn't regress it)

- Panic surface is small and mostly guarded (~24 non-test `unwrap`/`expect`/`panic`, most provably unreachable). async-graphql catches resolver panics.
- The collaboration actor model (single coordinator + per-project actor over mpsc) is a clean serialisation strategy; presence is in-memory only, so a crash loses only ephemeral cursors.
- Replayable `GraphEdits` is the right idea and the *legacy* applicator has ordering **and** idempotence tests. Migrations are properly versioned via SeaORM.
- Dataset provenance/source tracking is tracked end-to-end (`DATASET_SOURCE_TRACKING_SUMMARY.md`).

### 1.7 Testing gaps that matter

264 tests exist, but the critical failure paths are exactly the untested ones: **collaboration coordinator/actor — zero tests**; DAG failure/partial-write/crash — none; `graph_data_edit_applicator` (the *current* unified path) ordering/idempotence — none (only the legacy applicator is covered). Any robustness work must land with these tests or it will regress.

---

## 2. Functional suitability for the consulting use case

**Stated use case:** assisting technical consultants with *discovery, mapping, simulation, projection* of complex client environments (operating model + technical environment + risk/controls overlays).

### 2.1 Workflow coverage today

| Stage | Status | Reality in the code |
|-------|--------|---------------------|
| **Discovery / ingest** | Partial–good | CSV/TSV/JSON + xlsx/ods round-trip; per-node free-form `attributes` JSON; dataset provenance tracked; an experimental code-analysis ingestion path. No live REST/SQL connectors (spec'd, not wired). |
| **Mapping / modelling** | **Strong** | Plan DAG (`DataSet→Graph→Transform→Filter→Merge→Artefact`); rich graph data with `layer`, `weight`, `belongs_to` partitions, `attributes`; visual + spreadsheet editors; replayable edits. This is the mature spine. |
| **Simulation** | **Absent** | No state, no propagation, no what-if. The 11 transforms are structural/cosmetic. |
| **Projection** | **Misnamed** | A "projection" is a saved 3D-force-graph *view* (`force3d`/`layer3d`), not a forecast or target-state derivation. `layer3d` is a "Coming Soon" stub. |
| **Risk & controls overlay** | **Weak / DIY** | `layer` is the only overlay primitive; risk/control data can only live in free-text `comment`, integer `weight`, or opaque `attributes`. No scored schema, no aggregation. |
| **Deliverable / reporting** | **Weak** | All 17 exporters are diagram/data formats. No register, no controls matrix, no formatted report (only a custom-Handlebars escape hatch). |

### 2.2 The three headline gaps

1. **"Simulation" is vapour.** A repo-wide search for `simulat|monte.carlo|what.if|propagat|state.transition|impact.analysis|quantitative` returns nothing in `layercake-core/src`. The only quantitative behaviour is mechanical weight-summation as a side effect of collapsing nodes (`graph.rs`, `merge_builder.rs`) — never a metric you can request. The one analysis service computes only node/edge counts, connected components, and depth-capped path enumeration — no centrality, no weighted metrics, no attribute rollups.
2. **"Projection" is a naming mismatch.** `entities/projections.rs` describes it as "metadata linking a project and a graph to a visualization type." `load_graph` returns the upstream graph 1:1 with only layer colours — no derivation, no current→future semantics. This is a viewer preset, not a projection in the consulting sense.
3. **Risk/controls is unmodelled.** Layers let a consultant *colour-code* an operating-model / tech / risk picture, and the `FilterNode` can select "nodes where `severity = High`" over the attribute bag. But you **cannot roll those up** — no "count uncontrolled high-risk assets per domain," no control-coverage/traceability computation. Note: `is_control()` in `graph.rs` refers to *control characters*, not risk controls — there is no domain vocabulary anywhere in the code or docs.

### 2.3 Missing consulting scaffolding

- **Frameworks/templates:** none — no TOGAF/ArchiMate metamodel, no NIST/CIS/ISO control libraries, no starter operating-model or risk taxonomies. Every model is hand-built from generic nodes/edges.
- **Quantitative rollups:** none.
- **Scenario comparison / what-if:** none — even the spec's own "Current State / Target State A/B" example has no diff or comparison view; they are just independent graphs.
- **Versioning / snapshots of client state:** none — no point-in-time baseline, no baseline-vs-reassessment.
- **Reporting/deliverables:** diagrams only.

### 2.4 Half-built / off-strategy surfaces

- **Stories & Sequences** — mature and wired, but a *narrative/presentation* feature emitting static Mermaid sequence diagrams. Good for walking a client through a flow; not simulation.
- **Code Analysis** — on-strategy in principle (auto-deriving an architecture graph from a client codebase is a legitimate *discovery* path) but Experimental, desktop-only (local file paths), crude status handling.
- **MCP/agentic access was recently *removed*** (`plans/20260122-de-feature.md`) despite `SPECIFICATION.md` calling it "a key goal." For a consulting tool where agent-assisted discovery is high-leverage, this is worth revisiting deliberately rather than by attrition.
- **Orphans:** `SourceManagementPage.tsx`, `ProjectSharingPage.tsx` exist but appear unrouted.

### 2.5 Suitability verdict

Layercake delivers roughly the **first third** of the stated workflow — discovery and mapping — competently, with real strength in layered overlays, replayable edits, and multi-format diagram export. It is a plausible substrate for *drawing* an operating-model + tech + risk picture. The analytical half of the pitch — simulation, projection, structured risk/controls, rollups, scenario comparison, deliverables — is not built. **Product-market fit for the stated use case is "capable mapping/diagramming tool," not "discovery-to-simulation consulting platform."** That gap is the roadmap.

---

## 3. Roadmap

Three horizons. Do them roughly in order: you cannot sell an analytical platform on top of an editor that loses edits, so **stabilise first**. Then make the *model* rich enough to carry consulting semantics. Then build the consulting features on top.

### Horizon 1 — Stabilise (make it trustworthy) — ~4–6 weeks

Goal: no silent data loss, no fail-open auth, no stranded state. This is a prerequisite for everything else and for any paid/client use.

- **H1.1 Security (R1, R5).** Flip the core auth-bypass default to `false`; make bypass an explicit, dev-only opt-in; assert it is off in server/release builds. Add `actor_for_request` + `authorize_project_write` to `execute_plan`/`execute_node`. Land the security smoke tests from `improvements.md` §1.3.
- **H1.2 Transactional integrity (R2, R8).** Wrap each DAG node's persist sequence and the edit-replay batch in one transaction. Call `mark_error` on failure. Add a startup reconciliation that resolves rows stuck in `Processing`. Add failure/partial-write tests.
- **H1.3 Kill wall-clock coordination (R3, R12).** Replace the 500 ms echo window and the 100/200 ms `setTimeout` guesses with client-scoped op-id acknowledgement (mutation carries op-id → subscription echoes it → suppress exactly that op). This is the fix `plans/20260128-fixes.md` already scoped.
- **H1.4 Durable editing (R4, R6, R7, R10, R11).** Add dirty-state tracking, flush pending saves on unmount, guard prop-reset effects behind a dirty check, add a `beforeunload` prompt, surface save failures (and restore `isDirty` on failure) instead of faking "Saved."
- **H1.5 Concurrency correctness (R9).** Unique constraint on `(graph_id, sequence_number)`; allocate sequence numbers atomically.
- **H1.6 Test the untested critical paths.** Collaboration actor concurrency; DAG failure paths; `graph_data_edit_applicator` ordering/idempotence.

*Exit criteria:* a scripted concurrent-edit + kill-mid-execution test suite passes with zero lost edits and zero stranded rows; auth cannot be bypassed without an explicit flag.

### Horizon 2 — Deepen the model (make it consulting-shaped) — ~6–10 weeks

Goal: the data model can *carry* operating-model + risk/controls semantics and answer questions, not just draw pictures.

- **H2.1 Typed overlays / multi-membership.** Today a node has a single `layer: String`. Introduce first-class, typed overlays so one asset can simultaneously sit in an operating-model domain, a tech-environment tier, and a risk plane. (Keep `attributes` as the escape hatch, but promote the consulting-critical fields to schema.)
- **H2.2 Structured risk/controls schema.** A defined shape for risk (impact, likelihood, severity, owner) and control (type, effectiveness, coverage, framework ref) attached to nodes/edges/layers — replacing free-text `comment`/opaque `attributes`.
- **H2.3 Quantitative rollups.** Aggregation across the layer/partition hierarchy: sum/count/coverage per domain (e.g. "uncontrolled high-risk assets per capability," "control coverage per tier"). This is the single highest-leverage analytical primitive and the tool has none.
- **H2.4 Snapshots & versioning.** Point-in-time capture of client state so an engagement has baselines and can compare reassessment-over-time.
- **H2.5 Scenario / target-state as real objects.** Make "Current vs Target A/B" a first-class comparison with diff, not independent graphs. This is where "projection" earns its name — a *derived* future state, not a saved camera angle.

### Horizon 3 — The consulting layer (make it a product) — ~ongoing

Goal: the features that turn a rich model into billable deliverables.

- **H3.1 Framework & control-library templates.** Starter metamodels (ArchiMate/TOGAF-style), and importable control libraries (NIST CSF, CIS, ISO 27001). Turns "hand-build every model" into "start from a framework."
- **H3.2 Deliverable/report exporter.** Risk registers, RACI/controls-coverage matrices, and formatted documents (not just diagrams). Build on the existing `to_custom.rs` Handlebars path and `to_csv_matrix.rs`, but ship curated report templates.
- **H3.3 "Simulation" — earn the word.** Start with the achievable, defensible version: **impact propagation** over the graph (if this control fails / this system is down, what's exposed downstream?) using the risk/controls schema from H2.2 and edge weights. This is a bounded, graph-native computation — not Monte-Carlo, but genuinely useful and honest to call analysis. Consider quantitative what-if (scenario weights) as a follow-on.
- **H3.4 Live connectors (discovery).** REST/SQL/CMDB ingestion (spec'd, never wired) so discovery isn't CSV-only.
- **H3.5 Reconsider agentic/MCP access.** Agent-assisted discovery and mapping is high-leverage for consulting; the removed MCP surface (`plans/20260122-de-feature.md`) is worth a deliberate decision, not silent attrition.
- **H3.6 Finish or cut the stubs.** Route or delete `SourceManagementPage`/`ProjectSharingPage`; resolve `layer3d` "Coming Soon"; decide whether Code Analysis is a maintained discovery feature or Experimental clutter.

### Sequencing rationale

- Horizon 1 is non-negotiable and first — the current defects mean a consultant can lose a client model mid-engagement, and the auth default is unshippable to any shared instance.
- Horizon 2 before Horizon 3 because rollups, reports, and simulation all depend on a *structured* risk/controls model; building reports on the current free-text attributes would bake in the wrong abstraction.
- Horizon 3's "simulation" is deliberately scoped to graph-native impact propagation first — it is the version you can actually ship and defend, and it reuses the H2 schema directly.

---

## 4. Immediate next actions (this sprint)

1. **R1** — flip `authorization.rs:197` default to `false`; add release-build assertion. *(security, hours)*
2. **R5** — authorize `execute_plan`/`execute_node`. *(security, hours)*
3. **R2** — wrap DAG node persist in a transaction + call `mark_error`; add a `Processing`-reconciliation on startup. *(integrity, days)*
4. **R3** — remove the 500 ms echo window; rely on the existing `clientId` filter, then design op-id acknowledgement. *(data loss, days)*
5. **R4/R6/R7** — flush-on-unmount, dirty-guard the reset effects, and stop faking "Saved." *(data loss, days)*

Each of these is small, high-severity, and independently shippable. Everything in Horizons 2–3 should wait behind Horizon 1's exit criteria.

---

*Cross-references: `plans/improvements.md` (2026-01-01 engineering review — this document updates its Critical/High items and adds the functional & product dimensions), `plans/20260128-fixes.md` (frontend race-condition investigation — corroborated and elevated to Critical here), `SPECIFICATION.md` (original vision, including the simulation/MCP goals that are not yet realised).*
