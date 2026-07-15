# Command: `layercake doctor`

Scan a project for structural health problems — the classes of issue that
otherwise only surface as a silently empty diagram or a stale computed graph.

## Usage

```bash
layercake doctor --project 36                 # resolves the DB from a running server
layercake doctor --project 36 --port 3001     # …on a non-default port
layercake doctor --project 36 --database ./my.db   # or point at the file directly
layercake doctor --project 36 --json          # machine-readable
layercake doctor --project 36 --strict        # exit non-zero on warnings too (CI)
```

If `--database` is omitted, the DB path is resolved from a running server's
`/health` (via `--host`/`--port`/`--url`), so `doctor` works from any directory
when the server is up — no need to be in the same folder as `layercake.db`.

Exits non-zero if any **error**-severity finding is present; with `--strict`,
any **warning** also fails.

## Checks

- **node-missing-dataset** (error) — a plan DAG node's `config.dataSetId` points
  at a dataset that no longer exists.
- **orphaned-computed-graph** (warning) — a computed `graph_data` row whose
  originating DAG node is gone.
- **empty-sequence-context** / **sequence-context-warning** (warning) — a
  persisted sequence context with no participants, or carrying build warnings.
- **story-sequence-unresolved** (warning) — a story sequence references an edge
  that doesn't resolve against its enabled datasets.
- **story-empty** (error) — a story has sequences but resolves to zero
  participants (its diagram will be blank).
- **story-build-failed** (error) — the story context builder errored.

## For agents

Run this first when a diagram renders empty or a graph looks stale. It
operationalises the manual diagnosis (inspecting `sequences[0].steps.length`,
matching hash ids to datasets, hunting orphaned graphs) into one command.
