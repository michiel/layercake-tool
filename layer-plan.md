## Goal
Introduce project-wide layer definitions that are independent of plan DAG assembly. Layers become a first-class, project-scoped resource; datasets can contribute layer rows that are opt-in for inclusion. All rendering (exports, previews, artefacts) reads from the project layer set.

## Architecture Changes
- **Data model**
  - Add `project_layers` table: `{ id, project_id, layer_id, name, background_color, text_color, border_color, source_dataset_id?, enabled, created_at, updated_at }`.
  - Keep a row per dataset contribution (source_dataset_id) plus optional manual rows (source_dataset_id = NULL). Enabled flag gates inclusion; uniqueness scoped by project/layer_id/source_dataset_id.
  - Migrate existing per-graph layer storage:
    - When reading legacy graphs, merge distinct layer rows into `project_layers`.
    - Update graphs to reference layer IDs only; drop embedded layer color blobs on write/export (or keep for backward compatibility but ignore if project layers exist).
- **Graph/plan execution**
  - When building graphs from datasets, collect layer IDs seen in nodes/edges but do not persist layer definitions to the graph; instead, look up colors from `project_layers` at render time.
  - If a node/edge references a layer not present in `project_layers`, mark as “missing layer” for the Layers UI tab and optionally flag in annotations.

- **GraphQL/REST**
  - New queries/mutations:
    - `projectLayers(projectId)` → list project layers with source metadata and enabled flag.
    - `setLayerDatasetEnabled(projectId, datasetId, enabled)` → include/exclude all layers from a dataset (parses dataset graph_json.layers).
    - `upsertProjectLayer(projectId, layer)`; `deleteProjectLayer(projectId, layerId, sourceDatasetId?)`.
    - `missingLayers(projectId)` → unique layer IDs found in nodes/edges not present/enabled in project layers.
  - Extend existing plan/graph preview resolvers to attach project layer map (fallback to legacy if empty).

- **Rendering/export**
  - Export pipeline (PlantUML, DOT, Mermaid, CSV, etc.) pulls `layers` from project scope instead of graph scope.
  - Common renderer context (`export::renderer::create_standard_context`) loads project layers by ID for the graph being rendered.
  - Layer color lookup: `layer_id -> project_layers` map; if missing, optionally default neutral palette and surface warning.

- **Backend services**
  - `GraphService`/plan executor: when materialising graphs, do not persist layers to `graph_layers`; instead, ensure project layer set is loaded for render-time.
  - Migration helpers: a one-time job to populate `project_layers` from existing `graph_layers` per project and mark legacy data migrated.

## UI Changes (Workbench / Layers page)
- New “Layers” page under Workbench with three tabs:
  1. **Sources**: list all datasets that declare layers; toggle include/exclude; show counts and preview of layer IDs/colors.
  2. **Palette**: editable table of current project layers with color pickers and hex inputs; add/delete rows; edit labels/IDs/colors; source info displayed (manual vs dataset).
  3. **Missing**: unique layer IDs discovered in nodes/edges of datasets/graphs that are not in the project set; provide “Add selected” action to create them with default colors.
- Persist changes via new GraphQL mutations; optimistic UI updates and error toasts.

## Data Flow
1. Import/ingest datasets → extract layer rows (if present) and store in `project_layer_sources`.
2. User toggles sources → service regenerates project palette (or keeps manual edits overriding source colors).
3. Graph execution uses only project layers for color/style resolution.
4. Missing layers are detected by scanning node/edge layer fields during graph build and exposed to the Layers UI tab.

## Migrations & Compatibility
- New tables as above; backfill existing `graph_layers` into `project_layers` per project.
- Keep legacy exports working: if `project_layers` empty, fall back to embedded graph layers; log deprecation.
- Ensure API versioning: clients without layer page still work because renderers fallback.

## Testing
- Unit: layer merge logic, missing layer detection, renderer context uses project layers, backfill script.
- Integration: GraphQL mutations for add/update/delete/toggle; export renders with project layers; missing layer tab shows expected IDs.
- UI: Cypress/Playwright smoke for Layers page tabs and mutations; snapshot color picker values; toggle datasets on/off and verify renderer uses palette.
