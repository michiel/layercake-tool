# Workflow: Join as a collaborator (agent presence)

Make an agent appear as a distinct user in the multi-user UI — with an avatar,
an "Agent" badge, and a live cursor — just like a human browser tab.

## How presence works

The collaboration protocol is client-driven: a client opens the collaboration
WebSocket for a project and announces itself with a `join_session` message
carrying `userId`, `userName`, `avatarColor`, and `isAgent`. The server
broadcasts that presence to every other connected client. Marking `isAgent`
makes the UI render the user with a robot icon + "Agent" badge.

## Hold a presence session

```bash
layercake api join --project 1 --name "Claude agent" --agent
```

Options:
- `--project N` (required) — the project id to join.
- `--name "…"` (required) — display name shown to others.
- `--agent` — mark as an agent (robot icon + Agent badge). Omit to appear as a
  plain user.
- `--id <stable-id>` — reuse a stable user id across reconnects (defaults to
  `agent-<project>-<pid>`).
- `--color "#7c3aed"` — avatar colour.
- `--url` / `--host` / `--port` — target a specific instance.

The command holds the connection open (heartbeating) and prints presence/errors
it receives on stdout. Press Ctrl-C to leave; the UI then shows the agent going
offline.

## Typical pattern

Run `api join` in one process while doing work (edits via `api call`) in
another, so humans watching the project see the agent as an active collaborator
for the duration of the task:

```bash
# terminal 1 — presence for the whole session
layercake api join --project 1 --name "Claude agent" --agent

# terminal 2 — do the work
layercake api call --query 'mutation { … }'
```

## Notes

- Presence identity (this WS session) is currently separate from the identity on
  GraphQL mutations (`x-layercake-session`). The UI shows the agent as *present*;
  attributing individual mutations to that same agent identity is a possible
  future enhancement.
- Humans (browser tabs) omit `isAgent` and render normally — this is fully
  backward-compatible.
