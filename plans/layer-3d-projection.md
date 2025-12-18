# Layer3D Projection Implementation Plan

## Overview

Implement the "Layercake" 3D visualization as the `layer3d` projection type in the projections-frontend. Use the existing ProjectionGraph GraphQL feed (same as Force3D) as the sole data source; no CSV ingestion. Layer3D will introduce new A-Frame primitives (no reuse required from Force3D visuals) but must integrate with the existing projections routing and ProjectionState GraphQL pipeline.

### Critical Implementation Notes

**Directory Structure:** Force3D and Layer3D **must** be in separate subdirectories:
- `src/projections/force3d-projection/` - Existing Force3D implementation
- `src/projections/layer3d-projection/` - New Layer3D implementation (this plan)

**THREE.js Version Isolation:**
- **Force3D:** Uses `three@0.182.0` (standard THREE.js)
- **Layer3D:** Uses `super-three@0.173.5` via `aframe@1.7.1` (A-Frame's THREE.js fork)
- **DO NOT** share THREE.js code/components between projections
- Each projection imports its own THREE.js dependencies

**A-Frame Version:** 1.7.1 (installed)
- Breaking changes from 1.4: removed `physicallyCorrectLights`, removed WebVR, updated to THREE.js r173
- See [A-Frame 1.7.0 Release Notes](https://aframe.io/blog/aframe-v1.7.0/) and [Changelog](https://github.com/aframevr/aframe/blob/master/CHANGELOG.md)

**Dependencies Already Installed:**
- ✅ `aframe@1.7.1`
- ✅ `d3-hierarchy@3.1.2`
- ✅ `d3-scale@4.0.2`
- ✅ `@types/aframe`

**Key Characteristics:**
- **Vertical (Y-axis):** Represents layer stratification (e.g., software layers, security levels)
- **Horizontal (XZ-plane):** Represents containment/grouping via treemap layout with clear fallbacks when hierarchy metadata is missing
- **Edges:** Orthogonal routing to reduce visual clutter
- **Interactivity:** Camera controls, node selection; narrative/story mode deferred
- **Tech Stack:** A-Frame for 3D rendering, D3 for layout; ProjectionGraph GraphQL as input

## Coordinate System Conventions

**Standard 3D Coordinate System:**
- **Y+ is up** (layer stratification)
- **Z- is forward** (A-Frame default camera direction)
- **X+ is right**
- **Right-handed coordinate system**
- **Units:** Meters (for correct VR scale; 1 unit = 1 meter)

**Camera Default:**
- Position calculated from graph bounding box
- Distance formula: `distance = boundingBoxSize / (2 * tan(fov/2)) * 1.1` (10% padding)
- Default FOV: 60 degrees
- Look at graph center point

**Layer Y-Positions:**
- Layer 0: Y = 0
- Layer N: Y = N * layerSpacing (default layerSpacing = 10)
- Layers stack upward (positive Y)

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
- Primary: use `attrs.belongs_to` or `attrs.parent_id` if present
- Secondary: infer from edges with semantic relation (e.g., `attrs.relation in {"contains","parent_of"}`)
- Fallback: flat hierarchy grouped by layer only; treemap operates per layer with no nesting
- **Validation:** Detect cycles using DFS; break cycles at arbitrary edge and log warning

2) **Sizing/weight**
- Use `node.weight` when present; fallback to degree (edge count) or constant 1 for treemap sizing
- **Validation:** Ensure weight > 0, default to 1 if invalid

3) **Layer mapping**
- Use `node.layer` to map Y stratification
- **Validation:** Create default "Unnamed Layer 0" if layers array is empty
- **Validation:** Assign nodes with missing/invalid layer to first available layer

## Architecture

### Component Structure (Separate subdirectories for each projection type)

**IMPORTANT:** Force3D and Layer3D projection code must be in **separate subdirectories** to avoid code conflicts and THREE.js version issues.

```
projections-frontend/
├── src/
│   ├── projections/
│   │   ├── force3d-projection/          # Force3D projection (existing)
│   │   │   ├── Force3DScene.tsx
│   │   │   ├── Force3DControls.tsx
│   │   │   └── ...
│   │   └── layer3d-projection/          # Layer3D projection (new)
│   │       ├── Layer3DScene.tsx         # Main A-Frame scene component
│   │       ├── components/
│   │       │   ├── Layer3DNode.tsx      # Individual node rendering
│   │       │   ├── Layer3DEdge.tsx      # Orthogonal edge rendering
│   │       │   └── Layer3DControls.tsx  # Leva controls for Layer3D
│   │       ├── hooks/
│   │       │   ├── useLayercakeLayout.ts    # D3-based layout calculator
│   │       │   ├── useLayer3DCamera.ts      # Camera movement system
│   │       │   └── useLayer3DState.ts       # ProjectionState management
│   │       └── lib/
│   │           ├── layercake-layout.ts      # Core layout algorithms
│   │           ├── orthogonal-routing.ts    # Edge path calculation
│   │           └── layer3d-validation.ts    # Data validation and sanitisation
│   └── app.tsx                          # Render appropriate scene based on projection.type
```

**THREE.js Version Isolation:**
- **Force3D**: Uses `three@0.182.0` (standard THREE.js)
- **Layer3D**: Uses `super-three@0.173.5` (A-Frame's THREE.js fork)
- **DO NOT** share THREE.js code/components between projections due to version incompatibility
- Each projection must manage its own THREE.js imports and dependencies

### A-Frame + React Integration Pattern (Hybrid Approach)

**Decision:** Use **hybrid pattern** for optimal balance:
- **React:** Manages state, GraphQL subscriptions, and UI controls (Leva)
- **A-Frame:** Manages 3D scene via refs and imperative updates
- **Pattern:** React components return A-Frame JSX, but updates happen via refs

**Why not react-aframe?**
- Performance overhead for large graphs
- Complex state synchronisation issues
- Limited community support

**Why not pure imperative?**
- Loses React's declarative benefits
- Harder to maintain and debug
- State management becomes complex

**Implementation approach:**
```typescript
// Layer3DScene.tsx
const sceneRef = useRef<HTMLElement>(null)

useEffect(() => {
  // Register custom A-Frame components once
  if (!AFRAME.components['layer3d-node']) {
    AFRAME.registerComponent('layer3d-node', { /* ... */ })
  }
}, [])

useEffect(() => {
  // Update scene imperatively when data changes
  const scene = sceneRef.current
  if (!scene) return

  // Create/update entities via DOM manipulation
  updateNodes(scene, graphData)
}, [graphData])

return <a-scene ref={sceneRef}>{/* Initial structure */}</a-scene>
```

**Component Naming Convention:**
- Prefix all custom A-Frame components with `layer3d-` (e.g., `layer3d-node-interaction`)
- Check existence before registering: `if (!AFRAME.components['layer3d-foo']) { ... }`
- No unregistration needed (A-Frame doesn't support it cleanly)

## Implementation Phases

### Phase 0: Architecture Decisions & Setup 🎯 START HERE

**Goal:** Make critical architectural decisions and set up development environment before implementation begins.

**Decisions to Document:**
1. ✅ A-Frame integration pattern: **Hybrid** (React state + imperative scene updates)
2. ✅ Coordinate system: **Y+ up, Z- forward, right-handed, meters**
3. ✅ Camera initial position: **Calculated from bounding box**
4. ✅ Component naming: **`layer3d-*` prefix**
5. ✅ Error handling strategy: **Validate, fallback, log warnings**

**Tasks:**

1. **Audit Dependencies:**
   ```bash
   cd projections-frontend
   npm list three
   npm list aframe
   # Document versions in ADR
   ```
   - Check Three.js version used by Force3D (`3d-force-graph` dependency)
   - Verify A-Frame compatible with that Three.js version
   - Document version constraints in plan

2. **Create Architecture Decision Record (ADR):**
   - Create `docs/adr/004-layer3d-architecture.md`
   - Document A-Frame + React integration pattern
   - Document coordinate system conventions
   - Document component naming strategy
   - **Document THREE.js version isolation:**
     - A-Frame 1.7.1 uses THREE.js r173 (super-three@0.173.5)
     - Force3D uses THREE.js 0.182.0 (standard three)
     - Separate subdirectories required: no shared THREE.js code
     - Import statements must be projection-specific
   - **Document A-Frame 1.7 breaking changes:**
     - Removed physicallyCorrectLights (use default modern lighting)
     - Removed WebVR (use WebXR instead)
     - Updated THREE.js to r173 with ES module support
   - Get review/approval before proceeding

3. **Set Up Test Data Fixtures:**
   - Create `projections-frontend/src/fixtures/layer3d-test-data.ts`
   - Include test cases:
     - Minimal: 3 nodes, 2 edges, 2 layers
     - Hierarchical: Parent-child relationships via attrs
     - Flat: No hierarchy metadata (fallback test)
     - Large: 100 nodes, 200 edges, 5 layers (performance test)
     - Edge cases: Cycles, missing layers, invalid data

4. **Configure Development Tools:**
   - Add A-Frame Inspector integration (Ctrl+Alt+I)
   - Configure Chrome DevTools for 3D debugging
   - Set up performance monitoring (FPS, triangle count)
   - Document debugging workflow

5. **Verify Installed Dependencies:**
   ```bash
   cd projections-frontend
   npm list aframe d3-hierarchy d3-scale three
   # Verify installed: aframe@^1.7.1, d3-hierarchy@^3.1.2, d3-scale@^4.0.2
   ```
   - **A-Frame version:** 1.7.1 (uses THREE.js r173 via super-three@0.173.5)
   - **THREE.js version conflict:** Force3D uses three@0.182.0, A-Frame uses super-three@0.173.5
   - **Resolution:** Keep projections in separate subdirectories, do not share THREE.js code
   - Run `npm test` and `npm run build` to ensure no breakage

**Success Criteria:**
- ✅ Architecture decisions documented in ADR and reviewed
- ✅ Dependency version compatibility verified (no conflicts)
- ✅ Test data fixtures available for all test cases
- ✅ Development environment working with HMR
- ✅ A-Frame Inspector loads without errors
- ✅ Performance monitoring tools configured

**Deliverables:**
- `docs/adr/004-layer3d-architecture.md` (documents THREE.js version isolation)
- `projections-frontend/src/fixtures/layer3d-test-data.ts`
- `docs/debugging-layer3d.md`
- Updated directory structure: `src/projections/layer3d-projection/`
- Version compatibility matrix documented

**Key Decisions Documented:**
- ✅ A-Frame 1.7.1 with THREE.js r173 (super-three)
- ✅ Separate subdirectories for Force3D (three@0.182) and Layer3D (super-three@0.173.5)
- ✅ No shared THREE.js code between projections
- ✅ A-Frame 1.7 breaking changes accounted for

---

### Phase 1: Foundation & Basic Layout ⭐

**Goal:** Replace the "Coming Soon" placeholder with basic 3D visualisation of nodes positioned by layer using ProjectionGraph data.

**Tasks:**

1. **Create `lib/layer3d-validation.ts`:**
   - Validate ProjectionGraph data structure
   - Detect and break cycles in hierarchy (DFS algorithm)
   - Sanitise missing/invalid layers (create default layer)
   - Sanitise missing/invalid node properties
   - Return validated data or throw descriptive errors

2. **Create `useLayercakeLayout.ts` hook:**
   - Accept ProjectionGraph data (nodes, edges, layers)
   - Validate input using `layer3d-validation.ts`
   - Build hierarchy: prefer attrs.belongs_to/parent_id; else edge semantics; else flat-by-layer fallback
   - Phase 1 layout: Simple grid layout on XZ plane (rows/columns based on sqrt of node count)
   - Map Y by layer index: `y = layerIndex * layerSpacing`
   - Calculate graph bounding box for camera positioning
   - Return positioned nodes `{ id, x, y, z, width, depth, height, color, label, isPartition, layerId, boundingBox }`

3. **Create `Layer3DScene.tsx`:**
   - Location: `src/projections/layer3d-projection/Layer3DScene.tsx`
   - Hybrid pattern: React manages state, imperative scene updates
   - A-Frame 1.7 scene with:
     - Camera with WASD + mouse look controls
     - Updated lighting setup (A-Frame 1.7 removed physicallyCorrectLights):
       - Ambient lighting (color: #888, intensity: 0.6)
       - Directional light (position: "10 20 10", intensity: 0.8)
       - Hemisphere light for sky/ground variation
     - Shadow system (type: pcfsoft)
     - Stats component for debugging (FPS, triangles)
   - Render nodes as `<a-box>` entities at calculated positions
   - Apply layer colors from ProjectionGraph
   - Register custom component `layer3d-node-interaction` for click handling
   - Calculate and set camera initial position from bounding box
   - Handle WebGL context loss (show reload prompt)
   - **Note:** A-Frame 1.7 uses THREE.js r173; do not import from Force3D's THREE.js (r182)

4. **Update `app.tsx`:**
   - Import from new directory: `import Layer3DScene from './projections/layer3d-projection/Layer3DScene'`
   - Import from Force3D directory: `import Force3DScene from './projections/force3d-projection/Force3DScene'`
   - Add conditional rendering: `{isLayer3d ? <Layer3DScene /> : <Force3DScene />}`
   - Pass GraphQL data (graphData, layers, loading state) to Layer3DScene
   - Ensure error boundary catches Layer3D errors without crashing app
   - Show loading spinner while GraphQL query pending
   - **Note:** Both projections use different THREE.js versions; keep imports separate

5. **Add Error Handling:**
   - GraphQL query failure: Show error toast, retry 3x with exponential backoff
   - Validation errors: Show specific error message in UI (e.g., "Graph contains cycles")
   - Layout calculation errors: Fall back to grid layout, log warning
   - WebGL not supported: Show fallback message with browser requirements

**Success Criteria:**
- ✅ Layer3D projections show nodes as coloured boxes in 3D space from GraphQL feed
- ✅ Nodes are vertically separated by layer with correct spacing
- ✅ Flat grouping works when no hierarchy metadata is present
- ✅ Camera positioned correctly to view entire graph (10% padding)
- ✅ Camera controls work (WASD + mouse look)
- ✅ **Performance:** 60fps with 50 nodes on typical hardware (2019 MacBook Pro / equivalent)
- ✅ **Performance:** Layout calculation completes in <50ms for 50 nodes
- ✅ No console errors or warnings
- ✅ A-Frame Inspector works (Ctrl+Alt+I)
- ✅ Error handling prevents crashes with invalid data

**Test Plan:**
1. Create a Layer3D projection via workbench
2. Open it and verify nodes render in 3D
3. Verify different layers have different Y positions (measure: should be layerSpacing apart)
4. Verify layer colours are applied correctly (visual comparison with Force3D)
5. Test with fixture data (minimal, hierarchical, flat, large)
6. Test error cases (empty graph, invalid data, cycles)
7. Measure FPS (should be 60fps) and layout time (should be <50ms)
8. Test camera controls (WASD movement, mouse look)

**Performance Targets:**
- FPS: 60fps sustained (use Stats component)
- Layout time: <50ms (use console.time)
- Triangle count: <50k (use Stats component)
- Memory: <200MB (use Chrome Task Manager)

---

### Phase 2: Treemap Layout & Hierarchy

**Goal:** Implement proper XZ-plane layout using D3 treemap for containment relationships with explicit fallbacks when hierarchy is absent.

**Tasks:**

1. **Enhance Hierarchy Detection:**
   - Parse `attrs.belongs_to` and `attrs.parent_id` from node attributes
   - Parse edges with semantic relations: `attrs.relation in {"contains","parent_of","has","includes"}`
   - Build D3 hierarchy structure using `d3.hierarchy()`
   - Detect cycles using DFS; break at arbitrary edge and log warning
   - Fallback: Group nodes by layer only (no nesting); treemap operates per layer

2. **Update `useLayercakeLayout.ts` with Treemap:**
   ```typescript
   import { hierarchy, treemap } from 'd3-hierarchy'
   import { scaleLinear } from 'd3-scale'

   export const useLayercakeLayout = (nodes, edges, layers, config) => {
     const { canvasSize, layerSpacing, partitionPadding } = config

     // 1. Build hierarchy (with cycle detection)
     const hierarchyData = buildHierarchy(nodes, edges)

     // 2. Calculate node weights
     const weighted = hierarchyData.map(n => ({
       ...n,
       value: n.weight || calculateDegree(n.id, edges) || 1
     }))

     // 3. Apply D3 treemap to each layer separately (if hierarchy per layer)
     const root = hierarchy(weighted)
       .sum(d => d.value)
       .sort((a, b) => b.value - a.value)

     const layoutFn = treemap()
       .size([canvasSize, canvasSize])
       .paddingOuter(partitionPadding)
       .paddingInner(partitionPadding / 2)

     layoutFn(root)

     // 4. Map to 3D coordinates
     const layerMap = new Map(layers.map((l, i) => [l.layerId, i]))

     return root.descendants().map(d => {
       const layerIndex = layerMap.get(d.data.layer) || 0
       return {
         id: d.data.id,
         isPartition: !!d.children,
         x: (d.x0 + d.x1) / 2 - canvasSize / 2,
         y: layerIndex * layerSpacing,
         z: (d.y0 + d.y1) / 2 - canvasSize / 2,
         width: d.x1 - d.x0,
         depth: d.y1 - d.y0,
         height: d.children ? (layers.length * layerSpacing) : 1,
         color: getLayerColor(d.data.layer, layers),
         label: d.data.label,
         layerId: d.data.layer
       }
     })
   }
   ```

3. **Add Leva Controls for Layout:**
   ```typescript
   const controls = useControls('Layer3D Layout', {
     canvasSize: { value: 100, min: 50, max: 200, step: 10 },
     layerSpacing: { value: 10, min: 5, max: 30, step: 1 },
     partitionPadding: { value: 2, min: 0, max: 10, step: 0.5 },
   })
   ```

4. **Implement Node Type Differentiation:**
   - **Partition nodes** (have children): Wireframe pillars with opacity 0.3
   - **Leaf nodes**: Solid boxes with opacity 0.9
   - Update `Layer3DNode.tsx` to handle both types
   - Partition nodes span all layers (Y from 0 to maxLayer * layerSpacing)
   - Leaf nodes only at their specific layer height

5. **Add State Persistence:**
   - Sync Leva controls with ProjectionState
   - Use `useLayer3DState.ts` hook to debounce updates (300ms)
   - Mutation: `saveProjectionState(id, { layout: { canvasSize, layerSpacing, partitionPadding } })`
   - Subscription: Listen to `projectionStateUpdated` and update controls

**Success Criteria:**
- ✅ Nodes with children render as wireframe containers (pillar shape)
- ✅ Leaf nodes are contained within their parents (XZ-plane)
- ✅ When hierarchy is absent, nodes are still grouped per layer with stable layout
- ✅ Treemap layout respects node weights when provided
- ✅ Configuration changes update layout in real-time (<100ms recalculation)
- ✅ Settings persist via ProjectionState across page refreshes
- ✅ **Performance:** 60fps with 100 nodes, 30fps with 500 nodes
- ✅ **Performance:** Layout calculation <100ms for 200 nodes
- ✅ Cycle detection works (log warning, break cycle, continue rendering)

**Test Plan:**
1. Test with hierarchical fixture (nodes with parent_id)
2. Test with edge-based hierarchy (edges with relation="contains")
3. Test with flat fixture (no hierarchy metadata)
4. Test with cycles (should detect and break)
5. Test weight-based sizing (nodes with different weights)
6. Test configuration changes (adjust sliders, verify immediate update)
7. Test persistence (change settings, refresh page, verify settings retained)
8. Measure performance (layout time, FPS)

**Performance Targets:**
- FPS: 60fps for <100 nodes, 30fps for 500 nodes
- Layout time: <100ms for 200 nodes, <500ms for 500 nodes
- Re-layout on config change: <100ms (memoisation required)

---

### Phase 3: Orthogonal Edge Routing

**Goal:** Implement clean, orthogonal edge routing to visualise relationships without clutter.

**Tasks:**

1. **Research Orthogonal Routing Algorithms:**
   - Review existing libraries (e.g., `dagre`, `elkjs`)
   - Decide on routing strategy:
     - **Simple 3-point:** source → vertical lift → target (Phase 3 MVP)
     - **Channel-based:** Allocate gutter lanes to avoid overlaps (future)
     - **Bundled:** Group parallel edges (future)
   - Document chosen algorithm and trade-offs

2. **Create `lib/orthogonal-routing.ts`:**
   ```typescript
   import { Vector3 } from 'three'

   export interface EdgeSegment {
     start: Vector3
     end: Vector3
     color: string
     width: number
   }

   export function calculateOrthogonalPath(
     source: { x: number, y: number, z: number },
     target: { x: number, y: number, z: number },
     options: {
       gutterOffset?: number
       edgeIndex?: number  // For multi-edge offset
       totalEdges?: number
     }
   ): EdgeSegment[] {
     // Simple 3-point routing for Phase 3:
     // 1. Start at source
     // 2. Rise/fall vertically to target Y
     // 3. Move horizontally to target

     const midpoint = {
       x: source.x,
       y: target.y,
       z: source.z
     }

     return [
       { start: new Vector3(source.x, source.y, source.z),
         end: new Vector3(midpoint.x, midpoint.y, midpoint.z),
         color: options.color || '#555',
         width: options.width || 0.3 },
       { start: new Vector3(midpoint.x, midpoint.y, midpoint.z),
         end: new Vector3(target.x, target.y, target.z),
         color: options.color || '#555',
         width: options.width || 0.3 }
     ]
   }
   ```

3. **Implement `Layer3DEdge.tsx`:**
   - Render edges as A-Frame line components or THREE.js Line2
   - Calculate path using `calculateOrthogonalPath`
   - Support edge labels positioned at path midpoint
   - Visibility toggle from Leva controls
   - Color edges based on:
     - Source layer color (default)
     - Target layer color (option)
     - Custom color from edge attributes (option)
   - Width based on edge weight (if present)

4. **Add Edge Controls:**
   ```typescript
   const edgeControls = useControls('Layer3D Edges', {
     showEdges: true,
     edgeOpacity: { value: 0.6, min: 0, max: 1, step: 0.1 },
     edgeWidth: { value: 0.3, min: 0.1, max: 2, step: 0.1 },
     edgeColorMode: { value: 'source', options: ['source', 'target', 'custom'] },
   })
   ```

5. **Optimise Edge Rendering:**
   - **<100 edges:** Individual `<a-entity line>` components per segment
   - **100-500 edges:** Switch to THREE.js BufferGeometry
   - **>500 edges:** Implement frustum culling, only render visible edges
   - Add performance monitoring for edge rendering

6. **Add Edge Labels:**
   - Use `three-spritetext` (same as Force3D)
   - Position at midpoint of edge path
   - Billboard (always face camera)
   - Visibility based on camera distance (LOD)
   - Toggle via Leva control

**Success Criteria:**
- ✅ Edges render as orthogonal paths (no diagonal lines between layers)
- ✅ Edges don't overlap with partition boxes
- ✅ Edge labels are readable and positioned correctly at path midpoint
- ✅ Toggle edges on/off without breaking layout
- ✅ Edge colors reflect source/target layer correctly
- ✅ Edge width scales with weight
- ✅ **Performance:** 60fps with 100 nodes + 200 edges
- ✅ **Performance:** 30fps with 500 nodes + 1000 edges
- ✅ **Performance:** Edge calculation <50ms for 200 edges
- ✅ Settings persist via ProjectionState

**Test Plan:**
1. Test with simple graph (10 nodes, 15 edges)
2. Test with complex graph (100 nodes, 200 edges)
3. Test with cross-layer edges (different Y positions)
4. Test with multiple edges between same nodes (visual inspection for clarity)
5. Test edge label readability (zoom in/out)
6. Test performance (FPS with many edges)
7. Test toggle edges on/off
8. Test edge coloring modes (source, target, custom)

**Performance Targets:**
- FPS: 60fps for 200 edges, 30fps for 1000 edges
- Edge calculation time: <50ms for 200 edges
- Re-render on toggle: <100ms (use memoisation)

---

### Phase 4: Interactive Controls & State Management

**Goal:** Add comprehensive controls and persist user preferences via ProjectionState (existing GraphQL mutations/subscriptions).

**Tasks:**

1. **Create `Layer3DControls.tsx`:**
   ```typescript
   import { useControls, folder } from 'leva'

   export const useLayer3DControls = () => {
     return useControls('Layer3D', {
       Layout: folder({
         canvasSize: { value: 100, min: 50, max: 200, step: 10 },
         layerSpacing: { value: 10, min: 5, max: 30, step: 1 },
         partitionPadding: { value: 2, min: 0, max: 10, step: 0.5 },
       }),
       Display: folder({
         showEdges: true,
         showLabels: true,
         showPartitions: true,
         nodeScale: { value: 1, min: 0.5, max: 3, step: 0.1 },
         edgeOpacity: { value: 0.6, min: 0, max: 1, step: 0.1 },
         labelDistance: { value: 20, min: 5, max: 50, step: 5 }, // LOD threshold
       }),
       Camera: folder({
         fov: { value: 60, min: 30, max: 120, step: 5 },
         moveSpeed: { value: 5, min: 1, max: 20, step: 1 },
         resetCamera: button(() => resetCameraPosition()),
       }),
       Debug: folder({
         showStats: true,
         showBoundingBoxes: false,
         showLayerPlanes: false,
       })
     })
   }
   ```

2. **Implement `useLayer3DState.ts`:**
   - Sync Leva controls with ProjectionState via GraphQL mutations
   - Debounce updates (300ms) to avoid excessive mutations
   - Load initial state from ProjectionState on mount
   - Handle subscription updates (merge remote changes with local state)
   - Type-safe state schema:
     ```typescript
     interface Layer3DState {
       version: '1.0.0'
       layout: {
         canvasSize: number
         layerSpacing: number
         partitionPadding: number
       }
       display: {
         showEdges: boolean
         showLabels: boolean
         showPartitions: boolean
         nodeScale: number
         edgeOpacity: number
         labelDistance: number
       }
       camera: {
         position: { x: number, y: number, z: number }
         target: { x: number, y: number, z: number }
         fov: number
         moveSpeed: number
       }
       debug: {
         showStats: boolean
         showBoundingBoxes: boolean
         showLayerPlanes: boolean
       }
     }
     ```

3. **Implement Camera Improvements:**
   - **Smooth transitions:** Use TWEEN.js or A-Frame animation component
   - **Focus on node:** Click node to focus camera with smooth zoom (duration: 1s)
   - **Reset camera:** Button to return to initial calculated position
   - **Save camera position:** Persist camera position in ProjectionState
   - **Restore on mount:** Load saved camera position if available

4. **Implement Node Interactivity:**
   - **Click to select:** Highlight selected node (emissive material)
   - **Hover tooltip:** Show node details (id, label, layer, degree)
   - **Double-click to focus:** Zoom camera to node
   - **Keyboard shortcuts:**
     - `R`: Reset camera
     - `E`: Toggle edges
     - `L`: Toggle labels
     - `Escape`: Deselect node
   - **VR controller support:** Raycasting for node selection (basic)

5. **Add Error Handling:**
   - GraphQL mutation failures: Retry 3x, revert UI on final failure, show toast
   - Subscription disconnect: Auto-reconnect with exponential backoff, show "reconnecting..." indicator
   - Camera calculation errors: Fall back to default position, log warning

**Success Criteria:**
- ✅ All controls update visualisation in real-time (<100ms response)
- ✅ Settings persist via ProjectionState across page refreshes
- ✅ Camera focuses smoothly on clicked nodes (1s animation)
- ✅ Hover tooltips show correct node information
- ✅ Keyboard shortcuts work reliably
- ✅ No regressions to other projection types (Force3D still works)
- ✅ Layer3D state is scoped by projection.type (no interference)
- ✅ **Performance:** 60fps maintained during camera animations
- ✅ **Performance:** Control changes apply within 100ms
- ✅ Error handling prevents crashes (invalid state, network failures)

**Test Plan:**
1. Test all Leva controls (adjust each, verify update)
2. Test state persistence (change settings, refresh, verify retained)
3. Test camera focus (click nodes, verify smooth zoom)
4. Test camera reset (move camera, press R or button, verify reset)
5. Test node selection (click, verify highlight, escape to deselect)
6. Test hover tooltips (hover nodes, verify correct data)
7. Test keyboard shortcuts (E, L, R, Escape)
8. Test error scenarios (disconnect network, verify auto-reconnect)
9. Test performance (FPS during animations, control responsiveness)
10. Test Force3D (verify no interference)

**Performance Targets:**
- FPS: 60fps sustained during camera animations
- Control response time: <100ms from slider change to visual update
- State save debounce: 300ms (balance responsiveness vs. network load)
- Camera focus animation: 1000ms (smooth, not jarring)

---

### Phase 5: Text Labels & Visual Polish

**Goal:** Make the visualisation production-ready with readable text and professional appearance.

**Tasks:**

1. **Implement Label Rendering System:**
   - Use `three-spritetext` (same as Force3D for consistency)
   - Billboard labels (always face camera via `look-at="#rig"`)
   - LOD system: Only show labels for nodes within `labelDistance` threshold
   - Configurable label size based on layer or node importance
   - Label colors from layer textColor or node labelColor

2. **Implement Lighting & Shadows (A-Frame 1.7 compatible):**
   ```typescript
   // A-Frame 1.7 uses THREE.js r173, which removed useLegacyLights
   // Default lighting behavior changed; physicallyCorrectLights no longer available
   <a-scene shadow="type: pcfsoft">
     {/* Ambient light for overall illumination */}
     <a-light type="ambient" color="#888888" intensity="0.6" />

     {/* Directional light for depth perception and shadows */}
     <a-light
       type="directional"
       position="10 20 10"
       intensity="0.8"
       castShadow="true"
       shadow-camera-near="0.5"
       shadow-camera-far="50"
       shadow-camera-left="-20"
       shadow-camera-right="20"
       shadow-camera-top="20"
       shadow-camera-bottom="-20"
     />

     {/* Hemisphere light for sky/ground color variation (THREE.js r173) */}
     <a-light type="hemisphere" groundColor="#444444" skyColor="#AAAAAA" intensity="0.4" />
   </a-scene>
   ```
   **Note:** A-Frame 1.7 removed physicallyCorrectLights property since THREE.js r165+ no longer supports useLegacyLights. The default lighting now uses modern physically-based rendering.

3. **Implement Visual Effects:**
   - **Node hover highlighting:** Change emissive color on hover
   - **Selected node outline:** Add wireframe overlay or scale slightly
   - **Particle effects for edges:** Optional animated particles flowing along edges (use A-Frame particle system)
   - **Layer boundaries:** Optional semi-transparent grid planes at each layer Y-position
   - **Transition animations:** Fade in nodes when first loaded, animate layout changes

4. **Implement Layer Visualization:**
   - **Layer planes:** Semi-transparent planes at each layer Y-position
     - Color from layer backgroundColor with opacity 0.1
     - Grid lines for spatial reference
     - Toggle via Leva control
   - **Layer labels:** 3D text anchored at layer corner
     - Shows layer name
     - Always visible (not LOD-culled)
     - Positioned at (canvasSize/2 + 5, layerY, canvasSize/2 + 5)

5. **Add Accessibility Features:**
   - **Keyboard navigation:** Tab through nodes, Enter to select
   - **Reduced motion:** Respect `prefers-reduced-motion` media query (disable animations)
   - **High contrast mode:** Detect and adjust colors
   - **Screen reader support:** Add ARIA labels to interactive elements (limited in WebXR)

6. **Polish Camera Controls:**
   - Add smooth damping to mouse look (not jarring)
   - Add zoom controls (scroll wheel or ±keys)
   - Add pan controls (Shift+WASD or middle mouse button)
   - Clamp camera position (don't let user fly infinitely far away)

**Success Criteria:**
- ✅ Labels are readable at various distances (LOD working)
- ✅ Labels only show for nearby nodes (within threshold)
- ✅ Lighting makes depth perception clear (can judge distances)
- ✅ Shadows render correctly (nodes cast shadows on layer planes)
- ✅ Selected nodes are clearly highlighted (visual feedback)
- ✅ Hover effects work smoothly (no jank)
- ✅ Layer planes and labels aid spatial orientation
- ✅ Professional, polished appearance (comparable to commercial tools)
- ✅ **Performance:** 60fps maintained with all effects enabled (<100 nodes)
- ✅ **Performance:** Shadows don't drop FPS below 30fps
- ✅ **Accessibility:** Keyboard navigation works, reduced motion respected

**Test Plan:**
1. Test label LOD (move camera, verify labels appear/disappear at threshold)
2. Test label readability (various distances, camera angles)
3. Test lighting (verify depth perception, shadows)
4. Test node highlighting (hover, click, verify visual feedback)
5. Test layer planes (toggle on, verify correct positioning and colors)
6. Test layer labels (verify visible, correct names, good positioning)
7. Test accessibility (keyboard navigation, reduced motion)
8. Test camera controls (smooth damping, zoom, pan, clamp)
9. Test performance (FPS with all effects enabled)
10. Visual inspection (professional appearance, no glitches)

**Performance Targets:**
- FPS: 60fps with labels, lighting, shadows enabled (<100 nodes)
- FPS: 30fps with all effects enabled (500 nodes)
- Label LOD: Update every 100ms (balance performance vs. responsiveness)
- Shadow quality: Medium (balance quality vs. performance)

---

### Phase 6: Narrative System (Stories/Tours) 🔮 FUTURE

**Goal:** Implement guided narrative mode for data storytelling (deferred to future release).

**Tasks:**

1. **Story Data Structure:**
   ```typescript
   interface Story {
     id: string
     title: string
     description?: string
     steps: StoryStep[]
     autoPlay: boolean
     looping: boolean
   }

   interface StoryStep {
     targetNodeId: string
     duration: number  // milliseconds
     label: string
     cameraPosition?: { x: number, y: number, z: number }  // Optional override
     cameraTarget?: { x: number, y: number, z: number }
     highlightNodes?: string[]  // Additional nodes to highlight
     annotation?: string  // Rich text annotation
   }
   ```

2. **Narrative Controls:**
   - Play/Pause/Stop buttons in UI overlay
   - Next/Previous step buttons
   - Progress indicator (current step / total steps)
   - Timeline scrubbing (seek to specific step)
   - Speed control (0.5x, 1x, 2x playback speed)

3. **Camera Animation System:**
   - Implement "dolly" system from design doc
   - Smooth interpolation between story steps using TWEEN.js or similar
   - Configurable animation curves (ease-in-out, linear, custom)
   - Auto-advance with configurable timing
   - Pause on user interaction (camera controls)

4. **Story Management UI:**
   - Story creation/editing UI in projection settings
   - Node selection from graph (click to add to sequence)
   - Sequence editor with drag-and-drop reordering
   - Preview mode (play story without saving)
   - Export story as JSON

5. **Story Storage:**
   - Store stories in ProjectionState
   - Multiple stories per projection
   - Stories included in standalone exports

**Success Criteria:**
- ✅ Stories play smoothly with camera following path
- ✅ Users can create stories via UI
- ✅ Stories can be edited and reordered
- ✅ Stories can be embedded in exports
- ✅ VR mode supports narrative playback
- ✅ Auto-advance works reliably
- ✅ User can interrupt and resume stories

**Deferred Rationale:** This is a complex feature that requires the base visualisation to be solid. Focus on core functionality first (Phases 1-5), then add storytelling capabilities in a future release.

---

## Error Handling Strategy

### GraphQL Failures

**Query Failures:**
- Retry up to 3 times with exponential backoff (1s, 2s, 4s)
- Show error toast with specific message: "Failed to load projection data. Retrying..."
- On final failure: Show error page with "Retry" button and support contact

**Subscription Disconnect:**
- Auto-reconnect with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- Show "Reconnecting..." indicator in UI (non-intrusive banner)
- On successful reconnect: Refetch full graph to sync state
- On repeated failures: Show error message and "Reload" button

**Mutation Failures:**
- Retry up to 3 times with exponential backoff
- On failure: Revert UI changes, show error toast
- Log error details for debugging (but don't expose to user)

### Layout Calculation Errors

**Circular Hierarchy:**
- Detect cycles using DFS before layout calculation
- Break cycle at arbitrary edge (choose edge with highest betweenness centrality)
- Log warning: "Detected cycle in hierarchy: [nodeA] -> [nodeB] -> [nodeA]. Breaking edge [nodeB] -> [nodeA]"
- Continue with acyclic hierarchy

**Invalid Node Positions:**
- Validate all coordinates are finite numbers
- Replace NaN/Infinity with default position (0, 0, 0)
- Log warning with node ID

**Missing Layers:**
- If layers array is empty: Create default layer "Unnamed Layer 0"
- If node references non-existent layer: Assign to first available layer, log warning

**Container with Too Many Children:**
- If container has >1000 children: Log warning, proceed anyway (treemap may be unreadable)
- Future: Add pagination or hierarchy flattening for large containers

### Rendering Errors

**WebGL Context Loss:**
- Detect via canvas `webglcontextlost` event
- Pause rendering, show alert: "Graphics context lost. Please reload the page."
- Provide "Reload" button
- Log error details

**Out of Memory:**
- Catch out-of-memory errors during scene creation
- Reduce render quality automatically (disable shadows, reduce texture resolution)
- Show warning: "Low memory. Rendering quality reduced."
- If still failing: Show error and suggest reducing graph size

**No WebGL Support:**
- Detect via `AFRAME.utils.device.checkHeadsetConnected()` or manual check
- Show fallback page with browser requirements:
  - Chrome 90+, Firefox 88+, Safari 15+, Edge 90+
  - WebGL 2.0 support required
  - Hardware acceleration enabled
- Provide link to browser compatibility info

### User Input Validation

**Invalid Configuration Values:**
- Clamp numeric inputs to valid ranges (e.g., canvasSize 50-200)
- Reject negative values for sizes/distances
- Log warning if clamping occurs

**Invalid Camera Positions:**
- Validate camera position is not NaN/Infinity
- Reset to default position if invalid
- Log warning

### Error Reporting

**Development Mode:**
- Show detailed error messages in console
- Include stack traces
- Display error overlays with component names

**Production Mode:**
- Show user-friendly error messages (no stack traces)
- Log errors to backend telemetry (if available)
- Include error ID for support reference

---

## Debugging Guide

### Tools

**A-Frame Inspector:**
- Press `Ctrl+Alt+I` (Windows/Linux) or `Cmd+Option+I` (Mac) in scene to open
- Inspect entity hierarchy, positions, materials
- Modify properties in real-time
- View performance stats

**Chrome DevTools:**
- **Elements panel:** Inspect A-Frame DOM
- **Console:** View logs, errors, warnings
- **Performance panel:** Record and analyse frame timing
- **3D View (Layers panel):** Visualise 3D DOM structure
- **Memory panel:** Check for memory leaks

**Stats Component:**
- Add `stats` attribute to `<a-scene stats>` to show FPS, triangles, calls
- Shows real-time performance metrics in top-left corner

**Leva Debug Panel:**
- Enable "Debug" folder in Leva controls
- Toggle `showBoundingBoxes` to visualise treemap rectangles
- Toggle `showLayerPlanes` to verify layer Y-positions
- Toggle `showStats` to show/hide performance stats

### Common Issues

**Entities Not Rendering:**
- Check position is within camera frustum (use A-Frame Inspector)
- Verify entity has valid geometry and material
- Check scale is not 0
- Verify parent entity is visible

**Low FPS:**
- Use Stats component to check triangle count (target: <50k)
- Check draw calls (target: <500)
- Disable shadows if FPS drops below 30
- Reduce node count or simplify geometry
- Check for memory leaks (use Chrome Memory panel)

**Layout Looks Wrong:**
- Log layout output: `console.log('Layout:', layoutData)`
- Verify treemap calculation: Check `d.x0, d.x1, d.y0, d.y1` values
- Check layer assignments: Verify node.layer maps to valid layer
- Use `showBoundingBoxes` debug option to visualise layout rectangles

**Camera Not Moving:**
- Verify WASD controls enabled: `<a-entity wasd-controls>`
- Check camera is not locked by other component
- Verify no JavaScript errors preventing input handling
- Check browser has focus (click on scene)

**WebGL Context Loss:**
- Check browser console for "webglcontextlost" event
- Verify hardware acceleration enabled in browser settings
- Reduce texture sizes and draw calls
- Check for too many entities (browser WebGL limits)

**Memory Leaks:**
- Use Chrome Task Manager (Shift+Esc) to monitor memory
- Check for uncleaned event listeners (use `removeEventListener`)
- Verify A-Frame components have proper `remove()` methods
- Dispose Three.js geometries and materials when unmounting

### Debugging Workflow

**Step 1: Reproduce Issue**
- Use test fixtures to reproduce consistently
- Document exact steps to reproduce
- Note browser, OS, hardware details

**Step 2: Isolate Problem**
- Disable features one by one (edges, labels, shadows)
- Use minimal test case (smallest graph that shows issue)
- Check if issue occurs in A-Frame Inspector

**Step 3: Gather Data**
- Check browser console for errors/warnings
- Record performance profile in Chrome DevTools
- Take screenshots/video of issue
- Export ProjectionState JSON for analysis

**Step 4: Fix & Verify**
- Implement fix with logging to confirm it works
- Test with original reproduction case
- Test with larger graphs to verify no regressions
- Remove debug logging before commit

### Development Tips

**Hot Reload Issues:**
- A-Frame components don't always hot-reload cleanly
- If scene breaks: Full page refresh (Ctrl+Shift+R)
- If components don't update: Clear browser cache

**Performance Testing:**
- Test with large graphs (100, 500, 1000 nodes)
- Test on lower-end hardware (not just dev machine)
- Monitor FPS over time (check for degradation)
- Use Chrome Performance panel to identify bottlenecks

**State Debugging:**
- Use React DevTools to inspect component state
- Log ProjectionState changes to console
- Verify GraphQL subscriptions are receiving updates
- Check for stale closures in useEffect hooks

---

## Live Update Strategy

### When Graph Data Changes

**Node Added:**
- Calculate position for new node using existing layout algorithm
- Animate node into position (fade in + scale up, duration: 500ms)
- Update related edges
- No full layout recalculation (maintain stability)

**Node Removed:**
- Animate node out (fade out + scale down, duration: 500ms)
- Remove related edges
- Optional: Recalculate layout to fill gap (user preference)

**Node Updated (label, color, layer):**
- Update properties in place (no position change)
- If layer changed: Animate vertical transition to new layer Y-position (duration: 1s)

**Edge Added:**
- Calculate orthogonal path
- Animate edge into view (fade in, duration: 500ms)

**Edge Removed:**
- Animate edge out (fade out, duration: 500ms)

**Edge Weight Changed:**
- Animate width change (duration: 300ms)
- Optionally recalculate orthogonal path if routing changed

### When ProjectionState Changes (External)

**Control Setting Changed by Another User:**
- Merge remote changes with local changes (conflict resolution)
- If no local unsaved changes: Apply remote changes immediately
- If local unsaved changes: Show notification "Remote changes available. Reload?"
- Apply changes with animation (smooth transition)

**Camera Position Changed:**
- Don't force camera update (user may be navigating)
- Show indicator that another user's camera is at different position
- Optional: "Follow user X" mode for collaborative viewing

### Subscription Event Handling

**`projectionGraphUpdated` Event:**
```typescript
useSubscription(GRAPH_SUB, {
  onData: ({ data }) => {
    const updated = data.projectionGraphUpdated
    // Diff with current graph
    const { added, removed, updated } = diffGraphs(currentGraph, updated)
    // Apply changes with animations
    animateNodeChanges(added, removed, updated)
    setCurrentGraph(updated)
  }
})
```

**`projectionStateUpdated` Event:**
```typescript
useSubscription(STATE_SUB, {
  onData: ({ data }) => {
    const remoteState = data.projectionStateUpdated.stateJson
    // Merge with local state
    const merged = mergeStates(localState, remoteState, localDirty)
    if (localDirty) {
      // Show notification instead of force-applying
      showNotification('Remote changes available. Reload?')
    } else {
      // Apply immediately
      setLocalState(merged)
    }
  }
})
```

### Conflict Resolution Strategy

**When Local and Remote States Conflict:**
1. **User has unsaved local changes:** Prefer local, show notification
2. **User has no unsaved changes:** Prefer remote, apply immediately
3. **User is actively interacting:** Don't interrupt, queue remote changes
4. **User is idle (no input for 10s):** Apply remote changes smoothly

**Conflict Detection:**
- Track "dirty" flag for local changes not yet saved
- Track timestamp of last local change
- Track timestamp of last remote change received
- If remote timestamp > local save timestamp: Conflict possible

**Notification UI:**
- Non-intrusive banner at top: "New changes from [user/system]. [Reload] [Dismiss]"
- Auto-dismiss after 30s if user clicks elsewhere
- If dismissed: Apply changes on next page load

---

## Export Considerations

### Bundle Size Strategy

**A-Frame vs. THREE.js Direct:**
- **A-Frame:** ~1.5MB gzipped, easier implementation, full VR support
- **THREE.js direct:** ~500KB gzipped, more code, limited VR
- **Decision:** Use A-Frame for Phase 1-5, consider THREE.js optimization for Phase 6 if bundle size becomes issue

**Optimisation Techniques:**
- Load A-Frame from CDN in standalone exports (reduce bundle size)
- Use tree-shaking for unused A-Frame components
- Compress embedded graph data with gzip
- Use base64-encoded data URI for small graphs (<100KB)
- Use separate data.js file for large graphs (>100KB)

**Export Bundle Structure:**
```
export-<projection-name>.zip
├── index.html                 # Entry point
├── data.js                    # Graph data: window.PROJECTION_EXPORT = {...}
├── projection.js              # Layer3D renderer (bundled)
├── projection.css             # Styles
└── assets/
    ├── aframe.min.js          # A-Frame from CDN or bundled
    └── ...                    # Other dependencies
```

**Loading Strategy in Export:**
```html
<!-- index.html -->
<script src="https://aframe.io/releases/1.7.1/aframe.min.js"></script>
<script src="data.js"></script>  <!-- Defines window.PROJECTION_EXPORT -->
<script src="projection.js"></script>  <!-- Loads data from window scope -->
```
**Note:** Using A-Frame 1.7.1 from CDN (~1.5MB gzipped). Version 1.7 includes THREE.js r173 (super-three).

### Export Options

**User Configurable:**
- Include A-Frame from CDN (smaller) vs. bundle (offline-capable)
- Include controls (Leva) vs. read-only view
- Include full graph vs. filtered subset
- Include stories (Phase 6) vs. static view

**Backend Export Service:**
```rust
pub async fn export_projection(id: i32, options: ExportOptions) -> Result<Vec<u8>> {
    // 1. Fetch projection + graph + state
    let projection = fetch_projection(id).await?;
    let graph = fetch_projection_graph(id).await?;
    let state = fetch_projection_state(id).await?;

    // 2. Serialise to JSON
    let data_json = json!({
        "projection": projection,
        "graph": graph,
        "state": state
    });

    // 3. Bundle HTML + JS + data
    let html = include_str!("../templates/layer3d-export.html");
    let js = include_str!("../dist/projection.js");
    let data_js = format!("window.PROJECTION_EXPORT = {};", data_json);

    // 4. Create ZIP
    let zip = create_zip(vec![
        ("index.html", html),
        ("projection.js", js),
        ("data.js", &data_js),
    ])?;

    Ok(zip)
}
```

**Bundle Size Targets:**
- Small graph (<100 nodes): <2MB total
- Medium graph (100-500 nodes): <5MB total
- Large graph (500-1000 nodes): <10MB total

---

## Configuration Schema

**ProjectionState for Layer3D:**
```typescript
interface Layer3DState {
  version: '1.0.0'  // Semver string for migrations
  layout: {
    canvasSize: number          // 100
    layerSpacing: number        // 10
    partitionPadding: number    // 2
    algorithm: 'treemap' | 'grid' | 'radial'  // Default: 'treemap'
  }
  display: {
    showEdges: boolean          // true
    showLabels: boolean         // true
    showPartitions: boolean     // true
    nodeScale: number           // 1.0
    edgeOpacity: number         // 0.6
    edgeWidth: number           // 0.3
    edgeColorMode: 'source' | 'target' | 'custom'  // 'source'
    labelDistance: number       // 20 (LOD threshold)
  }
  camera: {
    position: { x: number, y: number, z: number }  // Calculated default
    target: { x: number, y: number, z: number }    // Graph center
    fov: number                 // 60
    moveSpeed: number           // 5
  }
  debug: {
    showStats: boolean          // false
    showBoundingBoxes: boolean  // false
    showLayerPlanes: boolean    // false
  }
  stories?: Story[]             // Phase 6
  customLayerColors?: Record<string, { body: string, label: string }>  // Override layer colors
}
```

**State Versioning & Migration:**
- Version format: Semver (major.minor.patch)
- Major version: Breaking changes (manual migration required)
- Minor version: New features (backward compatible)
- Patch version: Bug fixes (backward compatible)

**Migration Strategy:**
```typescript
function migrateLayer3DState(state: any): Layer3DState {
  const version = state.version || '0.0.0'

  // Migrate 0.x.x -> 1.0.0
  if (version.startsWith('0.')) {
    state = migrate_0_to_1(state)
  }

  // Future migrations
  // if (version === '1.0.0') state = migrate_1_0_to_1_1(state)

  return state
}
```

**Default State:**
```typescript
const DEFAULT_LAYER3D_STATE: Layer3DState = {
  version: '1.0.0',
  layout: {
    canvasSize: 100,
    layerSpacing: 10,
    partitionPadding: 2,
    algorithm: 'treemap'
  },
  display: {
    showEdges: true,
    showLabels: true,
    showPartitions: true,
    nodeScale: 1.0,
    edgeOpacity: 0.6,
    edgeWidth: 0.3,
    edgeColorMode: 'source',
    labelDistance: 20
  },
  camera: {
    position: { x: 0, y: 50, z: 100 },  // Recalculated on mount
    target: { x: 0, y: 0, z: 0 },
    fov: 60,
    moveSpeed: 5
  },
  debug: {
    showStats: false,
    showBoundingBoxes: false,
    showLayerPlanes: false
  }
}
```

---

## Testing Strategy

### Unit Tests

**Layout Calculations:**
```typescript
// layercake-layout.test.ts
describe('useLayercakeLayout', () => {
  it('calculates correct Y positions for layers', () => {
    const layout = calculateLayout(testData, { layerSpacing: 10 })
    expect(layout[0].y).toBe(0)    // Layer 0
    expect(layout[1].y).toBe(10)   // Layer 1
  })

  it('detects and breaks cycles in hierarchy', () => {
    const cyclicData = createCyclicGraph()
    const layout = calculateLayout(cyclicData, {})
    expect(layout).toBeDefined()  // Should not throw
    expect(console.warn).toHaveBeenCalledWith(/cycle/)
  })

  it('falls back to grid layout when hierarchy missing', () => {
    const flatData = createFlatGraph()
    const layout = calculateLayout(flatData, {})
    expect(layout.every(n => !n.isPartition)).toBe(true)
  })
})
```

**Orthogonal Routing:**
```typescript
// orthogonal-routing.test.ts
describe('calculateOrthogonalPath', () => {
  it('creates 3-point path for cross-layer edges', () => {
    const source = { x: 0, y: 0, z: 0 }
    const target = { x: 10, y: 10, z: 10 }
    const segments = calculateOrthogonalPath(source, target, {})
    expect(segments).toHaveLength(2)
    expect(segments[0].end.y).toBe(target.y)  // Vertical lift first
  })

  it('handles edges within same layer', () => {
    const source = { x: 0, y: 5, z: 0 }
    const target = { x: 10, y: 5, z: 10 }
    const segments = calculateOrthogonalPath(source, target, {})
    expect(segments[0].start.y).toBe(segments[0].end.y)  // No vertical
  })
})
```

**State Synchronisation:**
```typescript
// useLayer3DState.test.ts
describe('useLayer3DState', () => {
  it('debounces state updates', async () => {
    const { result } = renderHook(() => useLayer3DState())
    result.current.updateLayout({ canvasSize: 150 })
    result.current.updateLayout({ canvasSize: 200 })

    await waitFor(() => {
      expect(mockMutation).toHaveBeenCalledTimes(1)  // Debounced
      expect(mockMutation).toHaveBeenCalledWith({ canvasSize: 200 })
    })
  })

  it('merges remote state changes correctly', () => {
    // Test subscription update handling
  })
})
```

### Integration Tests

**GraphQL Flow:**
```typescript
// layer3d-graphql.test.ts
describe('Layer3D GraphQL integration', () => {
  it('fetches projection data and renders scene', async () => {
    render(<Layer3DScene id="1" />)

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument()
    })

    expect(screen.getByRole('canvas')).toBeInTheDocument()
  })

  it('updates scene when subscription emits new data', async () => {
    // Mock subscription
    // Emit update
    // Verify scene updated
  })
})
```

**Control Changes:**
```typescript
// layer3d-controls.test.ts
describe('Layer3D controls', () => {
  it('updates layout when slider changes', async () => {
    render(<Layer3DScene id="1" />)

    const slider = screen.getByRole('slider', { name: /canvas size/i })
    fireEvent.change(slider, { target: { value: '150' } })

    await waitFor(() => {
      expect(mockMutation).toHaveBeenCalledWith({
        layout: expect.objectContaining({ canvasSize: 150 })
      })
    })
  })
})
```

### Visual Tests

**Screenshot Comparison:**
```typescript
// layer3d-visual.test.ts
describe('Layer3D visual regression', () => {
  it('matches baseline screenshot', async () => {
    const screenshot = await takeScreenshot(<Layer3DScene data={fixtureData} />)
    expect(screenshot).toMatchImageSnapshot()
  })
})
```

**Manual Testing Checklist:**
- [ ] VR mode renders correctly (tested on Oculus/Vive)
- [ ] Performance acceptable on low-end hardware (2019 MacBook Air)
- [ ] Large graphs render without crashing (1000 nodes)
- [ ] No visual glitches or z-fighting
- [ ] Labels readable at all camera distances
- [ ] Lighting looks professional
- [ ] Shadows render correctly

### Performance Benchmarks

**Automated Benchmarks:**
```typescript
// layer3d-performance.test.ts
describe('Layer3D performance', () => {
  it('maintains 60fps with 100 nodes', async () => {
    const fps = await measureFPS(<Layer3DScene data={generate100Nodes()} />)
    expect(fps).toBeGreaterThan(55)  // Allow 5fps margin
  })

  it('calculates layout in <100ms for 200 nodes', () => {
    const start = performance.now()
    calculateLayout(generate200Nodes(), {})
    const duration = performance.now() - start
    expect(duration).toBeLessThan(100)
  })
})
```

**Performance Test Matrix:**
| Node Count | Edge Count | Target FPS | Layout Time | Status |
|-----------|-----------|-----------|------------|--------|
| 50        | 75        | 60        | <50ms      | ✅     |
| 100       | 200       | 60        | <100ms     | ✅     |
| 500       | 1000      | 30        | <500ms     | 🔄     |
| 1000      | 2000      | 15        | <1000ms    | ❌     |

### User Acceptance Testing

**Test Scenarios:**
1. **Create projection:** Create Layer3D projection from workbench, verify it appears in list
2. **Open projection:** Click "Open", verify new tab/window opens with 3D scene
3. **Navigate scene:** Use WASD + mouse to move around, verify smooth controls
4. **Adjust settings:** Change layout/display settings, verify immediate visual update
5. **Persist settings:** Change settings, refresh page, verify settings retained
6. **Select nodes:** Click nodes, verify highlighting and tooltip
7. **Toggle features:** Toggle edges/labels/partitions, verify correct visibility
8. **Export projection:** Export to ZIP, extract, open index.html, verify standalone works
9. **Large graph:** Test with 500+ nodes, verify acceptable performance
10. **Error handling:** Disconnect network, verify auto-reconnect and error messages

---

## Dependencies

**Installed NPM Packages:**
- `aframe` (1.7.1): Core 3D rendering engine with THREE.js r173 (super-three@0.173.5)
- `d3-hierarchy` (^3.1.2): Treemap layout calculations
- `d3-scale` (^4.0.2): Scaling utilities for layout
- `three` (0.182.0): Standard THREE.js used by Force3D
- `three-spritetext` (1.10.0): Text labels (uses three@0.182.0)
- `3d-force-graph` (1.79.0): Force3D visualization (uses three@0.182.0)

**TypeScript Definitions:**
- `@types/aframe` (^1.2.0)
- `@types/d3-hierarchy` (^3.1.0)
- `@types/d3-scale` (^4.0.0)

**THREE.js Version Compatibility Matrix:**

| Package | THREE.js Version | Notes |
|---------|-----------------|-------|
| Force3D (`3d-force-graph`) | 0.182.0 | Standard THREE.js |
| Layer3D (`aframe`) | 0.173.5 (super-three) | A-Frame's fork of THREE.js |
| `three-spritetext` | 0.182.0 | Compatible with Force3D |

**CRITICAL:** A-Frame uses a **different THREE.js version** (super-three 0.173.5) than Force3D (three 0.182.0). This is why Layer3D and Force3D must be in **separate subdirectories** and cannot share THREE.js code.

**Dependency Audit Checklist:**
1. ✅ Run `npm list three aframe` - verified versions above
2. ✅ Check A-Frame THREE.js compatibility: https://aframe.io/docs/1.7.0/introduction/faq.html#which-version-of-three-js-does-a-frame-use
3. ✅ Verify `3d-force-graph` THREE.js version: Confirmed 0.182.0
4. ✅ Dependencies installed: `aframe@1.7.1`, `d3-hierarchy@3.1.2`, `d3-scale@4.0.2`
5. ✅ Document version constraints in ADR (Phase 0)

---

## Success Metrics

**Phase 0 Complete:**
- ✅ Architecture decisions documented in ADR
- ✅ All dependencies compatible (no conflicts)
- ✅ Test fixtures available for all test cases
- ✅ Dev environment working with HMR
- ✅ Debugging tools configured

**Phase 1 Complete:**
- ✅ Basic 3D visualisation renders from GraphQL data
- ✅ Nodes positioned by layer (Y-axis correct)
- ✅ Camera controls work smoothly
- ✅ 60fps with 50 nodes, layout time <50ms
- ✅ No console errors, A-Frame Inspector works
- ✅ Error handling prevents crashes

**Phase 2 Complete:**
- ✅ Treemap layout implemented with hierarchy detection
- ✅ Partition vs. leaf nodes render differently
- ✅ Configuration controls functional and persist
- ✅ 60fps with 100 nodes, 30fps with 500 nodes
- ✅ Layout calculation <100ms for 200 nodes
- ✅ Cycle detection works

**Phase 3 Complete:**
- ✅ Edges render with orthogonal routing (no diagonals)
- ✅ Edge labels readable at >2m distance
- ✅ 60fps with 100 nodes + 200 edges
- ✅ Edge calculation <50ms for 200 edges
- ✅ Settings persist

**Phase 4 Complete:**
- ✅ All controls persist to ProjectionState
- ✅ Interactive node selection works
- ✅ Camera focus animations smooth (1s duration)
- ✅ 60fps maintained during animations
- ✅ Control response time <100ms
- ✅ Error handling works (network failures, etc.)

**Phase 5 Complete:**
- ✅ Labels readable at all distances (LOD works)
- ✅ Professional appearance with lighting/shadows
- ✅ Visual feedback for interactions (hover, select)
- ✅ 60fps with all effects enabled (<100 nodes)
- ✅ Accessibility features work (keyboard nav, reduced motion)

**Production Ready:**
- ✅ Works in desktop and VR modes
- ✅ Exports to standalone HTML
- ✅ Handles graphs up to 1000 nodes (15fps minimum)
- ✅ No console errors or warnings
- ✅ Documentation complete (user guide, developer guide)
- ✅ All tests passing (unit, integration, visual)

---

## Next Steps

1. **Immediate:** Start with Phase 0
   - Document architecture decisions in ADR
   - Audit dependencies and verify compatibility
   - Set up test fixtures and debugging tools
   - Get architecture review and approval

2. **Short Term:** Complete Phases 1-2 (Foundation + Layout)
   - Basic 3D visualisation working
   - Treemap layout with hierarchy detection
   - State persistence via ProjectionState

3. **Medium Term:** Complete Phases 3-4 (Edges + Interactivity)
   - Orthogonal edge routing
   - Comprehensive controls and state management
   - Camera animations and node interactions

4. **Long Term:** Complete Phase 5 (Polish)
   - Labels, lighting, shadows
   - Visual effects and layer visualisation
   - Accessibility features

5. **Future:** Phase 6 (Narrative System)
   - Guided tours and storytelling
   - Advanced VR interactions
   - Additional layout algorithms

---

## References

- Original Design Document: `docs/projections-lc.md`
- Projections Implementation Plan: `plans/projections.md`
- **A-Frame 1.7 Documentation:** https://aframe.io/docs/1.7.0/
- **A-Frame 1.7 Release Notes:** https://aframe.io/blog/aframe-v1.7.0/
- **A-Frame 1.7 THREE.js FAQ:** https://aframe.io/docs/1.7.0/introduction/faq.html
- **A-Frame 1.7 Changelog:** https://github.com/aframevr/aframe/blob/master/CHANGELOG.md
- **A-Frame + THREE.js Integration:** https://aframe.io/docs/1.7.0/introduction/developing-with-threejs.html
- D3 Hierarchy: https://d3js.org/d3-hierarchy
- D3 Scale: https://d3js.org/d3-scale
- Force3D Implementation: `projections-frontend/src/projections/force3d-projection/` (reference for patterns)
- Standalone Export Pattern: https://github.com/michiel/standalone-pack
- WebGL Best Practices: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices
- THREE.js r173 Documentation: https://threejs.org/docs/index.html#manual/en/introduction/Creating-a-scene

---

## Open Questions

### ✅ Resolved

1. **A-Frame + React Integration:** Hybrid pattern (React state + imperative scene updates)
2. **Coordinate System:** Y+ up, Z- forward, right-handed, meters
3. **Camera Initial Position:** Calculate from bounding box with 10% padding
4. **Component Naming:** `layer3d-*` prefix for all custom A-Frame components

### ❓ To Be Decided

1. **Hierarchy Detection:** What specific edge `attrs.relation` values indicate containment?
   - **Proposal:** `["contains", "parent_of", "has", "includes"]`
   - **Action:** Review existing graph data, document in Phase 0

2. **VR Priority:** How important is VR mode for initial release?
   - **Proposal:** Desktop mode is priority, basic VR support in Phase 5, advanced VR in Phase 6
   - **Action:** Confirm with stakeholders in Phase 0

3. **Performance Targets:** What's the maximum graph size we need to support?
   - **Proposal:** Primary target 100 nodes (60fps), stretch target 500 nodes (30fps), absolute max 1000 nodes (15fps)
   - **Action:** Test with real-world graphs in Phase 1

4. **Export Format:** Should Layer3D exports include A-Frame from CDN or bundle?
   - **Proposal:** Offer both options (CDN for smaller size, bundled for offline use)
   - **Action:** Implement both in export service

5. **Accessibility Priority:** How much accessibility support is required?
   - **Proposal:** Basic keyboard navigation (Phase 4), reduced motion support (Phase 5), full accessibility (Phase 6)
   - **Action:** Confirm requirements with accessibility team

---

## Glossary

- **A-Frame:** Web framework for building VR experiences using HTML
- **Treemap:** Space-filling visualisation of hierarchical data
- **LOD (Level of Detail):** Technique to show/hide details based on distance
- **Orthogonal routing:** Edge routing with only right-angle turns (Manhattan routing)
- **Partition node:** Container node with children in hierarchy
- **Leaf node:** Terminal node with no children
- **ProjectionState:** GraphQL type storing user preferences for a projection
- **ProjectionGraph:** GraphQL type containing nodes/edges/layers for visualisation
- **Dolly:** Camera rig that moves smoothly between positions (cinematography term)
- **Frustum culling:** Optimisation technique to skip rendering objects outside camera view
- **WebGL context loss:** Browser event when GPU context is lost (tab backgrounded, etc.)
- **Billboard:** 3D object that always faces the camera
- **Emissive material:** Material that appears to emit light (used for highlights)
