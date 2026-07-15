# Uplift plan — tool-review-002

**Date:** 2026-07-15
**Source:** `reviews/tool-review-002.md` (post-uplift session, project 36).
**Approach:** structural over patches (per project direction / memory). Validate every finding against current master first — the reviewer's build predated PR #87, so several "remaining" items are already shipped.

## Validation: what's stale vs. live (checked against master)

**Already on master (reviewer saw a stale build — NO action):**
- F1 `api info` health probe + port detection — shipped (PR #87, `8a610327`). `api.rs` has `probe_health`/`detected_ports`.
- F6 `pruneOrphanedGraphs` mutation — shipped (`2fd51015`).
- Rec4 `schema type <Name>` + `--only-inputs/--only-mutations` — shipped (`b4664072`).
- F7 `enabledGraphIds` (GraphNode sourcing real) — shipped (`dc59f5ba`).
- exportProject — reviewer already retracted FR5.

**Genuinely live findings (ACTION):**

| # | Finding | Verdict | Notes |
|---|---------|---------|-------|
| **N8** | `;` in note/label breaks Mermaid sequence parsing | **CONFIRMED, real render bug** | `inline` helper only does `split_whitespace().join(" ")` — `;` passes through. Same class as the frontmatter-title fix. Highest value. |
| **Rec1/F2** | `NodeExecutionResult`/`PlanExecutionResult` lack `warnings` | **CONFIRMED** | `helpers.rs:483-494` — only `success`/`message`. Pipeline collects warnings but they don't reach the direct-return path. Closes F2 fully. |
| **N1** | renderTarget ↔ outputPath extension drift (silent) | **CONFIRMED** | Editing a SequenceArtefactNode can flip `MermaidSequence`→`PlantUmlSequence` while `outputPath` stays `.mmd`. No validation. |
| **N5 / D2** | `containNodes` typed as both bool and enum; `SequenceRenderConfigInput` not wired | **CONFIRMED** | `config.rs`: `SequenceRenderConfig.contain_nodes: SequenceContainNodes` (enum) vs `RenderConfig.contain_nodes: Option<bool>`. `SequenceRenderConfigInput` exists but the artefact `config` is a free-form JSON string that bypasses it. |
| **N2** | `doctor` has `--database`, not `--port` (inconsistent w/ `api`) | CONFIRMED | Ergonomics. |
| **N3** | `doctor --database layercake.db` is cwd-relative; can't resolve from server | CONFIRMED | Ergonomics; should work when the server is up. |
| **N7** | null `notePosition` behaviour undocumented | CONFIRMED (doc) | Falls back to "both". Document + make explicit. |
| **Rec2** | auto-run doctor / `--strict` | Partially (doctor exits non-zero on errors; no `--strict` for warnings). |
| **Rec3/Rec6** | `doc guide render-config`, `agent-runbook` | Docs. |
| **N4/N6** | stylistic (`.smmd` ext; builtInStyles×applyLayers clarity) | Low — docs only. |

## Stages

Ordered by value. Each stage builds + tests + commits independently.

### Stage 1 — N8: sanitize sequence text against grammar hazards [CORRECTNESS, highest value]
Add a `mermaid_safe` handlebars helper: collapse whitespace/newlines (like `inline`), and replace `;` (Mermaid sequence statement terminator) with `—` (or `,`). Apply to every note/label/name in `to_mermaid_sequence.hbs`. For `to_plantuml_sequence.hbs`, add a `plantuml_safe` (or reuse, PlantUML's hazards differ — `;` is safe there, but survey). Regression test: a note containing `;` renders without breaking the statement.
**Status:** Not Started

### Stage 2 — Rec1/F2: surface warnings on execution results [ARCHITECTURE, closes F2]
Thread the `SequenceStoryContext.warnings` (already collected) up to the GraphQL execution result. Add `warnings: [String!]!` to `NodeExecutionResult` and `PlanExecutionResult`. The executor must collect per-node warnings and return them. This closes the F2 caveat — an agent calling `executeNode`/`executePlan` sees the warning stream directly, not just via doctor/logs.
**Status:** Not Started

### Stage 3 — N1 + N5/D2: artefact config validation [CORRECTNESS]
- Validate `renderTarget` ↔ `outputPath` extension on SequenceArtefactNode save (reject/warn: MermaidSequence needs `.mmd`/`.md`; PlantUmlSequence needs `.puml`/`.txt`), OR auto-rewrite the extension. Prefer a validation warning surfaced via the new warnings channel + a hard reject on clear mismatch.
- Wire `SequenceRenderConfigInput` (or document that `config` is JSON and normalise `containNodes` string/bool). At minimum, make the pipeline accept both `containNodes: "one"` and `true` consistently and document it. (Full schema-typed config is larger — evaluate.)
**Status:** Not Started

### Stage 4 — N2/N3: doctor ergonomics [UX]
- Add `--port`/`--host`/`--url` to `doctor` so it can resolve the DB path from a running server's `api info` (the server knows its `--database`). Keep `--database` as an override. Goal: `layercake doctor --project 36` works when the server is up, cwd-independent.
- Requires the server to expose its database path — check `/health` or add a lightweight info endpoint.
**Status:** Not Started

### Stage 5 — Rec2/Rec7: doctor `--strict` + auto-run hook [UX]
- `doctor --strict`: exit non-zero on warnings too (for CI).
- Consider an `executePlan` option or post-run doctor summary. Evaluate scope; at minimum `--strict`.
**Status:** Not Started

### Stage 6 — Docs: render-config guide, agent-runbook, N7 [DOCS]
- `layercake doc guide render-config` — every render target → accepted renderConfig keys, with examples (theming, containNodes vs classDef, sequence keys, showNotes).
- `layercake doc guide agent-runbook` (or extend `guide agent`) — task → workflow/guide index.
- Document null `notePosition` = "both" (N7) in develop-a-story.
- Reconcile port docs (D1): note 3000 default + per-project override.
**Status:** Not Started

## Deferred / file as issues (not building now)
- N4 (`.smmd` extension) — stylistic; issue.
- N6 (builtInStyles × applyLayers interaction) — clarify in render-config guide (Stage 6).
- Doctor `--fix` (Rec4 in review-002) — larger remediation feature; issue.
- Carrying FRs already tracked: #82 renameNode, #83 dataset diff, #84 exec timings, #85 palette+WCAG, #86 frontend authoring. FR6 story-preview — `previewStoryContext` already shipped (PR #87); reviewer's "not shipped" is stale.

## Guiding principle
Structural, seam-unifying fixes; make failures visible (N8 render bug, warnings on results); don't add patches. N8 and Rec1 are the two that most improve day-to-day agent use.
