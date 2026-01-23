# ADR 004: Layer3D Projection Architecture

## Status

Accepted

## Date

2025-12-18

## Context

We are implementing the Layer3D ("Layercake") projection type to provide a hierarchical 3D visualisation of graph data using vertical layer stratification and horizontal treemap layout. This projection type will coexist with the existing Force3D projection, requiring careful architectural decisions to avoid conflicts.

### Key Challenges

1. **THREE.js Version Conflict:**
   - Force3D uses standard `three@0.182.0`
   - A-Frame 1.7.1 uses `super-three@0.173.5` (A-Frame's fork of THREE.js r173)
   - Cannot share THREE.js code between projections

2. **A-Frame Integration:**
   - A-Frame uses entity-component-system (imperative)
   - React uses declarative rendering
   - Need pattern that balances performance and maintainability

3. **Breaking Changes in A-Frame 1.7:**
   - Removed `physicallyCorrectLights` property
   - Removed WebVR (replaced with WebXR)
   - Updated to THREE.js r173 with modern physically-based rendering

## Decision

### 1. Directory Structure: Separate Subdirectories

**Decision:** Force3D and Layer3D code **must** be in separate subdirectories to isolate THREE.js versions.

```
projections-frontend/src/projections/
├── force3d-projection/
│   ├── Force3DScene.tsx
│   ├── Force3DControls.tsx
│   └── ... (uses three@0.182.0)
└── layer3d-projection/
    ├── Layer3DScene.tsx
    ├── components/
    │   ├── Layer3DNode.tsx
    │   ├── Layer3DEdge.tsx
    │   └── Layer3DControls.tsx
    ├── hooks/
    │   ├── useLayercakeLayout.ts
    │   ├── useLayer3DCamera.ts
    │   └── useLayer3DState.ts
    └── lib/
        ├── layercake-layout.ts
        ├── orthogonal-routing.ts
        └── layer3d-validation.ts
        (uses super-three@0.173.5 via aframe)
```

**Rationale:**
- Prevents accidental THREE.js version mixing
- Clear code ownership and responsibility
- Enables independent updates and maintenance
- Webpack/Vite can properly tree-shake unused code

**Constraints:**
- NO shared THREE.js code between projections
- Each projection imports its own THREE.js dependencies
- Shared code must be THREE.js-agnostic (GraphQL, state management, utilities)

### 2. A-Frame + React Integration: Hybrid Pattern

**Decision:** Use **hybrid pattern** where React manages state and A-Frame manages 3D scene imperatively.

**Pattern:**
```typescript
// Layer3DScene.tsx
const Layer3DScene: React.FC<Props> = ({ graphData, layers }) => {
  const sceneRef = useRef<HTMLElement>(null)

  // Register A-Frame components once
  useEffect(() => {
    if (!AFRAME.components['layer3d-node-interaction']) {
      AFRAME.registerComponent('layer3d-node-interaction', {
        init() {
          this.el.addEventListener('click', (e) => {
            // Handle node click
          })
        }
      })
    }
  }, [])

  // Update scene imperatively when data changes
  useEffect(() => {
    const scene = sceneRef.current
    if (!scene) return
    updateNodes(scene, graphData)
  }, [graphData])

  return (
    <a-scene ref={sceneRef}>
      {/* Initial structure */}
    </a-scene>
  )
}
```

**Rationale:**
- **NOT react-aframe:** Performance overhead, complex state sync, limited community support
- **NOT pure imperative:** Loses React's declarative benefits, complex state management
- **Hybrid balances:** React for state/UI, A-Frame for 3D performance

**Constraints:**
- React components return A-Frame JSX
- Updates happen via refs and imperative DOM manipulation
- Avoid recreating entities on every render (use refs + updates)

### 3. Component Naming: `layer3d-*` Prefix

**Decision:** Prefix all custom A-Frame components with `layer3d-`.

**Examples:**
- `layer3d-node-interaction`
- `layer3d-edge-renderer`
- `layer3d-camera-controls`

**Rationale:**
- Prevents naming conflicts with Force3D or other A-Frame components
- Clear namespace for Layer3D-specific functionality
- Easy to identify Layer3D components in debugging

**Implementation:**
```typescript
// Check existence before registering
if (!AFRAME.components['layer3d-node-interaction']) {
  AFRAME.registerComponent('layer3d-node-interaction', { /* ... */ })
}
```

**Note:** A-Frame doesn't support component unregistration cleanly, so check before registering.

### 4. Coordinate System: Y+ Up, Z- Forward, Right-Handed

**Decision:** Use standard 3D coordinate system with Y-axis for layer stratification.

**Conventions:**
- **Y+ is up:** Layer stratification (layers stack upward)
- **Z- is forward:** A-Frame default camera direction
- **X+ is right:** Standard right-handed system
- **Units:** Meters (1 unit = 1 meter) for correct VR scale

**Layer Y-Positions:**
- Layer 0: Y = 0
- Layer N: Y = N × layerSpacing (default layerSpacing = 10)

**Camera Default:**
- Position calculated from graph bounding box
- Distance formula: `distance = boundingBoxSize / (2 * tan(fov/2)) * 1.1` (10% padding)
- Default FOV: 60 degrees
- Look at graph centre point

**Rationale:**
- Y+ up is standard in 3D visualisation (matches user mental model)
- Consistent with A-Frame defaults
- VR scale correctness (1 unit = 1 meter)
- Predictable layer positioning

### 5. Lighting Setup: Modern PBR (A-Frame 1.7)

**Decision:** Use modern physically-based rendering with THREE.js r173 defaults.

**Configuration:**
```typescript
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

  {/* Hemisphere light for sky/ground color variation */}
  <a-light type="hemisphere" groundColor="#444444" skyColor="#AAAAAA" intensity="0.4" />
</a-scene>
```

**Rationale:**
- A-Frame 1.7 removed `physicallyCorrectLights` (THREE.js r165+ removed `useLegacyLights`)
- Modern PBR is now default behaviour
- Provides good depth perception with shadows
- Hemisphere light adds realism with sky/ground colour variation

**Breaking Change Addressed:**
- ❌ ~~`physicallyCorrectLights` property~~ (removed)
- ✅ Modern PBR (default in THREE.js r173)

### 6. Error Handling Strategy

**Decision:** Validate early, fallback gracefully, log warnings for debugging.

**Principles:**
1. **Validate GraphQL data before layout calculation**
   - Detect cycles in hierarchy (DFS algorithm)
   - Sanitise missing/invalid layers
   - Provide sensible defaults

2. **Fallback gracefully on errors**
   - Cycles: Break at arbitrary edge, log warning, continue
   - Missing layers: Create default "Unnamed Layer 0"
   - Invalid positions: Replace NaN/Infinity with (0, 0, 0)

3. **Handle WebGL context loss**
   - Detect via `webglcontextlost` event
   - Show user-friendly reload prompt
   - Log error details for debugging

4. **GraphQL failure recovery**
   - Retry queries 3x with exponential backoff
   - Auto-reconnect subscriptions
   - Show non-intrusive "Reconnecting..." indicator

**Rationale:**
- Prevent crashes from invalid data
- Maintain user experience during transient failures
- Provide debugging information without exposing internals to users

## Consequences

### Positive

1. **Version Isolation:** Separate subdirectories prevent THREE.js conflicts
2. **Clear Ownership:** Each projection has distinct codebase
3. **Maintainability:** Hybrid pattern balances React and A-Frame benefits
4. **Modern Rendering:** A-Frame 1.7 provides latest WebXR and PBR features
5. **Graceful Degradation:** Error handling prevents crashes

### Negative

1. **Code Duplication:** Some utilities may be duplicated between projections
2. **Learning Curve:** Team must understand both React and A-Frame patterns
3. **Imperative Updates:** Hybrid pattern requires manual DOM manipulation
4. **Version Constraints:** A-Frame updates must be tested for THREE.js compatibility

### Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| THREE.js version leak | High | Linting rules, code review, separate directories |
| A-Frame hot reload issues | Medium | Document workaround (full page refresh), educate team |
| Performance with large graphs | High | Implement LOD, frustum culling, instanced rendering |
| WebGL context loss | Medium | Implement detection and graceful error messages |

## Implementation Notes

### Phase 0 Deliverables

- ✅ This ADR document
- ⏳ Test data fixtures (`src/fixtures/layer3d-test-data.ts`)
- ⏳ Debugging guide (`docs/debugging-layer3d.md`)
- ⏳ Directory structure (`src/projections/layer3d-projection/`)

### Dependencies Verified

```json
{
  "aframe": "1.7.1",           // THREE.js r173 (super-three@0.173.5)
  "d3-hierarchy": "3.1.2",     // Treemap layout
  "d3-scale": "4.0.2",         // Scaling utilities
  "three": "0.182.0",          // Force3D only
  "three-spritetext": "1.10.0" // Force3D only
}
```

### THREE.js Version Matrix

| Package | THREE.js Version | Usage |
|---------|-----------------|-------|
| Force3D (`3d-force-graph`) | 0.182.0 (standard) | Force3D projection only |
| Layer3D (`aframe`) | 0.173.5 (super-three) | Layer3D projection only |
| `three-spritetext` | 0.182.0 | Force3D only (not compatible with Layer3D) |

**CRITICAL:** Layer3D must use A-Frame's built-in text components, not `three-spritetext`.

## References

- A-Frame 1.7.0 Release Notes: https://aframe.io/blog/aframe-v1.7.0/
- A-Frame 1.7.0 Documentation: https://aframe.io/docs/1.7.0/
- A-Frame Changelog: https://github.com/aframevr/aframe/blob/master/CHANGELOG.md
- THREE.js r173 Documentation: https://threejs.org/docs/
- Layer3D Implementation Plan: `plans/layer-3d-projection.md`
- Original Design Document: `docs/projections-lc.md`

## Approval

- **Author:** Claude Sonnet 4.5
- **Date:** 2025-12-18
- **Status:** Accepted (auto-approved for implementation)

## Changelog

- 2025-12-18: Initial ADR created for Layer3D architecture
