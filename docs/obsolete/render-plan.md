# Render Target Configuration Plan

## Goals
- Allow artefact nodes to configure render-target–specific options (e.g., Graphviz orientation/layout, Mermaid look preset) in the UI.
- Persist target-specific options in node config JSON / GraphQL schema and surface them in the template context so Handlebars templates can emit the right directives.
- Keep the settings dialog manageable by only showing relevant options for the selected render target.

---

## 1. Configuration Model
1. Extend `RenderConfig` (Rust + GraphQL types) with a flexible `target_options: Option<serde_json::Value>` map keyed by render target identifier.
2. Introduce typed structs for known targets:
   - `GraphvizOptions { layout: GraphvizLayout, overlap: bool, spline: bool, nodesep: f32, ranksep: f32 }`.
   - `MermaidOptions { look: MermaidLook, display: MermaidDisplay }`.
   - Additional targets (PlantUML, CSV, etc.) can define their own structs over time.
3. Store the strongly typed options inside the render config under a `render_target` key so the backend knows which struct to deserialize.
4. Record defaults per target (e.g., Graphviz `layout=dot`, `spline=true`, `overlap=false`, `nodesep=0.5`, `ranksep=0.5`; Mermaid `look=default`, `display=full`).

---

## 2. Frontend / UI Work
1. Update artefact config forms (Graph + Tree) so the `renderTarget` select drives a conditional sub-form:
   - When `renderTarget=graphviz`, show dropdowns/toggles for the Graphviz options listed above.
   - When `renderTarget=mermaid`, show the Mermaid-specific select fields.
   - For other targets, either hide the sub-form or show a relevant subset.
2. Ensure the form serializes `renderConfig.targetOptions` (target-specific payload) alongside generic render config fields.
3. Provide contextual helper text (e.g., tooltip describing Graphviz layouts: Dot, Neato, Fdp, Circo).
4. Persist the target-specific config locally (React state) so switching targets preserves their configurations (per target) until saved.

---

## 3. Backend
1. GraphQL Input/Output:
   - Extend `RenderConfigInput` / `RenderConfig` to expose `target_options` as a JSON map.
   - Add GraphQL enums for Graphviz layouts and Mermaid looks/displays for validation in typed fields (optional but preferred).
2. Node Config Serialization:
   - `StoredGraphArtefactNodeConfig` should persist the `target_options` payload in `render_config`.
   - Migration logic should default legacy nodes to empty `target_options`.
3. Template Context:
   - `export::create_standard_context` must insert a `target_options` map (or a strongly typed struct) into the Handlebars context so templates can do `{{target_options.graphviz.layout}}` or similar.
4. Validation:
   - Add validation when saving node configs to ensure combos are legal (e.g., Graphviz layout must be one of the enumerated values, nodesep/ranksep >= 0).

---

## 4. Template Updates
1. DOT / Graphviz templates:
   - Read `config.target_options.graphviz` to emit `layout="dot"`, `splines=true/false`, `overlap=false`, `nodesep=0.5`, `ranksep=0.5`, etc.
   - Provide defaults when options are missing.
2. Mermaid templates:
   - Read `config.target_options.mermaid` to emit `%%{init: { 'themeVariables': { ... }, 'theme': ...}}%%` or class definitions for `look=handDrawn` vs. `display=compact`.
3. Keep templates clean by using helpers or partials for the new settings blocks.

---

## 5. Testing
1. Unit tests for render config serialization/deserialization with `target_options` payloads.
2. Template snapshot tests verifying Graphviz/Mermaid outputs change when options change.
3. Frontend tests (or manual QA) ensuring the dependent sub-form updates when the render target changes.

---

## 6. Rollout / Migration
1. Back-fill existing node configs by treating `target_options` as `null` (renderers fallback to defaults).
2. Update docs + tooltips so users understand the new per-target controls.
3. After the feature stabilizes, consider extending other render targets (PlantUML, CSV exporters) with their own option sets.

## Progress
- ✅ Backend/schema support: Added strongly typed render-target option structs, GraphQL fields, and stored-config parsing so templates receive `config.target_options`. Frontend/UI + template wiring still pending.
- ✅ Frontend config UI: Artefact dialogs now expose Graphviz/Mermaid-specific controls and serialize `renderConfig.targetOptions`.
- ⚙️ Template consumption: DOT and Mermaid templates read `config.target_options` to emit layout/look/display directives. Additional targets can adopt the same helpers next.
