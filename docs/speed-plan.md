# DAG Speed & In-Memory Execution Plan

## Current Bottlenecks
- `DagExecutor::execute_node` rebuilds upstream graphs from the database for every Merge/Graph/Transform/Filter run (`layercake-core/src/pipeline/dag_executor.rs:91`, `layercake-core/src/pipeline/dag_executor.rs:107`, `layercake-core/src/pipeline/dag_executor.rs:233`). Even when nothing changed, each node issues several `SELECT` queries (plan nodes, graphs, layers, edges) and then rewrites the destination tables, so a DAG with a few hundred nodes results in thousands of round-trips.
- Persisting node results performs one insert per node/edge (`layercake-core/src/pipeline/dag_executor.rs:709`, `layercake-core/src/pipeline/graph_builder.rs:568`). Every execution truncates `graph_nodes`/`graph_edges` and reinserts rows individually, which is the dominant cost once graphs exceed ~100 elements.
- Dataset ingestion still writes every CSV row into `dataset_rows` sequentially even though downstream builders only consume `graph_json` (`layercake-core/src/pipeline/dataset_importer.rs:150`, `layercake-core/src/pipeline/dataset_importer.rs:173`). Importing a 500-row file therefore issues 500 inserts plus later JSON parsing, doubling both DB time and memory.
- Graph construction repeatedly queries upstream metadata (`layercake-core/src/pipeline/graph_builder.rs:68`, `layercake-core/src/pipeline/graph_builder.rs:289`) and does not cache `data_sets` or prior graph materializations, so wide DAGs pull the same blobs from disk and deserialize them for each consumer.

## Recommended Improvements

### 1. Add an in-memory DAG runtime (priority: high)
1. Build a `DagExecutionContext` that, before execution, loads all relevant plan nodes, edges, `data_sets` records, and previously materialized graphs into hash maps. The context should expose `fn graph(&node_id) -> Option<&Graph>` so downstream nodes never hit the database unless explicitly asked to persist.
2. Update `DagExecutor::execute_node` to read and write through this context:
   - DataSet nodes deserialize their `data_sets.graph_json` once and stash the `Graph` struct in memory.
   - Merge/Graph/Transform/Filter nodes consume the cached upstream `Graph` values instead of calling `GraphService::build_graph_from_dag_graph`.
   - Artefact/export nodes can request persistence, which converts the cached `Graph` into DB rows at the very end.
3. Expose a feature flag (e.g., `PIPELINE_IN_MEMORY=true`) to keep the current DB-first behavior available while rolling out the new path.

Expected impact: A 200-node DAG would drop from thousands of queries to a handful (initial preload + final persistence), matching the requirement that “datasets should be loaded into memory and processed there.”

### 2. Batch persistence when disk writes are required
1. Replace the per-row inserts in `persist_graph_contents` with `graph_nodes::Entity::insert_many` / `graph_edges::Entity::insert_many` using 500-row chunks to keep statements under DB limits (`layercake-core/src/pipeline/dag_executor.rs:709`).
2. Apply the same pattern inside `GraphBuilder::build_graph_from_data_sets` and `MergeBuilder::merge_data_from_sources` so we pay one insert per chunk instead of one per record.
3. Wrap each node’s persistence in a transaction to avoid interleaved truncates/inserts when multiple nodes run concurrently later.

This preserves the relational tables for UI consumers but removes the O(n) statement count that currently dominates run time.

### 3. Stop duplicating dataset rows on import
1. For CSV/TSV nodes, construct the `graph_json` payload directly (similar to `services/source_processing.rs`) and store it on the dataset record, skipping `dataset_rows` entirely when the pipeline does not need normalized rows (`layercake-core/src/pipeline/dataset_importer.rs:150`).
2. If normalized rows are still useful for future analytics, gate their creation behind a flag and insert them via `insert_many` batches so 500-row files only cost a handful of statements.
3. Record lightweight metadata (row counts, column info, checksum) on the dataset to keep progress reporting intact.

Eliminating the redundant insert path removes hundreds of queries before the DAG even starts.

### 4. Cache upstream descriptors & hashes
1. When building graphs, preload all `plan_dag_nodes` and `data_sets` referenced by the plan into maps keyed by node id and dataset id (`layercake-core/src/pipeline/graph_builder.rs:68`). Pass these maps into `GraphBuilder`/`MergeBuilder` so each upstream node is deserialized once per execution.
2. Extend the existing `source_hash` to include the actual dataset content checksum (already available once everything is in memory). Store it alongside the cached `Graph` so we can skip recomputing nodes when nothing changed.
3. Persist the cached graph as JSON (or a compressed blob) in `graphs.metadata` for rapid reload; only hit `graph_nodes`/`graph_edges` when the UI asks for row-level data.

### 5. Instrument and guardrail the rollout
1. Capture execution metrics per node (query count, elapsed time, rows written) so we can prove the benefits once the in-memory path ships.
2. Add smoke tests that execute sample DAGs (100+ nodes) under both the current DB path and the in-memory path to ensure parity before removing the legacy flow.

## Progress
- ✅ Introduced `DagExecutionContext` behind the `PIPELINE_IN_MEMORY` flag. DagExecutor entry points now create a shared context so DataSet, Merge, Graph, Transform, and Filter nodes reuse already-materialized graphs instead of querying `graph_nodes`/`graph_edges` every time. Dataset reference nodes also keep parsed `graph_json` payloads in memory for reuse.
- ⚙️ While the flag defaults to off, enabling it removes thousands of redundant queries for DAGs with hundreds of nodes without changing persisted results, establishing the framework needed for the remaining optimizations.
- ✅ Batched all graph persistence writes. `persist_graph_contents`, `GraphBuilder`, and `MergeBuilder` now wrap node/edge deletes plus inserts in a single transaction, performing chunked `insert_many` writes (500/item batches) via the new `persist_utils` helpers and reusing `insert_layers_to_db` against the same connection. This eliminates the per-row round-trips that previously dominated DAG execution time.
- ✅ Dataset ingestion no longer floods `dataset_rows` by default. `DatasourceImporter` skips row materialization unless `PIPELINE_PERSIST_DATASET_ROWS=true`, and when enabled it writes rows in 500-record batches. GraphBuilder/MergeBuilder also cache dataset descriptors and compute hashes over actual `graph_json`/`source_hash` content so upstream changes are detected precisely without extra queries.

## Next Steps
1. Expand the new `DagExecutionContext` so GraphBuilder/MergeBuilder operate directly on cached datasets/graphs (no DB hydrations) and validate the behavior with representative DAG fixtures.
2. Add metric emission (per-node counters/timers) alongside the new spans so runs can be aggregated outside of tracing.
3. Once the new runtime is battle-tested, remove the eager DB materialization path or keep it only for backward compatibility with legacy tooling.
