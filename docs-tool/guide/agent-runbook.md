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
