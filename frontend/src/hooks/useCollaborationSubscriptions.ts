// import { useSubscription } from '@apollo/client'
import { useEffect, useRef, useCallback } from 'react'
// import {
//   PLAN_DAG_UPDATED_SUBSCRIPTION,
//   USER_PRESENCE_CHANGED_SUBSCRIPTION,
//   COLLABORATION_EVENTS_SUBSCRIPTION,
//   PlanDagUpdateEvent,
//   UserPresenceEvent,
//   CollaborationEvent,
// } from '../graphql/subscriptions'

// Mock types for frontend-only development
interface PlanDagUpdateEvent {
  id: string;
  userId: string;
  timestamp: string;
}

interface UserPresenceEvent {
  userId: string;
  userName: string;
  avatarColor: string;
  isOnline: boolean;
  cursorPosition?: { x: number; y: number };
  selectedNodeId?: string;
  lastActive: string;
}

interface CollaborationEvent {
  eventType: string;
  userId: string;
  data: any;
}

// Hook for Plan DAG real-time updates - Mock for frontend-only development
export const usePlanDagSubscription = (
  planId: string,
  onUpdate?: (event: PlanDagUpdateEvent) => void
) => {
  // Mock implementation for frontend-only development
  return {
    data: null,
    loading: false,
    error: null,
  }
}

// Hook for user presence tracking - Mock for frontend-only development
export const useUserPresenceSubscription = (
  planId: string,
  onPresenceChange?: (event: UserPresenceEvent) => void
) => {
  // Mock implementation for frontend-only development
  return {
    data: null,
    loading: false,
    error: null,
  }
}

// Hook for all collaboration events - Mock for frontend-only development
export const useCollaborationEventsSubscription = (
  planId: string,
  onEvent?: (event: CollaborationEvent) => void
) => {
  // Mock implementation for frontend-only development
  return {
    data: null,
    loading: false,
    error: null,
  }
}

// User presence state management
export interface UserPresence {
  userId: string
  userName: string
  avatarColor: string
  isOnline: boolean
  cursorPosition?: { x: number; y: number }
  selectedNodeId?: string
  lastActive: string
}

// Combined hook for managing user presence state
export const useUserPresence = (planId: string, currentUserId?: string) => {
  const usersRef = useRef<Map<string, UserPresence>>(new Map())

  const updateUserPresence = useCallback((event: UserPresenceEvent) => {
    const user: UserPresence = {
      userId: event.userId,
      userName: event.userName,
      avatarColor: event.avatarColor,
      isOnline: event.isOnline,
      cursorPosition: event.cursorPosition,
      selectedNodeId: event.selectedNodeId,
      lastActive: event.lastActive,
    }

    if (event.isOnline) {
      usersRef.current.set(event.userId, user)
    } else {
      usersRef.current.delete(event.userId)
    }
  }, [])

  const { loading, error } = useUserPresenceSubscription(planId, updateUserPresence)

  const getOnlineUsers = useCallback(() => {
    return Array.from(usersRef.current.values()).filter(
      user => user.isOnline && user.userId !== currentUserId
    )
  }, [currentUserId])

  const getUserCursor = useCallback((userId: string) => {
    return usersRef.current.get(userId)?.cursorPosition
  }, [])

  return {
    loading,
    error,
    getOnlineUsers,
    getUserCursor,
  }
}

// Connection status management
export const useCollaborationConnection = (planId: string) => {
  const connectionRef = useRef<'connecting' | 'connected' | 'disconnected' | 'error'>('connecting')

  const { loading: planLoading, error: planError } = usePlanDagSubscription(planId)
  const { loading: presenceLoading, error: presenceError } = useUserPresenceSubscription(planId)

  useEffect(() => {
    if (planError || presenceError) {
      connectionRef.current = 'error'
    } else if (planLoading || presenceLoading) {
      connectionRef.current = 'connecting'
    } else {
      connectionRef.current = 'connected'
    }
  }, [planLoading, presenceLoading, planError, presenceError])

  return {
    status: connectionRef.current,
    isConnected: connectionRef.current === 'connected',
    hasError: connectionRef.current === 'error',
    error: planError || presenceError,
  }
}

// Hook for broadcasting cursor position
export const useCursorBroadcast = (planId: string, userId: string) => {
  const lastPositionRef = useRef<{ x: number; y: number } | null>(null)
  const broadcastTimeoutRef = useRef<number | null>(null)

  const broadcastCursorPosition = useCallback((x: number, y: number, selectedNodeId?: string) => {
    // Throttle cursor updates to avoid spam
    if (broadcastTimeoutRef.current) {
      clearTimeout(broadcastTimeoutRef.current)
    }

    broadcastTimeoutRef.current = window.setTimeout(() => {
      lastPositionRef.current = { x, y }

      // In a real implementation, this would publish a cursor moved event
      console.log('Broadcasting cursor position:', {
        planId,
        userId,
        position: { x, y },
        selectedNodeId
      })

      // TODO: Implement actual cursor position broadcasting
      // This would typically be done through a mutation that triggers
      // the collaboration event system
    }, 100) // Throttle to 10 updates per second
  }, [planId, userId])

  useEffect(() => {
    return () => {
      if (broadcastTimeoutRef.current) {
        clearTimeout(broadcastTimeoutRef.current)
      }
    }
  }, [])

  return {
    broadcastCursorPosition,
  }
}

// Hook for conflict detection and resolution
export interface ConflictEvent {
  type: 'node_conflict' | 'edge_conflict'
  nodeId?: string
  edgeId?: string
  conflictingUsers: string[]
  timestamp: string
}

export const useConflictDetection = (planId: string) => {
  const conflictsRef = useRef<ConflictEvent[]>([])

  const detectConflict = useCallback((event: CollaborationEvent) => {
    // Simple conflict detection - multiple users editing same node/edge
    // In a real implementation, this would be more sophisticated

    const now = new Date().toISOString()

    if (event.eventType === 'NODE_UPDATED' && event.data.nodeEvent) {
      const nodeId = event.data.nodeEvent.node.id

      // Check for recent edits to same node by different users
      const recentEdits = conflictsRef.current.filter(
        conflict =>
          conflict.nodeId === nodeId &&
          new Date(conflict.timestamp).getTime() > Date.now() - 5000 // 5 seconds
      )

      if (recentEdits.length > 0) {
        const conflict: ConflictEvent = {
          type: 'node_conflict',
          nodeId,
          conflictingUsers: [...new Set([...recentEdits.flatMap(c => c.conflictingUsers), event.userId])],
          timestamp: now,
        }

        conflictsRef.current.push(conflict)
        console.warn('Node conflict detected:', conflict)
      }
    }
  }, [])

  useCollaborationEventsSubscription(planId, detectConflict)

  const getActiveConflicts = useCallback(() => {
    // Filter out old conflicts (older than 30 seconds)
    const cutoff = Date.now() - 30000
    conflictsRef.current = conflictsRef.current.filter(
      conflict => new Date(conflict.timestamp).getTime() > cutoff
    )

    return conflictsRef.current
  }, [])

  return {
    getActiveConflicts,
  }
}