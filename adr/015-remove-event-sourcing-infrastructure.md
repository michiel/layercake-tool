# ADR 015: Remove Event Sourcing Infrastructure

**Status:** Accepted

**Date:** 2025-10-20

**Deciders:** Development Team

## Context

The frontend codebase included event sourcing infrastructure (`PlanDagEventStore` and `PlanDagEvents`) totalling approximately 600 lines of code. This infrastructure was intended to provide state reconstruction from an event log for the Plan DAG editor.

However, during a comprehensive review of event handling for real-time collaboration, we discovered that:

1. **No Integration:** The event sourcing infrastructure was never integrated into any components
2. **Alternative Solution:** Delta-based subscriptions using JSON Patch already provide efficient state synchronisation
3. **Cognitive Overhead:** The unused infrastructure added confusion for developers trying to understand the system architecture
4. **Maintenance Burden:** Maintaining code that isn't used increases the risk of bugs and technical debt

## Decision

We have decided to **remove the event sourcing infrastructure** from the frontend codebase, specifically:

- `frontend/src/events/PlanDagEvents.ts` (323 lines)
- `frontend/src/stores/PlanDagEventStore.ts` (280 lines)

## Rationale

### Why Event Sourcing Was Not Needed

1. **JSON Patch Delta Subscription:** The existing `PLAN_DAG_DELTA_SUBSCRIPTION` already provides incremental state updates via RFC 6902 JSON Patch operations, achieving the same efficiency goals as event sourcing

2. **Simpler Architecture:** Direct state updates via subscriptions are easier to reason about than event log reconstruction

3. **Real-time Collaboration:** Our collaboration model uses WebSocket subscriptions with delta updates, which integrates seamlessly with GraphQL subscriptions without needing a separate event store

4. **No Historical Replay Requirement:** The application does not require replaying historical changes or reconstructing state from past events

### Current State Management Approach

Instead of event sourcing, we use:

```
┌─────────────────┐
│  Backend        │
│  (Source of     │
│   Truth)        │
└────────┬────────┘
         │
         │ GraphQL Subscriptions
         │ (Delta Updates via JSON Patch)
         ▼
┌─────────────────┐
│  usePlanDagCQRS │
│  Hook           │
│                 │
│  - Delta Sub    │◄── Structural changes (nodes, edges, positions)
│  - Exec Status  │◄── Execution status (processing, complete, error)
│    Sub          │
└────────┬────────┘
         │
         │ ReactFlow Adapter
         ▼
┌─────────────────┐
│  ReactFlow      │
│  Canvas         │
└─────────────────┘
```

This approach provides:
- **Real-time updates:** Changes appear immediately via WebSocket subscriptions
- **Efficient bandwidth:** Only deltas are transmitted, not full state
- **Simpler codebase:** No dual state management between event log and current state
- **Proven reliability:** JSON Patch is an established standard (RFC 6902)

## Consequences

### Positive

- **Reduced Complexity:** 600 lines of unused infrastructure removed
- **Clearer Architecture:** Developers can focus on the actual state management approach in use
- **Lower Maintenance Burden:** Fewer files to maintain, update, and potentially debug
- **Easier Onboarding:** New developers won't be confused by unused patterns

### Negative

- **Loss of Historical Context:** If future requirements demand event sourcing, it would need to be reimplemented
  - *Mitigation:* The code is preserved in Git history and can be restored if needed

### Neutral

- **No Functional Impact:** Since the code was never integrated, its removal doesn't affect any existing functionality

## Implementation Notes

The removal was straightforward:
1. Deleted `frontend/src/events/PlanDagEvents.ts`
2. Deleted `frontend/src/stores/PlanDagEventStore.ts`
3. Verified no imports or references existed in the codebase
4. All tests continued to pass

## Future Considerations

If event sourcing becomes necessary in the future (e.g., for audit trails, undo/redo, or time-travel debugging), consider:

1. **Assess Actual Needs:** Determine if simpler alternatives (e.g., command history, snapshot-based undo) suffice
2. **Backend Event Sourcing:** Implement event sourcing on the backend where it can serve multiple clients
3. **Existing Patterns:** Build on the delta subscription pattern already in use
4. **Proven Libraries:** Use established event sourcing libraries rather than custom implementation

## References

- [RFC 6902: JSON Patch](https://tools.ietf.org/html/rfc6902)
- Technical Review: `docs/docs/reviews/2025-10-20-editor_events.md`
- Event Sourcing Pattern: https://martinfowler.com/eaaDev/EventSourcing.html
