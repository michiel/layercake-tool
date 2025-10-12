import { PlanDag, PlanDagNode, ReactFlowEdge } from '../types/plan-dag'

/**
 * Event Sourcing Foundation for Plan DAG operations
 * Defines all possible events that can occur in the Plan DAG system
 * Events are immutable facts about what happened
 */

// Base event structure
export interface BaseEvent {
  eventId: string
  timestamp: number
  clientId: string
  projectId: number
  version: number
}

// Node Events
export interface NodeCreatedEvent extends BaseEvent {
  type: 'NODE_CREATED'
  payload: {
    node: PlanDagNode
    position: { x: number; y: number }
  }
}

export interface NodeUpdatedEvent extends BaseEvent {
  type: 'NODE_UPDATED'
  payload: {
    nodeId: string
    updates: Partial<PlanDagNode>
    previousState: PlanDagNode
  }
}

export interface NodeDeletedEvent extends BaseEvent {
  type: 'NODE_DELETED'
  payload: {
    nodeId: string
    deletedNode: PlanDagNode
  }
}

export interface NodeMovedEvent extends BaseEvent {
  type: 'NODE_MOVED'
  payload: {
    nodeId: string
    newPosition: { x: number; y: number }
    previousPosition: { x: number; y: number }
  }
}

// Edge Events
export interface EdgeCreatedEvent extends BaseEvent {
  type: 'EDGE_CREATED'
  payload: {
    edge: ReactFlowEdge
  }
}

export interface EdgeDeletedEvent extends BaseEvent {
  type: 'EDGE_DELETED'
  payload: {
    edgeId: string
    deletedEdge: ReactFlowEdge
  }
}

// Plan DAG Events
export interface PlanDagUpdatedEvent extends BaseEvent {
  type: 'PLAN_DAG_UPDATED'
  payload: {
    planDag: PlanDag
    previousVersion: string
  }
}

export interface PlanDagValidatedEvent extends BaseEvent {
  type: 'PLAN_DAG_VALIDATED'
  payload: {
    planDag: PlanDag
    validationResult: any
    isValid: boolean
  }
}

// Remote Events (from other clients)
export interface RemoteChangeReceivedEvent extends BaseEvent {
  type: 'REMOTE_CHANGE_RECEIVED'
  payload: {
    remoteClientId: string
    changeType: string
    planDag: PlanDag
  }
}

// Optimistic Update Events
export interface OptimisticUpdateAppliedEvent extends BaseEvent {
  type: 'OPTIMISTIC_UPDATE_APPLIED'
  payload: {
    operationId: string
    operation: PlanDagOperation
    expectedResult: any
  }
}

export interface OptimisticUpdateConfirmedEvent extends BaseEvent {
  type: 'OPTIMISTIC_UPDATE_CONFIRMED'
  payload: {
    operationId: string
    serverResult: any
  }
}

export interface OptimisticUpdateRollbackEvent extends BaseEvent {
  type: 'OPTIMISTIC_UPDATE_ROLLBACK'
  payload: {
    operationId: string
    error: Error
    rollbackData: any
  }
}

// Union type of all possible events
export type PlanDagEvent =
  | NodeCreatedEvent
  | NodeUpdatedEvent
  | NodeDeletedEvent
  | NodeMovedEvent
  | EdgeCreatedEvent
  | EdgeDeletedEvent
  | PlanDagUpdatedEvent
  | PlanDagValidatedEvent
  | RemoteChangeReceivedEvent
  | OptimisticUpdateAppliedEvent
  | OptimisticUpdateConfirmedEvent
  | OptimisticUpdateRollbackEvent

// Operation types for optimistic updates
export interface PlanDagOperation {
  type: 'CREATE_NODE' | 'UPDATE_NODE' | 'DELETE_NODE' | 'MOVE_NODE' |
        'CREATE_EDGE' | 'DELETE_EDGE' | 'UPDATE_PLAN_DAG'
  payload: any
  clientId: string
  timestamp: number
}

// Event creation helpers
export const createEvent = {
  nodeCreated: (
    clientId: string,
    projectId: number,
    node: PlanDagNode,
    position: { x: number; y: number }
  ): NodeCreatedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'NODE_CREATED',
    payload: { node, position }
  }),

  nodeUpdated: (
    clientId: string,
    projectId: number,
    nodeId: string,
    updates: Partial<PlanDagNode>,
    previousState: PlanDagNode
  ): NodeUpdatedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'NODE_UPDATED',
    payload: { nodeId, updates, previousState }
  }),

  nodeDeleted: (
    clientId: string,
    projectId: number,
    nodeId: string,
    deletedNode: PlanDagNode
  ): NodeDeletedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'NODE_DELETED',
    payload: { nodeId, deletedNode }
  }),

  nodeMoved: (
    clientId: string,
    projectId: number,
    nodeId: string,
    newPosition: { x: number; y: number },
    previousPosition: { x: number; y: number }
  ): NodeMovedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'NODE_MOVED',
    payload: { nodeId, newPosition, previousPosition }
  }),

  edgeCreated: (
    clientId: string,
    projectId: number,
    edge: ReactFlowEdge
  ): EdgeCreatedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'EDGE_CREATED',
    payload: { edge }
  }),

  edgeDeleted: (
    clientId: string,
    projectId: number,
    edgeId: string,
    deletedEdge: ReactFlowEdge
  ): EdgeDeletedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'EDGE_DELETED',
    payload: { edgeId, deletedEdge }
  }),

  planDagUpdated: (
    clientId: string,
    projectId: number,
    planDag: PlanDag,
    previousVersion: string
  ): PlanDagUpdatedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'PLAN_DAG_UPDATED',
    payload: { planDag, previousVersion }
  }),

  remoteChangeReceived: (
    clientId: string,
    projectId: number,
    remoteClientId: string,
    changeType: string,
    planDag: PlanDag
  ): RemoteChangeReceivedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'REMOTE_CHANGE_RECEIVED',
    payload: { remoteClientId, changeType, planDag }
  }),

  optimisticUpdateApplied: (
    clientId: string,
    projectId: number,
    operationId: string,
    operation: PlanDagOperation,
    expectedResult: any
  ): OptimisticUpdateAppliedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'OPTIMISTIC_UPDATE_APPLIED',
    payload: { operationId, operation, expectedResult }
  }),

  optimisticUpdateConfirmed: (
    clientId: string,
    projectId: number,
    operationId: string,
    serverResult: any
  ): OptimisticUpdateConfirmedEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'OPTIMISTIC_UPDATE_CONFIRMED',
    payload: { operationId, serverResult }
  }),

  optimisticUpdateRollback: (
    clientId: string,
    projectId: number,
    operationId: string,
    error: Error,
    rollbackData: any
  ): OptimisticUpdateRollbackEvent => ({
    eventId: generateEventId(),
    timestamp: Date.now(),
    clientId,
    projectId,
    version: 1,
    type: 'OPTIMISTIC_UPDATE_ROLLBACK',
    payload: { operationId, error, rollbackData }
  })
}

// Event ID generation
function generateEventId(): string {
  return `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}