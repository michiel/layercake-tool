# Layer Alias Extension Plan

## Goal
Introduce an `alias` attribute on layer definitions so that palette entries can point at another project layer. This allows datasets to continue ingesting their own layer IDs while reusing a canonical palette color scheme (e.g., `{ layer_id: "threat_actor", alias: "warning" }` uses palette entry `warning` whenever it exists).

## Summary
1. Extend backend storage to persist the alias column and expose it through GraphQL.
2. Update CSV import/export, copy/paste flows, and dataset editors to read/write alias values.
3. Teach palette resolution (renderers, exports, UI previews) to map aliased IDs onto their target entries at runtime.
4. Provide validation, migration, and tooling to maintain referential integrity between aliases and palette entries.

## Status
- [x] Schema + ORM updates in `project_layers` / `graph_layers`, migration `m20251207_000002_add_alias_to_layers`, and GraphService alias resolution (including recursive palette lookup + cycle protection).
- [x] GraphQL types/mutations expose `alias` (`ProjectLayerInput`, `ProjectLayer`, graph `Layer`, layer bulk updates, edit replay).
- [x] Frontend palette + dataset editors present alias fields, persist via copy/paste (`ProjectLayersPage`, GraphSpreadsheet editor for datasets/graphs).
- [ ] CSV/JSON import/export, clipboard, and dataset ingestion/export flows emit/parse the alias column consistently.
- [ ] Artefact renderers / docs / regression tests updated to reflect alias semantics end-to-end.

## Detailed Steps

### 1. Backend Data Model
- **Schema changes**  
  - Add `alias` (TEXT, nullable) to `project_layers` and `graph_layers` tables (new migration).  
  - Update seed/migration scripts to carry alias data when backfilling from legacy rows.
- **ORM models**  
  - Update `layercake-core/src/database/entities/project_layers.rs` and `graph_layers.rs` to include `alias`.  
  - Adjust ActiveModels/Models, `GraphService` structs, and conversions (`ProjectLayer`, `Layer`) to read/write the new column.
- **Validation**  
  - Enforce that aliases refer to existing palette IDs within the same project (soft validation in service layer).  
  - Detect cycles or self-referential aliases and reject updates.
- **Migrations**  
  - Provide SQL/backfill script to set `alias = NULL` for existing rows.  
  - Document manual steps for self-hosted deployments.

### 2. GraphQL API Layer
- **Schema**  
  - Extend `ProjectLayerInput`, `ProjectLayer`, `Layer` (graph-level) types with `alias`.  
  - Update queries (`projectLayers`, `graph.layers`, `missingLayers`) to return alias information.
- **Mutations**  
  - Allow `upsertProjectLayer`, `deleteProjectLayer`, dataset layer toggles, and bulk updates to manipulate the alias field.  
  - Update `GraphSpreadsheetEditor` bulk inputs (layer update payloads) to include alias.
- **Resolvers**  
  - Propagate alias data when assembling `GraphService::list_project_layers`, `missing_layers`, alias resolution logic, etc.

### 3. Palette Resolution Logic
- **Runtime resolution**  
  - Enhance `GraphService::resolve_layer`, export renderers (`export::renderer`, `sequence_context`, etc.) to fall back to the alias target when present.  
  - Ensure traversal handles nested aliases by chasing until a concrete palette entry is found (with cycle protection).  
  - When no target exists, keep original colors but surface warnings.
- **Missing-layer detection**  
  - Treat aliased IDs as satisfied when their target exists/enabled; otherwise, mark them missing.

### 4. CSV/JSON Import & Export Workflows
- **Importers**  
  - Update CSV/JSON ingestion (`data_loader`, `GraphSpreadsheetEditor` CSV paste, dataset bulk import) to parse an optional `alias` column.  
  - Validate alias values against existing palette entries (soft warning if absent).
- **Exporters**  
  - Include `alias` column in CSV exports (`to_csv_nodes`, dataset downloads) and JSON outputs (`GraphSpreadsheetEditor`, `download_json`).  
  - Update custom template contexts to expose alias metadata for advanced rendering.

### 5. Frontend UI Updates
- **Palette management** (`ProjectLayersPage`)  
  - Add alias field to add/edit rows, copy/paste buffer, dataset toggles, and validations.  
  - Update missing-layer tab to suggest aliasing instead of manual color entry when possible.
- **Graph editor palette panel** (`LayerListItem`)  
  - Display alias relationships (e.g., tooltip “alias of warning”).  
  - Provide quick actions to set alias from existing palette entries.
- **Dataset editor / GraphSpreadsheetEditor**  
  - Add alias column to the spreadsheet view (nodes/edges remain unchanged).  
  - Ensure layers tab copy/paste includes the alias column header.  
  - Palette view should show resolved alias targets and raw alias input.
- **Artefact previews**  
  - Update mermaid/PlantUML renderers, story views, and sequence diagrams to consume resolved aliases (no UI change, but tests needed).

### 6. Copy/Paste & Clipboard Flows
- **GraphSpreadsheetEditor**  
  - Include `alias` column in node/edge/layer CSV templates (especially layers).  
  - Update parse/serialize logic to handle the additional field, preserving blanks when absent.  
  - Ensure read-only palettes still display alias values.

### 7. Testing & Validation
- **Backend tests**  
  - Extend GraphService unit tests to cover alias resolution, cycles, missing targets, and migration fallback.  
  - Add GraphQL resolver tests for alias fields.
- **Frontend tests / QA**  
  - Manual smoke tests: palette editing, dataset edit copy/paste, Graph artifacts render, story preview uses alias colors.  
  - Add Cypress/Playwright coverage if available (palette tab interactions).

### 8. Documentation & Migration Notes
- Update `docs/ProjectLayers.md` (or relevant docs) with alias semantics and examples.  
- Provide CSV template snippet showing the new column.  
- Mention alias behavior in README / release notes, including instructions for migrating existing datasets.

## Timeline / Sequencing
1. Backend schema + service updates (alias column, migrations, GraphQL types).  
2. Runtime resolution + missing-layer logic.  
3. Frontend palette editors + dataset spreadsheet alias handling.  
4. CSV/JSON import/export adjustments.  
5. Artefact renderers + story/sequence integrations.  
6. Documentation, QA, and release notes.
