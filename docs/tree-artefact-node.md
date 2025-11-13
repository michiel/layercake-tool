# Plan: Rename OutputNode → GraphArtefactNode and Introduce TreeArtefactNode

This document outlines the implementation plan for modernizing the artefact-generation nodes in the plan DAG. The work renames the existing `OutputNode` to the clearer `GraphArtefactNode`, then adds a new `TreeArtefactNode` (TreeMap Artefact) that reuses the same export/preview pipeline.

## Goals
1. Improve naming clarity by renaming OutputNode → GraphArtefactNode everywhere (Rust backend, TypeScript frontend, persisted configs, docs).
2. Support tree/mindmap exports via a new TreeArtefactNode that can render both PlantUML Mindmap and Mermaid Mindmap outputs.
3. Preserve the current export/download/preview workflow so both artefact nodes use the same command execution, templates, and Handlebars rendering pipeline.

## Work Breakdown

### 1. Type & Schema Renaming (GraphArtefactNode)
- Update Rust GraphQL types (`GraphNode`, `PlanDag` DTOs) and enums to expose `GraphArtefactNode`.
- Rename matching TypeScript types (`GraphNodeType`, union discriminators, component props) to keep the schema/SDK in sync.
- Write a small migration that rewrites existing persisted configs/JSON to accept both the legacy `"output"` node type and the new `"graph_artefact"` variant for backwards compatibility.
- Refresh docs/examples to use the new terminology.

### 2. Extract Common Artefact Node Behaviour
- Identify the shared behaviour in the current OutputNode implementation (render target selection, template lookup, export file naming, download button state).
- Move the reusable pieces into a shared helper module (e.g., `artefact_node.rs` on the backend, a `useArtefactNode` hook or utility on the frontend).
- Ensure the helper abstracts:
  * Render target metadata (id, label, template path).
  * Execution pipeline (kick off export job, stream preview, write downloadable artefact).
  * Shared UI state (loading, error, preview text).

### 3. Rebrand Existing Node to GraphArtefactNode
- Swap the backend switch/case branches from `OutputNode` to `GraphArtefactNode`, using the helper from step 2.
- Update command routing, MCP tool prompts, and resolvers that referenced OutputNode IDs.
- Rename Handlebars templates/directories to match (e.g., `graph_artefact/plantuml.hbs`), leaving symlinks or aliases if needed to avoid breaking existing exports.
- Verify that existing render targets (PlantUML Sequence, Mermaid Flow, DOT, etc.) still appear with the same options under the new name.

### 4. Add TreeArtefactNode (TreeMap Artefact)
- Extend schema/types with a `TreeArtefactNode` entry, using the same config shape as GraphArtefactNode (render target selection, output filename, optional prompt metadata).
- Register two render targets for TreeArtefactNode:
  * `tree_plantuml_mindmap` → PlantUML Mindmap template.
  * `tree_mermaid_mindmap` → Mermaid Mindmap template.
- Copy/adapt the Handlebars templates from the graph pipeline; ensure they live under a tree-specific directory for clarity (`tree_artefact/plantuml_mindmap.hbs`, `tree_artefact/mermaid_mindmap.hbs`).
- Wire the node into the artefact helper so export/download/preview flows automatically match the GraphArtefactNode capabilities.

### 5. Frontend UX Updates
- Update the node palette/config UI so users can select between Graph Artefact and Tree Artefact nodes, each with a render target dropdown containing the appropriate options.
- Ensure the preview panel, download button, and status indicators share the helper logic to avoid duplicating React code.
- Add user-facing copy describing the new Mindmap outputs in the node configuration form/tooltips.

### 6. QA & Docs
- Add/refresh unit tests covering:
  * Artefact node resolution after renaming.
  * TreeArtefactNode render target validation.
  * Export pipeline selects the correct Handlebars template IDs.
- Extend integration snapshots if the API output changes (GraphQL node types, plan serialization).
- Document the new node capabilities in `docs/` and update any samples (plan YAMLs) that referenced OutputNode.

## Dependencies & Considerations
- Ensure renaming does not break existing plans: keep deserializers tolerant of both `"output"` and `"graph_artefact"` until old data is migrated.
- Templates for mindmap targets should match the existing `OutputNode` structure (context data shape, partials) to reuse the helper infrastructure.
- Coordinate with UI/UX to confirm naming (“Graph Artefact” vs “Graph Export”) before shipping.

With this plan, both graph and tree artefact nodes share one implementation path, making it easy to add new render targets in the future.
