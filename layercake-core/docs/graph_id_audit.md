## Legacy `graph_id` references audit (pending unified migration)

This note lists high-traffic areas still tied to legacy `graphs`/`graph_nodes`/`graph_edges` IDs that must be adapted or bridged to unified `graph_data` during migration.

### High-risk paths
- `services/graph_edit_service.rs`, `graph_edit_applicator.rs`, `graph_edit_service_test.rs`, `graph_edit_replay_test.rs`: all edit replay flows depend on `graph_id` and legacy tables.
- `graph_service.rs`, `graph_analysis_service.rs`, `graph_builder.rs`, `merge_builder.rs`, `layer_operations.rs`, `persist_utils.rs`, `dag_executor` legacy path: execution, validation, and layer handling tied to legacy graphs.
- GraphQL mutations/queries/types (`graphql/mutations/graph.rs`, `graph_edit.rs`, `queries/mod.rs`, `types/graph*.rs`, `types/graph_edit.rs`, `types/layer.rs`, plan_dag filter/metadata types) expose legacy IDs to clients.
- MCP tools/resources (`mcp/tools/graph_edit.rs`, `mcp/tools/graph_data.rs`, `mcp/tools/analysis.rs`, `mcp/resources.rs`, `mcp/server.rs`) accept `graph_id`.
- Console/CLI flows (`console/context.rs`, `console/commands.rs`) read legacy graphs.

### Medium-risk / data model hooks
- `app_context` ops (`graph_operations.rs`, `preview_operations.rs`, `plan_dag_operations.rs`) rely on legacy graph IDs when manipulating nodes/edges/layers.
- `migrations/m20251010_000002_create_graph_edits.rs` (schema) and tests that insert edits against legacy graph IDs.
- `database/entities/*` (`graph_edits`, `graph_nodes`, `graph_edges`, `graph_layers`) remain active; user_sessions has `layercake_graph_id`.

### Guidance for migration
1) Introduce compatibility layer: GraphData -> legacy graph view (for GraphQL/MCP) or route clients to unified IDs via new APIs.
2) Update DAG executor to exclusively emit `graph_data` IDs and stop writing legacy tables; bridge reads until clients switch.
3) Adapt edit replay to operate on `graph_data` (or provide translation layer) and migrate edit records with offsets.
4) Update GraphQL/MCP schemas to surface `graphDataId` and deprecate `graphId`.
5) Clean up console/CLI to read from `graph_data`.
