# Query Builder Integration Plan

## Goal
- Filter nodes execute a single query powered by [`react-querybuilder`], replacing the legacy “preset + multi-filter stack” UI entirely.
- Each filter node persists a canonical `QueryFilterConfig` JSON payload so DAG executions remain reproducible and auditable.
- The backend compiles that JSON into SQL/SeaORM predicates that run against `graph_nodes`, `graph_edges`, and `graph_layers` before materializing filtered graphs (with configurable pruning).

## Feasibility Snapshot
- **Frontend**: `react-querybuilder` ships with the necessary primitives; we wrap it with Mantine controls, surface entity-specific fields, and serialize the `RuleGroupType` JSON into the node config.
- **Backend**: Filter nodes already hydrate upstream graphs from SQLite via SeaORM. SQLite JSON1 (`json_extract`) supports arbitrary `attrs.*` lookups, so the Rust query compiler can emit parameterized SQL safely.
- **Data model**: `FilterNodeConfig` now consists of a single `query: QueryFilterConfig`. Stored plans that still contain `filters: GraphFilter[]` are auto-upgraded in both the UI and backend, so no DB migration is required.

## Confirmed Requirements
1. **Multi-entity queries** – one query can include rules for nodes, edges, and layers simultaneously.
2. **Configurable pruning** – users can choose how dangling edges / orphaned nodes are handled after filtering.
3. **Attribute-only v1** – aggregations or cross-entity comparisons are out of scope for now; mark TODO hooks for later.
4. **Manual attrs keys** – users type `attrs.customKey` values themselves (no auto-discovery yet).
5. **Rename variant** – rename `GraphFilterKind::QueryText` ➝ `Query` everywhere (no legacy configs to migrate).

## Proposed Implementation Plan
1. **Data model & schema updates**
   - Replace `filters: GraphFilter[]` with `query: QueryFilterConfig` in TypeScript, GraphQL, and Rust.
   - Provide a serde shim that can deserialize the legacy multi-filter payload, extract the first query definition, normalize it, and drop preset-only entries.
   - Normalize every saved query (targets default to nodes, rule group defaults to `{ combinator: 'and', rules: [] }`, metadata version defaults to `v1`) before persisting or executing.

2. **Frontend integration (react-querybuilder)**
   - Expose a single `QueryFilterBuilder` inside the Filter node dialog (no add/remove list, no preset dropdown).
   - Summarize the configured query directly in the node body (mode + targets + rule count) so users can scan plans quickly.
   - Migrate any legacy configs client-side when nodes are loaded to avoid crashes while editing.

3. **Backend query compilation & execution**
   - Invoke the existing SQL compiler once per Filter node execution, using the normalized `QueryFilterConfig`.
   - Maintain include/exclude semantics and link-pruning behavior (auto-drop dangling edges, retain edges, or drop orphan nodes) as part of the single query execution.
   - Store `metadata.query` alongside the filtered graph so executions remain auditable, and keep a legacy parser for stored configs that still include `filters`.

4. **Testing, validation, docs**
   - Run `npm run frontend:build` and `cargo test -p layercake-core`; the end-to-end integration test regenerates the reference Mermaid export, which has been updated to the new template output.
   - This document and any user-facing materials will note that filter presets/multi-filter stacks have been removed in favor of the query builder.

## Risks & Considerations
- **Operator parity**: the UI must only expose operators the backend understands, or we risk runtime compile errors.
- **JSON attribute performance**: SQLite JSON queries are unindexed; large graphs may need additional indexes or virtual columns later.
- **Entity coupling**: multi-entity queries complicate pruning semantics. We must clearly document how link-pruning modes interact with include/exclude filters.
- **Manual attrs input**: without auto-discovery, typos in `attrs.*` keys silently yield empty result sets; validation helpers are recommended.
- **Complex SQL generation**: mixing numeric and text operators in nested groups requires careful typing and thorough test coverage.
- **Future extensions**: aggregations or cross-entity predicates are deferred but must be considered when designing the config schema (hence `notes` + TODO hooks).

## Open Questions
1. UX detail: should entity prefixes be displayed inline with field names (`node.label`) or separated via grouped dropdown sections to keep multi-entity queries readable?
2. Defaulting: which `linkPruningMode` should be preselected (`autoDropDanglingEdges` vs. `retainEdges`), and should the UI warn users before changing it?

## Status
- [x] Data model + enum rename applied across TypeScript and GraphQL/Rust types.
- [x] Frontend query builder integration (react-querybuilder UI, multi-entity targeting, config persistence).
- [x] Backend query compiler and execution pipeline (SQL generation + SeaORM execution + link pruning controls).
- [x] Tests, documentation updates, and golden file refresh.
