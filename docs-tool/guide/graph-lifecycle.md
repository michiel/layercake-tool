# Guide: Graph lifecycle

How a dataset becomes exported diagram bytes, and where a `graph_data` row is
created, invalidated, and pruned along the way. Read this when a diagram is
empty, a graph looks stale, or you're cleaning up orphans.

## The pipeline, stage by stage

```
dataset  →  DataSetNode  →  GraphNode / MergeNode / TransformNode  →  graph_data row  →  GraphArtefactNode  →  exported bytes
```

| Stage | What it is | Graph state |
|-------|------------|-------------|
| **dataset** | Imported rows (nodes/edges/layers), addressed by PK. | No graph yet — raw source data. |
| **DataSetNode** | A DAG node that references a dataset (`linkedDataSetId`). | Names a source; still no computed graph. |
| **GraphNode / MergeNode / TransformNode** | Compute nodes: build/merge/transform a graph from upstream. | Executing one **creates or refreshes** a `graph_data` row keyed by `(project_id, dag_node_id)`. |
| **`graph_data` row** | The materialised graph for that DAG node — the computed-graph cache. | Exists once its producing node has executed. |
| **GraphArtefactNode** (also Tree/Sequence artefacts) | Render nodes: read a `graph_data` row and render a diagram. | Reads, does not create, graph state. |
| **exported bytes** | The rendered `.mmd`/`.puml`/… written to `outputPath`. | Terminal output. |

## Created / invalidated / pruned

- **Created / refreshed** — when a compute node (Graph/Merge/Transform) executes,
  its `graph_data` row is written (upsert on `dag_node_id`). Re-running replaces it.
- **Invalidated** — editing a compute node or its upstream leaves the old
  `graph_data` row in place until the node re-executes; a render reading a stale
  row is the usual cause of a "green but wrong/empty" diagram. Re-execute the
  producing node.
- **Origin rewritten** — `renamePlanDagNode` rewrites `graph_data.dag_node_id`
  so a computed graph follows its node's new id (no orphan created by a rename).
- **Pruned** — a `graph_data` row whose `dag_node_id` no longer matches any node
  in the plan is an **orphan** (e.g. the producing node was deleted). Clean these
  with the `pruneOrphanedGraphs(projectId)` mutation.

## Diagnosing

- **Empty or stale diagram** → run `layercake doctor --project <id>`. It flags
  missing/stale `graph_data` and orphaned graphs, and resolves the DB from a
  running server so it works regardless of cwd.
- **Orphaned graphs after deleting nodes** → `pruneOrphanedGraphs(projectId)`.
- **Which node produced a graph** → `graph_data.dag_node_id` is the producing
  DAG node's id.
