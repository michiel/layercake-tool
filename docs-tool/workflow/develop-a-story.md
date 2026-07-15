# Workflow: Develop a story (agent-authored sequence diagrams)

Stories turn graph nodes and edges into **sequence diagrams with commentary** —
"stories of sequences" built from datasets. Having an agent construct stories is
a core use case: you read a dataset's graph, choose an ordered path of edges,
attach commentary, and render a sequence diagram.

## Sourcing: datasets vs computed graphs

A story sources its edges from either:
- **`enabledDatasetIds`** — raw dataset PKs (`data_sets`), or
- **`enabledGraphIds`** — computed graph ids (`graph_data`, a GraphNode's
  output). Use this to build a story from a *merged* graph rather than raw
  datasets.

A `SequenceEdgeRef.datasetId` may be either a dataset id OR a computed graph id;
whatever you enabled on the story. (Frontend picker for computed graphs is a
follow-up — set `enabledGraphIds` via the API for now.)

## Mental model

- A **Story** is a curated collection of **Sequences** (belongs to a project,
  draws from one or more datasets).
- A **Sequence** is an **ordered list of edge references** (`edgeOrder`). Each
  edge becomes a sequence-diagram message `source ->> target: <edge label>`.
- Each edge reference (`SequenceEdgeRef`) is one **step + commentary**:
  `{ datasetId, edgeId, note?, notePosition? }`. The `note` is the commentary;
  `notePosition` is `Source | Target | Both`.
- **Participants (lifelines) are derived** from the edges' endpoints — you do not
  author them.
- Rendering happens through the plan DAG: a **StoryNode** (`{ storyId }`) feeds a
  **SequenceArtefactNode** (`{ renderTarget, outputPath, renderConfig }`) which
  produces Mermaid or PlantUML.

All operations below run against a running server via `layercake api call`.
Confirm shapes with `layercake schema dump` (e.g. `grep -A15 "input CreateStoryInput"`).

## 1. Find datasets and their edges

A sequence references edges by `edgeId` within a `datasetId`. Read the graph to
pick edges and learn their ids/labels:

```bash
layercake api call --query '{ project(id: 1) { id name } }'
# Inspect a dataset's graph (nodes + edges) to choose an ordered path of edges.
# Use schema dump to find the exact query for graph nodes/edges in your version.
layercake schema dump | grep -iE "graphNodes|graphEdges|edges\("
```

## 2. Create the story

```bash
layercake api call --query '
  mutation($input: CreateStoryInput!) {
    createStory(input: $input) { id name }
  }' --variables '{
    "input": {
      "projectId": 1,
      "name": "Checkout flow",
      "description": "How an order moves through the system",
      "enabledDatasetIds": [10]
    }
  }'
```

## 3. Create sequences (the ordered steps + commentary)

`edgeOrder` is the heart of the story: the **order is the message order**, and
`note` is the commentary shown on the diagram.

```bash
layercake api call --query '
  mutation($input: CreateSequenceInput!) {
    createSequence(input: $input) { id name edgeCount }
  }' --variables '{
    "input": {
      "storyId": 42,
      "name": "Happy path",
      "enabledDatasetIds": [10],
      "edgeOrder": [
        { "datasetId": 10, "edgeId": "e-user-cart",   "note": "User adds item",        "notePosition": "Source" },
        { "datasetId": 10, "edgeId": "e-cart-order",   "note": "Cart becomes an order", "notePosition": "Both" },
        { "datasetId": 10, "edgeId": "e-order-payment","note": "Charge the card",        "notePosition": "Target" }
      ]
    }
  }'
```

Repeat `createSequence` for each sequence in the story. Reorder or revise later
with `updateSequence(id, input)`.

## 4. Wire the render nodes into the plan DAG

Add a `StoryNode` referencing the story, connected downstream to a
`SequenceArtefactNode`. Read the current DAG, add the two nodes + the edge, and
send it back with `updatePlanDag` (see `layercake doc workflow edit-a-plan`).

Key config:
- StoryNode config: `{ "storyId": 42 }`
- SequenceArtefactNode config: `{ "renderTarget": "MermaidSequence", "outputPath": "checkout.mmd", "renderConfig": { "showNotes": true } }`

**Important:** commentary (`note`) only appears in the output when the artefact's
`renderConfig.showNotes` is `true`. Set the notes on the edges AND enable
`showNotes`.

The SequenceArtefactNode must be connected downstream of a StoryNode, or export
will error.

## 5. Render the diagram

Execute the plan (so the StoryNode builds its context), then export the artefact
node to get the diagram text:

```bash
layercake api call --query '
  mutation($projectId: Int!, $nodeId: String!) {
    exportNodeOutput(projectId: $projectId, nodeId: $nodeId) {
      success filename mimeType content   # content is base64
    }
  }' --variables '{"projectId": 1, "nodeId": "<sequence-artefact-node-id>"}'
```

Decode `content` (base64) to get the Mermaid/PlantUML source, e.g.:

```
sequenceDiagram
  participant user as "User"
  participant cart as "Cart"
  Note over user,payment: Happy path
  Note over user: User adds item
  user->>cart: add item
  ...
```

## Preview without rendering

To inspect the resolved participants/steps/warnings (or render the diagram
yourself client-side) without the Handlebars template:

```bash
layercake api call --query '
  query($p:Int!,$s:Int!){ previewStoryContext(projectId:$p, storyId:$s) }' \
  --variables '{"p":1,"s":42}'
```

Returns the `SequenceStoryContext` as JSON (participants, sequences[].steps,
and any build `warnings`). If `steps` is empty or `warnings` is non-empty, the
diagram will be blank — run `layercake doctor --project <id>` to see why.

## Summary of the ops, in order

`createStory` → `createSequence` (×N) → `updatePlanDag` (add StoryNode +
SequenceArtefactNode) → run the plan → `exportNodeOutput`.

Node types: `DataSetNode` (source graphs) → `StoryNode` (`{storyId}`) →
`SequenceArtefactNode` (`{renderTarget, outputPath, renderConfig}`).

## Export/import a whole story

```bash
# Export a story as JSON or CSV (base64 content back)
layercake api call --query '
  mutation($id: Int!) { exportStory(storyId: $id, format: JSON) { filename content } }' \
  --variables '{"id": 42}'
```

Use `importStory(projectId, format, content)` to load one back.
