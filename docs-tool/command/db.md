# Command: `layercake db`

Manage and inspect the SQLite database file.

## Subcommands

```bash
layercake db info [--database layercake.db] [--json]   # file location + size
layercake db init [--database layercake.db]            # create + migrate a new DB
layercake db migrate <up|down|fresh> [--database ...]  # run migrations
```

## `db info`

Reports the database file path, absolute location, existence, and size. It is
**filesystem-only** (no DB connection), so it is safe to run while a server has
the database open.

```bash
layercake db info --json
# {"path":"layercake.db","absolute_path":"/…/layercake.db","exists":true,
#  "size_bytes":466944,"size_mb":"0.45"}
```

## Direct database access for agents

The database is a single SQLite file. For read-only inspection you can open it
directly (e.g. `sqlite3 layercake.db`) without going through the server. Get the
path with `layercake db info`.

Prefer the API (`layercake api call`) for anything a running server also touches
— writing directly to the file while a server is running can conflict with it
and will not broadcast changes to connected browsers. See
`layercake doc workflow inspect-database`.
