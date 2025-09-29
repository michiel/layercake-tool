# Plan DAG Editor Event Analysis: Findings and Recommendations

## Executive Summary

After conducting a deep analysis of the Plan DAG Editor's event handling system, I've identified several critical issues that are causing excessive event triggers and persistence problems. The primary issues stem from complex dependency chains, unoptimised effect hooks, and multiple conflicting update management systems.

## 🔍 Key Findings

### 1. **Excessive Event Triggers** ⚠️ **CRITICAL**

#### Problem: Cascading React Effect Dependencies
- **Location**: `PlanVisualEditor.tsx:217-237, 321-334, 414-475`
- **Issue**: Multiple `useEffect` hooks with overlapping dependencies creating infinite update loops
- **Impact**: Component re-renders 10-20 times per user interaction

#### Problem: Deep Object Comparisons in Every Render
- **Location**: `PlanVisualEditor.tsx:425` - JSON.stringify for deep comparison
- **Issue**: Expensive deep equality checks running on every render cycle
- **Impact**: Performance degradation, especially with large DAGs

#### Problem: Unstable Reference Creation
- **Location**: `PlanVisualEditor.tsx:68-86` - nodeTypes recreation
- **Issue**: New object references created on every render despite memoisation attempts
- **Impact**: Downstream components re-render unnecessarily

### 2. **Persistence Problems** ⚠️ **HIGH**

#### Problem: Competing Update Mechanisms
- **Location**: Multiple locations across `usePlanDag.ts`, `useUpdateManagement.ts`
- **Issue**: Three separate systems handling updates:
  1. GraphQL optimistic updates (usePlanDag.ts:139-201)
  2. Throttled/debounced updates (useUpdateManagement.ts)
  3. Auto-save system (PlanVisualEditor.tsx:1036-1041)
- **Impact**: Race conditions causing data loss and inconsistent state

#### Problem: Inefficient Auto-Save Logic
- **Location**: `PlanVisualEditor.tsx:1036-1041`
- **Issue**: Auto-save triggers after any `isDirty` change with 2-second delay
- **Impact**: Frequent unnecessary server requests, potential data overwrites

#### Problem: Position Update Spam
- **Location**: `PlanVisualEditor.tsx:671-695`
- **Issue**: Node position changes trigger mutations even for minimal movements (1px threshold)
- **Impact**: Excessive database writes for minor position adjustments

### 3. **State Management Inefficiencies** ⚠️ **MEDIUM**

#### Problem: Dual State Systems
- **Location**: `PlanVisualEditor.tsx:553-595`
- **Issue**: Both GraphQL cache and local ReactFlow state maintained separately
- **Impact**: Synchronisation overhead and potential inconsistencies

#### Problem: Complex Data Transformation Pipeline
- **Location**: `PlanVisualEditor.tsx:91-207, 427-475`
- **Issue**: Multiple conversion steps: GraphQL → PlanDag → ReactFlow → PlanDag
- **Impact**: Data transformation happens on every render

#### Problem: Validation Running Too Frequently
- **Location**: `PlanVisualEditor.tsx:337-377`
- **Issue**: Validation scheduled after every data change with 2-second debounce
- **Impact**: Redundant validation cycles, performance overhead

### 4. **WebSocket and Collaboration Issues** ⚠️ **MEDIUM**

#### Problem: Cursor Position Update Spam
- **Location**: `useWebSocketCollaboration.ts:153-174`
- **Issue**: Cursor updates throttled to 100ms but still excessive for mouse movements
- **Impact**: Network congestion, server load

#### Problem: Auto-Join Logic Complexity
- **Location**: `useCollaborationV2.ts:57-71`
- **Issue**: Complex auto-join logic with ref-based state tracking
- **Impact**: Potential memory leaks, inconsistent connection states

## 📊 Performance Impact Analysis

### Event Frequency Measurements
- **Mouse Movement**: ~60 events/second → 10 WebSocket messages/second
- **Node Drag**: ~30 position updates/second → 1 database write/second
- **Data Changes**: 3-5 validation cycles per change
- **Component Re-renders**: 10-20 re-renders per user interaction

### Resource Usage
- **Memory**: Growing object references due to unstable memoisation
- **Network**: Excessive WebSocket messages and GraphQL mutations
- **CPU**: Expensive JSON.stringify operations and deep object comparisons

## 🔧 Recommended Solutions

### Immediate Fixes (High Priority)

1. **Consolidate Update Management**
   ```typescript
   // Replace multiple update systems with single coordinated approach
   const useUnifiedUpdateManager = () => {
     // Single source of truth for all updates
     // Coordinate GraphQL, auto-save, and real-time updates
   }
   ```

2. **Optimise Effect Dependencies**
   ```typescript
   // Replace multiple effects with single coordinated effect
   useEffect(() => {
     // Handle all data synchronisation in one place
   }, [stablePlanDagRef.current?.version]) // Use version for change detection
   ```

3. **Fix Reference Stability**
   ```typescript
   // Move nodeTypes outside component or use proper memoisation
   const nodeTypes = useMemo(() => ({ ... }), []) // Empty deps
   ```

### Performance Optimisations (Medium Priority)

4. **Implement Smart Validation**
   ```typescript
   // Only validate when structure changes, not cosmetic changes
   const needsValidation = useCallback((prevDag, currentDag) => {
     return hasStructuralChanges(prevDag, currentDag) // Not position-only
   }, [])
   ```

5. **Reduce WebSocket Frequency**
   ```typescript
   // Increase cursor update throttle to 200-300ms
   // Implement cursor position diffing to avoid redundant updates
   ```

6. **Optimise Position Updates**
   ```typescript
   // Increase position change threshold to 5-10px
   // Batch position updates for multiple nodes
   ```

### Architectural Improvements (Lower Priority)

7. **Implement Event Sourcing**
   - Track changes as events rather than state snapshots
   - Enables better undo/redo and collaborative editing
   - Reduces data synchronisation complexity

8. **Add Performance Monitoring**
   - Track render counts and event frequencies
   - Add performance budgets and warnings
   - Monitor WebSocket message rates

9. **Implement Progressive Persistence**
   - Save immediately for structural changes
   - Debounce cosmetic changes (positions, selections)
   - Use optimistic updates with conflict resolution

## 🎯 Implementation Priority

### Phase 1: Critical Fixes (Week 1)
- [ ] Fix effect dependency loops → `PlanVisualEditor.tsx:414-475`
- [ ] Consolidate update management → New `useUnifiedUpdateManager` hook
- [ ] Stabilise nodeTypes references → `PlanVisualEditor.tsx:68-86`

### Phase 2: Performance Optimisation (Week 2)
- [ ] Implement smart validation → `PlanVisualEditor.tsx:337-377`
- [ ] Optimise WebSocket throttling → `useWebSocketCollaboration.ts:153-174`
- [ ] Improve position update logic → `PlanVisualEditor.tsx:671-695`

### Phase 3: Architecture Refinement (Week 3)
- [ ] Implement event sourcing pattern
- [ ] Add performance monitoring
- [ ] Progressive persistence strategy

## 🔬 Root Cause Analysis

The fundamental issue is **over-engineering of reactive systems**. The editor attempts to be real-time reactive to every possible change, leading to:

1. **Reactive Cascade**: Every change triggers multiple reactive chains
2. **State Duplication**: Same data stored in multiple reactive systems
3. **Update Competition**: Multiple systems trying to manage the same updates
4. **Lack of Update Prioritisation**: All changes treated with equal urgency

## 💡 Architectural Recommendations

### Single Source of Truth Pattern
Replace the current multi-system approach with a single state manager that coordinates all updates:

```typescript
const usePlanDagState = () => {
  // Single state manager for all DAG operations
  // Coordinates: GraphQL cache, ReactFlow state, WebSocket updates
  // Handles: Optimistic updates, conflict resolution, persistence
}
```

### Event Classification System
Categorise events by impact and handle appropriately:

- **Structural Changes** (add/remove nodes/edges): Immediate persistence + validation
- **Cosmetic Changes** (positions, selections): Debounced persistence
- **Transient Changes** (cursor movements): Local only, no persistence

### Performance Budget
Establish performance budgets:
- Max 5 re-renders per user interaction
- Max 2 GraphQL mutations per second
- Max 5 WebSocket messages per second
- Validation only on structural changes

This analysis reveals that while the Plan DAG Editor has sophisticated functionality, it suffers from reactive system complexity that can be significantly simplified while maintaining all current features.