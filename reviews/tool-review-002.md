# Tool review 002 — layercake

Session: 2026-07-15 (post-uplift). Same project 36, same working directory. Verifying what was addressed after review 001 and driving the plan through to rendered artefacts.

## The plan now runs end-to-end

Six artefacts on disk under `./output/`:

```
graph1.mmd   (nested subgraphs, threats vs reference, layer classDefs applied)
graph1.dot   (Graphviz)
graph2.mmd   (NW × Foundry × DSR, 61 nodes / 53 edges, containNodes)
scenario_a.mmd  (9 messages, 9 notes — pre-release feature spill)
scenario_b.mmd  (10 messages, 10 notes — bidirectional tool exfil / injection)
scenario_c.mmd  (9 messages, 9 notes — persistent lifecycle debt)
```

All three scenario diagrams have `source->>target: label` messages and note-position routing that matches what I authored. Before the uplift they were empty. This is a large step up.

## Review 001 findings — status

### Blockers — both fixed structurally

- **B1** (SequenceEdgeRef serde mismatch) — **fixed structurally, not just patched.** Commit `96705ded` picked the harder option: the mutation now converts API→pipeline `SequenceEdgeRef` before writing to disk (`edge_refs_to_persisted_json`), and added a migration (`m20260715_000002`) to rewrite existing rows. There is now one on-disk shape, owned by one type. This is exactly the "unify the seam" ask.
- **B2** (Handlebars `{{else if}}`) — **fixed.** Commit `05848025` rewrote both templates as nested `{{else}}{{#if …}}…{{/if}}` and added regression tests for all three notePositions. The commit message correctly identifies the subtler failure mode: the block form doesn't error, it silently mis-routes.
- **Bonus fix** — `to_mermaid.hbs` frontmatter title now quoted (`54de9036`). My Graph 1 label is `Graph 1: Foundry x DSR threats` — the colon was going to bite the next author; caught during the same pass.

### Non-blockers — mostly done

- **F2** (silent skips) — **done, with a caveat.** `SequenceStoryContext.warnings` now carries structured messages; `tracing::warn!` logs them; doctor surfaces them. But **`NodeExecutionResult` (GraphQL type) still has no `warnings` field** — an agent calling `executeNode` from the outside still gets `success: true, message: "…"` with no visible warning stream. Discoverable via doctor or by reading `sequence_contexts.context_json`, but not surfaced on the direct-return path. The commit message calls out only the DAG-executor log path.
- **F3** (dataset PK bridge) — **done.** `PlanDagNode.linkedDataSetId: Int` is populated and matches. Query it, get the PK, mutate the dataset. Clean.
- **F4** (graph-json boilerplate) — **done.** New `layercake doc guide graph-json` covers the exact shape (nodes/edges/layers required, weight/comment/attributes on each item). The `layers: []` requirement is now visible without diffing files. Small: the guide says `weight` is `int` required — my last session's data set `weight: 1` numerically and it worked, but a float would also probably work; not exercised.
- **F5** (NodeMetadata subfield selection) — the workflow now points at the pattern more clearly. Not verified as *documented* in every workflow — I still had to remember `metadata { label description }` from prior context — but the failure mode is more searchable.
- **D3** (Story.sequences) — **done.** `story.sequences { id name edgeCount }` works.
- **D5** (node types) — **done.** `layercake doc guide node-types` describes all 10 variants with io types and config hints. Explicitly calls out that `StoryNode` context uses `enabledDatasetIds` from the DB rather than DAG upstream — that was the trap I hit last session; now it's a documented invariant.
- **Rec1** (`layercake doctor`) — **done, and it's the most useful new tool.** Structural checks (orphaned graphs, unresolved sequence edges, missing datasets, empty stories, layer dupes) in one command with `--json` for scripting. This is the "green diagram with empty output" antidote review 001 asked for. Should be the first thing any new agent runs.

### Remaining from review 001

- **F1** (`api info` port) — **not addressed.** Still prints 3000, still cost me a mental double-take when I resumed. Deferred per the uplift plan.
- **F6** (orphaned computed graphs) — partially handled: doctor detects them ("orphaned-computed-graph" warning). No `pruneOrphanedGraphs` mutation.
- **F7** (StoryNode upstream is decorative) — **not addressed on the code side, but documented.** Per the node-types guide: "the context builder uses the Story's `enabledDatasetIds` from the DB, not the DAG upstream — wire DataSetNodes upstream of it." That converts a trap into a documented invariant. Fine for now; the real fix (either enforce or make the graph upstream real) is deferred.
- **D1** (port doc) — not reconciled; `layercake doc workflow drive-via-api` still shows 3000.
- **D2** (`showNotes` in schema) — **partially done.** The uplift plan notes `SequenceRenderConfigInput` exists in `config.rs:176`, but `layercake schema dump` still only shows the general `RenderConfigInput` (no `showNotes` field). Either the type isn't wired to the GraphQL schema yet, or it's only used server-side. From a caller's perspective: still pass `showNotes` via the free-form JSON `config`.
- **D4** (session header) — no change; still undocumented.
- **D6** (layer `source_dataset_id` semantics) — no change.
- **Rec2** (warnings on execution result) — covered under F2 caveat.
- **Rec3** (CLI shorthands) — no change.
- **Rec4** (`schema type <Name>`) — no change. `schema dump` is still the entrypoint.
- **Rec5** (id bridge on types) — covered by F3.
- **Rec6** (guides expansion) — three new guides shipped, others still open.
- **FR1** (dataset diff) — no change.
- **FR2** (renameNode) — no change.
- **FR3** (sequence-authoring UI) — I noticed the UI overwrites SequenceArtefactNode `renderTarget` on save; see new friction N1 below. So the UI did move, but not in the direction of easier sequence authoring per se.
- **FR4** (palette presets + WCAG) — no change.
- **FR5** (exportProject) — **already existed**, I just missed it in review 001. `exportProjectArchive`, `exportProjectToDirectory`, `reexportProject`, `exportProjectAsTemplate` are all in the schema. Retracting FR5 as satisfied.
- **FR6** (story preview endpoint) — not shipped, but B1/B2 fixes make the export path work, so the immediate need is gone.
- **FR7** (per-node execution timings) — no change.

## New friction points found this session

### N1. UI-side edits to SequenceArtefactNode configs silently flip renderTarget

Between review 001 and this session, someone (probably the UI's "story render-settings" button, added in `e95a01ef`) rewrote `seq_b` and `seq_c` from `MermaidSequence` to `PlantUmlSequence`. The `outputPath` fields still ended in `.mmd`. So my first re-export produced `.mmd` files containing PlantUML markup.

Two things bit here:
- Editing the artefact node externally mutated my authored config in a way the API path (`updatePlanDagNode`) couldn't see coming.
- The `outputPath` extension and the `renderTarget` can drift out of sync silently.

**Fix**: warn or reject when `renderTarget` and `outputPath` extension don't match (`MermaidSequence` should reject non-`.mmd`/non-`.md`; `PlantUmlSequence` should reject non-`.puml`/`.txt`). Or auto-rewrite the extension on renderTarget change.

### N2. `layercake doctor --port <n>` isn't recognized

`layercake api call` takes `--port`. `layercake doctor` takes `--database <path>` instead, no `--port`. Same operation ("point at the running server / underlying DB") uses different options depending on whether the command is talking over HTTP or reading SQLite directly. Cost me one confused invocation.

**Fix**: normalize. Either `--port` everywhere (doctor talks to the server too), or a top-level `--connection` / config file that all subcommands honour.

### N3. `layercake doctor` needs an absolute DB path

Default `--database layercake.db` resolves relative to cwd, so `doctor` from `/Users/…/netwealth/model/` fails to find the file that lives in `/Users/…/nst/layercake-tool/`. The tool is agnostic-cwd for `api` (talks over HTTP), but `doctor` isn't. Minor but surprising.

**Fix**: doctor should be able to resolve the DB path from a running server (`api info` already prints "Database file: layercake.db"); ideally `layercake doctor --project 36` should just work when the server is up.

### N4. Sequence artefact output extension defaults are surprising

For a Mermaid *graph* the extension is `.mmd`. For a Mermaid *sequence* the extension is also `.mmd`. Users can end up with `graph1.mmd` and `scenario_a.mmd` that require completely different rendering paths — the first is flowchart, the second is `sequenceDiagram`. Fine — that's how Mermaid works — but consider `.seq.mmd` or `.smmd` to reduce confusion in `output/`. Purely stylistic.

### N5. `containNodes` accepts both boolean and string ("one")

In my seq configs `containNodes` was stored as string `"one"` after UI touch, and as boolean `true` in the graph artefacts. Both seem to work. Confusing typing. Schema check would help.

### N6. When I set `builtInStyles: "light"` in a Mermaid graph artefact, the layer classDefs still applied

Not sure if intended. The `graph1.mmd` output has both no built-in theme applied (per the leading comment) and the `classDef reference fill:#DBE7F5,...` from the project palette. Result is fine — I'm noting that the interaction of `builtInStyles` and `applyLayers` isn't clear from the schema alone.

### N7. Sequence rendering uses **default** notePosition, not the one on the edge, when `notePosition` is null

The workflow doc says "Optional note text… `notePosition` is `Source | Target | Both`". Undefined behaviour for `null` in the doc. My testing suggests unset falls back to "both" (Note over source,target), but the code path isn't stated.

### N8. Sequence output isn't escaped against Mermaid grammar hazards (`;` breaks parsing)

Reproduced with `mmdc`: any `;` in a note or a message label truncates the statement early because Mermaid's sequence-diagram grammar treats `;` as a statement terminator. The parser's error position is misleading — it points at the *next* line, not the semicolon.

```
sequenceDiagram
    participant A
    participant B
    Note over A: DSR-05-C — Lineage tracking opacity: the SDK no longer resolves records; the audit trail is broken
    A->>B: hello

# Parse error on line 5: Expecting 'SOLID_ARROW' ... got 'NEWLINE'
```

Three notes in this project hit it. Workaround: replace `;` with `—` in the sequence note text via `updateSequence`. But the *right* fix is in the layercake renderer:

- Escape or replace forbidden chars in `note` and `label` before emitting into the `.hbs`.
- Known hazards to sanitize on the sequence path: `;` (statement terminator), and probably `\n` inside a note (already rare) — worth a survey.
- Same class of bug as the `to_mermaid.hbs` frontmatter title quoting fix (`54de9036`); apply the same rigour to sequence text.

Suggested implementation: a handlebars helper `{{mermaid_safe note}}` that replaces `;` with a comma or em-dash, escapes control chars, and (optionally) wraps in `<br/>` on `\n`. Apply it in both `to_mermaid_sequence.hbs` and `to_plantuml_sequence.hbs` (PlantUML has its own hazards — semicolons are safer there, but `//` isn't).

Alternative: PR upstream in Mermaid to allow `;` inside note text. Slower path.

## What worked really well

- **The uplift plan file** (`plans/20260715-tool-review-001-uplift.md`) is itself a nice artefact. It validates each finding against master, marks status, breaks work into structural / correctness / docs stages, and calls out deferred items with rationale. Any agent picking this up next would find the state in one file.
- **Commit messages point back to review IDs** (B1/B2/F2/F3/D3/D5/Rec1). Cross-referencing is fast.
- **Doctor doc's "For agents" section** — one-line explicit guidance on when to run it. More docs should end with a "For agents" section.
- **The uplift chose structural over patches.** B1 could have been a one-line `#[serde(rename_all = "camelCase")]` on the API type — it was, but *behind* a boundary conversion so the on-disk shape belongs to the pipeline type. The migration ensures no split-brain remains. This is the right call for a project-in-motion.

## Recommendations (net-new + carrying)

1. **Surface `warnings` on `NodeExecutionResult` and `PlanExecutionResult`.** The pipeline collects them; the log path prints them; but the direct-response path an agent hits via `executeNode` still has no `warnings: [String!]!` field. Adding it would close F2 completely.

2. **Auto-run doctor at end of `executePlan`** or expose a `strict: Boolean` argument that turns doctor findings into hard errors. Right now doctor is opt-in — a great tool that a new agent may not know about.

3. **`layercake doc` should have an `agent-runbook`** — a one-page top-level "when doing X in this repo, use Y" that indexes all the guides. `layercake doc guide agent` exists; extend it with a table of common tasks → workflow/guide.

4. **Doctor `--fix`.** For each check, if there's an obvious fix (prune orphaned graph, re-run empty story context after migration, etc.), offer to apply it. Would take doctor from diagnostic to remediation tool.

5. **Config validation for artefact nodes.** The `.config` JSON string bypasses the GraphQL schema. `SequenceRenderConfigInput` is defined but not wired; wire it. And validate extension ↔ renderTarget as in N1.

6. **A `layercake doc guide render-config`** that maps every rendered target to its accepted renderConfig keys — the biggest remaining "why isn't this doing what I expect" area. Add examples: light/dark theming, containNodes vs classDef interaction, sequence-specific keys.

7. **`layercake doctor --project 36 --strict` for CI.** Exit non-zero on any warning, not just errors.

## Feature requests carrying forward

FR1 (dataset diff), FR2 (renameNode), FR4 (palette presets + WCAG), FR6 (story preview) still open. FR3 partially moved via UI settings work but not toward CLI/API easier-authoring. FR7 (execution timings) still open.

Retracting FR5 (`exportProject`) — already exists as `exportProjectArchive`.

## Bottom line

**Both review-001 blockers are fixed structurally.** The three scenario diagrams and both graph diagrams render cleanly. The project is now usable end-to-end. Doctor is a great new diagnostic. The main remaining gap is that `NodeExecutionResult` should carry warnings — but doctor makes this easy to work around. Docs are noticeably better; two new guides + workflow updates cover most of the "shape not obvious" issues from review 001.

Highly usable state. If this were an OSS project I'd say "ship it, close the milestone, open follow-up issues for F1/F6/F7 and the FRs".
