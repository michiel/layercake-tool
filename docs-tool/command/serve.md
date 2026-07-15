# Command: `layercake serve`

Start the Layercake server: the GraphQL API plus the embedded web UI, served
same-origin over HTTP.

## Usage

```bash
layercake serve                                # http://127.0.0.1:3000 (loopback)
layercake serve --port 8080                    # different port
layercake serve --host 0.0.0.0 --port 8080     # expose on the network (self-host)
layercake serve --open                          # open the web UI in a browser
layercake serve --database ./my.db              # use a specific database file
```

## Flags

- `--host <addr>` — bind address. Default `127.0.0.1` (local-only). Use
  `0.0.0.0` to accept connections from other machines.
- `--port <n>` — default `3000`.
- `--database <path>` — SQLite file. Default `layercake.db` (created if absent).
- `--cors-origin <url>` — allow a cross-origin frontend (dev only).
- `--open` — launch the default browser at the UI once ready.

## Security

There is **no authentication gate**. On the default loopback bind that's fine —
only local processes can reach it. If you use `--host 0.0.0.0`, the server is
reachable by anyone on the network; put it behind your own auth / reverse proxy.

## For agents

Once running, talk to it with `layercake api call` / `layercake api info`, or
hit `POST /graphql` directly. Probe readiness with `GET /health` (returns
`{"service","status","version"}`). Learn the API with `layercake schema dump`.
