# TransformNode Transformation Support Plan

## Objectives
- Enable Plan DAG `TransformNode` instances to apply the same graph transformations that `graph_config` supports in export profiles (`sample/ref/plan.yaml`).
- Expose each transformation (and its parameters) in the Plan Visual Editor so users can configure them through the TransformNode edit dialog.
- Reuse the existing Rust transformation pipeline in `layercake-core/src/graph.rs` and `layercake-core/src/plan_execution.rs` for consistency and to avoid duplicate logic.

## Current State
- `layercake-core/src/graph.rs` already provides transformation primitives such as `modify_graph_limit_partition_depth`, `modify_graph_limit_partition_width`, `truncate_node_labels`, `insert_newlines_in_node_labels`, `truncate_edge_labels`, `insert_newlines_in_edge_labels`, `aggregate_edges`, and `invert_graph`.
- `apply_graph_transformations` in `layercake-core/src/plan_execution.rs` applies these based on `GraphConfig` populated from `graph_config` in YAML plans.
- TransformNodes in GraphQL/TypeScript (`layercake-core/src/graphql/types/plan_dag.rs`, `frontend/src/types/plan-dag.ts`) currently expose only a single `transformType` and a loosely-typed `transformConfig`, blocking parity with YAML capabilities and preventing multiple transformations.
- The Plan DAG executor (`layercake-core/src/pipeline/dag_executor.rs`) returns `Node type TransformNode not yet implemented`, so TransformNodes cannot execute today.
- The editor UI (`frontend/src/components/editors/PlanVisualEditor/forms/TransformNodeConfigForm.tsx`) offers a simple selector for four stub transforms and lacks parameter controls for the YAML-backed operations.

## Implementation Plan

### 1. Model Transformations Explicitly
- Introduce a canonical transformation descriptor shared across backend and frontend:
  - Rust: add `GraphTransformKind` enum and `GraphTransform` struct with `kind: GraphTransformKind` and typed `params` in `layercake-core/src/graphql/types/plan_dag.rs`. Derive `Serialize`, `Deserialize`, `SimpleObject`, and `InputObject`.
  - TypeScript: mirror as `export type GraphTransformKind = 'PartitionDepthLimit' | 'PartitionWidthLimit' | 'NodeLabelMaxLength' | 'NodeLabelInsertNewlines' | 'EdgeLabelMaxLength' | 'EdgeLabelInsertNewlines' | 'InvertGraph' | 'AggregateEdges';` and define `interface GraphTransform { kind: GraphTransformKind; params: GraphTransformParams; }`.
- Ensure JSON serialization remains stable for persistence in `plan_dag_nodes.config_json`.
- Include `generate_hierarchy` as its own transform type so users can trigger hierarchy generation separately from exports.
- Replace the current `transformType` / `transformConfig` fields with `transforms: Vec<GraphTransform>` in Rust and `GraphTransform[]` in TypeScript. Maintain backward compatibility by deserializing legacy configs into the new structure (single-entry shim) to avoid breaking existing drafts.
- **Status:** ✅ Rust GraphQL schema now exposes `GraphTransformKind`, `GraphTransform`, and `GraphTransformParams`, legacy config migrates into the new array, and `GraphConfig` defaults (including `aggregate_edges`) were added.

### 2. Map Descriptors to Existing GraphConfig
- Implement `impl GraphTransform { fn apply(&self, graph: &mut Graph) -> anyhow::Result<()> }` or convert all transforms into a `GraphConfig` via `TransformNodeConfig::to_graph_config(&self)`.
- Add conversion helper in Rust:
  ```rust
  impl TransformNodeConfig {
      pub fn to_graph_config(&self) -> crate::plan::GraphConfig {
          let mut cfg = crate::plan::GraphConfig::default();
          for transform in &self.transforms {
              match transform.kind {
                  GraphTransformKind::PartitionDepthLimit => {
                      cfg.max_partition_depth = transform.params.max_partition_depth.unwrap_or(0);
                  }
                  // ...remaining mappings
              }
          }
          cfg
      }
  }
  ```
  - Extend `crate::plan::GraphConfig` with `Default` impl to simplify zero initialization.
  - Support transformations that require boolean toggles (`InvertGraph`, `AggregateEdges`, `GenerateHierarchy`). Default booleans to `true` when the transform entry exists unless the user explicitly disables them.
- Confirm aggregate edges runs after label edits just like `apply_graph_transformations`. If we stick with the existing pipeline, ensure duplicates of `AggregateEdges` don't double-run (idempotent but log once).

### 3. Execute TransformNodes in the DAG
- Update `DagExecutor::execute_node` (`layercake-core/src/pipeline/dag_executor.rs`) to handle `"TransformNode"`:
  1. Resolve the incoming graph ID(s) using existing adjacency helpers (one input enforced by validation).
  2. Fetch the latest graph JSON for the upstream node (via `GraphBuilder`/`GraphService` once available) and deserialize to `Graph`.
  3. Convert the node's `TransformNodeConfig` to `GraphConfig` or apply transforms sequentially on a mutable graph using the new helper.
  4. Persist the transformed graph as a new graph record (never mutate upstream data) and emit execution metadata (similar to GraphNode builder). Store reference in an output `GraphNode` or update node execution state payload.
  5. Return meaningful execution errors (e.g., missing upstream graph, invalid parameter) and mark node status for downstream updates.
- Add unit tests for the conversion helpers and an integration test in `layercake-core/tests` validating that a serialized TransformNode config yields expected graph changes after execution.
- **Status:** ✅ `DagExecutor` now materializes upstream graphs, applies ordered transforms (with default edge aggregation), persists results as new graph records with metadata/source hashes, and unit tests cover default aggregation, disabled aggregation, and parameter validation.

### 4. Expand GraphQL API Surface
- Update GraphQL schema (`layercake-core/src/graphql/types/plan_dag.rs`) to expose the new transform list to the frontend. Regenerate schema artifacts if applicable.
- Adjust GraphQL resolvers/mutations reading `TransformNodeConfig` to accept the new payload shape and to down-convert legacy payloads.
- Ensure validation service (`layercake-core/src/services/validation.rs`) checks:
  - Non-empty `transforms` array.
  - Parameter ranges (e.g., depth/width > 0, max length within sensible bounds).
  - Enforce only one instance for mutually exclusive operations if needed (e.g., a single `InvertGraph` step).

### 5. Frontend Editor Enhancements
- Update `frontend/src/types/plan-dag.ts` and API adapters to use the new transform array.
- Refactor `TransformNodeConfigForm`:
  - Render a list of transformation steps with add/remove/reorder controls (use Mantine `Stack` + `Card` or `Accordion`).
  - Provide parameter inputs per transform type (number inputs for depth/width/length, switches for booleans) and keep ordering user-controlled so execution matches the configured sequence.
  - Validate locally before calling mutations: ensure required parameters filled, show inline errors.
  - Persist `aggregateEdges` as a checkbox rather than implicit flag to give users control; default it to checked when a new transform block is added.
- Update TransformNode badge/body (`frontend/src/components/editors/PlanVisualEditor/nodes/TransformNode.tsx` and `BaseNode` metadata) to display a concise summary (e.g., `Invert → Truncate Node Labels (32)`).
- Adjust DAG validation (`frontend/src/utils/planDagValidation.ts`) to recognize a node as configured when `transforms.length > 0`.
- **Status:** ✅ Frontend types now model `GraphTransform` arrays, node defaults inject aggregate edges, and the TransformNode editor supports ordered lists with add/remove/reorder controls, per-transform inputs, and validation of required parameters.

### 6. Persistence & Migration
- Write a migration script or runtime adapter that upgrades stored `TransformNode` configs when plans are loaded:
  - If legacy `transformType` exists, wrap it in `transforms = [{ kind, params }]`.
  - Insert an `AggregateEdges` transform (enabled) on upgrade if legacy configs relied on the implicit aggregate behavior, unless one already exists.
- Preserve step ordering during migration and execution so runtime transformations run in the configured order.
- Document the migration assumptions in `docs/historical/transformation-system-integration.md` and update `SPECIFICATION.md` with the new config schema.

### 7. Testing & Verification
- Rust:
  - Add unit tests for each `GraphTransformKind` to ensure parameter mapping into `GraphConfig` matches expectations.
  - Integration test that executes a mini DAG containing Input → Graph → Transform → Output and verifies adjacency plus transformed graph properties.
- Frontend:
  - Add React component tests (Vitest) covering form validation, parameter serialization, and reordering logic.
  - Run `npm run frontend:build` and `npm run backend:test` before submission.
- Manual smoke tests:
  - Create a plan in the UI, configure multiple transforms, save, reload, and verify the JSON stored in the DB matches the new schema.
  - Execute the DAG (once TransformNode execution implemented) and confirm the exported outputs reflect configured transforms.

### 8. Documentation & Developer Enablement
- Update `docs/historical/plan-dag-json-schema.md` and `SPECIFICATION.md` to document the new `transforms` array shape and supported operations.
- Add usage instructions to `docs/historical/transformation-system-integration.md` describing how TransformNodes map to YAML exports.
- Provide a migration note in `IMPLEMENTATION.md` near the Plan Visual Editor section on how to add new transform definitions.

## Decisions & Follow-ups
- `generate_hierarchy` remains available as a TransformNode operation with a boolean toggle.
- Transformations must execute strictly in the sequence configured in the node; ensure backend respects order and UI supports reordering.
- TransformNodes must never mutate upstream graph records; always create a new graph record to store the output.
- `aggregateEdges` is an optional transform; default to enabled when created but allow users to remove or disable it.
- Continue coordinating with product on migration copy strategies and any additional transform kinds before implementation begins.
