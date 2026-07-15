# Tool review 001 — layercake

Session: 2026-07-15. Driving project 36 ("Netwealth PDLC") entirely via `layercake api call` against a running server, no code changes to the layercake source tree.

## Summary

Layercake is a graph-editing service with an ambitious surface — projects, datasets, plan DAGs, computed graphs, stories, sequences, artefacts, project-level layer palettes, live collaboration. When it worked, it worked well: read-modify-write on the plan DAG, `updateDataSet` / `updateDataSetGraphData`, `upsertProjectLayer`, `createStory` + `createSequence` all round-trip cleanly, and `executePlan` reliably topological-sorted 17 nodes.

The two blocking bugs I hit both live at seam boundaries — GraphQL types vs. pipeline types, and template vs. Handlebars version. Both are subtle-looking edge cases that pretty much any user who tries to render a sequence diagram will hit, so the "core end-to-end story" (per `layercake doc workflow develop-a-story`) is currently broken in practice.

## Blocking bugs (documented, not fixed per instruction)

### B1. `SequenceEdgeRef` serialization case mismatch between API and pipeline

The GraphQL type at `layercake-server/src/graphql/types/sequence.rs:22`:

```rust
#[derive(..., Serialize, Deserialize, SimpleObject, InputObject)]
pub struct SequenceEdgeRef {
    #[graphql(name = "datasetId")]
    pub dataset_id: i32,
    #[graphql(name = "edgeId")]
    pub edge_id: String,
    pub note: Option<String>,
    #[graphql(name = "notePosition")]
    pub note_position: Option<NotePosition>,
}
```

has default (snake_case) serde renaming — it serializes to `dataset_id`, `edge_id`, `note_position` — and `NotePosition` has no serde rename so variants serialize as `"Source" / "Target" / "Both"`. This is what ends up in the SQLite `sequences.edge_order` column.

The pipeline consumer at `layercake-core/src/sequence_types.rs:15`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceEdgeRef {
    pub dataset_id: i32,
    pub edge_id: String,
    ...
    pub note_position: Option<NotePosition>,
}
```

expects `datasetId`, `edgeId`, `notePosition` and its `NotePosition` (also at `sequence_types.rs`) has `#[serde(rename_all = "camelCase")]` — variants deserialize as `"source" / "target" / "both"`.

The result: `serde_json::from_str::<Vec<SequenceEdgeRef>>(&sequence.edge_order)` at `sequence_context.rs:404` fails silently (`.unwrap_or_default()`), returning an empty vector. Every `SequenceRender.steps` ends up empty, participants come out empty, and the exported diagram has zero messages — just a wall of `participant …` declarations and a title `Note over …`.

There is no client-visible error. `executePlan` reports "success". The stored `sequence_contexts.context_json` looks structurally healthy — you have to inspect `sequences[0].steps.length` to see that it's zero.

**Workaround I used:** patch the DB column directly.

```bash
for sid in 7 8 9; do
  cur=$(sqlite3 layercake.db "SELECT edge_order FROM sequences WHERE id=$sid;")
  new=$(echo "$cur" | jq -c 'map({datasetId, edgeId, note, notePosition: (.note_position | ascii_downcase)})')
  ...
done
```

This makes the pipeline parse succeed — but any subsequent `updateSequence` or `createSequence` puts the DB back into snake_case. The workaround is not durable across normal editing.

**Fix (upstream)**: pick ONE of the two shapes and use it everywhere. Simplest: give the API's `SequenceEdgeRef` the same `#[serde(rename_all = "camelCase")]` attribute plus the same rename on `NotePosition`. Add a one-shot migration to rewrite existing DB rows. There's no reason for two types with the same name and incompatible serialization to coexist in the same crate graph.

### B2. Handlebars template uses `{{else if}}` which the runtime rejects

Once B1 is worked around, the export fails hard:

```
Service 'render_mermaid_sequence' error: Error rendering "Unnamed template"
line 31, col 11: Helper not defined: "else"
```

Template at `layercake-core/src/export/to_mermaid_sequence.hbs:29-35`:

```handlebars
{{#if (stringeq notePosition "source")}}
    Note over {{source.alias}}: {{note}}
{{else if (stringeq notePosition "target")}}
    Note over {{target.alias}}: {{note}}
{{else}}
    Note over {{source.alias}},{{target.alias}}: {{note}}
{{/if}}
```

The `handlebars-rust` crate has famously not supported `{{else if}}` as a single token — it must be written as nested `{{else}}{{#if …}}…{{/if}}`. This exact template line has never rendered without B1 masking it.

**Fix (upstream)**: rewrite the block as nested `{{else}}{{#if …}}…{{/if}}`. Same for any sibling template — check `to_plantuml_sequence.hbs` too.

## Friction points (non-blocking, but sharp corners)

### F1. Non-default port trap

`layercake api info` unconditionally prints `Base URL: http://127.0.0.1:3000` even when the actual running server is on 3001 (this session's setup). It reads from a config default, not from the running server's `/health` or process introspection. Cost me the first ~5 minutes; will cost every new agent the same 5 minutes.

**Fix**: `api info` should either scan localhost for actually-listening layercake ports, or accept `--port` and print live-verified endpoints, or read from a well-known status file the server writes on startup.

### F2. Silent failure modes throughout the sequence pipeline

Three places I hit where a bad input just produced an empty result with a "success" response:

1. Sequence edge_order parse failure (B1).
2. `resolve_edge` returning `None` because the edge id isn't in the specified dataset — `continue` with no log.
3. `all_nodes.get(&edge.source)` returning `None` — `continue` with no log.

**Fix**: `NodeExecutionResult` should carry a `warnings: [String!]` field; the pipeline should collect "sequence 7 skipped 9 of 9 steps: could not parse edge_order" and surface it. Right now the UX is "everything is green, diagram is empty, good luck".

### F3. Numeric vs. hash node IDs

Documented in CLAUDE.md but worth repeating: DataSet / Graph mutations take `Int!` (primary key), plan DAG nodes have hash string ids like `dataset_766f8beffdb5`. Nothing in the schema or docs surfaces the correspondence. I got there via `dataSets(projectId: 36) { id filename }` and matching by file, but that's a coincidence — a plan DAG node's `config` JSON contains `"dataSetId": 196`, which is the real link. The GraphQL type doesn't expose that as a first-class field.

**Fix**: add `linkedDataSetId: Int` (or `linkedGraphId`), computed by parsing config, to `PlanDagNode`. Alternatively expose a `plan_dag_node_for_dataset(dataSetId: Int!)` accessor.

### F4. Graph JSON must include boilerplate that isn't obvious

My first `updateDataSetGraphData` call failed with `missing field weight`. Second failed with `missing field layers`. The doc says nothing about the exact `.graphJson` schema. I only figured out the shape by dumping an existing dataset's `graphJson` and diffing.

**Fix**: publish the GraphJson schema in `layercake doc guide graph-json` or similar. Even a Rust struct printed as a comment somewhere would help. Bonus: default `weight: 1`, `comment: ""`, `attributes: {}`, `layers: []` server-side rather than requiring them.

### F5. `NodeMetadata` requires subfield selections but the error is generic GraphQL

The gotcha "must have a selection of subfields" is easy to hit and easy to fix, but the schema doesn't document *what* fields to pick and the workflow docs don't mention it. Trivial once known. Worth a note in `layercake doc workflow edit-a-plan` explaining "the metadata object needs `{ label description }`".

### F6. Static IDs vs. server-generated IDs in `updatePlanDag`

I passed `id: "story_a"` explicitly. Server accepted it, echoed it back — good. But edges pass unnamed and get `edge_XXXX` — inconsistent. Also, deleting nodes via `updatePlanDag` (full replace) evidently leaves orphaned `graph_data` rows behind — `graphs(projectId: 36)` still returns the 4 old computed graphs from the previous DAG shape even after they're removed from the DAG. Whether that's intentional caching or a leak is unclear.

**Fix**: document that IDs are user-choosable and idempotent for nodes, and clarify the lifecycle of computed graphs when their originating DAG node disappears. Add a `pruneOrphanedGraphs(projectId)` mutation.

### F7. Sequence artefact requires DAG connection from a StoryNode, but the doc says stories can source "one or more datasets"

The `develop-a-story` workflow doc says a Story "belongs to a project, draws from one or more datasets" and the DAG diagram shows `DataSetNode → StoryNode → SequenceArtefactNode`. I initially wired `graph2 → story_a` (a GraphNode → StoryNode edge), thinking a merged graph would be a natural upstream. The API accepted the edge. Execution silently produced no participants. When I switched to `ds_ref, ds_threats, ds_mapping, ds_nw → story_a`, participants populated.

The story context builder actually **ignores DAG upstream entirely** — it looks up `story.enabledDatasetIds` from the DB. So the DAG-edge is decorative (and possibly required-by-execution-scheduling but not by data flow).

**Fix**: either enforce that upstreams to a StoryNode are DataSetNodes only (and their `dataSetId` is auto-added to `enabledDatasetIds`), or make it real — a GraphNode upstream should let the story reference edges from the merged graph, not just from raw datasets. Right now the docs suggest one behavior and the code does another.

## Documentation inconsistencies

### D1. Port

`CLAUDE.md` (project-specific) says port 3001. `layercake doc workflow drive-via-api` says port 3000. `layercake api info` prints 3000. All three of these should agree with reality, or clearly explain "3000 is the default; your project may run on another".

### D2. The story workflow's rendering config

`layercake doc workflow develop-a-story` shows:

```
SequenceArtefactNode config: `{ "renderTarget": "MermaidSequence", "outputPath": "checkout.mmd", "renderConfig": { "showNotes": true } }`
```

This is the *only* mention of `showNotes`. It's not a field in `RenderConfigInput` in the schema. It's not in any GraphQL type. It's a magic key that the pipeline understands but the schema won't validate. If a user reads `layercake schema dump | grep -A15 "input RenderConfigInput"`, they'll see `applyLayers`, `orientation`, `builtInStyles`, etc. — but not `showNotes`. They'll assume the doc is stale.

**Fix**: add `showNotes: Boolean` and `renderAllSequences: Boolean` (and any others) to `RenderConfigInput`, or split off a `SequenceRenderConfigInput`. Right now `config` is a JSON string field that bypasses schema validation, and callers just have to know.

### D3. Sequence "sequences" field naming collision

The GraphQL type calls it `sequenceCount` on Story (not `sequences`). The workflow doc example query is:

```bash
layercake api call --query 'query($id: Int!) { stories(projectId: $id) { id name description sequences { id name edgeCount } } }'
```

That fails: `Unknown field "sequences" on type "Story". Did you mean "sequenceCount"?`.

**Fix**: update the doc — either use `sequenceCount`, or add a `sequences` field on Story that returns `[Sequence!]!`.

### D4. `layercake api info` reference to `Session header: x-layercake-session: <id>`

No workflow doc explains when the session header matters. Multi-user collaboration? Rate limiting? Locking? A one-paragraph guide would help.

### D5. Node type documentation

There are 10 `PlanDagNodeType` variants (`DataSetNode` through `SequenceArtefactNode`). None of them have a doc string in `schema dump`. `FilterNode`, `ProjectionNode`, `TreeArtefactNode` — I don't know what they do, when they're required, or what config they expect. `layercake doc` has no `guide node-types`. This is the biggest missing piece of onboarding.

**Fix**: add `layercake doc guide node-types` describing each variant, its `config` JSON schema, its input types (`DataType` on edges), and its output type.

### D6. Layer `source_dataset_id` semantics

`ProjectLayer.sourceDatasetId` is nullable. `upsertProjectLayer` optionally takes `sourceDatasetId`. `setLayerDatasetEnabled(dataSetId, enabled)` operates on layers *originating from* a dataset. But `resetProjectLayers` retires everything — including the dataset-scoped ones. It's unclear whether a `sourceDatasetId`-tagged layer is "owned" by that dataset (auto-generated), or just "hinted" (manually attached). The three UI states (project palette, dataset-derived, dataset-overridable) aren't described.

## Recommendations

1. **Ship a `layercake doctor` command.** Scan projects for orphaned computed graphs, dangling sequence contexts, plan DAG nodes referencing missing datasets, layer duplicates, sequence edge references that don't resolve, etc. Everything I diagnosed by hand today would be one command output.

2. **Warnings, not silent skips, in the pipeline.** Every `continue` in `build_sequence_entries` should push a warning; the executor should surface warnings in `NodeExecutionResult.message` or a dedicated `warnings` array. Silent green is worse than red.

3. **Introspection on running projects.** `layercake api call` is the workhorse but the CLI could add higher-level shorthands: `layercake api story-list --project 36`, `layercake api dataset-show --id 199`, `layercake api plan-show --project 36 --format tree`. The equivalent GraphQL is verbose enough that I spent maybe 15% of the session on jq-and-quotes rather than actual thinking.

4. **Schema `.graphql` in `layercake doc guide`** so agents don't need to run `layercake schema dump` and page through 3000+ lines to find one input type. Or: `layercake schema type <Name>` that prints just one type's SDL.

5. **Consistent "hash id ⇔ pk id" bridge.** Add `datasetPkById(hashId: String!)`, `datasetHashByPk(id: Int!)`, and expose these on the types themselves (`DataSet.dagNodeIds: [String!]`, `PlanDagNode.datasetId: Int`). Discoverability of these joins is currently zero.

6. **Docs — the good parts.** `layercake doc workflow …` is a great pattern. Extend it: `guide render-config`, `guide graph-json`, `guide sequence-authoring-checklist`, `guide troubleshooting-empty-diagrams`.

## Feature requests

1. **Diff between two datasets** (or between dataset and graph). "What did the merge do?" is an obvious question with no tool.

2. **`renameNode(hashId, newHumanReadableId)`** — accept a nicer id when the auto-generated hash gets in the way of authoring stories or discussing the DAG with humans.

3. **Direct sequence-authoring UI in the frontend.** If the CLI story authoring is this fiddly (nine edge references, three note positions, dataset ids), a graphical picker would be a big lift.

4. **Layer colour presets and WCAG check.** The default palette used `#f7f7f8` (near-white background) with `#0f172a` text, contrast 20:1 — fine — but next to a plain white UI it disappeared. A "suggest palette" button with an accessibility check would go a long way. `dataviz` skill's referenced palette would be a great starting point.

5. **`exportProject(id, format: JSON)`** that dumps the full project (datasets, plan DAG, stories, sequences, layers) so an agent-driven session can be snapshotted and re-loaded elsewhere. Bonus: makes reproducing bugs like B1/B2 trivial.

6. **Story preview endpoint** that returns the pre-rendered `SequenceStoryContext` JSON without going through the Handlebars template. Would have unblocked me on B2 immediately — I could have rendered it client-side.

7. **Live diagnostics on `executePlan`.** Currently returns `{success, message, outputFiles}`. Could return per-node timings, warnings, output sizes. "Executed 17 nodes in topological order" is a great one-liner but there's no drill-in.

## Miscellaneous observations

- The `EnterpriseAgenticHub` reference model is well-modelled in dataset 196. The DSR framework in 195 is well-organized. The mapping edges in 197 are precise. The domain data quality is high — the tool is the bottleneck, not the content.
- `layercake schema dump` output is deterministic and grep-friendly — good. I would kill for `--only-mutations` and `--only-inputs` flags.
- The MCP-like agent presence feature (`layercake api join --project 36 --agent`) is a nice touch. I didn't exercise it in this session but the affordance is thoughtful.
- Deleting a `DataSetNode` from the plan DAG doesn't delete its underlying dataset — good. But it also doesn't warn if the dataset is orphaned afterward. See recommendation 1.

## Bottom line

The Netwealth PDLC project is up: layers harmonised (3), NW dataset rewritten with actors + Entra + MCP bridge + connectors (19 nodes, 18 edges), plan DAG rebuilt (17 nodes, 16 edges), Graph 1 and Graph 2 executed (42/35 and 61/53 nodes/edges respectively), three stories created with 9/10/9 sequence steps. Story context builds correctly *after DB patching*, but the final Mermaid sequence render is blocked by B2 (Handlebars `{{else if}}`).

Without touching layercake code, the sequence diagrams cannot render for this project. Once B1 and B2 are fixed upstream, the plan should render three publication-quality scenario diagrams end-to-end.
