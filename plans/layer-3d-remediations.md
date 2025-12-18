# Layer3D Remediation Plan

## Findings to Address
- GraphQL data mismatch: ProjectionGraph queries/subs omit attrs/weight fields needed for hierarchy and sizing, and edges omit attrs.relation. Layer3D always falls back to flat-per-layer and constant sizing.
- Edges not rendered: Layer3DScene ignores edges entirely; no orthogonal routing or edge visibility.
- Layer planes desynced: Planes use fixed spacing/size, ignoring layout controls (layerSpacing/canvasSize), so planes can drift from node positions.
- Partition Y/height incorrect: Partitions rendered at y=0 spanning all layers regardless of actual grouping, misaligning containers.
- State persistence missing: Layer3D controls are not loaded from or saved to ProjectionState; Leva changes are ephemeral.
- Ground plane size fixed: Fixed 100x100 plane unrelated to canvasSize; large layouts exceed the ground/frame.

## Remediation Tasks
1) GraphQL/Data Path
   - Extend ProjectionGraph query/subscription (projections-frontend/src/app.tsx) to request node attrs and weight, and edge attrs (relation). Ensure downstream mapping passes attrs/weight into Layer3D layout inputs.
2) Edge Rendering
   - Add orthogonal edge rendering in Layer3DScene using a simple segment-based A-Frame path (source→(x,z gutter at target y)→target). Honor show/hide toggle (initially always on). Use edge color/opacity defaults.
3) Layer Plane Sync
   - Drive layer plane Y from layer index * layerSpacing and plane size from canvasSize so they align with node layout controls.
4) Partition Placement
   - Use node.layer for partition Y; set partition height to layerSpacing (or a small multiple) instead of spanning all layers. Keep partitions bounded to their layer group center.
5) State Persistence
   - Load Layer3D controls from projectionState when projectionType === 'layer3d'; save Leva changes back via SAVE_STATE, namespaced under `layer3d`. Default to existing control defaults if state missing.
6) Ground Size
   - Tie ground plane width/height to canvasSize, not a hard-coded 100.

## Deliverables
- Updated plan implemented across Layer3DScene, layout hook, and app.tsx data/state wiring.
- Basic edge rendering present and visible; hierarchy honors attrs/edge relations when provided.
- Layer planes/partitions/ground aligned with layout controls.
- Layer3D controls persist via ProjectionState.
