import { memo, useCallback, useEffect, useState } from 'react'
import { useViewport } from 'reactflow'
import { CollaborativeCursor } from './CollaborativeCursor'
import { UserPresenceData } from '../../types/websocket'

interface CollaborativeCursorsProps {
  users: UserPresenceData[]
  currentUserId?: string
}

export const CollaborativeCursors = memo(({
  users,
  currentUserId
}: CollaborativeCursorsProps) => {
  const viewport = useViewport()
  const [visibleCursors, setVisibleCursors] = useState<UserPresenceData[]>([])

  // Filter users to show cursors for (exclude current user, only show online users with canvas cursor positions)
  const updateVisibleCursors = useCallback(() => {
    const cursorsToShow = users.filter(user => {
      const canvasData = user.documents['plan-dag-canvas']
      return (
        user.userId !== currentUserId &&
        user.isOnline &&
        canvasData?.position?.type === 'canvas'
      )
    })
    setVisibleCursors(cursorsToShow)
  }, [users, currentUserId])

  useEffect(() => {
    updateVisibleCursors()
  }, [updateVisibleCursors])

  // Only render if we have cursors to show
  if (visibleCursors.length === 0) {
    return null
  }

  return (
    <>
      {visibleCursors.map((user) => {
        const canvasData = user.documents['plan-dag-canvas']
        const position = canvasData?.position

        // Type guard to ensure we have a canvas position
        if (!position || position.type !== 'canvas') return null

        return (
          <CollaborativeCursor
            key={user.userId}
            userId={user.userId}
            userName={user.userName}
            avatarColor={user.avatarColor}
            position={{ x: position.x, y: position.y }}
            viewport={viewport}
            selectedNodeId={canvasData.selectedNodeId}
          />
        )
      })}
    </>
  )
})

CollaborativeCursors.displayName = 'CollaborativeCursors'