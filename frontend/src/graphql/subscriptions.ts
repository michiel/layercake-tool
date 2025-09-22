import { gql } from '@apollo/client'

// Plan DAG Update Subscription
export const PLAN_DAG_UPDATED_SUBSCRIPTION = gql`
  subscription PlanDagUpdated($planId: ID!) {
    planDagUpdated(planId: $planId) {
      planId
      updateType
      data {
        node {
          id
          type
          config
          metadata {
            label
            description
          }
        }
        edge {
          id
          source
          target
          connectionType
        }
        metadata
      }
      userId
      timestamp
    }
  }
`

// User Presence Subscription
export const USER_PRESENCE_CHANGED_SUBSCRIPTION = gql`
  subscription UserPresenceChanged($planId: ID!) {
    userPresenceChanged(planId: $planId) {
      userId
      userName
      avatarColor
      planId
      isOnline
      cursorPosition {
        x
        y
      }
      selectedNodeId
      lastActive
    }
  }
`

// All Collaboration Events Subscription
export const COLLABORATION_EVENTS_SUBSCRIPTION = gql`
  subscription CollaborationEvents($planId: ID!) {
    collaborationEvents(planId: $planId) {
      eventId
      planId
      userId
      eventType
      timestamp
      data {
        nodeEvent {
          node {
            id
            type
            config
            metadata {
              label
              description
            }
          }
        }
        edgeEvent {
          edge {
            id
            source
            target
            connectionType
          }
        }
        userEvent {
          userId
          userName
          avatarColor
        }
        cursorEvent {
          userId
          userName
          avatarColor
          positionX
          positionY
          selectedNodeId
        }
      }
    }
  }
`

// TypeScript interfaces for subscription data
export interface PlanDagUpdateEvent {
  planId: string
  updateType: 'NODE_ADDED' | 'NODE_UPDATED' | 'NODE_REMOVED' | 'EDGE_ADDED' | 'EDGE_REMOVED' | 'METADATA_UPDATED'
  data: {
    node?: {
      id: string
      type: string
      config: any
      metadata: {
        label: string
        description?: string
      }
    }
    edge?: {
      id: string
      source: string
      target: string
      connectionType: string
    }
    metadata?: string
  }
  userId: string
  timestamp: string
}

export interface UserPresenceEvent {
  userId: string
  userName: string
  avatarColor: string
  planId: string
  isOnline: boolean
  cursorPosition?: {
    x: number
    y: number
  }
  selectedNodeId?: string
  lastActive: string
}

export interface CollaborationEvent {
  eventId: string
  planId: string
  userId: string
  eventType: 'NODE_CREATED' | 'NODE_UPDATED' | 'NODE_DELETED' | 'EDGE_CREATED' | 'EDGE_DELETED' | 'USER_JOINED' | 'USER_LEFT' | 'CURSOR_MOVED'
  timestamp: string
  data: {
    nodeEvent?: {
      node: {
        id: string
        type: string
        config: any
        metadata: {
          label: string
          description?: string
        }
      }
    }
    edgeEvent?: {
      edge: {
        id: string
        source: string
        target: string
        connectionType: string
      }
    }
    userEvent?: {
      userId: string
      userName: string
      avatarColor: string
    }
    cursorEvent?: {
      userId: string
      userName: string
      avatarColor: string
      positionX: number
      positionY: number
      selectedNodeId?: string
    }
  }
}