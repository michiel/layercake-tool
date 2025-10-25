# Query Builder Integration Plan

## Goal
- Replace the deprecated "Query Text Filter" option in the Filter node with a first-class `Query` filter powered by [`react-querybuilder`] so users can build nested conditions across nodes, edges, and layers.
- Persist each query’s configuration JSON in `GraphFilter` so DAG executions are reproducible and auditable.
- Compile the saved query JSON into SQL/SeaORM predicates that execute against `graph_nodes`, `graph_edges`, and `graph_layers` before materializing filtered graphs.

## Feasibility Snapshot
- **Frontend**: `react-querybuilder` is already installed. It can be wrapped with Mantine components, fed with our entity-specific field lists, and the resulting `RuleGroupType` can be serialized directly into the plan node config.
- **Backend**: Filter nodes already load upstream graphs through SeaORM-backed SQLite tables that expose every field we need. SQLite JSON1 (`json_extract`) lets us query arbitrary `attrs.*` keys, so a Rust query-compiler module can translate the builder config into `Condition`s executed server-side.
- **Data model**: `GraphFilterParams` (TS + GraphQL + Rust) currently holds `preset/query_text/enabled`. We can replace the unused `query_text` with a strongly typed `query_config` payload without DB migrations because plan configs are stored as JSON blobs.

## Confirmed Requirements
1. **Multi-entity queries** – one query can include rules for nodes, edges, and layers simultaneously.
2. **Configurable pruning** – users can choose how dangling edges / orphaned nodes are handled after filtering.
3. **Attribute-only v1** – aggregations or cross-entity comparisons are out of scope for now; mark TODO hooks for later.
4. **Manual attrs keys** – users type `attrs.customKey` values themselves (no auto-discovery yet).
5. **Rename variant** – rename `GraphFilterKind::QueryText` ➝ `Query` everywhere (no legacy configs to migrate).

## Proposed Implementation Plan
1. **Data model & schema updates**
   - Rename the enum variant and UI label from `QueryText` to `Query` in TypeScript, GraphQL, and Rust.
   - Define the new config payload:
     ```ts
     type QueryFilterTarget = 'nodes' | 'edges' | 'layers';
     type QueryFilterTargets = QueryFilterTarget[];
     type QueryLinkPruningMode = 'autoDropDanglingEdges' | 'retainEdges' | 'dropOrphanNodes';

     interface QueryFilterConfig {
       targets: QueryFilterTargets;      // multi-entity selection
       mode: 'include' | 'exclude';      // keep vs. remove matching rows
       linkPruningMode: QueryLinkPruningMode;
       ruleGroup: RuleGroupType;         // react-querybuilder JSON schema
       fieldMetadataVersion: string;     // future-proofing
       notes?: string;                   // optional TODOs for future augments
     }
     ```
   - Encode entity context inside each rule’s `field` (e.g., `node.label`, `edge.attrs.priority`). This allows a single `ruleGroup` to mix targets.
   - Update `GraphFilterParams` to drop `query_text` and add `query_config: Option<QueryFilterConfig>` (with matching TS/GraphQL mirror).

2. **Frontend integration (react-querybuilder)**
   - Implement `QueryFilterBuilder` (e.g., `frontend/src/components/editors/PlanVisualEditor/forms/QueryFilterBuilder.tsx`) that:
     - Wraps `<QueryBuilder />` with Mantine-styled controls.
     - Offers multiselect for entity targets, include/exclude toggle, link-pruning dropdown, and optional notes textarea.
     - Presents fields grouped by entity with clear prefixes, plus a simple text input for manual `attrs.*` keys.
     - Restricts operators to the subset the backend supports (`=`, `!=`, `<`, `<=`, `>`, `>=`, `between`, `in`, `contains`, `startsWith`, `endsWith`).
   - Replace the disabled text input in `FilterNodeConfigForm` with the new builder when the user selects `Query`.
   - Ensure configs round-trip: loading a node pre-populates the builder, and editing updates `query_config` in `GraphFilter.params`.

3. **Backend query compilation**
   - Add `layercake-core/src/filters/query.rs` containing:
     - Rust equivalents of `QueryFilterConfig`, `RuleGroup`, `Rule`, etc. (derive `Serialize/Deserialize`).
     - A compiler that walks the `ruleGroup`, and for each entity prefix, emits a SeaORM `Condition`.
     - Operator translation logic that maps builder operators to SQLite expressions, always using bound parameters. For JSON attributes emit `Expr::cust("json_extract(attrs, ?)")`.
     - `mode` handling: `include` → `IN (matches)`; `exclude` → `NOT IN (matches)`.
     - `TODO` placeholders (with `tracing::warn!`) for future aggregation/cross-entity functionality so the config format stays forward-compatible.

4. **Filter node execution path**
   - Update `FilterNodeConfig::apply_filters` so `GraphFilterKind::Query`:
     - Executes the compiler for each targeted entity, retrieving the matching IDs per table from SQLite (before building the in-memory `Graph`).
     - Applies include/exclude semantics to the loaded graph data, then enforces the chosen `linkPruningMode`:
       - `autoDropDanglingEdges`: remove edges referencing missing nodes automatically.
       - `retainEdges`: keep edges even if endpoints were removed (documented risk).
       - `dropOrphanNodes`: when filtering edges, optionally drop nodes that lose all incident edges.
     - Persists the resulting graph via the existing `persist_filtered_graph`.
   - Record metrics / debug logs describing how many entities were kept/dropped to help tune performance.

5. **Testing, validation, docs**
   - Rust unit tests for the compiler: nested groups, numeric vs. text comparisons, include vs. exclude, JSON attribute rules.
   - Integration test in `layercake-core/tests` exercising a Filter node with multi-entity rules to ensure graph consistency.
   - React Testing Library test confirming the builder UI saves/loads `query_config`.
   - Update docs/README to include instructions for building query filters, supported operators, and link-pruning behavior.

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
- [ ] Frontend query builder integration.
- [ ] Backend query compiler and execution pipeline.
- [ ] Tests, documentation, and UX validation.
