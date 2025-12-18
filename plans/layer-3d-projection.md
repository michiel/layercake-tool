# Layer3D Projection Implementation Plan

## Overview

Implement the "Layercake" 3D visualization as the `layer3d` projection type in the projections-frontend. Use the existing ProjectionGraph GraphQL feed (same as Force3D) as the sole data source; no CSV ingestion. Layer3D will introduce new A-Frame primitives (no reuse required from Force3D visuals) but must integrate with the existing projections routing and ProjectionState GraphQL pipeline. Add a validation step that prevents re-import from connection from creating new projects; Layer3D projections should always bind to their existing graph/projection entities.

**Key Characteristics:**
- **Vertical (Y-axis):** Represents layer stratification (e.g., software layers, security levels).
- **Horizontal (XZ-plane):** Represents containment/grouping via treemap layout with clear fallbacks when hierarchy metadata is missing.
- **Edges:** Orthogonal routing to reduce visual clutter.
- **Interactivity:** Camera controls, node selection; narrative/story mode deferred.
- **Tech Stack:** A-Frame for 3D rendering, D3 for layout; ProjectionGraph GraphQL as input.

## Data Model Adaptation

The Layer3D design expects hierarchical data with containment relationships. The GraphQL interface is:

```graphql
type ProjectionGraph {
  nodes: [Node]    # id, label, layer, color?, labelColor?, weight?, attrs?
  edges: [Edge]    # id, source, target, label, weight?, attrs?
  layers: [Layer]  # layerId, name, backgroundColor, textColor, borderColor
}
```

**Challenges & Solutions:**

1) **Hierarchy inference**
- Primary: use `attrs.belongs_to` or `attrs.parent_id` if present.
- Secondary: infer from edges with semantic relation (e.g., `attrs.relation in {"contains","parent_of"}`).
- Fallback: flat hierarchy grouped by layer only; treemap operates per layer with no nesting.

2) **Sizing/weight**
- Use `node.weight` when present; fallback to degree (edge count) or constant 1 for treemap sizing.

3) **Layer mapping**
- Use `node.layer` to map Y stratification; default to first layer if missing.

## Architecture

### Component Structure (new A-Frame implementation; integrates with existing projections routing/state)

```
projections-frontend/
├── src/
│   ├── components/
│   │   ├── Layer3DScene.tsx        # Main A-Frame scene component
│   │   ├── Layer3DControls.tsx     # Leva controls for Layer3D
│   │   ├── Layer3DNode.tsx         # Individual node rendering
│   │   └── Layer3DEdge.tsx         # Orthogonal edge rendering
│   ├── hooks/
│   │   ├── useLayercakeLayout.ts   # D3-based layout calculator
│   │   ├── useLayer3DCamera.ts     # Camera movement/narrative system
│   │   └── useLayer3DState.ts      # ProjectionState management
│   ├── lib/
│   │   ├── layercake-layout.ts     # Core layout algorithms
│   │   └── orthogonal-routing.ts   # Edge path calculation
│   └── app.tsx / routing           # Render Layer3DScene when projection.type === 'layer3d'
```

## Implementation Phases

### Phase 1: Foundation & Basic Layout ⭐ START HERE

**Goal:** Replace the "Coming Soon" placeholder with basic 3D visualization of nodes positioned by layer using ProjectionGraph data.

**Tasks:**
1. Install dependencies (audit existing versions to avoid duplicates):
   ```bash
   npm install aframe d3-hierarchy d3-scale three
   npm install -D @types/aframe
   ```

2. Create `useLayercakeLayout.ts` hook:
   - Accept ProjectionGraph data (nodes, edges, layers).
   - Build hierarchy: prefer attrs.belongs_to/parent_id; else edge semantics; else flat-by-layer fallback.
   - Phase 1 layout: flat XZ per layer (grid/circle) or treemap if hierarchy resolves; map Y by layer index.
   - Return positioned nodes { id, x, y, z, width, depth, height, color, label, isPartition, layerId }.

3. Create `Layer3DScene.tsx`:
   - Render when projection.type === 'layer3d' (app.tsx/routing update).
   - A-Frame scene with camera controls (WASD + mouse look), basic lighting.
   - Render nodes as `<a-box>` (solid) and wireframe partitions; apply layer colors from ProjectionGraph.
   - Basic selection/highlight hooks (new primitives; no Force3D reuse).

4. Update app/routing:
   - Conditional rendering: `{projection.type === 'layer3d' ? <Layer3DScene /> : <Force3DScene />}`.
   - Ensure ProjectionGraph query feeds both projection types; no CSV path.

**Success Criteria:**
- Layer3D projections show nodes as colored boxes in 3D space from GraphQL feed.
- Nodes are vertically separated by layer; flat grouping works when no hierarchy metadata is present.
- Camera moves around the scene; no crashes or console errors.

**Test Plan:**
- Create a Layer3D projection via workbench
- Open it and verify nodes render in 3D
- Verify different layers have different Y positions
- Verify layer colors are applied correctly

---

### Phase 2: Treemap Layout & Hierarchy

**Goal:** Implement proper XZ-plane layout using D3 treemap for containment relationships with explicit fallbacks when hierarchy is absent.

**Tasks:**

1. **Hierarchy Detection (GraphQL-based):**
   - Use attrs.belongs_to/parent_id when present.
   - Otherwise detect edges with semantic relation (attrs.relation in {"contains","parent_of"}) to build hierarchy.
   - Fallback: group nodes by layer only (no nesting); treemap operates per layer.

2. **Update `useLayercakeLayout.ts`:**
   ```typescript
   import { hierarchy, treemap } from 'd3-hierarchy'

   export const useLayercakeLayout = (nodes, edges, layers, config) => {
     // 1. Build hierarchy from edges or flat list
     // 2. Apply d3.treemap() to calculate XZ positions
     // 3. Assign Y position based on layer
     // 4. Return positioned nodes
   }
   ```

3. **Configuration Options:**
   - Canvas size (XZ bounds)
   - Layer spacing (Y-axis distance between layers)
   - Partition padding (treemap padding)
   - Add these to Leva controls

4. **Node Type Differentiation:**
   - Partition nodes (have children): Wireframe pillars spanning all layers
   - Leaf nodes: Solid boxes at their specific layer
   - Implement in `Layer3DNode.tsx`

**Success Criteria:**
- Nodes with children render as wireframe containers; leaves sit within parents when hierarchy exists.
- When hierarchy is absent, nodes are still grouped per layer with a stable XZ layout.
- Treemap layout respects node weights/sizes when provided.
- Configuration changes update layout in real-time and persist via ProjectionState.

---

### Phase 3: Orthogonal Edge Routing

**Goal:** Implement clean, orthogonal edge routing to visualize relationships without clutter.

**Tasks:**

1. **Create `orthogonal-routing.ts`:**
   ```typescript
   export function calculateOrthogonalPath(
     source: { x, y, z },
     target: { x, y, z },
     gutterOffset: number
   ): Vector3[] {
     // Returns array of points for Manhattan-style routing:
     // source -> gutter point (XZ) -> vertical lift -> target
   }
   ```

2. **Implement `Layer3DEdge.tsx`:**
   - Use A-Frame line components or THREE.js Line2 for thick lines
   - Calculate path using orthogonal routing
   - Support edge labels (position at midpoint of path)
   - Visibility toggle from controls

3. **Edge Rendering Strategies:**
   - **Option A:** Individual `<a-entity line>` components per segment (start here).
   - **Option B:** THREE.js geometry for better performance when edge count >500; add threshold switching.

4. **Visual Enhancements:**
   - Directional arrows (reuse from Force3D implementation)
   - Edge color based on source/target layer
   - Edge width based on weight
   - Transparency controls to reduce clutter

**Success Criteria:**
- Edges render as orthogonal paths (no diagonal lines)
- Edges don't overlap with partition boxes
- Edge labels are readable and positioned correctly
- Toggle edges on/off without breaking layout

---

### Phase 4: Interactive Controls & State Management

**Goal:** Add comprehensive controls and persist user preferences via ProjectionState (existing GraphQL mutations/subscriptions).

**Tasks:**

1. **Create `Layer3DControls.tsx`:**
   ```typescript
   const controls = useControls('Layer3D', {
     Layout: folder({
       canvasSize: { value: 100, min: 50, max: 200 },
       layerSpacing: { value: 10, min: 5, max: 30 },
       partitionPadding: { value: 2, min: 0, max: 10 },
     }),
     Display: folder({
       showEdges: true,
       showLabels: true,
       showPartitions: true,
       nodeScale: { value: 1, min: 0.5, max: 3 },
       edgeOpacity: { value: 0.6, min: 0, max: 1 },
     }),
     Camera: folder({
       fov: { value: 60, min: 30, max: 120 },
       moveSpeed: { value: 5, min: 1, max: 20 },
     })
   })
   ```

2. **Implement `useLayer3DState.ts`:**
   - Sync controls with ProjectionState via existing projection state mutations/subscriptions.
   - Define Layer3D state schema (e.g., layout: {canvasSize, layerSpacing, partitionPadding}, display: {showEdges, showLabels, showPartitions, nodeScale, edgeOpacity}, camera: {fov, moveSpeed}).
   - Debounce updates (300–500ms) and ensure backward compatibility with other projection types.

3. **Camera Improvements:**
   - Smooth camera transitions (tweening)
   - Focus on node (click to zoom)
   - Reset camera button
   - Save/restore camera position in state

4. **Node Interactivity:**
   - Click to select node (highlight)
   - Hover to show tooltip with node details
   - Double-click to focus camera on node
   - VR controller raycasting support

**Success Criteria:**
- All controls update visualization in real-time and persist via ProjectionState.
- Camera focuses smoothly on clicked nodes.
- No regressions to other projection types; Layer3D state is scoped by projection.type.

---

### Phase 5: Text Labels & Visual Polish

**Goal:** Make the visualization production-ready with readable text and professional appearance. (New primitives allowed; no requirement to reuse Force3D visuals.)

**Tasks:**

1. **Label Rendering:**
   - Use `three-spritetext` (like Force3D) or A-Frame text component
   - Billboard labels (always face camera)
   - LOD system: Only show labels for nearby nodes
   - Configurable label size and color per layer

2. **Lighting & Shadows:**
   ```typescript
   <a-scene shadow="type: pcfsoft">
     <a-light type="ambient" color="#888" />
     <a-light type="directional" position="10 20 10" intensity="0.8"
              shadow="cast: true" />
   </a-scene>
   ```

3. **Visual Effects:**
   - Node hover highlighting (emissive color)
   - Selected node outline (wireframe overlay)
   - Particle effects for active edges
   - Layer boundaries (optional grid planes)

4. **Layer Visualization:**
   - Semi-transparent layer planes as floor/ceiling
   - Layer labels anchored in 3D space
   - Color-coded layer backgrounds

**Success Criteria:**
- Labels are readable at various distances
- Lighting makes depth perception clear
- Selected nodes are clearly highlighted
- Professional, polished appearance

---

### Phase 6: Narrative System (Stories/Tours) 🔮 FUTURE

**Goal:** Implement guided narrative mode for data storytelling.

**Tasks:**

1. **Story Data Structure:**
   ```typescript
   interface Story {
     id: string
     title: string
     steps: StoryStep[]
   }

   interface StoryStep {
     targetNodeId: string
     duration: number
     label: string
     cameraPosition?: Vector3  // Optional override
   }
   ```

2. **Narrative Controls:**
   - Play/Pause/Stop buttons
   - Next/Previous step
   - Progress indicator
   - Timeline scrubbing

3. **Camera Animation:**
   - Implement "dolly" system from design doc
   - Smooth interpolation between story steps
   - Configurable animation curves (ease-in-out)
   - Auto-advance with configurable timing

4. **Story Management:**
   - Store stories in ProjectionState
   - UI to create/edit stories in projection settings
   - Select target nodes from graph
   - Sequence editor with drag-and-drop

**Success Criteria:**
- Stories play smoothly with camera following path
- Users can create stories via UI
- Stories can be embedded in exports
- VR mode supports narrative playback

**Deferred Rationale:** This is a complex feature that requires the base visualization to be solid. Focus on core functionality first, then add storytelling capabilities.

---

## Technical Considerations

### Performance Optimization

**For Large Graphs (>500 nodes):**
- Use THREE.js InstancedMesh for nodes instead of individual `<a-box>` entities (Layer3D-specific implementation).
- Frustum culling for labels and edges.
- Level-of-detail (LOD) system for distant objects.
- Lazy loading of edges (only render visible connections).

**Rendering Strategy:**
- < 100 nodes: Individual A-Frame entities (easier to interact with)
- 100-500 nodes: Optimize but keep individual entities
- \> 500 nodes: Switch to instanced rendering, implement custom picking

### A-Frame Integration Patterns

**Component Registration:**
```typescript
// Register custom A-Frame components in Layer3DScene
AFRAME.registerComponent('node-interaction', {
  init() {
    this.el.addEventListener('click', (e) => {
      // Handle node click
    })
  }
})
```

**React-A-Frame Considerations:**
- Use `react-aframe` package for better React integration
- OR: Use refs and imperative DOM manipulation
- Avoid recreating entities on every render (use refs + updates)

### VR Support

**Desktop Mode (Default):**
- WASD + mouse look controls
- Click to interact
- Standard web UI overlay (Leva controls)

**VR Mode (Optional):**
- Enable via `<a-scene vr-mode-ui="enabled: true">`
- VR controller raycasting for node selection
- 3D control panel within scene (buttons as interactive entities)
- Voice commands (future: Web Speech API)

### State Synchronization

**GraphQL Mutations:**
- Debounce control changes (300–500ms).
- Batch multiple changes into single mutation.
- Handle mutation failures gracefully (revert UI).

**Subscriptions:**
- Listen to `projectionStateUpdated` for external changes.
- Merge incoming state with local changes (conflict resolution).
- Update layout when graph structure changes.

## Data Model Extensions (Optional/Future)

To fully leverage Layer3D capabilities, consider adding:

**Backend Schema Changes:**
```sql
ALTER TABLE nodes ADD COLUMN parent_id TEXT;
ALTER TABLE nodes ADD COLUMN weight FLOAT DEFAULT 1.0;
ALTER TABLE nodes ADD COLUMN node_type TEXT DEFAULT 'leaf'; -- 'partition' | 'leaf'
```

**GraphQL Schema:**
```graphql
type Node {
  id: ID!
  label: String
  layer: String
  parentId: ID         # For hierarchy
  weight: Float        # For sizing in treemap
  nodeType: NodeType   # PARTITION | LEAF
  # ... existing fields
}
```

**Migration Strategy:**
- Make fields optional (nullable)
- Provide sensible defaults
- Layer3D works without these, but uses them if present

## Testing Strategy

### Unit Tests
- Layout calculations (layercake algorithm)
- Orthogonal routing logic
- State synchronization helpers

### Integration Tests
- GraphQL query/mutation flows
- Subscription updates trigger re-renders
- Control changes persist correctly

### Visual Tests
- Screenshot comparison for known datasets
- VR mode rendering (manual testing)
- Performance benchmarks with large graphs

### User Acceptance Testing
- Create Layer3D projection from workbench
- Verify all controls work
- Test with various graph sizes and structures
- Export and verify standalone version works

## Configuration Schema

**ProjectionState for Layer3D:**
```typescript
interface Layer3DState {
  version: string  // Schema version for migrations
  layout: {
    canvasSize: number
    layerSpacing: number
    partitionPadding: number
    algorithm: 'treemap' | 'grid' | 'radial'  // Future layout options
  }
  display: {
    showEdges: boolean
    showLabels: boolean
    showPartitions: boolean
    nodeScale: number
    edgeOpacity: number
    labelThreshold: number  // Min distance to show labels
  }
  camera: {
    position: { x: number, y: number, z: number }
    target: { x: number, y: number, z: number }
    fov: number
  }
  stories?: Story[]  // Phase 6
  customLayerColors?: Record<string, string>  // Override layer colors
}
```

## Migration from Force3D

**For Existing Projections:**
- Users can switch projection type via settings UI (future enhancement)
- Backend mutation: `updateProjectionType(id, newType)`
- Reinitialize ProjectionState with type-specific defaults
- Preserve common settings where applicable (e.g., layer colors)

## Development Workflow

### Setup Development Environment
```bash
cd projections-frontend
npm install aframe d3-hierarchy three
npm install -D @types/aframe
npm run dev
```

### Local Testing
1. Start backend: `cargo run --bin layercake-api`
2. Start frontend: `npm run dev` in `projections-frontend/`
3. Create test projection in workbench with type `layer3d`
4. Open projection and iterate on implementation

### Code Style
- Follow existing TypeScript conventions in projections-frontend
- Use functional components with hooks
- Extract complex logic into custom hooks
- Document complex algorithms with comments
- Keep components under 300 lines (split if larger)

## Dependencies

**New NPM Packages:**
- `aframe` (^1.4.0): Core 3D rendering engine
- `react-aframe` (^4.6.0): React bindings for A-Frame (optional)
- `d3-hierarchy` (^3.1.2): Treemap layout calculations
- `three` (^0.155.0): Already used by Force3D, needed for advanced features
- `three-spritetext` (^1.8.0): Already used for labels

**TypeScript Definitions:**
- `@types/aframe`
- `@types/d3-hierarchy`

## Success Metrics

**Phase 1 Complete:**
- ✅ Basic 3D visualization renders
- ✅ Nodes positioned by layer (Y-axis)
- ✅ Camera controls work

**Phase 2 Complete:**
- ✅ Treemap layout implemented
- ✅ Hierarchical containment visible
- ✅ Configuration controls functional

**Phase 3 Complete:**
- ✅ Edges render with orthogonal routing
- ✅ Edge labels readable
- ✅ Performance acceptable (<100ms render time for 100 nodes)

**Phase 4 Complete:**
- ✅ All controls persist to ProjectionState
- ✅ Interactive node selection works
- ✅ Camera focus animations smooth

**Phase 5 Complete:**
- ✅ Labels readable at all distances (LOD works)
- ✅ Professional appearance with lighting/shadows
- ✅ Visual feedback for interactions

**Production Ready:**
- ✅ Works in desktop and VR modes
- ✅ Exports to standalone HTML
- ✅ Handles graphs up to 1000 nodes
- ✅ No console errors or warnings
- ✅ Documentation complete

## Next Steps

1. **Immediate:** Start with Phase 1 implementation
   - Set up A-Frame in projections-frontend
   - Create basic Layer3DScene component
   - Hook into existing app.tsx conditional rendering

2. **Short Term:** Complete Phases 2-3
   - Treemap layout implementation
   - Orthogonal edge routing
   - Basic interactivity

3. **Medium Term:** Phase 4-5
   - Polish controls and state management
   - Visual refinements and performance optimization

4. **Long Term:** Phase 6
   - Narrative/story system for guided tours
   - Advanced VR interactions
   - Additional layout algorithms

## References

- Original Design Document: `docs/projections-lc.md`
- A-Frame Documentation: https://aframe.io/docs/
- D3 Hierarchy: https://d3js.org/d3-hierarchy
- Force3D Implementation: `projections-frontend/src/app.tsx` (reference for patterns)
- Standalone Export Pattern: https://github.com/michiel/standalone-pack

## Open Questions

1. **Hierarchy Detection:** What edge labels/types indicate containment?
   - Need to define semantic meaning or add explicit parent_id field

2. **Default Camera Position:** What's the best initial view?
   - Bird's eye (top-down) vs. perspective (angled) vs. side view?

3. **VR Priority:** How important is VR mode for initial release?
   - Can defer full VR optimization to Phase 6 if desktop is priority

4. **Performance Targets:** What's the max graph size we need to support?
   - Affects choice of rendering strategy (entities vs. instanced mesh)

5. **Export Format:** Should Layer3D exports include A-Frame or use simpler THREE.js?
   - A-Frame adds ~1MB to bundle but simplifies implementation
   - THREE.js direct is smaller but requires more custom code
