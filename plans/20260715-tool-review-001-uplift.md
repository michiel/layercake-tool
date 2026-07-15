# Uplift plan — tool-review-001

**Date:** 2026-07-15
**Source:** `reviews/tool-review-001.md` (agent-driven session on project 36).
**Status:** Stages 1–7 complete (branch `fix/tool-review-uplift`).
**Approach:** structural fixes over patches (unify seams, remove masking fallbacks, make failures visible). Validate every finding against current `master` before acting — several sequence-area items overlap recent PRs (#79/#80), so state may differ from the review.

## Validation of review findings (done against current master)

| # | Finding | Status after validation | Notes |
|---|---------|--------------------------|-------|
| **B1** | `SequenceEdgeRef` serde case mismatch API↔pipeline | **CONFIRMED, still broken** | API type (`sequence.rs:20`) has only `#[graphql(name=...)]`, no `#[serde(rename_all)]` → writes snake_case + `"Source"/"Target"/"Both"` to `sequences.edge_order`. Pipeline type (`sequence_types.rs`) is camelCase + lowercase. `serde_json::to_string(&input.edge_order)` (mutations/sequence.rs:36,101) writes the API shape; `sequence_context.rs` reads the pipeline shape → parses to empty. **This is the true root cause of empty diagrams**; PR #80's `resolve_edge` is downstream and never runs. |
| **B2** | Handlebars `{{else if}}` rejected | **CONFIRMED, nuanced** | Isolated `render_template` with inline `{{else if}}` errors `Helper not defined: "else"`. The `.hbs` uses the block-standalone form, which does NOT error but **silently mis-routes** — `notePosition:"target"` renders as the `{{else}}` (both) branch. So the target-note branch is dead. Present in `to_mermaid_sequence.hbs:31` and `to_plantuml_sequence.hbs:30`. |
| **NEW** | Mermaid frontmatter title breaks on `:` | **CONFIRMED** | `title: {{graph_name}}` (unquoted YAML) in `to_mermaid.hbs:2`, `to_mermaid_mindmap.hbs:2`, `to_mermaid_treemap.hbs:2`. A title containing `:` produces invalid frontmatter → Mermaid parse error. |
| F1 | `api info` prints hardcoded 3000 | CONFIRMED | `api.rs` builds URL from `--port` default 3000; doesn't verify against a running server. |
| F2 | Silent skips in sequence pipeline | CONFIRMED (partially mitigated) | PR #80 removed the all-nodes fallback; still `continue`s silently on unresolved edges/nodes with no warning surfaced. |
| F3 | Numeric vs hash node id bridge | CONFIRMED | `PlanDagNode` doesn't expose the `config.dataSetId` link as a first-class field. |
| F4 | GraphJson boilerplate undocumented | CONFIRMED | No `graph-json` guide; `updateDataSetGraphData` requires `weight`/`layers`. |
| F5 | NodeMetadata subfield selection | CONFIRMED (doc) | Docs don't mention `{ label description }`. |
| F6 | Static vs generated ids; orphaned graphs | CONFIRMED | No prune; lifecycle undocumented. |
| F7 | StoryNode upstream is decorative | CONFIRMED | Context builder uses `story.enabledDatasetIds`, ignores DAG upstream. |
| D2 | `showNotes` not in schema | **PARTIALLY FIXED** | `SequenceRenderConfigInput` with `show_notes`/`render_all_sequences`/`contain_nodes` EXISTS (`config.rs:176`). Need to verify it's wired to the artefact config path (vs the JSON-string `config` bypass) and update the doc. |
| D3 | Story `sequences` field missing | CONFIRMED | Story exposes `sequenceCount` (`story.rs:73`), not `sequences`. Doc query fails. |
| D5 | Node types undocumented | CONFIRMED | No `guide node-types`. |

## Stages

Ordered by impact and independence. Each stage builds, tests, and commits on its own (structural fixes first, then diagnostics, then docs). Backend fixes on `fix/tool-review-uplift`; larger additive features may split to their own branches.

### Stage 1 — B1: unify `SequenceEdgeRef` serialization (root seam) [STRUCTURAL]
The two same-named types with incompatible serde is the architectural defect. **Single source of truth:** make the persisted representation canonical and unambiguous.
- Give the API `SequenceEdgeRef` + `NotePosition` (`sequence.rs`) `#[serde(rename_all = "camelCase")]` and lowercase `NotePosition` serde renames so the *serde* (DB) shape matches the pipeline exactly. (GraphQL names already match via `#[graphql(name)]`.)
- **Better structural option (evaluate):** have the mutation persist via the *pipeline* `sequence_types::SequenceEdgeRef` (convert API→pipeline once at the boundary), so there is one on-disk shape owned by one type. Prefer this if the conversion is clean.
- One-shot migration to rewrite existing `sequences.edge_order` rows from snake_case→camelCase (and `NotePosition` case), idempotent.
- Test: round-trip create→persist→`build_story_context` yields non-empty steps; a golden test asserting the persisted JSON shape.
**Status:** Not Started

### Stage 2 — B2: fix `{{else if}}` mis-routing in sequence templates [CORRECTNESS]
Rewrite the note-position branch as nested `{{else}}{{#if …}}…{{/if}}` (handlebars-rust supported) in both `to_mermaid_sequence.hbs` and `to_plantuml_sequence.hbs`. Add a render test asserting `notePosition:"target"` → `Note over <target>` (not both).
**Status:** Not Started

### Stage 3 — NEW: quote Mermaid frontmatter title [CORRECTNESS]
Quote + escape the title in `to_mermaid.hbs`, `to_mermaid_mindmap.hbs`, `to_mermaid_treemap.hbs` (`title: "{{graph_name}}"` with `"`→`\"`), or add a handlebars helper. Test a title containing `:` produces valid frontmatter.
**Status:** Not Started

### Stage 4 — F2/Rec2: pipeline warnings instead of silent skips [ARCHITECTURE]
Structural: thread a warnings channel through sequence building so "sequence N skipped M of K steps: unresolved edge X" is collected and surfaced in the node execution result. Make silent-green impossible.
- Add `warnings: Vec<String>` to the story-context build path; propagate into `NodeExecutionResult`/execution annotations.
- Test: an unresolvable edge yields a warning, not silence.
**Status:** Not Started

### Stage 5 — F3/Rec5: expose the dataset id bridge on PlanDagNode [ARCHITECTURE]
Add `linkedDataSetId: Int` (computed from `config.dataSetId`) to `PlanDagNode` GraphQL type. Low-risk, high-discoverability.
**Status:** Not Started

### Stage 6 — D2/D3/D5 + docs: schema + guides [DOCS/SCHEMA]
- D3: add a `sequences: [Sequence!]!` resolver on `Story` (the doc's query should work), keep `sequenceCount`.
- D2: verify `SequenceRenderConfigInput` is honoured on the artefact config path; update `develop-a-story` doc to reference real fields; document the JSON-`config` vs typed-input situation.
- D5: `layercake doc guide node-types` (each `PlanDagNodeType`, its config, io types).
- F4: `layercake doc guide graph-json`.
- F5: note in `edit-a-plan` about `{ label description }`.
- D1: reconcile port docs (3000 default, note per-project override).
**Status:** Not Started

### Stage 7 (own branch) — Rec1: `layercake doctor` diagnostics command [FEATURE]
Scan a project for: orphaned computed graphs, sequence edge refs that don't resolve, plan DAG nodes referencing missing datasets, empty sequence contexts, layer duplicates. Emits a report. This operationalises everything the reviewer diagnosed by hand.
**Status:** Not Started (candidate for separate PR)

### Deferred / evaluate (not committed without sign-off)
- F1 `api info` live port detection — modest; do a lightweight `/health` probe of a candidate port and report verified vs default.
- F6 `pruneOrphanedGraphs` mutation + lifecycle docs.
- F7 make StoryNode DAG upstream real, or enforce DataSetNode-only + auto-enable — a behaviour decision, needs product input.
- Rec3 higher-level CLI shorthands (`api story-list`, etc.).
- Rec4/FR: `schema type <Name>`, `--only-mutations/--only-inputs`, `exportProject`, story-preview endpoint, palette presets. Feature requests — surface as issues, implement on request.

## Guiding principle
Per user: prefer structural/architecturally sound solutions; uplift design; unify seams rather than patch. B1 and F2 are the clearest examples — fix the seam and remove the masking, don't add a translator.
