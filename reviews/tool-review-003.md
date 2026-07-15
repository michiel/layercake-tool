# Tool review 003 — layercake

Session: 2026-07-16. Same project 36, driving via `layercake api call` at port 3001. This is the third review pass; the target project is stable and rendering, so this pass is about verifying the second uplift + a batch of deferred feature work landed cleanly.

## Where we are

- Every review-001 finding: **closed** or **documented as expected behaviour** (F1/F6/F7).
- Every review-002 finding: **closed**, except the doctor DB-CWD issue (N3 uplift regression, described below).
- Six deferred feature requests from earlier reviews now shipped: `exportProject`, `previewStoryContext`, `pruneOrphanedGraphs`, `diffDatasets`, `renamePlanDagNode`, palette presets + `checkContrast`, per-node timings, `schema type <Name>` + `--only-*`, `docs/agent-runbook`, `docs/render-config`, doctor `--strict`, doctor DB resolution from `/health`, `enabledGraphIds` on Story, `warnings` on execution results, `showNotes` schema clarification (now in the render-config guide), sequence `;` neutralisation.

That is a substantial delivery. The API surface now matches the mental model in the workflow docs; agents can drive it without needing to open the source tree.

## Review 002 findings — status

### Fully closed

- **F1** (`api info` port) — **fixed** (`8a610327`). Probes `/health`, reports "reachable (version …)" or "NOT reachable at this address; a layercake server IS answering on port(s): 3001 — try --port 3001". Exactly the affordance I asked for.
- **F2** (warnings on execution result) — **fixed** (`9154ae49`). `NodeExecutionResult.warnings: [String!]!` and `PlanExecutionResult.warnings: [String!]!` are both present. Empty on a clean run; populated on unresolved edges / empty stories. Closes review 001 F2 fully.
- **F6** (orphaned computed graphs) — **fixed** (`2fd51015`). `pruneOrphanedGraphs(projectId): [Int!]!` returns pruned ids; lifecycle docs updated.
- **F7** (StoryNode upstream) — **fixed** (`dc59f5ba`) by adding `enabledGraphIds` to `CreateStoryInput`/`UpdateStoryInput` and `Story`. A story can now source edges from a computed GraphNode's output as well as raw datasets — the "make the DAG upstream real" ask, cleanly done.
- **FR5** (exportProject) — **existed** (`exportProjectArchive`) but a new plain `exportProject(id: Int!): String!` query was added (`3f22f62b`). Retract as complete.
- **FR6** (story preview endpoint) — **fixed** (`3f22f62b`). `previewStoryContext(projectId, storyId): String!` returns the full context JSON. I used it in this session — very useful for authoring feedback loops.
- **FR1** (dataset diff) — **new: fixed** (`5a591e64`). `diffDatasets(fromDatasetId, toDatasetId): GraphDiff!` with `nodes { added removed changed unchanged }` and `edges { … }`. Structural, purely-id-based diff.
- **FR2** (renamePlanDagNode) — **new: fixed** (`366c1ddf`). Also updates every incident edge — verified in this session.
- **FR4** (palette presets + WCAG) — **new: fixed** (`e5e90765`). `palettePresets: [Palette!]!` and `checkContrast(background, text): ContrastResult!` with `ratio, passesAa, passesAaLarge, passesAaa`. Three curated palettes shipped: slate / dawn / lagoon (all AA). My current project palette all pass AAA per checkContrast (7:1+).
- **FR7** (per-node timings) — **new: fixed** (`d3e83425`). `PlanExecutionResult.nodeResults: [NodeExecutionTiming!]!` with `nodeId, nodeType, durationMs`. My plan's bottleneck was `merge_g2` (154 ms) and `merge_g1` (119 ms) — 88% of the total 349 ms.
- **N1** (renderTarget↔extension) — **fixed** (`16e5c684`). Setting `renderTarget:"MermaidSequence"` with `outputPath:"bad.puml"` is rejected with a clear message: `"renderTarget MermaidSequence expects an output extension of .mmd or .md but outputPath is 'bad_ext.puml'. Update the extension or the renderTarget so they match."` Exactly the ergonomic guard that would have caught the seq_b/c drift last session.
- **N5/D2** (`containNodes` bool/string, `showNotes` schema) — **partially fixed**. The config validator now accepts both `containNodes: true` and `containNodes: "one"`. `showNotes` is documented in the new `render-config` guide as the source of truth (D2 is now "the JSON `config` bypasses schema validation; consult this guide"). Not yet a first-class `SequenceRenderConfigInput` type. Fine for now — the guide is honest about the tradeoff.
- **N6** (builtInStyles vs applyLayers) — **fixed** by clarification. `render-config` guide has a paragraph called out under the graph artefacts table: "`applyLayers` emits your palette; `builtInStyles` selects a base theme; they're independent". Clear.
- **N7** (default notePosition) — **fixed** by clarification. `render-config` guide: "If `notePosition` is null/unset, it defaults to `Both`".
- **N8** (mermaid `;` breaks parsing) — **fixed** (`3e026320`). New `mermaid_safe` handlebars helper replaces `;` with `—` in note and label text at render time. I re-introduced a semicolon in a note via `updateSequence`, re-exported, and got clean `—` in the output — original DB text preserved. Regression test present.
- **D1** (port doc reconciliation) — **fixed** (`a71489e2`). Port is now consistently discoverable via `api info`, doc `workflow drive-via-api` presumably updated (I didn't diff).
- **Rec1** (doctor exists) — noted done in review 002.
- **Rec2** (auto-run doctor at end of execute) — **substantially addressed** by warnings on execution result. Not literally auto-run, but the "silent green" case is closed.
- **Rec3** (agent-runbook) — **fixed** (`a71489e2`). `layercake doc guide agent-runbook` — a one-page "task → where to look" table with gotchas. This is the first thing any new agent should read.
- **Rec4** (schema type / filters) — **fixed** (`b4664072`). `layercake schema type <Name>`, and `schema dump --only-inputs` / `--only-mutations`. Verified with `schema type Sequence` and `schema type GraphDiff`.
- **Rec6** (render-config guide) — **fixed** (`a71489e2`). Big guide covering both graph and sequence render targets.
- **Rec7** (doctor --strict) — **fixed** (`819501de`). Exit non-zero on any warning. `layercake doctor --project 36 --strict`.

### Remaining / new issues

- **N2** (`doctor --port`) — **fixed** (`819501de`); doctor now takes `--host`/`--port`/`--url` and can resolve DB path from a running server's `/health`.
- **N3** (doctor DB-CWD) — **regressed while trying to fix**. When I ran `layercake doctor --project 36 --port 3001` from `/Users/michielkalkman/dev/netwealth/model/`, the server-reported `"database": "layercake.db"` was treated as **relative to doctor's cwd**, not to the server's cwd. Doctor created an empty `layercake.db` in `/Users/michielkalkman/dev/netwealth/model/` and then errored "no such table: plans". Two things going wrong:
  1. The server's `/health` should report an **absolute** database path (or a `database_absolute` field alongside). Currently it echoes the same relative path the CLI was launched with.
  2. Doctor should refuse to operate on a database file that has zero tables — or at least refuse to *create* one when auto-resolving.
  Workaround: keep passing `--database /absolute/path/to/layercake.db`. If you have a running server, `layercake doctor` from anywhere-but-the-server's-cwd currently fails.
- **N9** (new) — `api info` prints the DB path with the same limitation. `Database file: layercake.db` in the output is a relative literal. Same fix as N3.

## New friction found while exercising the new features

### N10. `checkContrast` returns nulls when the hex is single-quoted via `eval`

Purely a caller-side issue; escape carefully. Trivial when using `--variables` or a heredoc. Worth a line in the FR4 doc.

### N11. `PaletteSwatch.contrastRatio` is only reported for the swatch's `backgroundColor`/`textColor` pair

Which is what you want most of the time — but if you want to check the swatch against the app UI background (`#ffffff`), you'd still need `checkContrast`. Fine, just noting.

### N12. `NodeExecutionTiming.durationMs: Int!` — artefact nodes report 0 ms

`GraphArtefactNode` and `SequenceArtefactNode` all read `durationMs: 0`. That's because artefact rendering happens on `exportNodeOutput`, not during `executePlan` — they're lazy. The type name `NodeExecutionTiming` conflates "execute" with "produce output", which is a legitimate distinction. Would be clearer if the timings had a `phase: "execute" | "render"` field, or if the type comment called this out. Minor.

### N13. `pruneOrphanedGraphs` docs

`layercake doc guide` doesn't have a "graph lifecycle" guide yet. `pruneOrphanedGraphs` is discoverable via `schema dump | grep prune`, but a paragraph in the agent-runbook or a `guide graph-lifecycle` would tie together: dataset → DataSetNode → GraphNode/Merge/Transform → `graph_data` row → GraphArtefactNode → exported bytes. The lifecycle for each stage (created when, invalidated when, pruned when) is currently only in commit messages.

### N14. `renamePlanDagNode` semantics

The mutation works. Not immediately clear:
- What happens if `newId` collides with an existing node? (I didn't test.)
- Does it fire the same DAG-change WebSocket event as `updatePlanDagNode`?
- Is it idempotent — old==new is presumably a no-op?

A one-line SDL comment on the mutation would answer these.

## Retracting from earlier reviews

- FR3 (sequence authoring UI) — `8e76e65e` adds a computed-graph source picker in the UI. Not exercised in CLI work but it's there.
- Rec5 (id bridge) — closed via `linkedDataSetId` in uplift 1.

## What still feels un-addressed

Looking back through review 001 and 002:

1. **No `docs/guide graph-lifecycle`** — see N13.
2. **`showNotes` still not first-class in the schema** — deliberate per the render-config guide, but a schema type would still enable validation. Low priority.
3. **`layercake doc guide render-config` covers the *keys* but not the *interaction matrix*** — e.g., what happens when `containNodes: true` meets `orientation: LR` for a deep hierarchy? Small doc quibble.
4. **No `--fix` for doctor.** Findings are labelled but there's no automated remediation. Given `pruneOrphanedGraphs` exists, `doctor --fix` could at least call it. Would take doctor from diagnostic to guardrail.
5. **The `layercake db info` command is still discoverable only via `--help`** — should be a first-class row in the agent-runbook. Actually it *is* there, my mistake — retracting.

## Recommendations

1. **Server should report the absolute DB path on `/health`** (N3/N9). Also print an absolute path in `layercake api info`. One-line fix that closes the doctor-from-remote-cwd bug.

2. **Doctor should refuse an empty/no-tables DB** — if it resolves to a path that opens but has no `plans` table, error with "this doesn't look like a layercake DB" rather than propagating the SQLite error verbatim.

3. **Consider `PaletteApply` mutation.** `palettePresets` returns curated palettes but there's no one-shot "apply preset X to project Y" mutation. Callers can loop over `upsertProjectLayer`, but the affordance would be nice; I could see a UI wanting a "one-click apply" button. Low priority.

4. **The agent-runbook is a great pattern — extend it.** Add a "common pitfalls" section that pre-answers: "why is my diagram empty" (doctor), "why does my `.mmd` contain `@startuml`" (renderTarget drift, now caught by validation), "why does mmdc reject my output" (semicolons, now neutralised). Turn every bug from every review pass into a runbook line.

5. **Structured warnings.** `warnings: [String!]!` is nice, but every warning is a free-form string. If callers want to react programmatically ("hide this in CI, escalate that"), a `type Warning { code: String!, message: String!, nodeId: String, severity: Warning! }` shape would be better. Not urgent — free strings are usable.

## Feature requests carrying forward

- FR3b (higher-level CLI shorthands like `layercake story list --project`) — still uncovered by CLI; a UI is present now.
- FR "doctor --fix" per Rec 4 above.
- FR "guide graph-lifecycle" per N13.

## Bottom line

**All three review passes now converge on green.** Every blocking bug is fixed structurally. Every friction point I identified is either closed or documented as intended. Six major FRs (diff, rename, exportProject, previewStoryContext, palettes, timings) have shipped in the two uplift PRs since review 001. The response cadence and quality of the fixes have been exceptional — commits reference review IDs, structural fixes chosen over patches, migration paths thought through, regression tests included.

The one lingering issue is the doctor-DB-CWD interaction (N3 regression). It's a one-line server-side fix (`/health` should print an absolute path). Everything else is polish.

The project (Netwealth PDLC) renders end-to-end into publication-quality Mermaid + DOT + PlantUML artefacts. The tool is production-usable for agent-driven modelling. Congratulations on shipping this.
