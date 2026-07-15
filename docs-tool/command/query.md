# Command: `layercake query`

Run one-shot data operations **directly against the database file** — no server
required. Use this for offline scripting. To drive a *running* server (with live
UI updates), use `layercake api call` instead.

## Usage

```bash
layercake query --entity <entity> --action <action> [options]
```

- `--entity` — one of: `datasets`, `plans`, `nodes`, `edges`, `exports`,
  `schema`, `analysis`, `annotations`.
- `--action` — one of: `list`, `get`, `create`, `update`, `delete`, `move`,
  `download`, `traverse`, `batch`, `search`, `clone`.
- `--project <id>` / `--plan <id>` — scope (as required by the entity).
- `--payload-json '<json>'` or `--payload-file <path>` — request body.
- `--output-file <path>` — where to write export downloads.
- `--pretty` — pretty-print JSON output.
- `--database <path>` — DB file (default `layercake.db`).
- `--dry-run` — validate without executing.

## Examples

```bash
layercake query --entity plans --action list --project 1 --pretty
layercake query --entity nodes --action list --project 1 --plan 2 --pretty
layercake query --entity nodes --action create --project 1 --plan 2 \
  --payload-json '{"nodeType":"GraphNode", …}'
```

Inspect the payload schema for an operation:

```bash
layercake query --entity schema --action get --payload-json '{"target":"nodes.create"}'
```

## `query` vs `api call`

| | `layercake query` | `layercake api call` |
|---|---|---|
| Talks to | the DB file directly | a running server (HTTP) |
| Server needed | no | yes |
| Live UI updates | no | yes |
| Use when | offline scripting | driving a live instance |

If a server is running against the same DB file, prefer `api call` — writing to
the file underneath a running server can conflict and won't update the UI.
