# Workflow: Edit a plan (as an agent)

Goal: modify a project's plan DAG (nodes/edges) programmatically against a
running Layercake instance, then have those changes appear live in any open
browser.

## Prerequisites

- A running server: `layercake serve` (defaults to `http://127.0.0.1:3000`).
- The project id you want to edit. List them:

  ```bash
  layercake api call --query '{ projects { id name } }'
  ```

## 1. Read the current plan DAG

Every plan-editing operation works on the plan DAG document. Fetch it first:

```bash
layercake api call --query '
  query($projectId: Int!) {
    getPlanDag(projectId: $projectId) {
      nodes { id nodeType position { x y } metadata config }
      edges { id source target }
    }
  }' --variables '{"projectId": 1}'
```

## 2. Apply an edit

Plan edits go through `updatePlanDag`, which replaces the DAG document with a
new version. Read → modify the returned structure → send it back. (Prefer
reading first so you preserve nodes/edges you are not changing.)

```bash
layercake api call --query '
  mutation($projectId: Int!, $input: PlanDagInput!) {
    updatePlanDag(projectId: $projectId, planDag: $input) {
      metadata { lastModified }
    }
  }' --variables @plan.json
```

Inspect the exact input shape with the schema:

```bash
layercake schema dump | grep -A20 "input PlanDagInput"
```

## 3. Appear as a collaborator while you work (optional)

So humans see the agent is active on the project, hold a presence session in a
separate process:

```bash
layercake api join --project 1 --name "Claude agent" --agent
```

The agent shows up in the multi-user presence UI with an "Agent" badge until
you stop it (Ctrl-C). See `layercake doc workflow join-as-collaborator`.

## Notes

- `layercake api call` targets a **running server** over HTTP. This is different
  from `layercake query`, which operates **directly on the database file** with
  no server. Use `api call` when a server is up and you want live UI updates.
- Mutations broadcast deltas to connected browsers automatically — no extra step
  is needed for the edit to appear live.
- **`metadata` needs a subfield selection**: query it as `metadata { label description }`
  and write it as `{ label description }`. A bare `metadata` errors with the
  generic GraphQL "must have a selection of subfields".
- Node **`id`s are user-choosable and idempotent** — pass your own string id and
  the server keeps it. Edges without ids get generated `edge_XXXX` ids.
- To join a node to its dataset PK, read `PlanDagNode.linkedDataSetId` rather than
  parsing `config` by hand.
- Node types, their config, and io: `layercake doc guide node-types`. The
  `graphJson` shape: `layercake doc guide graph-json`.
- Full API surface: `layercake schema dump`. Endpoints/headers: `layercake api info`.
