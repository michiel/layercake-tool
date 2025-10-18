# Plan DAG Editor Performance Analysis and Remediation Plan

## Current Performance Issues

### Observed Symptoms
- **Choppy node dragging**: Nodes lag behind cursor during drag operations
- **Frame drops**: Visible stuttering and delayed visual feedback
- **Sluggish response**: Noticeable delay between mouse movement and node position updates

## Root Cause Analysis

### 1. Excessive Re-renders During Drag (Critical)

**Location**: `PlanVisualEditor.tsx:1161-1171`

```typescript
const nodesWithEdges = useMemo(() => {
  return nodes.map(node => ({
    ...node,
    data: {
      ...node.data,
      edges: edges,
      isUnconfigured: !isNodeFullyConfigured(node.id),
      projectId: projectId
    }
  }))
}, [nodes, edges, isNodeFullyConfigured, projectId])
```

**Problem**:
- This memo recreates **all node objects** whenever `nodes` changes
- During drag, ReactFlow updates `nodes` on every mouse move to update positions
- Every position change triggers complete recreation of all node data objects
- This causes all BaseNode components to re-render even though only one node moved

**Impact**: High - Major cause of choppy dragging

### 2. Edge Marker Recalculation (Moderate)

**Location**: `PlanVisualEditor.tsx:1175-1199`

```typescript
const edgesWithMarkers = useMemo(() => {
  return edges.map(edge => {
    const sourceConfigured = isNodeFullyConfigured(edge.source)
    // ... creates new edge objects
  })
}, [edges, isNodeFullyConfigured, readonly])
```

**Problem**:
- Recreates all edge objects whenever edges change
- Calls `isNodeFullyConfigured` for every edge source
- Even though edges don't change during node dragging, the memo still processes all edges

**Impact**: Moderate - Adds overhead but less critical than node re-renders

### 3. Configuration Validation During Drag (Low-Moderate)

**Location**: `PlanVisualEditor.tsx:1116-1152`

```typescript
const nodeConfigMap = useMemo(() => {
  const map = new Map<string, boolean>()
  nodes.forEach(node => {
    // Complex validation logic for each node
  })
  return map
}, [nodeConfigKey])
```

**Problem**:
- `nodeConfigKey` is derived from nodes/edges state
- Recalculates configuration status for all nodes when state changes
- Validation logic runs even though node configuration doesn't change during position updates

**Impact**: Low-Moderate - Adds computational overhead

### 4. Data Injection Anti-Pattern (High)

**Problem**:
- Injecting entire `edges` array into every node's data
- Creates unnecessary data dependencies
- Breaks React memo optimization because node data objects are always new

**Impact**: High - Prevents effective memoization

### 5. Missing Position-Only Update Path (Critical)

**Problem**:
- No optimized code path for position-only updates
- Position changes trigger full data pipeline
- ReactFlow's position updates are fast, but our data transformations slow them down

**Impact**: Critical - Direct cause of drag performance issues

## Remediation Plan

### Phase 1: Immediate Fixes (High Priority)

#### 1.1 Separate Position Updates from Data Updates

**Goal**: Prevent full node data recreation on position changes

**Implementation**:
```typescript
// Split nodes into position and data concerns
const nodeDataMap = useMemo(() => {
  const map = new Map()
  nodes.forEach(node => {
    map.set(node.id, {
      edges: edges,
      isUnconfigured: !isNodeFullyConfigured(node.id),
      projectId: projectId,
      // ... other data
    })
  })
  return map
}, [edges, isNodeFullyConfigured, projectId]) // No 'nodes' dependency!

// Inject data only when it changes, not on position updates
const nodesWithData = useMemo(() => {
  return nodes.map(node => ({
    ...node,
    data: {
      ...node.data,
      ...nodeDataMap.get(node.id)
    }
  }))
}, [nodes, nodeDataMap])
```

**Benefit**: Position changes won't trigger data recreation

#### 1.2 Use Ref-Based Position Tracking During Drag

**Implementation**:
```typescript
const isDragging = useRef(false)
const draggedNodePositions = useRef<Map<string, {x: number, y: number}>>(new Map())

// In handleNodesChange, skip data injection during drag
const handleNodesChange = useCallback((changes: NodeChange[]) => {
  if (isDragging.current) {
    // Fast path: only update positions, don't regenerate data
    onNodesChange(changes)
    return
  }

  // Normal path: full processing
  onNodesChange(changes)
  // ... rest of logic
}, [onNodesChange])
```

**Benefit**: Eliminates memo recalculation during drag

#### 1.3 Optimize BaseNode Memo Comparison

**Implementation**:
```typescript
export const BaseNode = memo(({
  // ... props
}: BaseNodeProps) => {
  // ... component
}, (prevProps, nextProps) => {
  // Custom comparison: ignore position changes during drag
  if (prevProps.id !== nextProps.id) return false
  if (prevProps.selected !== nextProps.selected) return false

  // Shallow compare data except position-related fields
  const prevData = prevProps.data
  const nextData = nextProps.data

  return (
    prevData.nodeType === nextData.nodeType &&
    prevData.config === nextData.config &&
    prevData.metadata === nextData.metadata
    // Don't compare edges or other frequently changing data
  )
})
```

**Benefit**: Prevent re-renders of nodes not being dragged

### Phase 2: Optimization (Medium Priority)

#### 2.1 Cache Node Configuration Status

**Implementation**:
```typescript
// Use a stable cache that only updates when configuration actually changes
const configCache = useRef(new Map<string, boolean>())

const isNodeFullyConfigured = useCallback((nodeId: string): boolean => {
  if (configCache.current.has(nodeId)) {
    return configCache.current.get(nodeId)!
  }

  const node = nodes.find(n => n.id === nodeId)
  if (!node) return false

  const isConfigured = computeConfiguration(node, edges)
  configCache.current.set(nodeId, isConfigured)
  return isConfigured
}, [nodes, edges])

// Invalidate cache only when nodes/edges change structurally
useEffect(() => {
  configCache.current.clear()
}, [nodeConfigKey])
```

**Benefit**: Avoid repeated configuration calculations

#### 2.2 Lazy Edge Marker Calculation

**Implementation**:
```typescript
// Only recalculate markers when edge source configuration changes
const edgeConfigKey = useMemo(() => {
  return edges.map(e => `${e.id}:${isNodeFullyConfigured(e.source)}`).join(',')
}, [edges, isNodeFullyConfigured])

const edgesWithMarkers = useMemo(() => {
  // ... edge marker logic
}, [edgeConfigKey, readonly]) // More stable dependency
```

**Benefit**: Reduce edge recalculations

#### 2.3 Batch Position Updates

**Implementation**:
```typescript
const positionUpdateQueue = useRef<Array<{nodeId: string, position: Position}>>([])
const updateTimer = useRef<NodeJS.Timeout>()

const queuePositionUpdate = useCallback((nodeId: string, position: Position) => {
  positionUpdateQueue.current.push({nodeId, position})

  if (updateTimer.current) {
    clearTimeout(updateTimer.current)
  }

  updateTimer.current = setTimeout(() => {
    // Batch update all positions at once
    const updates = positionUpdateQueue.current
    positionUpdateQueue.current = []

    mutations.batchMoveNodes(projectId, updates)
  }, 100)
}, [mutations, projectId])
```

**Benefit**: Reduce backend mutation overhead

### Phase 3: Advanced Optimizations (Lower Priority)

#### 3.1 Virtual Rendering for Large DAGs

**When**: If DAG has >100 nodes

**Implementation**: Use react-window or similar to only render visible nodes

#### 3.2 WebWorker for Configuration Validation

**Implementation**: Move expensive validation logic to background thread

#### 3.3 Canvas-Based Edge Rendering

**Implementation**: Replace React-based edge rendering with canvas for better performance

## Implementation Order

1. **✅ COMPLETED**: Phase 1.1 + 1.2 + 1.3 (Critical fixes)
   - ✅ Separate position/data updates using Map-based approach
   - ✅ Ref-based drag tracking (already implemented, enhanced with comments)
   - ✅ Custom memo comparison in BaseNode

2. **Week 2**: Phase 2.1 (Memo optimization)
   - Configuration caching

3. **Week 3**: Phase 2.2 + 2.3 (Additional optimizations)
   - Lazy edge markers
   - Batch updates

4. **Future**: Phase 3 (As needed based on DAG size)

## Implementation Status

### Phase 1 - COMPLETED

#### Phase 1.1: Separate Position Updates from Data Updates ✅
**Status**: Implemented and tested
**File**: `PlanVisualEditor.tsx` lines 1159-1182

**Changes Made**:
```typescript
// Created nodeDataMap that only depends on actual data changes
const nodeDataMap = useMemo(() => {
  const map = new Map<string, any>()
  nodes.forEach(node => {
    map.set(node.id, {
      edges: edges,
      isUnconfigured: !isNodeFullyConfigured(node.id),
      projectId: projectId,
      ...node.data
    })
  })
  return map
}, [edges, isNodeFullyConfigured, projectId, nodeConfigKey]) // Uses nodeConfigKey instead of nodes!
```

**Impact**: Node data objects are no longer recreated on every position change during drag. The `nodeDataMap` only recalculates when configuration actually changes (tracked by `nodeConfigKey`), not when nodes move.

#### Phase 1.2: Use Ref-Based Position Tracking During Drag ✅
**Status**: Already implemented, enhanced with performance comments
**File**: `PlanVisualEditor.tsx` lines 267-284

**Changes Made**:
- Added performance comments to clarify the existing drag optimization
- Verified that `isDragging.current` correctly skips side effects during drag
- Position updates during drag now bypass validation, sync, and other expensive operations

**Impact**: During drag, only ReactFlow's fast position updates occur. All expensive processing is skipped.

#### Phase 1.3: Optimize BaseNode Memo Comparison ✅
**Status**: Implemented with custom comparison function
**File**: `BaseNode.tsx` lines 507-535

**Changes Made**:
```typescript
}, (prevProps, nextProps) => {
  // Critical props that force re-render
  if (prevProps.id !== nextProps.id) return false
  if (prevProps.selected !== nextProps.selected) return false
  if (prevProps.nodeType !== nextProps.nodeType) return false

  // Fast reference check
  if (prevProps.data === nextProps.data) return true

  // Deep comparison of important fields only
  const prevData = prevProps.data || {}
  const nextData = nextProps.data || {}
  if (prevData.isUnconfigured !== nextData.isUnconfigured) return false
  if (prevData.hasValidConfig !== nextData.hasValidConfig) return false
  if (prevData.config !== nextData.config) return false
  if (prevData.metadata !== nextData.metadata) return false

  return true
})
```

**Impact**: BaseNode components only re-render when meaningful props change. With Phase 1.1's stable data references during drag, this prevents unnecessary re-renders of non-dragged nodes.

## Success Metrics

- **Target**: 60 FPS during node dragging
- **Measurement**: Chrome DevTools Performance profiler
- **Criteria**:
  - No frame drops during drag
  - <16ms per frame
  - Smooth visual feedback
  - Node follows cursor with <50ms latency

## Testing Plan

1. Create test DAG with 50 nodes, 100 edges
2. Record baseline performance metrics
3. Implement Phase 1 fixes
4. Re-measure performance
5. Iterate on remaining phases if needed

## Related Files

- `src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx` - Main editor
- `src/components/editors/PlanVisualEditor/nodes/BaseNode.tsx` - Node component
- `src/components/editors/PlanVisualEditor/hooks/usePlanDagCQRS.ts` - State management
- `src/components/editors/PlanVisualEditor/edges/FloatingEdge.tsx` - Edge rendering

## References

- React Memo: https://react.dev/reference/react/memo
- ReactFlow Performance: https://reactflow.dev/learn/advanced-use/performance
- Chrome DevTools Performance: https://developer.chrome.com/docs/devtools/performance
