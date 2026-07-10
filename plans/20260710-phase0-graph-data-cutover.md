# Phase 0 — Complete the graph_data single-schema cutover

**Date:** 2026-07-10
**Status:** Complete
**Owner:** Data model direction workstream
**Related:**
`reviews/20260710-data-model-direction-roadmap.md` (§15 Phase 0),
`reviews/20260710-data-model-direction-assessment.md`

---

## 1. Goal

Finish the graph_data cutover so that **no source code references a table that
`m20251215_000001_drop_legacy_graph_tables` has already dropped**, and prove it with
a repeatable check. This is the hard gate that must close before any semantic-layer
work (Phases 1+) begins.

Dropped tables that must have zero remaining code references (outside `migrations/`):

- `graphs`
- `graph_nodes`
- `graph_edges`
- `graph_layers`
- `dataset_graph_nodes`
- `dataset_graph_edges`
- `dataset_graph_layers`

Authoritative replacements:

| Legacy table | Canonical replacement |
|---|---|
| `graphs` | `graph_data` |
| `graph_nodes` | `graph_data_nodes` |
| `graph_edges` | `graph_data_edges` |
| `graph_layers` | `project_layers` (project-scoped layers) |
| `dataset_graph_nodes` | `graph_data_nodes` (of the dataset's `graph_data` row) |
| `dataset_graph_edges` | `graph_data_edges` (of the dataset's `graph_data` row) |
| `dataset_graph_layers` | `project_layers` |

**Not in scope:** any new semantic tables (record types, assertions, dimensions,
provenance). Those are Phases 1+. `data_sets`, `dataset_nodes`, `dataset_rows`,
`graph_edits`, `graph_data*`, `project_layers`, `layer_aliases` are current and stay.

---

## 2. Why this is a gate (evidence)

The legacy tables are gone from the database but not from the code, so several
reachable paths query tables that no longer exist:

- `graphql/queries/mod.rs:109,127` → `data_set_service::get_graph_summary` /
  `get_graph_page` → `dataset_graph_*` (dropped). Dataset preview is broken.
- `services/graph_edit_service.rs:137-158` → `else` branch loads/updates `graphs`
  (dropped).
- `pipeline/dag_executor.rs`, `pipeline/merge_builder.rs`,
  `services/graph_service.rs` → "fall back to legacy `graphs` table" branches
  (dropped).
- `app_context/graph_operations.rs`, `services/import_service.rs` → write/read
  `graph_layers` / `graph_nodes` (dropped).
- `graphql/types/{graph,graph_node,graph_edge,preview}.rs` → map `graphs::Model` /
  `graph_nodes::Model` / `graph_edges::Model` (dropped).

`m20260709_000001_rebuild_graph_edits_drop_graphs_fk` already had to fix one silent
breakage (a dangling `graph_edits → graphs` FK) caused by this same incomplete drop.

---

## 3. Verification method

Environment note: the full workspace build fails on `src-tauri` (`gdk-sys` needs
system GTK not installed here). Phase 0 code is entirely in `layercake-core`,
`layercake-server`, `layercake-cli`, so verification uses:

```bash
# 1. No legacy table references remain in source (outside migrations).
grep -rnE '\b(graphs|graph_nodes|graph_edges|graph_layers|dataset_graph_nodes|dataset_graph_edges|dataset_graph_layers)\b' \
  layercake-core/src layercake-server/src layercake-cli/src layercake-projections/src \
  | grep -v '/migrations/'
# expected: no entity/table hits (comments mentioning history are fine)

# 2. Targeted build.
cargo build -p layercake-core -p layercake-server -p layercake-cli

# 3. Targeted tests.
cargo test -p layercake-core -p layercake-server -p layercake-cli
```

A guard test in `layercake-core` will assert the entity modules are gone so the
regression cannot silently return.

---

## 4. Workstreams

Each workstream is independently compilable and committed separately. Ordering is
chosen so the tree keeps building between commits (leaf rewrites first, entity-module
deletion last).

### WS1 — Dataset preview/summary → graph_data
**Files:** `services/data_set_service.rs` (`get_graph_summary` ~L474, `get_graph_page`
~L520, `update_graph_data` cleanup ~L291-300), import (`data_sets` L11).
**Action:** Resolve a `data_sets` row to its `graph_data` row (dataset import writes
`graph_data` with `source_type='dataset'`; confirm the link — `graph_data.name`/a
dataset id in metadata, or a lookup by dataset). Read nodes/edges from
`graph_data_nodes`/`graph_data_edges`; derive layers from `project_layers`. Drop the
`dataset_graph_layers` delete in `update_graph_data`.

### WS2 — DAG executor / merge builder / graph_service legacy fallbacks
**Files:** `pipeline/dag_executor.rs` (fallbacks ~L922, L1062, L1767; in-memory
`graphs::Model` for hashing ~L900, L1040; helper sigs ~L1255, L1273),
`pipeline/merge_builder.rs` (~L245), `services/graph_service.rs` (~L266 fallback,
`graph_layers` reads/writes L252-259, 675-684).
**Action:** Delete every "fall back to legacy `graphs` table" branch (graph_data is
now the only source; return a proper not-found error instead). Replace the in-memory
`graphs::Model` built for hash input with a small local struct or by passing the
needed fields directly; update the two helpers that take `&graphs::Model`. Repoint
`graph_service` layer accessors to `project_layers`.

### WS3 — Layer/node editing methods → project_layers / graph_data
**Files:** `app_context/graph_operations.rs` (`create_layer` L71, `update_layer`
L410, node edit via `graph_nodes` L297), `services/import_service.rs` (`graph_layers`
write L89).
**Action:** Determine reachability (GraphQL mutations). Repoint layer create/update
to `project_layers`; repoint node editing to `graph_data_nodes`. If a method is
superseded by an existing graph_data path, remove it and its wiring rather than
rewire.

### WS4 — graph_edit_service else-branch
**Files:** `services/graph_edit_service.rs` (L137-158, L216-..., L283-...).
**Action:** `graph_id` is always a `graph_data` id in the current schema (see
`graph_operations.rs:590-593`). Remove the `graphs` `else` branches; when the id is
absent from `graph_data`, return `not_found` instead of touching a dropped table.

### WS5 — GraphQL legacy type mappers
**Files:** `graphql/types/graph.rs` (L125 `From<graphs::Model>`, resolvers L68/106/115),
`graph_node.rs` (L39), `graph_edge.rs` (L25), `preview.rs` (L82,112).
**Action:** Remove `From<legacy::Model>` impls and any resolver that reads a dropped
table. Where a GraphQL type is still served, populate it from `graph_data*`
/`project_layers`. Delete now-unused GraphQL fields only if nothing in the schema
depends on them; otherwise back them with graph_data.

### WS6 — Delete legacy entity modules
**Files:** `database/entities/mod.rs` (L23-33), and delete
`entities/{graphs,graph_nodes,graph_edges,graph_layers,dataset_graph_nodes,dataset_graph_edges,dataset_graph_layers}.rs`;
remove the `graphs` relation from `entities/graph_edits.rs` (L31-42). Review the
`pub use dataset_nodes as datasets` alias (L45) — keep only if still referenced.
**Action:** Delete modules and fix `mod.rs`. This must be the last step; it will not
compile until WS1–WS5 remove all callers.

### WS7 — Guard + verification
**Files:** new test in `layercake-core` (e.g. a compile-time/reference guard) plus a
short note in `LAYERCAKE_MODEL_GUIDE.md` recording the single-path invariant.
**Action:** Add a test asserting the cutover invariant and run §3 checks. Optionally
wire the grep guard into CI.

---

## 5. Progress tracker

Legend: ☐ not started · ◐ in progress · ☑ done · ⚠ blocked/needs decision

| WS | Description | Status | Notes |
|----|-------------|--------|-------|
| — | Baseline: targeted build of core/server/cli green | ☑ | `cargo build -p layercake-core -p layercake-server -p layercake-cli` exit 0 |
| WS1 | Dataset preview/summary → graph_data | ☑ | `get_graph_summary`/`get_graph_page` now read the parsed graph from `data_sets.graph_json`; `update_graph_data` dropped-table cleanup removed |
| WS2 | DAG/merge/graph_service legacy fallbacks removed | ☑ | `dag_executor`/`merge_builder` fallbacks gone; hash helpers now take `&graph_data::Model`; `graph_service` doc/dead-code cleaned |
| WS3 | Layer/node editing → project_layers/graph_data | ☑ | node old-value fetch repointed to `graph_data_nodes`; per-graph layer-editing **removed** (decision below) across backend + frontend |
| WS4 | graph_edit_service else-branch removed | ☑ | three `graphs` else-branches now return `not_found` |
| WS5 | GraphQL legacy mappers removed/repointed | ☑ | deleted 5 dead `From<legacy::Model>` impls (graph/graph_node/graph_edge/preview) + dead `publish_graph_status` |
| WS6 | Legacy entity modules deleted | ☑ | deleted all 7 legacy modules (`graphs`, `graph_nodes`, `graph_edges`, `graph_layers`, `dataset_graph_{nodes,edges,layers}`); removed `graphs` relations from `graph_edits`/`graph_layers` |
| WS7 | Guard test + verification (§3) | ☑ | `layercake-core/tests/no_legacy_graph_tables.rs` guards all 7 dropped tables; §3 grep clean; all crate tests green; frontend `tsc` clean |

### Resolved decision — per-graph layer editing (was deferred)

**Decision (user): Option B — remove the per-graph layer mutations.** The
mutations targeted the dropped `graph_layers` table and were wired to a
graph-scoped GraphQL `Layer` type distinct from `ProjectLayer`. Layers now live
in `project_layers`. Removed across the stack:

- backend: `create_layer` / `update_layer_properties` GraphQL mutations;
  `bulk_update_graph_data` no longer accepts `layers`; `AppContext::{create_layer,
  update_layer_properties}`; `GraphService::update_layer_properties`;
  `From<graph_layers::Model> for Layer`; dead `ImportService` layer-import methods;
  unused `CreateLayerInput` / `LayerUpdateInput` / `GraphLayerUpdateRequest` types;
  the `graph_layers` entity module.
- frontend: `GraphDataDialog` no longer calls `createLayer` / sends `layers` to
  `bulkUpdateGraphData` (its layer grid was already `layersReadOnly`, so no UX
  regression); removed `CREATE_LAYER` / `UPDATE_LAYER_PROPERTIES` gql and the
  `layers` arg on `BULK_UPDATE_GRAPH_DATA`. Verified with `tsc --noEmit`.

Project-scoped layer styling continues to be edited via the existing
project-layer mutations (`upsertProjectLayer` etc.). Phase 0 is fully closed.

### Change log

- 2026-07-10 — Plan created; legacy-reference inventory completed (12 source files).
  Review and assessment updated to promote Phase 0 to a verified gate.
- 2026-07-10 — Implemented WS1, WS2, WS4, WS5, WS6, WS7 and the node half of WS3.
  Six dropped-table entity modules deleted; §3 grep clean for all six; guard test
  added. Build green for core/server/cli; core (175 unit + integration + doctests),
  server, cli, and integration-tests suites all pass. Layer-editing (WS3 remainder)
  deferred pending a decision.
- 2026-07-10 — WS3 resolved (Option B: remove per-graph layer editing). Deleted the
  `graph_layers` module and all layer-mutation code across backend and frontend;
  guard now covers all 7 tables. §3 grep clean; backend suites green; frontend
  `tsc --noEmit` clean. **Phase 0 complete.**

---

## 6. Exit criteria

- §3 grep returns no legacy entity/table references outside `migrations/`.
- `cargo build -p layercake-core -p layercake-server -p layercake-cli` is green.
- `cargo test` for those crates is green.
- Legacy entity modules are deleted; `entities/mod.rs` no longer declares them.
- Guard test in place so the regression cannot silently return.
