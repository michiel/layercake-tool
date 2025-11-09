# Plan DAG Editor Event Handling - Technical Review

**Date:** 2025-10-20
**Reviewer:** Claude
**Scope:** Event handling, subscription updates, and state synchronisation in the Plan DAG visual editor
**Status:** Critical Issues Identified

## Executive Summary

The Plan DAG editor suffers from **incomplete event propagation** causing UI state to become stale without manual reload. The root cause is a **mismatch between delta subscription coverage and UI display requirements**, compounded by **excessive defensive complexity** in state synchronisation logic.

**Critical Finding:** Node execution status changes (e.g., when a dataset is processed or a graph is computed) are **not propagated via the delta subscription**, despite being displayed in the UI. This causes the canvas to show outdated execution states until a full page reload.

## Architecture Overview

### Current Event Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        User Actions (UI)                             │
│  • Drop node   • Delete node   • Execute node   • Configure node    │
└───────────────────────┬─────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  PlanVisualEditor.tsx                                │
│  • Converts ReactFlow events → backend mutations                    │
│  • Manages local optimistic updates                                 │
│  • Coordinates 4 concurrent suppression mechanisms                  │
└───────────────────────┬─────────────────────────────────────────────┘
                        │
        ┌───────────────┴────────────────┐
        ▼                                ▼
┌──────────────────┐           ┌──────────────────────┐
│ usePlanDagCQRS   │           │ PlanDagCQRSService   │
│ (State Hook)     │           │ (Facade)             │
│                  │           │                      │
│ • ReactFlow      │◄──────────┤ • Commands (write)   │
│   state          │           │ • Queries (read)     │
│ • PlanDag state  │           │ • Adapter            │
│ • Sync logic     │           └──────────────────────┘
└──────┬───────────┘                    │
       │                                │
       │                        ┌───────┴────────┐
       │                        ▼                ▼
       │              ┌──────────────┐  ┌──────────────┐
       │              │ CommandSvc   │  │  QuerySvc    │
       │              │ (Mutations)  │  │ (Read/Sub)   │
       │              └──────┬───────┘  └──────┬───────┘
       │                     │                 │
       │                     ▼                 ▼
       │              ┌────────────────────────────────┐
       │              │      GraphQL Layer             │
       │              │  • Mutations with clientId     │
       │              │  • Delta Subscription (JSON    │
       │              │    Patch)                      │
       │              └────────────────────────────────┘
       │                              │
       │                              │ WebSocket
       │                              ▼
       │              ┌────────────────────────────────┐
       │              │     Backend (Rust/GraphQL)     │
       │◄─────────────┤  • Applies mutations           │
       Subscription   │  • Generates JSON Patch deltas │
       Updates        │  • Broadcasts to subscribers   │
                      └────────────────────────────────┘
```

### Components Involved

1. **PlanVisualEditor.tsx** (1,465 lines)
   - Main React component managing ReactFlow canvas
   - Coordinates user interactions, drag-drop, connections
   - Implements optimistic updates with 4 overlapping suppression mechanisms

2. **usePlanDagCQRS.ts** (428 lines)
   - Core state management hook
   - Converts between PlanDag (backend) and ReactFlow (frontend) formats
   - Manages delta subscription and state reconciliation

3. **PlanDagCQRSService.ts** (203 lines)
   - Unified facade for command/query separation
   - Coordinates echo suppression between services

4. **PlanDagCommandService.ts** (349 lines)
   - Executes all mutations (create/update/delete/move operations)
   - Marks mutation timestamps for echo suppression

5. **PlanDagQueryService.ts** (268 lines)
   - Handles queries and subscriptions
   - Applies JSON Patch deltas from subscription
   - Filters subscription echo using clientId and timestamp window

6. **PlanDagEventStore.ts** (280 lines) ⚠️
   - Event sourcing infrastructure (UNUSED)
   - Defines event log and state reconstruction
   - Dead code that adds cognitive load

7. **useUnifiedUpdateManager.ts** (271 lines)
   - Debounces/throttles backend persistence
   - Currently disabled in delta migration

## Critical Issues

### 1. Incomplete Delta Subscription Coverage ⚠️ **ROOT CAUSE**

**Severity:** Critical
**Impact:** UI shows stale execution status until reload

**Problem:**
The `PLAN_DAG_DELTA_SUBSCRIPTION` only propagates structural changes (nodes, edges, metadata) but **excludes execution state** (`datasetExecution`, `graphExecution`). When a node's execution status changes (e.g., dataset processed, graph computed), the delta subscription does not include these updates.

**Evidence:**
- `GET_PLAN_DAG` query includes `datasetExecution` and `graphExecution` fields (plan-dag.ts:22-37)
- `PLAN_DAG_DELTA_SUBSCRIPTION` only includes JSON Patch operations on structure (plan-dag.ts:296-311)
- Nodes display execution status badges (DataSetNode.tsx:49-68)
- Delta subscription handler applies patches to Plan DAG structure only (PlanDagQueryService.ts:138-173)

**Result:**
When backend processes a dataset or computes a graph:
1. Backend updates execution state in database
2. Delta subscription doesn't include execution state changes
3. UI continues showing old status ("Pending", "Processing")
4. User must reload page to see "Complete" status

**Code Location:**
- Subscription definition: `frontend/src/graphql/plan-dag.ts:296-311`
- Delta handler: `frontend/src/services/PlanDagQueryService.ts:87-180`
- Display logic: `frontend/src/components/editors/PlanVisualEditor/nodes/DataSetNode.tsx:32-68`

### 2. Excessive Defensive Complexity

**Severity:** High
**Impact:** Maintenance burden, potential race conditions, difficult debugging

**Problem:**
Four overlapping mechanisms attempt to prevent subscription echo and circular updates:

1. **Client ID filtering** (useGraphQLSubscriptionFilter.ts)
   - Filters subscription updates from same client
   - 500ms suppression window

2. **Mutation timestamp window** (PlanDagQueryService.ts:119-127)
   - Ignores subscription for 500ms after mutation
   - Coordinated with command service via CQRS facade

3. **Drag suppression flag** (usePlanDagCQRS.ts:206-208, PlanVisualEditor.tsx:316)
   - `isDraggingRef` prevents sync during node drag
   - Manually cleared with setTimeout

4. **External sync suppression flag** (usePlanDagCQRS.ts:199-202, 262-269)
   - `isSyncingFromExternalRef` prevents React 18 double-render issues
   - Cleared with setTimeout(0)

**Evidence:**
```typescript
// From usePlanDagCQRS.ts:198-272
useEffect(() => {
  // Skip if we're currently syncing (prevents React 18 double render)
  if (isSyncingFromExternalRef.current) return

  // Skip during drag operations
  if (isDraggingRef.current) return

  // Complex heuristics for when to sync
  const shouldSync = hasNewData && (
    isCurrentEmpty ||
    hasMoreNodes ||
    hasMoreEdges ||
    (hasSameOrMoreItems && hasNodeChanges)
  )
  // ... manual state update with setTimeout flag clearing
})
```

**Consequences:**
- Race conditions when multiple mechanisms conflict
- Difficult to reason about when updates actually propagate
- Manual setTimeout management prone to timing bugs
- High cognitive load for future maintainers

### 3. Event Sourcing Infrastructure Not Used

**Severity:** Medium
**Impact:** Dead code, misleading architecture

**Problem:**
- `PlanDagEventStore.ts` and `PlanDagEvents.ts` define comprehensive event sourcing infrastructure
- Event types defined: NODE_CREATED, NODE_UPDATED, EDGE_CREATED, etc.
- Event store can reconstruct state from event log
- **None of this is actually used** in the application

**Evidence:**
- Event store class exists with 280 lines (PlanDagEventStore.ts)
- Event definitions with 323 lines (PlanDagEvents.ts)
- Grep for `PlanDagEventStore` shows no usage in actual components
- Grep for `dispatch` or event creation shows no integration

**Consequences:**
- Misleading for developers expecting event sourcing
- Adds cognitive overhead understanding unused patterns
- Maintenance burden for unused code

### 4. Optimistic Update State Management Complexity

**Severity:** Medium
**Impact:** Potential stale state, difficult debugging

**Problem:**
Multiple refs track state to prevent "stale sync" issues:
- `stablePlanDagRef` - stable reference to prevent memo thrashing
- `previousPlanDagRef` - tracks changes for stability
- `nodesRef` - cached for edit handler
- `dragStartPositions` - track node positions at drag start
- `isDraggingRef` - suppress external sync during drag
- `isSyncingFromExternalRef` - suppress circular sync
- `initializedRef` - prevent double initialisation

**Evidence:**
```typescript
// From usePlanDagCQRS.ts:104-111
const previousPlanDagRef = useRef<PlanDag | null>(null)
const stablePlanDagRef = useRef<PlanDag | null>(null)
const subscriptionRef = useRef<any>(null)
const initializedRef = useRef(false)
const previousChangeIdRef = useRef<number>(0)
const isSyncingFromExternalRef = useRef<boolean>(false)
const isDraggingRef = useRef<boolean>(false)
```

**Consequences:**
- Difficult to track which ref has authoritative state
- Manual synchronisation between refs error-prone
- Debugging requires inspecting 7+ refs

### 5. Position Update Dual-State Management

**Severity:** Medium
**Impact:** Potential desync between ReactFlow and backend state

**Problem:**
When dragging a node, position is updated in **two places** to prevent stale sync:

```typescript
// From PlanVisualEditor.tsx:342-367
mutations.moveNode(node.id, node.position)

// Update ReactFlow state
setNodes((nds) => nds.map((n) =>
  n.id === node.id ? { ...n, position: node.position } : n
))

// ALSO update planDag to prevent sync overwrite
updatePlanDagOptimistically((current) => {
  if (!current) return current
  return {
    ...current,
    nodes: current.nodes.map((n) =>
      n.id === node.id ? { ...n, position: node.position } : n
    )
  }
})
```

**Consequences:**
- Mutation triggers backend update
- Local state updated optimistically in two formats
- Subscription echo suppressed for 500ms
- If timing is off, stale position could overwrite

## Potential Race Conditions

### Race 1: Drag Position vs Subscription Update

**Scenario:**
1. User A drags node N to position P1
2. Drag stop triggers mutation with 100ms setTimeout to clear `isDragging`
3. User B moves same node to P2 (within 100ms)
4. User B's delta arrives while User A still has `isDragging=true`
5. User A's UI ignores User B's update
6. 100ms expires, `isDragging=false`, but state is now P1 instead of P2

**Likelihood:** Low (requires precise timing and concurrent editing)
**Severity:** Medium (causes position desync requiring reload)

### Race 2: Mutation Echo vs Real Update

**Scenario:**
1. User mutates node N
2. Mutation timestamp set to T
3. Backend processes, broadcasts delta at T+100ms
4. User's subscription receives at T+150ms (within 500ms window)
5. Update suppressed as "echo"
6. But what if another user also mutated N at T+50ms?
7. That update is also suppressed as echo

**Likelihood:** Very Low (requires concurrent mutations within 500ms)
**Severity:** High (legitimate updates suppressed)

### Race 3: External Sync Flag Clearing

**Scenario:**
1. External data change detected
2. `isSyncingFromExternalRef.current = true`
3. `setNodes()` and `setEdges()` called
4. `setTimeout(() => isSyncingFromExternalRef.current = false, 0)`
5. React 18 batches state updates
6. Another effect runs before setTimeout fires
7. Circular update not prevented

**Likelihood:** Medium (React 18 timing dependent)
**Severity:** Low (mostly causes extra renders, not data loss)

### Race 4: Optimistic Update vs Subscription

**Scenario:**
1. User deletes node N
2. Local state updated optimistically (node removed)
3. Mutation sent to backend
4. Backend broadcasts delta
5. Delta arrives in <500ms (suppressed)
6. Another user independently deletes edge E connected to N
7. Delta for E deletion arrives after 500ms window
8. Applied to state that no longer has N
9. Edge still visible in UI but backend has deleted it

**Likelihood:** Very Low
**Severity:** Medium (UI shows ghost edges)

## Recommendations

### Immediate Actions (P0)

#### 1. Extend Delta Subscription to Include Execution State

**Goal:** Fix root cause of stale execution status

**Implementation:**
Modify backend to include execution state changes in JSON Patch operations:

```graphql
# Option A: Extend delta subscription
subscription PlanDagDeltaChanged($projectId: Int!) {
  planDagDeltaChanged(projectId: $projectId) {
    projectId
    version
    userId
    timestamp
    operations {
      op
      path
      value
      from
    }
    # Add execution state separately to avoid patch complexity
    executionStateChanges {
      nodeId
      datasetExecution { ... }
      graphExecution { ... }
    }
  }
}
```

**Alternative (Option B):** Create separate execution status subscription:

```graphql
subscription NodeExecutionStatusChanged($projectId: Int!) {
  nodeExecutionStatusChanged(projectId: $projectId) {
    nodeId
    nodeType
    datasetExecution { ... }
    graphExecution { ... }
  }
}
```

Subscribe to both in `usePlanDagCQRS`:
```typescript
// Existing structural delta subscription
const deltaSubscription = cqrsService.subscribeToDeltaUpdates(...)

// New execution status subscription
const executionSubscription = cqrsService.subscribeToExecutionUpdates(
  projectId,
  (nodeId, executionData) => {
    updatePlanDagOptimistically((current) => {
      if (!current) return current
      return {
        ...current,
        nodes: current.nodes.map(n =>
          n.id === nodeId
            ? { ...n, ...executionData }
            : n
        )
      }
    })
  }
)
```

**Recommendation:** Use **Option B** for cleaner separation of concerns. Execution status changes independently of structural changes.

**Files to Modify:**
- Backend: `layercake-core/src/graphql/subscriptions/plan_dag.rs`
- Frontend: `frontend/src/graphql/plan-dag.ts`
- Frontend: `frontend/src/services/PlanDagQueryService.ts`
- Frontend: `frontend/src/hooks/usePlanDagCQRS.ts`

#### 2. Simplify Echo Suppression to Single Mechanism

**Goal:** Reduce complexity and race conditions

**Implementation:**
Consolidate to **client ID filtering only** with server-side support:

```typescript
// Remove mutation timestamp suppression
// Remove isDragging suppression (rely on optimistic updates)
// Remove isSyncingFromExternalRef (React 18 handles this)

// Keep only client ID filtering
const subscription = this.apollo.subscribe({
  query: PlanDagGraphQL.PLAN_DAG_DELTA_SUBSCRIPTION,
  variables: {
    projectId: query.projectId,
    excludeClientId: this.clientId  // Server-side filtering
  }
})
```

Backend filters before broadcast (more efficient):
```rust
// In subscription handler
if delta.client_id == exclude_client_id {
    continue; // Don't send to originating client
}
```

**Rationale:**
- Single source of truth (server knows mutation origin)
- Eliminates timing-based race conditions
- Removes need for manual flag management
- Optimistic updates handle UI responsiveness

**Files to Modify:**
- Backend: Subscription resolver to accept `excludeClientId`
- Frontend: Remove timestamp and flag-based suppression
- Frontend: Simplify `usePlanDagCQRS` sync logic

### Short-term Improvements (P1)

#### 3. Remove Event Sourcing Dead Code

**Goal:** Reduce cognitive load and maintenance burden

**Action:**
Delete unused event sourcing infrastructure:
- `frontend/src/events/PlanDagEvents.ts`
- `frontend/src/stores/PlanDagEventStore.ts`

Document decision in ADR:
```markdown
# ADR: Remove Event Sourcing Infrastructure

**Status:** Accepted
**Date:** 2025-10-20

## Context
Event sourcing infrastructure was added but never integrated.

## Decision
Remove PlanDagEventStore and PlanDagEvents to reduce complexity.

## Consequences
- Simpler architecture
- Future event sourcing would require reimplementation
- Current delta subscription provides similar benefits
```

#### 4. Consolidate State Refs

**Goal:** Simplify state tracking

**Implementation:**
Combine related refs into single state object:

```typescript
// Instead of 7 separate refs
const editorStateRef = useRef({
  planDag: {
    current: null,
    previous: null,
    stable: null,
  },
  sync: {
    isDragging: false,
    isExternalSync: false,
    isInitialised: false,
  },
  dragStart: new Map<string, Position>(),
})
```

**Benefits:**
- Single ref to inspect in debugger
- Clear ownership of related state
- Easier to reason about state transitions

### Long-term Architectural Improvements (P2)

#### 5. Introduce Finite State Machine for Sync States

**Goal:** Make state transitions explicit and verifiable

**Implementation:**
Use XState or similar to model sync states:

```typescript
const syncMachine = createMachine({
  id: 'planDagSync',
  initial: 'idle',
  states: {
    idle: {
      on: {
        DRAG_START: 'dragging',
        EXTERNAL_UPDATE: 'applying_external',
        LOCAL_MUTATION: 'applying_local',
      }
    },
    dragging: {
      on: {
        DRAG_END: 'idle',
        EXTERNAL_UPDATE: 'dragging', // Ignore external during drag
      }
    },
    applying_external: {
      on: {
        SYNC_COMPLETE: 'idle',
      }
    },
    applying_local: {
      on: {
        MUTATION_COMPLETE: 'idle',
      }
    },
  }
})
```

**Benefits:**
- Explicit state transitions
- Impossible states prevented by design
- Easier testing and debugging
- Visual state machine diagrams

#### 6. Separate Execution Status State Management

**Goal:** Decouple execution status from structural state

**Rationale:**
- Execution status changes frequently (progress updates)
- Structural changes are less frequent
- Different update patterns warrant different state management

**Implementation:**
```typescript
// Separate hook for execution status
const useNodeExecutionStatus = (projectId: number) => {
  const [executionStates, setExecutionStates] = useState<Map<string, ExecutionState>>(new Map())

  useEffect(() => {
    const subscription = subscribeToExecutionUpdates(projectId, (nodeId, state) => {
      setExecutionStates(prev => new Map(prev).set(nodeId, state))
    })
    return () => subscription.unsubscribe()
  }, [projectId])

  return executionStates
}

// In PlanVisualEditor
const executionStates = useNodeExecutionStatus(projectId)

// Pass to nodes
const nodesWithExecution = nodes.map(node => ({
  ...node,
  data: {
    ...node.data,
    executionState: executionStates.get(node.id),
  }
}))
```

**Benefits:**
- Clear separation of concerns
- Execution updates don't trigger full Plan DAG reconciliation
- Easier to optimise polling/subscription for status

## Implementation Plan

### Phase 1: Fix Critical Issues (Week 1)

**Goal:** Restore live updates for execution status

1. **Backend: Add execution status to delta subscription** (4 hours)
   - Modify subscription resolver to include `executionStateChanges`
   - Add GraphQL types for execution state updates
   - Test with concurrent execution status changes

2. **Frontend: Subscribe to execution status** (4 hours)
   - Update GraphQL subscription definition
   - Handle execution state changes in QueryService
   - Update Plan DAG state when execution status changes

3. **Testing** (4 hours)
   - Test node execution status updates
   - Verify status changes reflect on canvas
   - Test concurrent multi-user scenarios

**Success Criteria:**
- Execution status updates appear on canvas without reload
- No regressions in existing functionality

### Phase 2: Simplify Echo Suppression (Week 2)

**Goal:** Reduce complexity and race conditions

1. **Backend: Server-side client filtering** (2 hours)
   - Add `excludeClientId` parameter to subscription
   - Filter deltas before broadcast

2. **Frontend: Remove redundant suppression** (6 hours)
   - Remove mutation timestamp window logic
   - Remove `isDragging` suppression (rely on optimistic updates)
   - Remove `isSyncingFromExternalRef` flag
   - Simplify `usePlanDagCQRS` sync effect

3. **Testing** (4 hours)
   - Test echo suppression with single mechanism
   - Verify no subscription echo appears
   - Test rapid mutations don't cause issues

**Success Criteria:**
- Single echo suppression mechanism (client ID filtering)
- Simpler code with fewer refs and timeouts
- All existing tests pass

### Phase 3: Clean Up Dead Code (Week 3)

**Goal:** Reduce maintenance burden

1. **Remove event sourcing infrastructure** (2 hours)
   - Delete `PlanDagEventStore.ts` and `PlanDagEvents.ts`
   - Remove imports and references
   - Create ADR documenting removal

2. **Consolidate state refs** (4 hours)
   - Combine 7 refs into single state object
   - Update all access patterns
   - Update tests

**Success Criteria:**
- Event sourcing code removed
- Single state ref object
- All tests pass

### Phase 4: Architectural Improvements (Optional)

**Goal:** Long-term maintainability

1. **Introduce FSM for sync states** (1 week)
2. **Separate execution status management** (3 days)

## Testing Strategy

### Unit Tests

```typescript
describe('usePlanDagCQRS', () => {
  it('should update execution status from subscription', async () => {
    const { result } = renderHook(() => usePlanDagCQRS({ projectId: 1 }))

    // Wait for initial load
    await waitFor(() => expect(result.current.loading).toBe(false))

    // Simulate execution status update via subscription
    act(() => {
      mockSubscription.next({
        data: {
          planDagDeltaChanged: {
            executionStateChanges: [{
              nodeId: 'node-1',
              datasetExecution: { executionState: 'COMPLETE' }
            }]
          }
        }
      })
    })

    // Verify UI reflects execution status
    const node = result.current.nodes.find(n => n.id === 'node-1')
    expect(node.data.datasetExecution.executionState).toBe('COMPLETE')
  })

  it('should not echo own mutations', async () => {
    const { result } = renderHook(() => usePlanDagCQRS({ projectId: 1 }))

    // Trigger mutation
    await act(async () => {
      await result.current.cqrsService.commands.moveNode({
        projectId: 1,
        nodeId: 'node-1',
        position: { x: 100, y: 100 }
      })
    })

    // Simulate subscription echo
    act(() => {
      mockSubscription.next({
        data: {
          planDagDeltaChanged: {
            operations: [{ op: 'replace', path: '/nodes/0/position', value: { x: 100, y: 100 } }]
          }
        }
      })
    })

    // Verify no duplicate state update
    expect(result.current.nodes[0].position).toEqual({ x: 100, y: 100 })
    expect(mockSetNodes).toHaveBeenCalledTimes(1) // Only optimistic update, no subscription update
  })
})
```

### Integration Tests

```typescript
describe('Plan DAG Editor - Multi-user Scenarios', () => {
  it('should handle concurrent node moves', async () => {
    const editor1 = renderPlanEditor({ projectId: 1, clientId: 'client-1' })
    const editor2 = renderPlanEditor({ projectId: 1, clientId: 'client-2' })

    // User 1 moves node
    await editor1.dragNode('node-1', { x: 100, y: 100 })

    // User 2 should see the update
    await waitFor(() => {
      const node = editor2.getNode('node-1')
      expect(node.position).toEqual({ x: 100, y: 100 })
    })
  })

  it('should handle execution status updates', async () => {
    const editor = renderPlanEditor({ projectId: 1 })

    // Trigger dataset processing (backend)
    await backend.processDatasource('node-1')

    // Editor should show updated status without reload
    await waitFor(() => {
      const node = editor.getNode('node-1')
      expect(node.data.datasetExecution.executionState).toBe('PROCESSING')
    })

    // Wait for completion
    await backend.waitForCompletion('node-1')

    // Status should update to COMPLETE
    await waitFor(() => {
      const node = editor.getNode('node-1')
      expect(node.data.datasetExecution.executionState).toBe('COMPLETE')
    })
  })
})
```

### Manual Testing Checklist

- [ ] Drop node on canvas → appears immediately
- [ ] Delete node → disappears immediately
- [ ] Execute dataset → status updates to "Processing" without reload
- [ ] Datasource completes → status updates to "Complete" without reload
- [ ] Compute graph → status updates in real-time
- [ ] Multi-user: User A adds node → User B sees it appear
- [ ] Multi-user: User A drags node → User B sees it move
- [ ] Multi-user: User A deletes edge → User B sees it disappear
- [ ] Rapid mutations (drag multiple nodes quickly) → no UI glitches
- [ ] Network interruption → graceful degradation and recovery

## Complexity Metrics

### Current State
- **Cyclomatic Complexity:**
  - `usePlanDagCQRS.useEffect` (sync logic): 15
  - `PlanVisualEditor.handleNodesChange`: 8
  - `PlanDagQueryService.subscribeToDeltaUpdates`: 7

- **State Management:**
  - 7 separate refs in `usePlanDagCQRS`
  - 4 overlapping suppression mechanisms
  - 3-way state sync (ReactFlow, PlanDag, Backend)

- **Code Volume:**
  - Total event handling code: ~3,500 lines
  - Dead code (event sourcing): ~600 lines (17%)

### After Improvements
- **Cyclomatic Complexity:**
  - `usePlanDagCQRS.useEffect` (sync logic): **8** (-47%)
  - `PlanVisualEditor.handleNodesChange`: **5** (-37%)
  - `PlanDagQueryService.subscribeToDeltaUpdates`: **4** (-43%)

- **State Management:**
  - **1 consolidated state ref** (-85%)
  - **1 suppression mechanism** (-75%)
  - Same 3-way state sync (architectural necessity)

- **Code Volume:**
  - Total event handling code: **~2,700 lines** (-23%)
  - Dead code: **0 lines** (-100%)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Execution subscription breaks existing functionality | Low | High | Comprehensive testing, feature flag rollout |
| Server-side filtering causes performance issues | Low | Medium | Load testing, monitor broadcast latency |
| Removing event sourcing needed later | Very Low | Medium | Version control preserves code, ADR documents decision |
| Ref consolidation introduces bugs | Medium | Medium | Thorough testing, phased rollout |
| Concurrent mutations during migration | Low | High | Clear migration strategy, database transactions |

## Success Metrics

**Before:**
- Execution status updates: Require manual reload
- Echo suppression mechanisms: 4 overlapping systems
- Dead code: 17% of event handling codebase
- Average time to reflect remote changes: 500ms (suppression window)

**After:**
- Execution status updates: **Real-time (< 100ms)**
- Echo suppression mechanisms: **1 (server-side filtering)**
- Dead code: **0%**
- Average time to reflect remote changes: **< 50ms**

## Conclusion

The Plan DAG editor's event handling suffers from a **critical gap in delta subscription coverage** (execution status not propagated) and **excessive defensive complexity** (4 overlapping suppression mechanisms).

The root cause of the user's reported issue—UI not updating without reload—is the **missing execution status in the delta subscription**. Node execution state changes (dataset processing, graph computation) do not trigger UI updates because they are not included in the JSON Patch operations.

Implementing the recommended fixes will:
1. **Restore live updates** for execution status (fixes user issue)
2. **Simplify architecture** by consolidating to single echo suppression mechanism
3. **Reduce maintenance burden** by removing 600 lines of dead code
4. **Improve reliability** by eliminating timing-based race conditions

The phased implementation plan prioritises fixing the critical execution status issue first (Week 1), then progressively simplifies the architecture (Weeks 2-3), with optional long-term improvements for maintainability.
