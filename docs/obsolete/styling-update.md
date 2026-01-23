# Layer Styling Overhaul Plan

The current render-config contract (`use_default_styling` + optional theme) does not map cleanly to the requirements for future exports:
* Layer colors should always be respected (nodes grouped/styled by layer where the format supports grouping).
* Built-in themes (`light`/`dark`) should only affect global graph elements (background, default fonts), not override per-layer styling.
* We need an explicit `apply_layers` boolean (default `true`) to turn layer-driven styling on/off.
* We need a `built_in_styles` enum (`none`, `light`, `dark`; default `light`) that controls template-level defaults (background, base node shape, fonts) without interfering with per-layer overrides.

This document outlines the end-to-end plan to evolve the UI, backend, and templates.

---

## 1. Requirements Summary

1. Replace `use_default_styling` with:
   - `apply_layers: bool` (default `true`) controlling whether layer colors drive node group/edge styling.
   - `built_in_styles: enum { none, light, dark }` (default `light`) controlling high-level template defaults (background, font colors, default node style).
2. Graph & Tree artefact nodes must default to `apply_layers=true` and `built_in_styles='light'`, but UIs should allow toggling.
3. Every export template (DOT, DOTHierarchy, PlantUML, Mermaid, CSV, JSON, GML, etc.) must:
   - Use `built_in_styles` only for global graph defaults.
   - Respect `apply_layers` by grouping/styling nodes per layer (DOT node subgraphs, PlantUML stereotypes, Mermaid class definitions, etc.).
   - Continue to honor layer palette (background/text/border colors) pulled from `graph_layers`.
4. GraphQL inputs/outputs, stored plan node configs, and MCP specs must expose the new fields.
5. Migration: existing sessions/configs (`use_default_styling`) need to be interpreted as:
   - `use_default_styling=true` → `apply_layers=true`, `built_in_styles='light'`.
   - `use_default_styling=false` → `apply_layers=true`, `built_in_styles='none'` (preserve "no theme" behavior).

---

## 2. Frontend Work

### 2.1 Data Models & Hooks
1. Update `frontend/src/types/plan-dag.ts`:
   - Replace `useDefaultStyling?: boolean` & `theme?: 'Light' | 'Dark'` with:
     ```ts
     applyLayers?: boolean;
     builtInStyles?: 'none' | 'light' | 'dark';
     ```
   - Keep legacy parsing helpers so cached data continues to load (map old fields to new ones).
2. Update `getDefaultNodeConfig` in `PlanVisualEditor/utils/nodeDefaults.ts` so graph/tree artefact nodes default to:
   ```ts
   renderConfig: {
     containNodes: true,
     orientation: 'TB',
     applyLayers: true,
     builtInStyles: 'light'
   }
   ```

### 2.2 Artefact Config Forms
1. `GraphArtefactNodeConfigForm` & `TreeArtefactNodeConfigForm`:
   - Replace the “Use default styling” toggle with:
     * Checkbox: “Apply layer colors” (`applyLayers`).
     * Select: “Built-in style” (`none`, `light`, `dark`).
   - Ensure the select is independent (always enabled) since built-in styles now control only global defaults.
2. Ensure form submission serializes `applyLayers` & `builtInStyles`.

### 2.3 Rendering Preview / Config Display
1. Anywhere we display render settings (e.g. `ProjectArtefactsPage`), update descriptive text to refer to `applyLayers` & `builtInStyles`.
2. Update GraphQL mutations/hooks that send the old `useDefaultStyling` to use the new structure (with fallback to keep compatibility).

---

## 3. Backend / GraphQL

### 3.1 Plan DAG GraphQL Types
1. In `layercake-core/src/graphql/types/plan_dag/config.rs`:
   - Add `apply_layers: Option<bool>` & `built_in_styles: Option<RenderBuiltinStyle>` to the `RenderConfig` input/object.
   - Introduce `enum RenderBuiltinStyle { NONE, LIGHT, DARK }`.
2. Update `StoredRenderConfig` (GraphQL helpers) to parse legacy data:
   - Map `use_default_styling` to `built_in_styles='light'` and `apply_layers=true`.
   - Map `theme` to `built_in_styles` (`Dark` → `dark`, `Light`/missing → `light`).
   - When legacy `use_default_styling=false`, set `built_in_styles='none'`.

### 3.2 Plan DAG Mutation Serialization
1. `StoredGraphArtefactNodeConfig` / `StoredTreeArtefactNodeConfig` should store the new fields.
2. `default_artefact_render_config()` should set:
   ```rust
   RenderConfig {
       contain_nodes: true,
       orientation: RenderConfigOrientation::TB,
       apply_layers: true,
       built_in_styles: RenderConfigBuiltinStyle::Light,
   }
   ```
3. Update `render_config.into_render_config()` to map legacy fields & set defaults described above.

### 3.3 Plan/Graph Execution
1. Ensure `Plan::get_render_config()` (layercake-core/src/plan.rs) returns the new fields.
2. `PlanExecution` flows that call `export::to_*::render` already pass `RenderConfig`; these renderers/templates just need to read the new fields.
3. Update `RenderConfig` struct (Rust) to include `apply_layers: bool` and `built_in_styles: RenderBuiltinStyle`.

### 3.4 Legacy Data Migration
1. Implement fallback parsing so existing JSON stored in `plan_dag_nodes.config_json` still works:
   - When `render_config` lacks new fields, map from old ones.
2. Consider writing a DB migration (optional) to rewrite stored config JSON with the new structure; not required if runtime parsing handles it.

---

## 4. Templates & Exporters

### 4.1 Shared Context
1. `export::create_standard_context` must expose `config.apply_layers` & `config.built_in_styles` to templates.
2. Remove `config.use_default_styling` references after migrating templates.

### 4.2 Individual Templates
For every template (DOT, DOTHierarchy, PlantUML, PlantUML mindmap/WBS, Mermaid variants, CSV/JSON/GML export where relevant):
1. Top-level theme:
   - Use `config.built_in_styles` to set background/font defaults. Definitions:
     - **none**: do not emit background/font defaults; rely on engine defaults.
     - **light**: e.g., DOT `bgcolor="#ffffff"`, `fontcolor="#1f2933"`, nodes `fillcolor="#f7f7f8"`, `color="#1f2933"`, `fontname="Lato"`. PlantUML: `skinparam BackgroundColor #ffffff`, `skinparam DefaultFontColor #1f2933`, etc.
     - **dark**: e.g., DOT `bgcolor="#1e1e1e"`, `fontcolor="#f5f5f5"`, nodes `fillcolor="#2b2b2b"`, `color="#f5f5f5"`; PlantUML: `skinparam BackgroundColor #1e1e1e`, `skinparam DefaultFontColor #f5f5f5`. Templates must reuse these values consistently.
2. Layer styling & readability:
   - If `config.apply_layers` is true, ensure:
     * DOT: node subgraphs per layer with fill/text/border colors.
     * DOTHierarchy: same as DOT but inside clusters.
     * PlantUML: define stereotypes per layer (<<LayerID>>) with colors; assign nodes to stereotypes.
     * Mermaid: define class/style per layer (e.g., `classDef layerA fill:#...`), assign nodes to classes.
     * JSGraph/MermaidMindmap/Treemap: ensure nodes carry layer-specific colors.
     * CSV/JSON: include layer color metadata columns (if not already).
   - If `apply_layers=false`, do not emit layer-specific color blocks; nodes fall back to built-in/global defaults.
   - Keep templates simple and readable:
     * Extract repeated snippets into partials (layer node blocks, stereotype/class definitions).
     * Push branching logic into helpers (`layer_block`, `edge_style`) instead of embedding large conditionals inline.
     * Maintain consistent indentation/comments so contributors can navigate the templates quickly.
3. Edge styling:
   - If needed, extend templates so edges referencing a layer also pick up colors when `apply_layers=true`.

### 4.3 Renderer Helpers
1. Update `dot_render_tree` and other helper functions (layercake-core/src/common/handlebars.rs) to respect `apply_layers` and `built_in_styles`.
2. Add helper functions (Handlebars) for:
   - `built_in_style_is`: to conditionally emit theme blocks.
   - `layer_style` partials reusable between dot/plantuml/etc.
3. Keep helper APIs focused so templates remain uncluttered (prefer `{{#layer_group layer}} ... {{/layer_group}}` patterns).

---

## 5. UI / UX Copy Updates

1. Tooltip/labels referencing “Use default styling” should change to “Apply built-in theme” or similar (clarify the difference between global styling and layer-driven styling).
2. Update any documentation/tooltips describing the meaning of the flags.

---

## 6. Testing & Validation

1. Unit tests:
   - Extend Rust tests for `StoredRenderConfig::into_render_config()` to cover legacy JSON inputs.
   - Add tests for `default_artefact_render_config`.
2. Template tests:
   - For each export type, add/extend golden tests verifying both `apply_layers=true` and `apply_layers=false`.
   - For built-in styles, verify DOT/PlantUML output matches expected background/font definitions.
3. Frontend:
   - Manual regression: configure artefact nodes with all combinations (layers on/off, built-in styles) and ensure GraphQL payloads match expectations.
   - Visual check: run `npm run frontend:build` and confirm no TypeScript errors.

---

## 7. Rollout Strategy

1. Implement backend parsing and template updates first (ensures existing nodes continue to export even before UI is updated).
2. Update frontend defaults/forms, coordinate with QA for new control labels.
3. Communicate the new config structure to documentation (README / docs/???) so users know how to configure exports programmatically.

---

This plan ensures artefact exports always honor layer colors by default, while giving users explicit control over whether layer styling and built-in themes are applied. All render targets will share the same configuration contract, simplifying both UI/UX and backend logic.***
