## Divergence from SPECIFICATION.md

### 1. `graph_json` structure for layers

**Specification:** "Each Project has DataSources. DataSources are tables belonging to a project that are a raw import of their source (attr: blob), and contain a graph_json attribute that imports the raw attributes to the appropriate graph_json={{nodes:[], edges:[], layers:[]}} attributes"

**Divergence:** The `graph_json` generated for layers contains an empty string key `""` in the layer objects (e.g., `{"" : "", "id": "base", ...}`). This is not a valid JSON structure and causes issues in the frontend when attempting to display or edit layer information. Additionally, the `color` field from imported CSV layers was previously mapped to `color` in `graph_json`, but the frontend's `GraphLayer` interface expects `background_color`. (This `color` to `background_color` mapping issue has been addressed in a recent fix).

### 2. `import_csv` bypasses `graph_json` generation

**Specification:** "Each Project has DataSources... contain a graph_json attribute that imports the raw attributes to the appropriate graph_json={{nodes:[], edges:[], layers:[]}} attributes"

**Divergence:** The `import_csv` tool, used for importing nodes, edges, and layers from CSV content, directly populates the respective database tables (`nodes`, `edges`, `layers`). It bypasses the `graph_json` generation logic present in `DataSourceService::create_from_file`. This means that if a datasource is created or updated solely via the `import_csv` tool, its `graph_json` attribute will either be empty or contain outdated/incorrect information, leading to inconsistencies when trying to display the datasource's graph data in the frontend.

### 3. Frontend `GraphLayer` field mismatch (Resolved)

**Specification:** Implied by the visual editing of layers, the frontend expects specific fields for layer properties.

**Divergence (Resolved):** The frontend's `GraphLayer` interface expects fields like `background_color`, `text_color`, and `border_color` for layer styling. Previously, the backend was only providing a generic `color` field from CSV imports. This has been corrected to map the `color` field to `background_color` in the `graph_json`.