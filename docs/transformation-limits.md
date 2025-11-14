# Transformation Limits Fix Plan

## Symptoms
- The *Partition Depth Limit* and *Partition Width Limit* transform options in the graph pipeline have no observable effect when running real plans.
- Users report that even with `max_partition_depth` / `max_partition_width` set in `GraphTransformKind::PartitionDepthLimit` and `GraphTransformKind::PartitionWidthLimit`, graphs keep their original hierarchy and node counts after a Transform node executes.

## Root Cause Analysis
1. **Transforms depend on partition metadata that is usually missing**  
   The implementations at `layercake-core/src/graph.rs:375-569` operate entirely on nodes that satisfy `is_partition == true` and maintain a `belongs_to` tree. They rely on `Graph::get_root_nodes()` / `Graph::get_children()` which filter on these fields.
2. **GraphBuilder / MergeBuilder do not populate `is_partition` / `belongs_to`**  
   After a dataset import/merge, most nodes enter the pipeline with `is_partition = false` and `belongs_to = None`, because the upstream CSV/JSON rarely carries hierarchical metadata and GraphBuilder (`layercake-core/src/pipeline/graph_builder.rs`) simply copies what is present. As a result, `get_root_nodes()` returns an empty list and both limit functions bail out without touching the graph.
3. **No diagnostics**  
   The transforms silently return `Ok(())` when there are no qualifying nodes, so users do not know that their width/depth settings were ignored.

## Fix Plan
1. **Detect missing partition metadata early**
   - In `Graph::modify_graph_limit_partition_depth` / `_width`, detect when `get_root_nodes()` is empty or when an entire level lacks `is_partition` data.
   - Emit a structured warning so users know partition metadata is missing. This also helps during future debugging.
2. **Auto-derive a partition tree when metadata is absent**
   - ✅ `Graph::ensure_partition_hierarchy` now synthesizes a hierarchy when no `is_partition` data exists, and both limit functions invoke it automatically. The transforms and CLI path also call the helper so users get automatic fixes plus informational logs when synthetic metadata is used.
3. **Refine trimming algorithms after hierarchy injection**
   - Depth: verify that aggregation preserves parent-child relationships by reassigning `belongs_to` and reusing or regenerating edges so downstream nodes truly collapse into the target depth.
   - Width: when generating `agg_*` nodes, ensure IDs are unique per level and that new nodes inherit the parent’s `belongs_to`. Keep partition children untouched while only collapsing non-partition siblings beyond `max_width`.
4. **Surface configuration-level safeguards**
   - In `TransformNodeConfig::apply_to` (`layercake-core/src/graphql/types/plan_dag/transforms.rs:70-110`), log a warning when the helper reports “no partitions found” so GraphQL clients understand why nothing happened.
   - Update `plan_execution::apply_graph_transformations` similarly when CLI users set `max_partition_depth/width`.
5. **Documentation updates**
   - Extend user docs / plan builder guidance to clarify that, after this change, depth/width limits will auto-generate hierarchy when metadata is missing, but supplying explicit partitions still yields the best fidelity.

## Testing Plan
1. **Unit tests on Graph helpers**
   - Add tests in `layercake-core/src/graph.rs` that feed a graph with zero partition metadata, run the new `ensure_partition_hierarchy`, and assert that partition nodes / belongs_to links are synthesized.
   - Extend `test_modify_graph_limit_partition_depth` and `_width` to cover both metadata-present and metadata-absent scenarios (created via helper) ensuring node counts and hierarchy depths change as expected.
2. **Transform application tests**
   - Introduce tests for `TransformNodeConfig::apply_to` that construct a `Graph` without partition metadata, run `PartitionDepthLimit` / `PartitionWidthLimit`, and verify the resulting graph shrinks appropriately.
3. **End-to-end plan test**
   - Create an integration test (e.g., under `layercake-core/tests/`) that loads a small plan with a Transform node using the limit settings, executes the DagExecutor, and asserts the persisted graph (via GraphService) contains aggregated nodes.
4. **Regression safeguards**
   - Add snapshot-based tests or detectors to ensure future changes that remove `is_partition` metadata do not silently break the transforms again.

Executing this plan will restore the limit/width transformations, provide clear warnings when hierarchy data is missing, and add coverage to prevent regressions.
