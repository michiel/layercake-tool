# Attributes Support Plan

## Goals & Scope
- Add optional `attributes` to nodes and edges: map of `string -> (string | integer)`.
- Persist attributes in the data model and expose through API/GraphQL, import/export, and UI (data edit + node hierarchy view).
- CSV import/export must serialize attributes into a single column; other formats may use JSON objects.

## Data Model Changes
- **Database (nodes, edges):** Reuse existing `attrs` JSON column as the persisted `attributes` payload for nodes/edges (string/int values only); no new migration required.
- **Domain structs:** Extend node/edge models (Rust `graph::Node`, `graph::Edge`, SeaORM entities) with `attributes: Option<serde_json::Value>` (map of string -> string|int).
- **Validation:** Enforce keys are non-empty strings; values are either strings or ints.
- **Migrations:** None required; keep JSONB/Text column as-is (optional future index when attribute queries arrive).

## API/GraphQL & Services
- **GraphQL types:** Add `attributes` field on node/edge types and inputs. Use `Json` scalar or a small `AttributesInput` list of `{ key, value, valueType }` to retain int vs string hint.
- **Services:** Ensure graph load/save flows persist attributes through:
  - Graph import/export paths (graph_service, graph_edit_service).
  - Plan DAG execution if nodes/edges are materialized into datasets/graphs.
- **Authorization:** No special handling; attributes ride along existing CRUD paths.

## Import/Export
- **CSV import:** Add optional `attributes` column. Parse as JSON object; on write, stringify compact JSON (one column). Reject malformed JSON with clear errors.
- **CSV export:** Emit compact JSON into the `attributes` column; empty/null -> blank.
- **Other formats (JSON, GraphQL, internal plans):** Use JSON object directly.
- **Backward compatibility:** When `attributes` missing, default to `None`/empty map; exporters should tolerate absence.

## UI Changes
- **Display:** In existing UI tables/forms, render attributes read-only as `k:v;k:v` (table) or multi-line `k:v\nk:v` (forms) when present.
- **Editing:** Add a dedicated `AttributesEditorDialog` component with an `AttributesEditor` subcomponent (key/value rows, value type selector string/int), opened from existing edit flows.
  - Data edit (nodes/edges): add “Edit attributes” affordance that opens dialog.
  - Node hierarchy view: show read-only string; provide button to open dialog for editing.
- **Validation:** Validate in dialog; apply changes on save; keep main forms light.
- **CSV UX:** When importing via UI, document the JSON-in-column requirement; show parse errors inline.

## Testing Strategy
- Unit: Attribute parsing/validation, CSV round-trip (string/int), GraphQL schema serialization.
- Integration: Import CSV with/without attributes; export and re-import matches; UI dialog submits attributes and persists.
- Migration: Ensure adding columns is backward compatible; run migration on sample DB.

## Risks & Gaps
- **CSV ergonomics:** JSON-in-column can be error-prone. Mitigation: strict error messages, sample templates.
- **Search/indexing:** No indexes on attributes; queries over attributes unsupported initially. Mitigation: call out limitation.
- **UI complexity:** Dialog workflow adds steps; mitigate with clear affordances and read-only preview in main views.

## Implementation Plan
1) **Schema/Migration**
   - Add nullable `attributes` JSON/JSONB (or TEXT) columns to node/edge tables.
   - Update SeaORM entities and ActiveModels.
2) **Domain & Services**
   - Add `attributes` field to domain structs and DTOs.
   - Plumb through graph load/save/edit services; default to empty map when absent.
3) **API/GraphQL**
   - Extend node/edge types and inputs with `attributes` (Json scalar or structured input).
   - Update resolvers/mutations to persist attributes.
4) **Import/Export**
   - CSV: add column, JSON parse/stringify; update templates and docs.
   - JSON/other exports: include attributes object.
5) **UI**
   - Build `AttributesEditorDialog` + `AttributesEditor` components (key/value/type) for editing.
   - Render read-only attributes strings in existing tables/forms; wire edit buttons to dialog.
   - Handle validation/errors and dirty state.
6) **Testing & Docs**
   - Add unit/integration tests for parsing, persistence, round-trips.
   - Update docs/templates/sample CSVs explaining the attributes column format and dialog usage.

## Recommendations
- Use `serde_json::Value` internally but expose a typed `AttributeValue` enum at API/validation boundaries for string vs int.
- Prefer JSONB columns if supported (for future querying), else TEXT with JSON validation in application layer.
- Keep CSV column name consistent (`attributes`) and documented with examples; document the dialog UX for edits.
