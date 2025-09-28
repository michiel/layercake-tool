import { useEffect, ReactNode } from 'react'
import { useCollaborationV2 } from '../../../../hooks/useCollaborationV2'
import { CollaborativeCursors } from '../../../collaboration/CollaborativeCursors'
import { UserPresenceData } from '../../../../types/websocket'

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
  // New WebSocket collaboration hook
  const collaboration = useCollaborationV2({
    projectId,
    documentId: 'plan-dag-canvas',
    documentType: 'canvas',
    enableWebSocket: true,
    userInfo: {
      id: currentUserId,
      name: `User ${currentUserId}`,
      avatarColor: '#3b82f6'
    }
  })

  // Use users directly from the new collaboration hook
  const onlineUsers: UserPresenceData[] = collaboration.users || []

  // Note: Conflict detection will be handled via WebSocket in the future
  // For now, collaboration events and conflicts are managed through the WebSocket system

  // Join project on mount, leave on unmount
  useEffect(() => {
    collaboration.joinProject()
    return () => {
      collaboration.leaveProject()
    }
  }, [collaboration])

  // Collaboration data available through props and component state
  // Future versions could expose this data via React Context if needed

  return (
    <>
      {children}

      {/* Collaborative Cursors - presence is now handled by TopBar */}
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