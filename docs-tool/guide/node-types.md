# Guide: Plan DAG node types

A plan DAG is a directed graph of nodes. Each node has a `nodeType`, an opaque
hash `id` (e.g. `dataset_766f8bef…`), a `metadata { label description }`, and a
`config` (a JSON string). Data flows along edges from sources → transforms →
artefacts.

Inspect the exact config input for any type with:

```bash
layercake schema dump | grep -A15 "input <Type>ConfigInput"
# e.g. input SequenceArtefactNodeConfigInput
```

To join a node back to its dataset PK, read `PlanDagNode.linkedDataSetId`
(computed from `config.dataSetId`) rather than parsing config by hand.

## The 10 node types

### Source nodes

- **DataSetNode** — brings a dataset's graph into the plan. `config.dataSetId`
  links to the dataset PK. Deleting the node does NOT delete the dataset.
- **GraphNode** — a computed graph produced by upstream processing; the unit
  that gets executed into `graph_data`.

### Transform / combine nodes

- **TransformNode** — applies graph transforms (see `config`).
- **FilterNode** — filters nodes/edges by criteria.
- **MergeNode** — merges multiple upstream graphs into one.

### Artefact (output) nodes — render during export, not eager execution

- **GraphArtefactNode** — renders a graph to an output (e.g. Mermaid/PlantUML/
  DOT). `config.renderTarget`, `config.outputPath`, `config.renderConfig`.
- **TreeArtefactNode** — renders a hierarchy/tree output.
- **ProjectionNode** — a projection artefact.

### Story / sequence nodes

- **StoryNode** — references a Story entity (`config.storyId`). On execution it
  builds the story's sequence context. **Important:** the context builder uses
  the Story's `enabledDatasetIds` from the DB, not the DAG upstream — see
  `layercake doc workflow develop-a-story`. Wire DataSetNodes upstream of it.
- **SequenceArtefactNode** — renders the story's sequences to a Mermaid or
  PlantUML sequence diagram. Must be connected downstream of a StoryNode.
  Config: `renderTarget` (`MermaidSequence`|`PlantUmlSequence`), `outputPath`,
  `useStoryLayers`, and `renderConfig` (see the render-config guide).

## Editing plan DAG nodes

- Node `id`s are user-choosable and idempotent in `updatePlanDag` (pass your own
  string id and the server keeps it). Edges without ids get `edge_XXXX`.
- `metadata` needs a subfield selection when queried: `metadata { label description }`.
- See `layercake doc workflow edit-a-plan` for the read-modify-write loop.
