# Assessment of the Data Model Direction and Roadmap Review

**Date:** 2026-07-10
**Status:** Independent validation and critique
**Reviews:** `reviews/20260710-data-model-direction-roadmap.md`
**Repository:** `michiel/layercake-tool`

---

## 1. Purpose

This document validates the factual claims in the data model direction review, then
offers an independent assessment of the recommended direction. It is deliberately
critical rather than confirmatory: the review is a strong piece of architecture
writing, and the most useful contribution here is to test its premises against the
code and to flag where its sequencing and risk weighting need adjustment.

Short version:

- The review's description of the **current schema is accurate**; every structural
  claim was checked against the entities and migrations and holds.
- The review's **central diagnosis is sound**: the model is graph-centric and
  conflates source, canonical, view, and rendered concerns.
- The review's **Phase 0 premise is not just theoretically correct — it is
  demonstrably unfinished today**, with live code still reading tables that
  migrations have dropped. This raises the priority of Phase 0 above where the
  review places it.
- The **target model (typed records + reified assertions + classifications +
  provenance, with GraphData as a materialised view) is a sound, conventional
  design.** The main reservations are about **scope, sequencing, and a re-introduced
  dual-write hazard**, not about the destination.

---

## 2. Validation of the review's factual claims

All claims below were checked directly against the code.

| Review claim | Verdict | Evidence |
|---|---|---|
| Nodes have `external_id`, `label`, `layer`, `weight`, `is_partition`, `belongs_to`, `comment`, `source_dataset_id`, `attributes`; no semantic type (§4.1) | **Confirmed** | `layercake-core/src/database/entities/graph_data_nodes.rs:16-32` |
| Edges have `label`, `layer`, `weight`, `comment`, `source_dataset_id`, `attributes`; no first-class relationship type (§4.2) | **Confirmed** | `graph_data_edges.rs:15-31` |
| Provenance is a single optional `source_dataset_id` per element (§4.3) | **Confirmed** | `graph_data_nodes.rs:28`, `graph_data_edges.rs:27`; merge copies one source (`merge_builder.rs:211,226`) |
| `graph_data` couples source-file metadata (blob, filename, file_size) with computed-graph lifecycle fields (§4.4) | **Confirmed** | `graph_data.rs:33-49` |
| `source_type` mixes resource/derivation/authorship dimensions: `dataset` / `computed` / `manual` (§4.5) | **Confirmed** | `graph_data.rs:30,101-119` |
| A single scalar `layer` on nodes and edges (§4.6) | **Confirmed** | `graph_data_nodes.rs:22`, `graph_data_edges.rs:23` |
| Partitions conflate display containment and domain hierarchy via `is_partition` + `belongs_to`; partitions cannot be edge endpoints (§4.7) | **Confirmed** | `graph_data_nodes.rs:24-25`; `LAYERCAKE_MODEL_GUIDE.md` "Partition Rules" |
| Graph-specific columns (`weight`, `is_partition`, `belongs_to`) are first-class while domain concepts sit in JSON (§4.8) | **Confirmed** | as above |
| Project layers carry stable id, name, three colours, optional dataset, alias (§2.5) | **Confirmed** | `project_layers.rs:6-20`; separate `layer_aliases.rs` table also exists |

**Conclusion:** the review is an accurate reading of the code. It is not arguing
against a straw model.

---

## 3. The decisive finding: the current cutover is genuinely incomplete

The review's Phase 0 ("finish the single-schema migration") is presented as
stabilisation before the interesting work. Investigation shows this is understated:
**the graph_data cutover is not finished, and the gap is producing latent runtime
failures, not just tidiness debt.**

Evidence:

1. **Legacy tables are dropped, but live services still query them.**
   `m20251215_000001_drop_legacy_graph_tables` drops `graphs`, `graph_nodes`,
   `graph_edges`, `dataset_graph_nodes`, `dataset_graph_edges`,
   `dataset_graph_layers`. Yet:
   - `services/data_set_service.rs` still reads `dataset_graph_nodes` /
     `dataset_graph_edges` in `get_graph_summary` (`:480-486`) and `get_graph_page`
     (`:529-552`), and deletes them in `update_graph_data` (`:291-296`) — these are
     dataset preview/summary paths against dropped tables.
   - `services/merge_builder.rs:245` runs a `legacy_graph = GraphEntity::find()`
     fallback against the dropped `graphs` table.
   - `services/graph_edit_service.rs:137-158` has an `else` branch that loads and
     updates a row from the dropped `graphs` table when an id is absent from
     `graph_data`.
   - GraphQL still maps from `graphs::Model`, `graph_nodes::Model`,
     `graph_edges::Model` (`graphql/types/graph.rs`, `graph_node.rs`,
     `graph_edge.rs`, `preview.rs`).

2. **A silent breakage from the same incomplete drop was fixed this month.**
   `m20260709_000001_rebuild_graph_edits_drop_graphs_fk` (dated today, 2026-07-09)
   documents that dropping `graphs` left a dangling foreign key on `graph_edits`,
   so "every insert into `graph_edits` then fails ... which silently breaks the
   entire graph-edit tracking / replay feature." This is precisely the class of
   hidden-legacy-read hazard Phase 0 is meant to eliminate, discovered reactively
   rather than by the telemetry/constraints the review recommends.

The primary write paths *were* migrated correctly — `pipeline/dataset_importer.rs`
imports into `graph_data` (`:90-159`) and `pipeline/dag_executor.rs` persists merge
and graph output into `graph_data` (`:155-211`). So the cutover is roughly
"happy-path done, edges of the surface still on dropped tables."

**Implication for the roadmap:** Phase 0 should be treated as a hard gate with a
higher bar than the review implies, and it should be *verified*, not assumed:

- delete or rewrite every reference to `graphs`, `graph_nodes`, `graph_edges`,
  `dataset_graph_*` (the entity modules themselves are still declared in
  `entities/mod.rs:29-33` and should go once callers are gone);
- add the "no legacy read" telemetry the review lists in Phase 0 deliverables
  *before* trusting the cutover;
- only then start semantic expansion.

Adding ~20 new tables on top of a surface that still half-references dropped tables
would compound the exact failure mode already observed.

---

## 4. Assessment of the recommended direction

### 4.1 Where the direction is right

- **Canonical records + reified assertions, with GraphData as a materialised
  view**, is a conventional and durable pattern (a lightweight property-graph over
  relational storage). It is the correct answer to the stated consulting use case,
  where the "facts" are tabular/documentary and the graph is one projection.
- **PostgreSQL + JSONB + join tables + recursive CTEs** (§13) is the pragmatic
  choice. The explicit rejection of a native graph DB / triple store / generic EAV
  is correct for this team and workload.
- **Separating classification (dimensions) from presentation (layers/themes)**
  (§4.6, §5.6) matches the evident intent of the existing alias mechanism and is
  the single highest-value modelling change for the consulting use case.
- **One application-service layer shared by UI/GraphQL/CLI/MCP** (§11.1) is right
  and is independently justified: the legacy-read problems in §3 exist partly
  because graph access is not funnelled through one service boundary.
- **Multi-source provenance and explicit identity reconciliation** (§4.3, §4.9)
  are genuine requirements for consulting-grade lineage, not gold-plating.

### 4.2 Where I would push back

1. **Scope is larger than the "evolutionary, not a rewrite" framing admits.**
   Eight phases and ~20 new tables is a multi-quarter programme. The repository's
   signals — a single normalised-graph ADR, an early-integration `status` file, a
   just-fixed migration bug — indicate a small team still stabilising the *last*
   unification. The dominant risk is not "overengineering an ontology" (the review's
   headline risk); it is **a permanently half-migrated system**, which already
   exists in miniature. The roadmap should be scoped so that every phase leaves the
   system in a shippable, single-authority state.

2. **§15 (horizontal phases) contradicts §17 (vertical slice), and §17 is right.**
   The phased roadmap builds each layer broadly (all record types, then all
   provenance, then all dimensions, then views). The "concrete first slice" builds
   one control-ownership product end-to-end. These are different strategies. Building
   horizontally means none of the semantic investment produces a user-visible product
   until Phase 5+. I would **invert the roadmap around §17**: implement the
   control-ownership slice as a thin vertical (a few record types, a handful of
   relationship types, one dimension set, one AnalysisFrame, graph + matrix
   projectors) *before* generalising any layer. If the consulting product does not
   land, most of Phases 1–4 is over-build.

3. **Reified assertions re-introduce the dual-write problem the graph_data cutover
   just fought.** The proposal keeps GraphData edges as materialisations of canonical
   assertions, with optional `assertion_id` / `record_id` back-references on
   GraphData rows (§6). That is two stores of the same relationship with a
   synchronisation contract — structurally identical to the `graphs` vs `graph_data`
   split whose incomplete resolution is documented in §3. The review names this risk
   (§18) but its mitigation ("define clear authority") is too weak given the evidence.
   This needs a concrete rule set before Phase 5: GraphData that is a projection
   target is **never** hand-authored; materialisation is driven by content hashes and
   a `Materialization` record; staleness is detected, not assumed; and there is a
   single code path that rebuilds a projection from assertions.

4. **Identity reconciliation (§4.9) is a product, not a table.** Fuzzy matching plus a
   human review queue plus agent-proposed matches is a significant surface with its
   own UI, states, and failure modes. It is billed at roughly one phase alongside
   provenance. It should be flagged as disproportionately expensive and, where
   possible, deferred behind exact/alias matching (which the existing
   `layer_aliases` pattern already models) until the slice in §17 proves it is
   needed.

5. **The semantic layer is a product bet on a new use case, and should be framed as
   one.** The codebase's centre of gravity today is graph visualisation and code
   analysis (`layercake-code-analysis`, `layercake-projections`, Mermaid/DOT/3D
   export). Security-consulting knowledge modelling is a *new* direction. The review
   presents typed records as the "long-term canonical" model as though it is the
   obvious evolution of the current tool; it is better understood as a strategic
   pivot that the §17 slice exists to de-risk. Existing graph/code-analysis data has
   no clean semantic type and will live indefinitely under the "GraphData as
   graph-native" escape hatch — so the semantic benefits accrue mostly to new
   projects. That is fine, but it should be stated so expectations are set and so the
   backfill cost of existing projects is not assumed away.

### 4.3 Smaller points

- The review lists `source_dataset_id` becoming `primary_source_dataset_id` for
  compatibility (§4.3). Good — but note there are already two provenance-ish
  mechanisms in play (`source_dataset_id` on rows and `source_dataset_id` on
  `project_layers`); the migration to `ProvenanceLink` should reconcile both, not
  just the row-level one.
- `entities/mod.rs` still declares the legacy modules and a
  `pub use dataset_nodes as datasets` backwards-compat alias (`:45`). These are cheap
  Phase 0 deletions once callers are removed and are a good "is the cutover actually
  done?" tripwire.
- The review is silent on `graph_edits` replay semantics under the new model. Since
  §12.2 wants manual edits represented as assertions/overrides rather than
  mutations, the existing edit-replay machinery (`graph_edit_service`,
  `graph_edits`) needs an explicit place in the target model or an explicit
  deprecation.

---

## 5. Recommended adjustments to the roadmap

1. **Harden Phase 0 into a verified gate.** Remove all references to dropped tables
   (`data_set_service` preview paths, `merge_builder` legacy fallback,
   `graph_edit_service` else-branch, GraphQL legacy mappers), drop the legacy entity
   modules, and add "no legacy read" telemetry and DB constraints *before*
   proceeding. Treat the July 2026 `graph_edits` FK breakage as the motivating
   incident.

2. **Reorder around the §17 vertical slice.** Deliver control-ownership (records →
   assertions → one dimension set → AnalysisFrame → graph + matrix projectors →
   shared theme → explain-to-source) as the first semantic increment, on a small
   fixed set of types, before generalising any single layer horizontally.

3. **Write the materialisation contract before assertions exist.** Decide and
   document authority, staleness detection, and the single rebuild path for
   projections, so the assertion/GraphData split does not recreate the dual-write
   problem.

4. **Record the ADRs the review calls for (§16), plus one it omits:** an explicit
   decision that Layercake is committing to a consulting knowledge-modelling
   product, with the §17 slice as the go/no-go checkpoint. The other seven decisions
   are well chosen and should be recorded as written.

5. **Keep the escape hatch explicit.** Existing graph and code-analysis data remains
   pure GraphData with no semantic type; the semantic layer is additive and opt-in
   per project. This bounds migration cost and lets the two use cases coexist.

---

## 6. Overall

The review is accurate, well-structured, and points at a real and defensible target
architecture. Its weaknesses are not in the destination but in the trip: it
under-weights that the *previous* unification is still unfinished (with live reads
against dropped tables and a breakage fixed only this month), it presents a
horizontal roadmap that its own §17 contradicts, and it re-introduces a dual-write
hazard without a hard mitigation. Adopt the direction; finish and *prove* Phase 0
first; then validate the whole stack with the single vertical slice before building
any layer out broadly.
