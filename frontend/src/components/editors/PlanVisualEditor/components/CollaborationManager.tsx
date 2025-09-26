import { useState, useEffect, useRef, useCallback, ReactNode } from 'react'
import {
  UserPresence,
  useCollaborationEventsSubscription,
  useConflictDetection,
  useCollaborationConnection,
  type ConflictEvent
} from '../../../../hooks/useCollaborationSubscriptions'
import { CollaborationEvent } from '../../../../graphql/subscriptions'
import { useUserPresence, useCollaboration } from '../../../../hooks/usePlanDag'
import { UserPresenceIndicator } from '../../../collaboration/UserPresenceIndicator'
import { CollaborativeCursors } from '../../../collaboration/CollaborativeCursors'

interface CollaborationManagerProps {
  projectId: number
  currentUserId: string
  children?: ReactNode
}

export const CollaborationManager = ({
  projectId,
  currentUserId,
  children
}: CollaborationManagerProps) => {
  // Collaboration hooks
  const { users: onlineUsers } = useUserPresence(projectId, currentUserId)
  const { broadcastCursorPosition, joinProject, leaveProject } = useCollaboration(projectId)
  const { status: collaborationStatus, isConnected, hasError } = useCollaborationConnection(projectId.toString())
  const { getActiveConflicts } = useConflictDetection(projectId.toString())

  // Collaboration events state
  const [collaborationEvents, setCollaborationEvents] = useState<CollaborationEvent[]>([])
  const [activeConflicts, setActiveConflicts] = useState<ConflictEvent[]>([])
  const collaborationEventsRef = useRef<CollaborationEvent[]>([])

  // Subscribe to collaboration events
  const { events } = useCollaborationEventsSubscription(projectId.toString())

  // Handle collaboration events
  useEffect(() => {
    if (events && events.length > 0) {
      const newEvents = events.filter(
        event => !collaborationEventsRef.current.some(existing => existing.eventId === event.eventId)
      )

      if (newEvents.length > 0) {
        setCollaborationEvents(prevEvents => [...prevEvents, ...newEvents])
        collaborationEventsRef.current = [...collaborationEventsRef.current, ...newEvents]
        console.log('New collaboration events received:', newEvents)
      }
    }
  }, [events])

  // Handle conflicts
  useEffect(() => {
    const checkConflicts = async () => {
      const conflicts = await getActiveConflicts()
      setActiveConflicts(conflicts)
    }

    const interval = setInterval(checkConflicts, 5000) // Check every 5 seconds
    return () => clearInterval(interval)
  }, [getActiveConflicts])

  // Join project on mount, leave on unmount
  useEffect(() => {
    joinProject()
    return () => leaveProject()
  }, [joinProject, leaveProject])

  // Provide collaboration data and functions to children
  const collaborationData = {
    onlineUsers,
    collaborationStatus,
    isConnected,
    hasError,
    collaborationEvents,
    activeConflicts,
    broadcastCursorPosition
  }

  return (
    <>
      {children}

      {/* User Presence Indicator */}
      <UserPresenceIndicator
        users={onlineUsers}
        currentUserId={currentUserId}
        projectId={projectId}
      />

      {/* Collaborative Cursors */}
      <CollaborativeCursors
        users={onlineUsers}
        currentUserId={currentUserId}
      />
    </>
  )
}

// Hook to access collaboration data from context if needed
export const useCollaborationContext = () => {
  // This could be implemented with React Context if needed
  // For now, return null as a placeholder
  return null
}