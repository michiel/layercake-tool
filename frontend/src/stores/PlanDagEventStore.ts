import { PlanDag } from '../types/plan-dag'
import { PlanDagEvent } from '../events/PlanDagEvents'

/**
 * Event Store for Plan DAG state management
 * Implements event sourcing pattern to reconstruct state from events
 * Provides optimistic update handling and conflict resolution
 */
export class PlanDagEventStore {
  private events: PlanDagEvent[] = []
  private listeners: Set<(event: PlanDagEvent) => void> = new Set()
  private snapshotThreshold = 50 // Create snapshots every N events
  private lastSnapshot: { events: number; state: PlanDag } | null = null

  constructor(
    private projectId: number,
    private clientId: string,
    initialState?: PlanDag
  ) {
    console.log('[PlanDagEventStore] Initialized for project:', projectId)

    if (initialState) {
      this.createInitialSnapshot(initialState)
    }
  }

  // Event dispatch and notification
  dispatch(event: PlanDagEvent): void {
    console.log('[PlanDagEventStore] Dispatching event:', event.type, event.eventId)

    // Validate event
    if (event.projectId !== this.projectId) {
      console.warn('[PlanDagEventStore] Event project ID mismatch:', event.projectId, this.projectId)
      return
    }

    // Add to event log
    this.events.push(event)

    // Notify listeners
    this.notify(event)

    // Check if we should create a snapshot
    this.checkSnapshotThreshold()
  }

  // State reconstruction from events
  getState(): PlanDag {
    console.log('[PlanDagEventStore] Reconstructing state from', this.events.length, 'events')

    let state: PlanDag

    // Start from snapshot if available
    if (this.lastSnapshot) {
      state = { ...this.lastSnapshot.state }
      const eventsToApply = this.events.slice(this.lastSnapshot.events)
      console.log('[PlanDagEventStore] Applying', eventsToApply.length, 'events from snapshot')
      return eventsToApply.reduce(this.reducer, state)
    }

    // Reconstruct from scratch
    state = this.getInitialState()
    return this.events.reduce(this.reducer, state)
  }

  // Event reduction logic
  private reducer = (state: PlanDag, event: PlanDagEvent): PlanDag => {
    console.log('[PlanDagEventStore] Reducing event:', event.type)

    switch (event.type) {
      case 'NODE_CREATED':
        return {
          ...state,
          nodes: [...state.nodes, event.payload.node],
          metadata: {
            ...state.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'NODE_UPDATED':
        return {
          ...state,
          nodes: state.nodes.map(node =>
            node.id === event.payload.nodeId
              ? { ...node, ...event.payload.updates }
              : node
          ),
          metadata: {
            ...state.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'NODE_DELETED':
        return {
          ...state,
          nodes: state.nodes.filter(node => node.id !== event.payload.nodeId),
          // Also remove edges connected to deleted node
          edges: state.edges.filter(edge =>
            edge.source !== event.payload.nodeId && edge.target !== event.payload.nodeId
          ),
          metadata: {
            ...state.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'NODE_MOVED':
        return {
          ...state,
          nodes: state.nodes.map(node =>
            node.id === event.payload.nodeId
              ? { ...node, position: event.payload.newPosition }
              : node
          ),
          metadata: {
            ...state.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'EDGE_CREATED':
        return {
          ...state,
          edges: [...state.edges, event.payload.edge],
          metadata: {
            ...state.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'EDGE_DELETED':
        return {
          ...state,
          edges: state.edges.filter(edge => edge.id !== event.payload.edgeId),
          metadata: {
            ...state.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'PLAN_DAG_UPDATED':
        return {
          ...event.payload.planDag,
          metadata: {
            ...event.payload.planDag.metadata,
            lastModified: new Date(event.timestamp).toISOString(),
            version: event.version.toString()
          }
        }

      case 'REMOTE_CHANGE_RECEIVED':
        // Remote changes are complete state updates
        return {
          ...event.payload.planDag,
          metadata: {
            ...event.payload.planDag.metadata,
            lastModified: new Date(event.timestamp).toISOString()
          }
        }

      case 'OPTIMISTIC_UPDATE_APPLIED':
        // For optimistic updates, we don't change state here
        // The actual change events will be dispatched separately
        return state

      case 'OPTIMISTIC_UPDATE_CONFIRMED':
        // Confirmation doesn't change state, just marks operation as successful
        return state

      case 'OPTIMISTIC_UPDATE_ROLLBACK':
        // Rollback would require more complex logic to revert changes
        // For now, we'll trigger a full state refresh
        console.warn('[PlanDagEventStore] Optimistic rollback - consider full refresh')
        return state

      default:
        console.warn('[PlanDagEventStore] Unknown event type:', (event as any).type)
        return state
    }
  }

  // Event listener management
  subscribe(listener: (event: PlanDagEvent) => void): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  private notify(event: PlanDagEvent): void {
    this.listeners.forEach(listener => {
      try {
        listener(event)
      } catch (error) {
        console.error('[PlanDagEventStore] Error in event listener:', error)
      }
    })
  }

  // Snapshot management
  private createInitialSnapshot(state: PlanDag): void {
    this.lastSnapshot = {
      events: 0,
      state: { ...state }
    }
    console.log('[PlanDagEventStore] Created initial snapshot')
  }

  private checkSnapshotThreshold(): void {
    const eventsSinceSnapshot = this.lastSnapshot ? this.events.length - this.lastSnapshot.events : this.events.length

    if (eventsSinceSnapshot >= this.snapshotThreshold) {
      this.createSnapshot()
    }
  }

  private createSnapshot(): void {
    const currentState = this.getState()
    this.lastSnapshot = {
      events: this.events.length,
      state: { ...currentState }
    }
    console.log('[PlanDagEventStore] Created snapshot at event', this.events.length)
  }

  // Event history and debugging
  getEventHistory(): PlanDagEvent[] {
    return [...this.events]
  }

  getEventsAfter(timestamp: number): PlanDagEvent[] {
    return this.events.filter(event => event.timestamp > timestamp)
  }

  getEventsByType(type: PlanDagEvent['type']): PlanDagEvent[] {
    return this.events.filter(event => event.type === type)
  }

  // State management helpers
  private getInitialState(): PlanDag {
    return {
      version: '1',
      nodes: [],
      edges: [],
      metadata: {
        version: '1',
        name: 'New Plan',
        description: '',
        created: new Date().toISOString(),
        lastModified: new Date().toISOString(),
        author: 'Unknown'
      }
    }
  }

  // Clear events (useful for testing or reset)
  clear(): void {
    this.events = []
    this.lastSnapshot = null
    console.log('[PlanDagEventStore] Cleared all events')
  }

  // Get store statistics
  getStats() {
    return {
      totalEvents: this.events.length,
      eventsInSnapshot: this.lastSnapshot?.events || 0,
      eventsSinceSnapshot: this.lastSnapshot ? this.events.length - this.lastSnapshot.events : this.events.length,
      listeners: this.listeners.size,
      projectId: this.projectId,
      clientId: this.clientId
    }
  }
}