import { memo } from 'react'
import { IconPointer } from '@tabler/icons-react'

interface CollaborativeCursorProps {
  userId: string
  userName: string
  avatarColor: string
  position: { x: number; y: number }
  viewport: { x: number; y: number; zoom: number }
  selectedNodeId?: string
}

export const CollaborativeCursor = memo(({
  userName,
  avatarColor,
  position,
  viewport,
  selectedNodeId
}: CollaborativeCursorProps) => {
  // Convert world coordinates to screen coordinates within ReactFlow container
  // ReactFlow transformation: screen = (world * zoom) + viewport_offset
  const screenX = (position.x * viewport.zoom) + viewport.x
  const screenY = (position.y * viewport.zoom) + viewport.y

  // Don't render cursor if coordinates are way outside visible area
  if (screenX < -100 || screenY < -100 || screenX > 5000 || screenY > 5000) {
    return null
  }

  return (
    <div
      style={{
        position: 'absolute',
        left: screenX,
        top: screenY,
        pointerEvents: 'none',
        zIndex: 1000,
        transform: 'translate(-2px, -2px)',
      }}
    >
      {/* Cursor pointer */}
      <div
        style={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
          gap: 4,
        }}
      >
        <IconPointer
          size={20}
          style={{
            color: avatarColor,
            filter: 'drop-shadow(0 1px 2px rgba(0,0,0,0.3))',
            transform: 'rotate(-45deg)',
          }}
        />

        {/* User name label */}
        <div
          style={{
            backgroundColor: avatarColor,
            color: 'white',
            padding: '2px 6px',
            borderRadius: 4,
            fontSize: 11,
            fontWeight: 500,
            whiteSpace: 'nowrap',
            boxShadow: '0 1px 3px rgba(0,0,0,0.2)',
            marginLeft: 4,
            marginTop: -2,
          }}
        >
          {userName}
          {selectedNodeId && (
            <span style={{ opacity: 0.8, marginLeft: 4, fontSize: 10 }}>
              ({selectedNodeId})
            </span>
          )}
        </div>
      </div>
    </div>
  )
})

CollaborativeCursor.displayName = 'CollaborativeCursor'