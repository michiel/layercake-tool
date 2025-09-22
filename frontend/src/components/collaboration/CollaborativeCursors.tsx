import { memo, useCallback, useEffect, useState } from 'react'
import { useViewport } from 'reactflow'
import { CollaborativeCursor } from './CollaborativeCursor'
import { UserPresence } from '../../hooks/useCollaborationSubscriptions'

interface CollaborativeCursorsProps {
  users: UserPresence[]
  currentUserId?: string
}

export const CollaborativeCursors = memo(({
  users,
  currentUserId
}: CollaborativeCursorsProps) => {
  const viewport = useViewport()
  const [visibleCursors, setVisibleCursors] = useState<UserPresence[]>([])

  // Filter users to show cursors for (exclude current user, only show online users with cursor positions)
  const updateVisibleCursors = useCallback(() => {
    const cursorsToShow = users.filter(user =>
      user.userId !== currentUserId &&
      user.isOnline &&
      user.cursorPosition
    )
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
      {visibleCursors.map((user) => (
        user.cursorPosition && (
          <CollaborativeCursor
            key={user.userId}
            userId={user.userId}
            userName={user.userName}
            avatarColor={user.avatarColor}
            position={user.cursorPosition}
            viewport={viewport}
            selectedNodeId={user.selectedNodeId}
          />
        )
      ))}
    </>
  )
})

CollaborativeCursors.displayName = 'CollaborativeCursors'