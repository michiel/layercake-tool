# Performance Review: Datasets & Graph Processing in DAG

## Current Pain Point
- Graph-heavy DAG runs degrade when graphs exceed ~100 nodes/edges. Symptoms: slow UI load, slow GraphQL responses (dataset graph fetch), and sluggish DAG execution (graph transformations/merges).

## Findings (E2E Path)
- Graph generation: Code/infra analysis produces large `Graph` structs serialized to JSON and stored per dataset.
- DAG execution: graph data is often deserialized/merged multiple times (code + infra merge, annotations, partition aggregation).
- GraphQL/API: datasets return full graph JSON inline; client renders entire graph even when not needed, causing large payloads.
- Frontend: graph visualizations do not paginate; state stores entire graph, leading to heavy diffing and layout thrash >100 elements.
- Storage: graph JSON is stored as a single blob; partial reads are impossible.

## Recommendations
1) **Graph Storage & Access**
   - Move graphs to chunked storage: store nodes/edges/layers separately (tables or JSON columns) and paginate on API.
   - Add lightweight summaries (counts, top layers, bounding box) and small previews for list queries.
   - Introduce projection parameters to GraphQL (e.g., `limit`, `filters`, `layer` selectors) and only hydrate what’s requested.

2) **Graph Processing Pipeline**
   - Avoid repeated full deserialization: cache deserialized graphs per dataset in DAG context; reuse in merges.
   - Add streaming merge/transform functions operating on iterators to reduce peak memory.
   - Defer heavy transforms (partition aggregation, layout) to optional/background steps or on-demand.

3) **DAG Execution Optimizations**
   - Add node/edge caps per stage with clear errors; early-prune layers not used downstream.
   - Parallelize independent steps where safe (analysis per file, infra per file) and batch DB writes for graphs/datasets.
   - Add metrics instrumentation (timings per stage, serialized sizes, allocation counts) to identify hotspots.

4) **API/GraphQL Layer**
   - Add separate fields: `graphSummary` (counts, layers) and `graphPage(limit, offset, layers)` returning slices.
   - Compress graph responses (gzip/deflate) and avoid embedding large graphs in list queries; require explicit fetch.
   - Add ETags/If-None-Match on graph fetches; cache summaries.

5) **Frontend**
   - Lazy-load graph data when user opens a visualization tab; show summary first.
   - Add virtualized rendering and layer filters; default to max 100 nodes until user opts to load more.
   - Keep graph state normalized (maps keyed by ID) and avoid storing duplicate graph JSON in component state.

## Implementation Plan
1) **Schema & Storage**
   - Add new tables/columns: `graph_nodes(dataset_id, ...)`, `graph_edges(dataset_id, ...)`, `graph_layers(dataset_id, ...)` or use JSONB columns with indexes if staying in SQLite/JSON.
   - Add migrations and a background job to backfill existing datasets by splitting stored graph JSON.
   - Keep `graph_json` for compatibility but mark as legacy; new API routes read from paged storage.

2) **Graph API & Services**
   - Extend `DataSetService` with: `get_graph_summary(dataset_id)`, `get_graph_page(dataset_id, limit, offset, layers?)`.
   - Refactor graph merge to accept iterators and optional filters; add caching layer in DAG execution context.
   - Add config-driven limits (max nodes/edges per request) and meaningful errors when exceeded.

3) **GraphQL Changes**
   - Add types `GraphSummary { nodeCount, edgeCount, layers }` and `GraphPage { nodes, edges, layers, hasMore }`.
   - Update dataset queries to return summary only; add dedicated query for graph pages with filters.
   - Use response compression and ETags at the server layer.

4) **Frontend Updates**
   - Update dataset/code analysis pages to fetch summaries first; load graph pages on demand with pagination and layer filters.
   - Implement virtualized list/renderer for nodes/edges; add “Load more” and “Max nodes per view” guardrails.
   - Cache fetched pages client-side keyed by dataset/layer to avoid refetch on tab switches.

5) **Performance Instrumentation**
   - Add tracing spans and metrics around graph serialization/deserialization, merges, DB reads, and GraphQL resolvers.
   - Surface timings in dev logs and optionally in UI diagnostics for large graphs.

6) **Rollout & Migration**
   - Behind a feature flag, dual-write graphs to both legacy JSON and new paged storage; validate parity on read.
   - Provide a migration tool/CLI to backfill and verify checksums (node/edge counts) per dataset.
   - Once stable, switch defaults to paged storage and remove large-graph responses from list queries.
