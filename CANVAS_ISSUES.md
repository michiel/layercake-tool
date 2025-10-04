# Canvas Issues - Plan DAG Visual Editor

This document tracks issues identified and resolved in the Plan DAG Visual Editor component.

## Issue Timeline

### 1. Node Drag Position Reversion (Initial)

**Date**: January 2025 (pre-session)

**Symptoms**:
- Moving a node and dropping it caused it to jump back to original position
- Position mutations executed successfully but visual state reverted

**Root Cause**:
CQRS sync mechanism detected external data changes from subscription echoes and overrode local optimistic updates.

**Resolution**:
Implemented dragging state flag (`isDragging.current` and `setDragging`) to suppress external syncs during drag operations.

**Files Modified**:
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`

---

### 2. Edge Reconnection Format Mismatch

**Date**: January 2025

**Symptoms**:
- Edge reconnection worked visually but backend delete failed
- Error: "Edge not found" when deleting old edge
- ReactFlow generated IDs like `reactflow__edge-{source}-{target}{handle}`
- Backend expected simpler format with attachment indicators

**Console Evidence**:
```
[PlanDagCommandService] Deleting edge: reactflow__edge-graphnode_1759468672584_8kucj-outputnode_1759468714870_ksd1hinput-left
response: { "data": null, "errors": [{"message": "Edge not found"}] }
```

**Root Cause**:
Inconsistent edge ID format between frontend ReactFlow and backend storage.

**Resolution**:
1. Generate consistent edge ID format: `{source}-{target}-{sourceHandle}-{targetHandle}`
2. Extract backend edge ID from `edge.data.originalEdge.id` populated by ReactFlowAdapter
3. Changed from using `reconnectEdge` utility to manual edge deletion and creation

**Files Modified**:
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`

---

### 3. Rules of Hooks Violation

**Date**: January 2025

**Symptoms**:
- React error: "Rendered more hooks than during the previous render"
- Application crashed when loading editor

**Console Evidence**:
```
React has detected a change in the order of Hooks called by PlanVisualEditorInner
197. undefined                useCallback
Error: Rendered more hooks than during the previous render.
```

**Root Cause**:
Added `useCallback` and `useMemo` hooks for edge markers after early returns (loading/error checks), violating React's Rules of Hooks which require hooks to be called in same order every render.

**Resolution**:
Moved all hooks before any conditional early returns:

```typescript
// Helper function to check if all upstream nodes are configured
// Must be defined before any early returns to follow Rules of Hooks
const areAllUpstreamNodesConfigured = useCallback(...)
const edgesWithMarkers = useMemo(...)

// NOW the early returns
if (loading) return ...
if (error) return ...
if (!planDag) return ...
```

**Files Modified**:
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`

---

### 4. Auto-Layout Not Visually Updating

**Date**: January 2025

**Symptoms**:
- Auto-layout buttons triggered mutations successfully
- Console showed position update mutations executing
- Nodes did not move visually on canvas

**Console Evidence**:
```
[PlanDagCommandService] Moving node: graphnode_1759468672584_8kucj {x: 462, y: 87}
[usePlanDagCQRS] Syncing ReactFlow state from external data change
[PlanDagQueryService] Skipping subscription update (recent mutation echo)
```

**Root Cause**:
CQRS sync mechanism detected external changes and attempted to sync state, overriding local optimistic updates. While subscription echoes were correctly suppressed, the sync detection logic still triggered.

**Resolution**:
Added `setDragging(true/false)` wrapper around auto-layout operations to suppress external syncs:

```typescript
setDragging(true);  // Suppress external syncs
const layoutedNodes = await autoLayout(...);
setNodes(layoutedNodes);  // Local update
layoutedNodes.forEach(node => mutations.moveNode(...));  // Backend persist
setDragging(false);  // Re-enable syncs
```

**Files Modified**:
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`
  - `handleAutoLayoutHorizontal` (lines 795-836)
  - `handleAutoLayoutVertical` (lines 795-836)

---

### 5. Node Delete Not Visually Updating

**Date**: January 2025

**Symptoms**:
- Clicking delete icon triggered correct GraphQL mutation
- Console showed successful delete mutation
- Node remained visible on canvas

**Root Cause**:
Same as auto-layout issue - CQRS sync mechanism overriding optimistic delete operations.

**Resolution**:
Applied same dragging state pattern to delete handler:

```typescript
setDragging(true)
setNodes((nds) => nds.filter(node => node.id !== nodeId))
setEdges((eds) => eds.filter(...))
mutations.deleteNode(nodeId)
setTimeout(() => setDragging(false), 100)
```

**Files Modified**:
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`
  - Delete handler useEffect (lines 155-186)

---

### 6. Node Drag Position Reversion (Recurrence)

**Date**: January 2025

**Symptoms**:
- Moving a node and dropping it caused it to jump back to original position
- Position mutations executed successfully
- Subscription echoes were correctly suppressed but arrived after dragging state was cleared

**Console Evidence**:
```
[PlanDagCommandService] Moving node: datasourcenode_1759546637909_okjw8 {x: -1286.36, y: -285.91}
Node position saved: datasourcenode_1759546637909_okjw8
[usePlanDagCQRS] Syncing ReactFlow state from external data change
[PlanDagQueryService] Skipping subscription update (recent mutation echo): {timeSinceLastMutation: '74ms'}
```

**Root Cause**:
Race condition in `handleNodeDragStop` - `setDragging(false)` was called immediately at the start of the handler, but the mutation was sent later. The subscription echo arrived after dragging was already false, so it wasn't being suppressed properly.

**Resolution**:
Delay `setDragging(false)` until after mutation completes:

```typescript
if (hasMovedX || hasMovedY) {
  mutations.moveNode(node.id, node.position)
  console.log('Node position saved:', node.id, node.position)

  // Re-enable external syncs after a short delay to allow mutation to complete
  setTimeout(() => setDragging(false), 100)
} else {
  console.log('Node not moved significantly, skipping position save:', node.id)
  // Re-enable external syncs immediately if no mutation was sent
  setDragging(false)
}
```

**Files Modified**:
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`
  - `handleNodeDragStop` (lines 298-342)

---

## Common Patterns

### CQRS Sync Suppression

Multiple issues (#1, #4, #5, #6) were caused by the CQRS sync mechanism interfering with local optimistic updates. The consistent solution pattern is:

1. Set `dragging` state to `true` before local state changes
2. Perform optimistic local updates
3. Send backend mutations
4. Delay setting `dragging` to `false` by 100ms to allow subscription echoes to arrive and be suppressed

### Edge ID Consistency

Issue #2 highlighted the importance of maintaining consistent edge ID formats between frontend and backend. The solution uses:
- Format: `{source}-{target}-{sourceHandle}-{targetHandle}`
- Backend ID stored in `edge.data.originalEdge.id` by ReactFlowAdapter
- Extract backend ID when performing mutations

### React Hooks Compliance

Issue #3 demonstrated the importance of following React's Rules of Hooks:
- All hooks must be called before any conditional returns
- Hooks must be called in the same order every render
- Document with comments when hooks are positioned to satisfy these rules

## Prevention Guidelines

1. **Always use dragging state suppression** when performing local mutations that will echo back via subscriptions
2. **Delay clearing dragging state** by 100ms after mutations to allow echoes to arrive
3. **Position all hooks before early returns** to comply with Rules of Hooks
4. **Use consistent ID formats** between frontend and backend, storing backend IDs in `data.original*` fields
5. **Test optimistic updates** by observing console logs for subscription echoes and sync behaviour
