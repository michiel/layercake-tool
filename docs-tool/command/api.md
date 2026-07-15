# Command: `layercake api`

Talk to a **running** Layercake server over HTTP. This is the path for agents
driving a live instance (changes appear in the UI). Contrast with
`layercake query`, which hits the database file directly with no server.

## Subcommands

```bash
layercake api info [--url | --host --port] [--json]   # endpoints + headers
layercake api call --query '<gql>' [--variables ...]  # run a GraphQL operation
layercake api join --project N --name "…" --agent     # presence (see below)
```

## `api info`

Prints the endpoints and the session header an agent needs:

```bash
layercake api info --json
```

Fields: `graphql` (POST), `graphql_ws`, `health`, `collaboration_ws`,
`session_header` (`x-layercake-session`), `database`.

## `api call`

POST a GraphQL query or mutation and print the JSON response.

```bash
# Query
layercake api call --query '{ projects { id name } }'

# With variables (inline JSON or @file)
layercake api call \
  --query 'query($id:Int!){ project(id:$id){ name } }' \
  --variables '{"id":1}'
layercake api call --query "$(cat op.graphql)" --variables @vars.json

# Against a remote / non-default instance
layercake api call --url http://host:8080 --query '{ __typename }'

# Tag the session (shows up in server-side session correlation)
layercake api call --session my-agent --query '{ __typename }'
```

Discover the operations and argument shapes with `layercake schema dump`.

## `api join`

Hold a presence session so the agent appears as a collaborator in the
multi-user UI. See `layercake doc workflow join-as-collaborator`.

## `api call` vs `query`

| | `layercake api call` | `layercake query` |
|---|---|---|
| Talks to | a running server (HTTP) | the DB file directly |
| Server needed | yes | no |
| Live UI updates | yes | no |
| Use when | driving a live instance | offline scripting |
