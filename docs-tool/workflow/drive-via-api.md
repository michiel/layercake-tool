# Workflow: Drive Layercake via the API

Connect to a running instance and run GraphQL queries/mutations.

## Start / find the server

```bash
layercake serve                      # http://127.0.0.1:3000 (loopback; the default —
                                     #   your project may run on another port, e.g. 3001.
                                     #   `layercake api info` tells you the live one)
layercake serve --host 0.0.0.0 --port 8080   # expose on a network
layercake api info                   # print endpoints + headers for the running server
```

Endpoints:
- `POST /graphql` — queries & mutations
- `GET  /graphql/ws` — subscriptions (WebSocket, `graphql-ws` protocol)
- `GET  /health` — readiness probe: `{"service","status","version"}`
- `GET  /ws/collaboration?project_id=N` — multi-user presence (see join workflow)

Requests may include an `x-layercake-session: <id>` header to correlate a
session. There is no auth gate on a loopback bind; treat `--host 0.0.0.0` as
network-exposed and put it behind your own auth/reverse-proxy.

## Run an operation

```bash
# Simplest: via the CLI helper (adds headers, targets the right URL)
layercake api call --query '{ projects { id name } }'

# With variables
layercake api call \
  --query 'query($id:Int!){ project(id:$id){ name } }' \
  --variables '{"id":1}'

# Or raw curl
curl -s http://127.0.0.1:3000/graphql \
  -H 'content-type: application/json' \
  -H 'x-layercake-session: my-agent' \
  -d '{"query":"{ projects { id name } }"}'
```

## Learn the full surface

```bash
layercake schema dump            # GraphQL SDL (works offline, no server needed)
layercake schema dump --json     # introspection JSON
```

## `api call` vs `query`

- `layercake api call` → HTTP to a **running server**; changes appear live in the UI.
- `layercake query` → **direct database** access, **no server**; for offline scripting.
