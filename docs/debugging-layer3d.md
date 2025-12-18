# Layer3D Projection Debugging Guide

## Quick Reference

| Issue | Quick Fix | Details |
|-------|-----------|---------|
| Entities not rendering | Check position is within camera frustum | [View Details](#entities-not-rendering) |
| Low FPS | Disable shadows, check triangle count | [View Details](#low-fps) |
| Layout looks wrong | Log layout output, verify treemap | [View Details](#layout-looks-wrong) |
| Camera not moving | Verify WASD controls enabled | [View Details](#camera-not-moving) |
| WebGL context loss | Check hardware acceleration | [View Details](#webgl-context-loss) |
| Memory leaks | Check event listeners, dispose geometries | [View Details](#memory-leaks) |

## Development Tools

### A-Frame Inspector

**Open Inspector:** Press `Ctrl+Alt+I` (Windows/Linux) or `Cmd+Option+I` (Mac) while scene is focused

**Features:**
- Inspect entity hierarchy and properties
- Modify positions, materials, components in real-time
- View performance stats (FPS, triangles, draw calls)
- Toggle entity visibility
- Examine scene graph structure

**Common Uses:**
- Check if entity positions are correct
- Verify materials and geometries are loaded
- Debug positioning issues (entities outside frustum)
- Inspect camera position and rotation

### Chrome DevTools

**Elements Panel:**
- Inspect A-Frame DOM structure (entities are HTML elements)
- View entity attributes and component properties
- Use DOM tree to understand hierarchy

**Console:**
- View Layer3D logs and warnings
- Test layout calculations: `console.log('Layout:', layoutData)`
- Inspect THREE.js objects: Entities have `.object3D` property

**Performance Panel:**
- Record frame timing to identify bottlenecks
- Analyze JavaScript execution time
- Check for layout thrashing or excessive re-renders

**3D View (Layers Panel):**
- Visualise 3D DOM structure
- Rotate and zoom to understand spatial relationships
- Useful for debugging overlapping entities

**Memory Panel:**
- Check for memory leaks (watch memory over time)
- Take heap snapshots before/after actions
- Identify retained objects that should be garbage collected

### Stats Component

**Enable Stats:**
```typescript
<a-scene stats>
  {/* Scene contents */}
</a-scene>
```

**Displays:**
- **FPS:** Frames per second (target: 60fps)
- **MS:** Milliseconds per frame (target: <16.67ms)
- **MB:** Memory usage
- **Calls:** Draw calls (target: <500)
- **Triangles:** Triangle count (target: <50k)
- **Geometries:** Number of geometries in scene
- **Textures:** Number of loaded textures

**Interpreting:**
- FPS < 30: Performance issue (too many entities, expensive shaders)
- Draw calls > 500: Too many individual meshes (use instanced rendering)
- Triangles > 100k: Simplify geometry or implement LOD
- Memory growing: Potential memory leak (check disposal)

### Leva Debug Panel

**Enable Debug Controls:**
```typescript
const debugControls = useControls('Layer3D Debug', {
  showStats: true,
  showBoundingBoxes: false,
  showLayerPlanes: false,
  logLayout: button(() => console.log('Layout:', layoutData)),
  logState: button(() => console.log('State:', stateData)),
})
```

**Debug Options:**
- `showStats`: Toggle A-Frame stats panel
- `showBoundingBoxes`: Visualise treemap layout rectangles
- `showLayerPlanes`: Show semi-transparent planes at each layer Y-position
- `logLayout`: Dump layout calculation results to console
- `logState`: Dump ProjectionState to console

## Common Issues

### Entities Not Rendering

**Symptoms:** Entities don't appear in scene, A-Frame Inspector shows them in DOM

**Causes:**
1. Position outside camera frustum
2. Invalid geometry or material
3. Scale is 0 or negative
4. Parent entity is invisible
5. Z-fighting (entities at exact same position)

**Debugging Steps:**
1. Open A-Frame Inspector (`Ctrl+Alt+I`)
2. Find entity in hierarchy
3. Check `position` attribute (is it within view?)
4. Check `visible` attribute (should be `true`)
5. Check `scale` attribute (should be positive numbers)
6. Click entity in Inspector to highlight in scene
7. If highlighted but not visible: Check material/geometry

**Solutions:**
```typescript
// Verify position is within expected bounds
console.log('Node position:', node.x, node.y, node.z)
console.log('Camera position:', cameraPosition)
console.log('Distance:', calculateDistance(node, camera))

// Check if position has NaN/Infinity
if (!Number.isFinite(node.x)) {
  console.error('Invalid position for node:', node.id)
  node.x = 0 // Fallback
}

// Verify entity has valid geometry
<a-box width="1" height="1" depth="1" material="color: red" />
```

### Low FPS

**Symptoms:** FPS < 30, scene feels laggy, interactions are slow

**Causes:**
1. Too many entities (>1000 individual boxes)
2. Too many draw calls (each entity = 1 draw call)
3. Expensive shaders or post-processing
4. Shadows enabled with many entities
5. High triangle count
6. Memory leak (garbage piling up)

**Debugging Steps:**
1. Enable Stats component: `<a-scene stats>`
2. Check FPS, draw calls, triangle count
3. Disable shadows: Remove `shadow="type: pcfsoft"`
4. Disable edges: `showEdges: false` in controls
5. Reduce node count: Test with minimal fixture
6. Check memory: Chrome Task Manager (Shift+Esc)

**Solutions:**
```typescript
// Solution 1: Disable shadows if FPS < 30
<a-scene shadow={fps > 30 ? "type: pcfsoft" : undefined}>

// Solution 2: Use instanced rendering for >500 nodes
if (nodes.length > 500) {
  return <InstancedNodes nodes={nodes} />
} else {
  return nodes.map(n => <a-box key={n.id} {...n} />)
}

// Solution 3: Implement LOD (Level of Detail)
const distance = calculateDistance(node, camera)
const shouldRender = distance < labelDistance
if (!shouldRender) return null

// Solution 4: Simplify geometry
<a-box segments="1 1 1" /> // Fewer triangles
// Instead of:
<a-box segments="10 10 10" /> // More triangles
```

### Layout Looks Wrong

**Symptoms:** Nodes overlap, treemap doesn't look right, positions seem incorrect

**Causes:**
1. Treemap calculation error
2. Invalid node weights (negative, NaN, Infinity)
3. Incorrect layer assignments
4. Coordinate system confusion
5. Cycle in hierarchy (not properly broken)

**Debugging Steps:**
1. Log layout output: `console.log('Layout:', layoutData)`
2. Enable `showBoundingBoxes` in debug controls
3. Check treemap values: `console.log('Treemap:', d.x0, d.x1, d.y0, d.y1)`
4. Verify layer assignments: `console.log('Layer map:', layerMap)`
5. Check node weights: `console.log('Weights:', nodes.map(n => n.weight))`

**Solutions:**
```typescript
// Validate treemap output
root.descendants().forEach(d => {
  if (!Number.isFinite(d.x0) || !Number.isFinite(d.x1)) {
    console.error('Invalid treemap coordinates for:', d.data.id)
  }
})

// Sanitise weights
const weighted = nodes.map(n => ({
  ...n,
  weight: Math.max(1, n.weight || 1) // Ensure positive, non-zero
}))

// Visualise bounding boxes
if (debugControls.showBoundingBoxes) {
  return (
    <a-box
      width={node.width}
      depth={node.depth}
      height={0.1}
      material="color: red; wireframe: true; opacity: 0.5"
      position={`${node.x} ${node.y} ${node.z}`}
    />
  )
}
```

### Camera Not Moving

**Symptoms:** WASD keys don't move camera, mouse look doesn't work

**Causes:**
1. WASD controls not enabled on camera
2. Camera locked by another component
3. JavaScript error preventing input handling
4. Browser doesn't have focus
5. Camera entity not properly configured

**Debugging Steps:**
1. Check console for JavaScript errors
2. Verify WASD controls: `<a-entity wasd-controls>`
3. Click on scene to give it focus
4. Check if camera is locked: Inspector → Camera → `wasd-controls` component
5. Test with minimal scene (just camera, no other entities)

**Solutions:**
```typescript
// Ensure WASD controls are enabled
<a-entity
  id="rig"
  wasd-controls="enabled: true; acceleration: 65"
  look-controls="enabled: true; pointerLockEnabled: false"
>
  <a-camera />
</a-entity>

// Check for conflicting components
// Remove any custom camera controllers that might interfere

// Verify camera entity is properly set up
useEffect(() => {
  const camera = document.querySelector('#rig')
  if (!camera) {
    console.error('Camera rig not found!')
  }
}, [])
```

### WebGL Context Loss

**Symptoms:** Scene goes blank, console shows "webglcontextlost" event

**Causes:**
1. Browser suspended WebGL (tab in background too long)
2. GPU driver crash or reset
3. Too many WebGL contexts (multiple tabs)
4. Out of GPU memory
5. Hardware acceleration disabled

**Debugging Steps:**
1. Check browser console for "webglcontextlost" event
2. Check browser settings: Hardware acceleration enabled?
3. Close other WebGL tabs (limit: ~16 contexts per browser)
4. Check GPU memory usage (Task Manager)
5. Try different browser to rule out driver issue

**Solutions:**
```typescript
// Handle context loss gracefully
useEffect(() => {
  const canvas = document.querySelector('canvas')
  if (!canvas) return

  const handleContextLost = (e: Event) => {
    e.preventDefault()
    console.error('WebGL context lost')
    showError('Graphics context lost. Please reload the page.')
  }

  const handleContextRestored = () => {
    console.log('WebGL context restored')
    // Optionally: Recreate scene
  }

  canvas.addEventListener('webglcontextlost', handleContextLost)
  canvas.addEventListener('webglcontextrestored', handleContextRestored)

  return () => {
    canvas.removeEventListener('webglcontextlost', handleContextLost)
    canvas.removeEventListener('webglcontextrestored', handleContextRestored)
  }
}, [])
```

### Memory Leaks

**Symptoms:** Memory usage grows over time, browser becomes slow, eventually crashes

**Causes:**
1. Event listeners not removed
2. THREE.js geometries/materials not disposed
3. A-Frame components without proper `remove()` method
4. Circular references preventing garbage collection
5. Large closures retaining data

**Debugging Steps:**
1. Open Chrome Task Manager (Shift+Esc)
2. Monitor "JavaScript Memory" column
3. Perform actions, watch memory
4. Take heap snapshot before/after
5. Look for retained objects in heap snapshot

**Solutions:**
```typescript
// Solution 1: Clean up event listeners
useEffect(() => {
  const handleClick = (e: Event) => { /* ... */ }
  element.addEventListener('click', handleClick)

  return () => {
    element.removeEventListener('click', handleClick)
  }
}, [])

// Solution 2: Dispose THREE.js objects
useEffect(() => {
  return () => {
    // Dispose geometries
    scene.traverse((object) => {
      if (object.geometry) {
        object.geometry.dispose()
      }
      if (object.material) {
        if (Array.isArray(object.material)) {
          object.material.forEach(m => m.dispose())
        } else {
          object.material.dispose()
        }
      }
    })
  }
}, [])

// Solution 3: Implement proper A-Frame component removal
AFRAME.registerComponent('layer3d-node', {
  init() {
    this.geometry = new THREE.BoxGeometry()
    this.material = new THREE.MeshStandardMaterial()
  },
  remove() {
    // CRITICAL: Dispose resources
    if (this.geometry) this.geometry.dispose()
    if (this.material) this.material.dispose()
  }
})
```

## Debugging Workflow

### Step 1: Reproduce Issue

1. Use test fixtures to reproduce consistently
2. Document exact steps to reproduce
3. Note browser, OS, hardware details
4. Check if issue occurs in different browsers
5. Try with minimal test case (smallest graph that shows issue)

**Example:**
```
Issue: Nodes don't render
Steps: 1) Load Layer3D projection with hierarchical fixture
       2) Observe: Only 2 of 7 nodes visible
Browser: Chrome 131, macOS 14.2
Hardware: MacBook Pro 2019, 16GB RAM
Minimal test: hierarchicalGraph fixture
```

### Step 2: Isolate Problem

1. Disable features one by one
   - Disable edges: `showEdges: false`
   - Disable labels: `showLabels: false`
   - Disable shadows: Remove `shadow` attribute
2. Use minimal test case (smallest graph that reproduces issue)
3. Check if issue occurs in A-Frame Inspector
4. Try with different test fixtures

**Example:**
```typescript
// Isolate: Does issue occur without edges?
const [showEdges, setShowEdges] = useState(false)

// Isolate: Does issue occur with minimal data?
const testData = minimalGraph // 3 nodes instead of 100

// Isolate: Does issue occur in Inspector?
// Press Ctrl+Alt+I and check entity positions
```

### Step 3: Gather Data

1. Check browser console for errors/warnings
2. Record performance profile in Chrome DevTools
3. Take screenshots/video of issue
4. Export ProjectionState JSON for analysis
5. Log relevant data structures

**Example:**
```typescript
// Log layout data
console.log('Layout calculation:', {
  nodeCount: nodes.length,
  layerCount: layers.length,
  boundingBox,
  positions: nodes.map(n => ({ id: n.id, x: n.x, y: n.y, z: n.z }))
})

// Log state
console.log('ProjectionState:', JSON.stringify(state, null, 2))

// Log performance
console.time('Layout calculation')
const layout = calculateLayout(nodes, edges, layers, config)
console.timeEnd('Layout calculation')
```

### Step 4: Fix & Verify

1. Implement fix with logging to confirm it works
2. Test with original reproduction case
3. Test with larger graphs to verify no regressions
4. Remove debug logging before commit
5. Update tests if necessary

**Example:**
```typescript
// Fix: Validate node positions before rendering
const validatedNodes = nodes.map(node => {
  if (!Number.isFinite(node.x)) {
    console.warn(`Invalid X position for node ${node.id}, using 0`)
    return { ...node, x: 0 }
  }
  return node
})

// Verify: Log validation results
console.log('Validated nodes:', validatedNodes.length, 'nodes')
console.log('Invalid positions fixed:', nodes.length - validatedNodes.length)
```

## Development Tips

### Hot Reload Issues

**Problem:** A-Frame components don't hot-reload cleanly, scene breaks after code changes

**Solution:**
- Full page refresh: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
- Clear browser cache if components don't update
- Restart dev server if issue persists

**Prevention:**
- Use refs for scene updates instead of recreating entities
- Register A-Frame components outside React components (top level)
- Check component existence before registering

### Performance Testing

**Best Practices:**
- Test with large graphs (100, 500, 1000 nodes)
- Test on lower-end hardware (not just dev machine)
- Monitor FPS over time (check for degradation)
- Use Chrome Performance panel to identify bottlenecks
- Profile with Stats component enabled

**Target Metrics:**
- 60fps with 100 nodes (desktop)
- 30fps with 500 nodes (desktop)
- <100ms layout calculation (200 nodes)
- <50k triangles total
- <500 draw calls

### State Debugging

**Tools:**
- React DevTools: Inspect component state and props
- Redux DevTools: If using Redux for state management
- Console logs: `console.log('State updated:', state)`

**Common Issues:**
- Stale closures: State not updating in useEffect
- Re-renders: Too many re-renders causing performance issues
- Subscription not firing: GraphQL subscription not connected

**Solutions:**
```typescript
// Debug stale closures
useEffect(() => {
  console.log('Effect running with state:', state)
  // If state is stale, add it to dependency array
}, [state])

// Debug re-renders
useEffect(() => {
  console.log('Component re-rendered')
})

// Debug subscriptions
useSubscription(STATE_SUB, {
  onData: ({ data }) => {
    console.log('Subscription data received:', data)
  },
  onError: (error) => {
    console.error('Subscription error:', error)
  }
})
```

## Logging Best Practices

### Development Logging

```typescript
// Use prefixes for easy filtering
console.log('[Layer3D] Initializing scene')
console.log('[Layout] Calculating positions for', nodes.length, 'nodes')
console.warn('[Validation] Node missing layer assignment:', node.id)
console.error('[Layout] Cycle detected:', cycleNodes)

// Use console.time for performance measurement
console.time('[Layout] Treemap calculation')
const layout = calculateTreemap(nodes)
console.timeEnd('[Layout] Treemap calculation')

// Use console.table for structured data
console.table(nodes.map(n => ({ id: n.id, x: n.x, y: n.y, layer: n.layer })))
```

### Production Logging

```typescript
// Only log errors in production
if (import.meta.env.DEV) {
  console.log('[Layer3D] Debug info:', data)
} else {
  // Only log errors
  console.error('[Layer3D] Critical error:', error)
}

// Use structured logging for error tracking
logError({
  component: 'Layer3D',
  action: 'calculateLayout',
  error: error.message,
  stack: error.stack,
  context: { nodeCount, layerCount }
})
```

## Performance Profiling

### Chrome Performance Panel

1. Open DevTools → Performance tab
2. Click Record (red circle)
3. Perform action (e.g., load graph, move camera)
4. Stop recording
5. Analyze flame graph for bottlenecks

**What to Look For:**
- Long JavaScript tasks (>50ms)
- Excessive layout/reflow (purple bars)
- Paint operations (green bars)
- GPU activity (green in timeline)

**Common Bottlenecks:**
- Layout thrashing: Reading/writing DOM repeatedly
- Expensive React re-renders: Use React.memo, useMemo
- THREE.js scene updates: Batch updates, use object pooling

### A-Frame Stats Panel

**Interpreting Stats:**
- FPS drops below 30: Performance issue
- MS per frame >33ms: Too slow for 30fps
- Draw calls spike: Check entity count
- Triangles spike: Check geometry complexity
- Memory growing: Potential leak

## Getting Help

### Before Asking for Help

1. Check this debugging guide
2. Check A-Frame documentation: https://aframe.io/docs/1.7.0/
3. Check THREE.js documentation: https://threejs.org/docs/
4. Search existing issues: https://github.com/aframevr/aframe/issues
5. Create minimal reproduction case

### When Reporting Issues

Include:
- Browser and version
- Operating system
- Hardware specs (GPU, RAM)
- Steps to reproduce
- Expected vs. actual behavior
- Screenshots or video
- Console errors (full stack trace)
- ProjectionState JSON (if relevant)

### Useful Resources

- A-Frame Slack: https://aframe.io/slack-invite/
- THREE.js Forum: https://discourse.threejs.org/
- Stack Overflow: Tag questions with `aframe` or `three.js`
- Layer3D Implementation Plan: `plans/layer-3d-projection.md`
