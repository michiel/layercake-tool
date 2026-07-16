# Guide: Agent runbook

One-page "when doing X, use Y" index for driving layercake as an agent. Start
here; each row points at the workflow/guide/command with the detail.

## First moves

1. **Find the server** — `layercake api info` (probes `/health`, tells you the
   real port and version, hints if you're on the wrong one).
2. **Health-check the project** — `layercake doctor --project <id>` (resolves the
   DB from the running server; run it whenever a diagram is empty or a graph
   looks stale). Add `--strict` in CI.
3. **Learn the API** — `layercake schema dump` (full SDL, offline),
   `layercake schema type <Name>` (one type), `--only-inputs`/`--only-mutations`.

## Task → where to look

| I want to… | Use |
|------------|-----|
| Drive a running server over HTTP | `doc workflow drive-via-api`; `api call` |
| Edit a plan DAG (nodes/edges) | `doc workflow edit-a-plan` |
| Understand node types + their config | `doc guide node-types` |
| Trace dataset → computed graph → exported bytes | `doc guide graph-lifecycle` |
| List curated colour palettes / apply one in a call | `palettePresets` query; `applyPalettePreset(projectId, presetName)` mutation |
| Check a colour pair passes WCAG before using it | `checkContrast(background, text)` query |
| Know the `graphJson` shape for datasets | `doc guide graph-json` |
| Author a story → sequence diagram | `doc workflow develop-a-story` |
| Source a story from a merged graph | `develop-a-story` (§ `enabledGraphIds`) |
| Know which `renderConfig` keys a target accepts | `doc guide render-config` |
| Preview a story without rendering | `previewStoryContext(projectId, storyId)` |
| Run a plan headless (no server) | `doc workflow run-a-plan-headless` |
| Inspect the DB directly | `doc workflow inspect-database`; `db info` |
| Appear as a collaborator in the UI | `doc workflow join-as-collaborator` |
| Snapshot a whole project | `exportProject(id)` query |
| Diagnose "green but empty diagram" | `layercake doctor` |
| Clean up orphaned computed graphs | `pruneOrphanedGraphs(projectId)` mutation |

## Gotchas worth internalising

- `layercake query` hits the **DB file directly**; `layercake api call` hits a
  **running server**. Live UI updates need `api call`.
- `metadata` needs a subfield selection: `metadata { label description }`.
- Plan node ids are hash strings; join to a dataset PK via
  `PlanDagNode.linkedDataSetId`.
- A StoryNode sources from the story's `enabledDatasetIds`/`enabledGraphIds`,
  **not** the DAG upstream. Wire the sources on the story.
- Sequence notes render only when `renderConfig.showNotes` is true; an unset
  `notePosition` means `Both`.
- `execute*` results now carry a `warnings` array — check it.

## Common pitfalls (symptom → cause → fix)

| Symptom | Cause | Fix |
|---------|-------|-----|
| Diagram renders empty (no error) | Stale/missing computed graph for the artefact's source node | `layercake doctor --project <id>`; re-execute the producing Graph/Merge/Transform node (see `doc guide graph-lifecycle`) |
| A `.mmd` output contains `@startuml` (or `.puml` contains `sequenceDiagram`) | `renderTarget` was switched but `outputPath` extension stayed stale | Now guarded: the server rejects a mismatched target/extension, and the UI rewrites the extension on switch. Set `outputPath` extension to match the target (`.mmd`/`.md` for Mermaid, `.puml`/`.txt` for PlantUML) |
| `mmdc` rejects the rendered output | Stray semicolons in labels breaking Mermaid syntax | Now neutralised in the templates; re-render |
| `doctor` errors "database file does not exist" or resolves the wrong DB | Run from a different cwd than the server with a relative `--database` | `doctor` resolves a running server's **absolute** path via `/health`; pass `--port`/`--url`, or give an absolute `--database` |
| `api call` with a hex colour silently drops/garbles it | Unquoted `#rrggbb` in `--variables` — `#` starts a shell comment, and bare hex isn't valid JSON | Quote hex as a JSON string and pass variables via a file: `--variables @vars.json` with `{"backgroundColor": "1f2937"}` (colours are stored without the leading `#`) |
