# Dataset Model Update Plan

The goal is to retire the legacy `dataType` field (Nodes/Edges/Layers/etc.) and treat every dataset as a graph bundle that can contain any combination of nodes, edges, and layers. We also need to ensure UI flows and APIs no longer rely on the single-type assumption, and that layer sources derive their options by inspecting dataset contents rather than `dataType`.

## Scope & Tasks

1. **Schema & Backend adjustments**
   - Remove the `dataType` column/enum from GraphQL types (`DataSet`, `CreateDataSetInput`, etc.) and associated SQL migrations/ORM models.
   - Update dataset creation endpoints (`createDataSet` / `createEmptyDataSet`) so they no longer require or store a `dataType`.
   - Ensure import pipelines decide how to populate `nodes`, `edges`, and `layers` based on parsed files, not a declared type.
   - Verify GraphService and plan execution code paths that currently branch on `dataType`; refactor to inspect actual payload (e.g., check `graphJson.layers` length when needed).

2. **Frontend UI changes**
   - Remove the “Data Type” column from dataset tables and detail views.
   - Update the “New Data Set → Create Empty” dialog to drop the type selector; default to an empty graph `{ nodes: [], edges: [], layers: [] }`.
   - Audit components that display dataset metadata to ensure no stale references to `dataType`.

3. **Layer source discovery**
   - Change layer source selectors (e.g., Project Layers page, artefact forms) to gather datasets by checking whether their stored graph has any `layers` entries.
   - Ensure GraphQL queries expose enough information (e.g., count of layers or a boolean flag) so the UI can filter without fetching the entire graph payload unnecessarily.

4. **Data migration / compatibility**
   - Provide a migration or fallback logic to handle existing datasets that still have a `dataType` column. For older records, treat them as mixed graphs and keep ingesting data as before.
   - Update tests and fixtures (both backend Rust tests and frontend mock data) to remove `dataType`.

5. **Documentation**
   - Refresh READMEs/UX docs to explain the new dataset model and the removal of the type selector.
   - Mention the new behavior in release notes / CHANGELOG.

## Open Questions / Follow-ups

- Do we need to expose lightweight metadata (e.g., counts of nodes/edges/layers) so UIs can show “Layers present” badges without parsing full graphs?
- Should we opportunistically clean up unused `dataType` database columns, or keep them nullable for compatibility and drop later?

## Progress Log

- **2024-XX-XX**: Initial audit completed. `dataType` is still referenced in GraphQL types (`DataSet`, `LibraryItem` metadata), dataset services (`DataSetService`, `GraphService`), upload/import paths, Plan DAG validation, and multiple frontend components (dataset tables, uploader, plan editors, library UI). Upcoming work will remove these references while preserving file upload hints.
- **2025-11-21**: Project archive/template exports now bundle full dataset `graph_json` payloads (as `.json`) instead of CSV extracts, and imports rehydrate datasets directly from those graphs. This paves the way for dropping the per-dataset `dataType` classification entirely.
- **2025-11-21**: GraphService layer seeding now detects layer-capable datasets by inspecting stored `graph_json` rather than the legacy `data_type` column, so any mixed dataset containing layer entries can act as a layer source.
- **2025-11-21**: GraphQL `DataSet`/`DataSetReference` objects no longer expose `dataType`. They now surface `nodeCount`/`edgeCount`/`layerCount`/`hasLayers`, and AppContext summaries compute those counts directly from `graph_json`, aligning the public API with the new “datasets are complete graphs” model.
- **2025-11-21**: Internally every dataset is treated as a single graph object with `nodes`, `edges`, and `layers` arrays (any of which may be empty). Pipeline builders now iterate those collections unconditionally instead of branching on legacy `dataType` tags, simplifying ingestion and ensuring consistent behavior across the codebase.
- **2025-11-21**: Bulk export now writes nodes/edges/layers for each dataset directly from `graph_json`, emitting separate sheets per section and ignoring the deprecated `data_type` flag. Empty datasets produce placeholder sheets rather than failing.
- **2025-11-22**: File uploads (CLI, GraphQL, MCP, and UI) now auto-detect the tabular data type and only accept an optional CSV/TSV hint. Dataset uploader surfaced the new optional selector, JSON uploads always assume graph bundles, and the preview API derives the displayed section directly from stored `graph_json`.

This plan covers the high-level refactor; implementation should proceed in the order above so GraphQL schema/backend changes land before frontend removals.
