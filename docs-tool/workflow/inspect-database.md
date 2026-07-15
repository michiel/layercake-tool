# Workflow: Inspect the database directly

For read-only inspection you can open the SQLite file directly, without going
through the server.

## 1. Find the file

```bash
layercake db info                 # path, absolute location, size
layercake db info --json          # machine-readable
```

Default path is `layercake.db` in the working directory (or whatever
`--database` a server was started with).

## 2. Read it

```bash
sqlite3 layercake.db '.tables'
sqlite3 layercake.db 'SELECT id, name FROM projects;'
```

The schema is standard SQLite; tables map to the domain entities (projects,
plans, graph data, datasets, etc.).

## When NOT to write directly

Reading is safe any time. **Do not write** to the file while a server is running
against it:
- it can conflict with the server's own writes, and
- direct writes do not broadcast to connected browsers, so the UI goes stale.

For any change, use the API (`layercake api call`) against the running server,
or `layercake query` when no server is running. See
`layercake doc command api` and `layercake doc command query`.

## Prefer the API for structured reads

If a server is up, `layercake api call` gives you the same data already shaped by
the domain model and is safer than reasoning about raw tables:

```bash
layercake api call --query '{ projects { id name } }'
```
