import { useState, useEffect, useRef, useCallback, ReactNode } from 'react'
import {
  useCollaborationEventsSubscription,
  useConflictDetection,
  useCollaborationConnection,
  type ConflictEvent
} from '../../../../hooks/useCollaborationSubscriptions'
import { CollaborationEvent } from '../../../../graphql/subscriptions'
import { useUserPresence, useCollaboration } from '../../../../hooks/usePlanDag'
import { UserPresenceIndicator } from '../../../collaboration/UserPresenceIndicator'
import { CollaborativeCursors } from '../../../collaboration/CollaborativeCursors'
import { UserPresenceData, ConnectionState } from '../../../../types/websocket'

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
  const { users: onlineUsersLegacy } = useUserPresence(projectId, currentUserId)
  const { joinProject, leaveProject } = useCollaboration(projectId)
  const { status: _collaborationStatus, isConnected: _isConnected, hasError: _hasError } = useCollaborationConnection(projectId.toString())
  const { getActiveConflicts } = useConflictDetection(projectId.toString())

  // Convert old UserPresence type to new UserPresenceData type
  const convertToUserPresenceData = (user: any): UserPresenceData => {
    const documents: Record<string, any> = {};

    // Create a mock document entry if the user has cursor position
    if (user.cursorPosition) {
      documents['plan-dag-canvas'] = {
        position: {
          type: 'canvas' as const,
          x: user.cursorPosition.x,
          y: user.cursorPosition.y
        },
        selectedNodeId: user.selectedNodeId,
        lastActiveInDocument: user.lastActive
      };
    }

    return {
      userId: user.userId,
      userName: user.userName,
      avatarColor: user.avatarColor,
      isOnline: user.isOnline,
      lastActive: user.lastActive,
      documents
    };
  };

  // Convert legacy users to new format
  const onlineUsers: UserPresenceData[] = onlineUsersLegacy.map(convertToUserPresenceData);

  // Collaboration events state (for future use)
  const [_collaborationEvents, setCollaborationEvents] = useState<CollaborationEvent[]>([])
  const [_activeConflicts, setActiveConflicts] = useState<ConflictEvent[]>([])
  const collaborationEventsRef = useRef<CollaborationEvent[]>([])

  // Subscribe to collaboration events
  const handleCollaborationEvent = useCallback((event: CollaborationEvent) => {
    const newEvent = event
    if (!collaborationEventsRef.current.some(existing => existing.eventId === newEvent.eventId)) {
      setCollaborationEvents(prevEvents => [...prevEvents, newEvent])
      collaborationEventsRef.current = [...collaborationEventsRef.current, newEvent]
      console.log('New collaboration event received:', newEvent)
    }
  }, [])

  useCollaborationEventsSubscription(projectId.toString(), handleCollaborationEvent)

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
    return () => {
      leaveProject()
    }
  }, [joinProject, leaveProject])

  // Collaboration data available through props and component state
  // Future versions could expose this data via React Context if needed

  return (
    <>
      {children}

      {/* User Presence Indicator */}
      <UserPresenceIndicator
        users={onlineUsers}
        connectionState={ConnectionState.CONNECTED}
        maxVisible={5}
        size="sm"
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