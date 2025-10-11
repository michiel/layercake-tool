# Multi-User Presence and Cursor Tracking in the Plan DAG Editor

## Overview

The Layercake Plan DAG Editor implements a sophisticated real-time collaboration system that enables multiple users to work simultaneously on the same plan visualization. This system provides live cursor tracking, user presence indicators, and conflict detection to ensure smooth collaborative editing experiences.

## Architecture

### System Components

The collaboration system consists of several interconnected components:

1. **Backend Database Layer** - User presence persistence and session management
2. **GraphQL Subscriptions** - Real-time event broadcasting
3. **Frontend React Hooks** - Client-side state management and event handling
4. **UI Components** - Visual representation of collaborative features

### Data Flow

```
User Interaction → Frontend Hook → GraphQL Mutation → Database Update → Subscription Event → Other Clients
```

## Backend Implementation

### Database Schema

#### User Presence Table (`user_presence`)

The core of the presence system is the `user_presence` table that tracks detailed user activity:

```sql
CREATE TABLE user_presence (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    session_id VARCHAR(255) NOT NULL,
    layercake_graph_id INTEGER,
    cursor_position TEXT,          -- JSON: {x: f64, y: f64}
    selected_node_id VARCHAR(255),
    viewport_position TEXT,        -- JSON: {x: f64, y: f64, zoom: f64}
    current_tool VARCHAR(50),      -- "select", "pan", "node_creation", etc.
    is_online BOOLEAN DEFAULT true,
    last_seen TIMESTAMP NOT NULL,
    last_heartbeat TIMESTAMP NOT NULL,
    status VARCHAR(20) DEFAULT 'active', -- "active", "idle", "away", "offline"
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Key Fields Explained:**

- **`cursor_position`**: JSON-encoded cursor coordinates in world space (`{x: 100.5, y: 250.3}`)
- **`selected_node_id`**: ID of the currently selected/edited node
- **`viewport_position`**: Current view state including pan and zoom (`{x: -50, y: 100, zoom: 1.2}`)
- **`current_tool`**: Active editor tool (select, pan, node creation, etc.)
- **`status`**: User activity level with automatic idle detection
- **`last_heartbeat`**: Ping timestamp for connection monitoring

### User Status Management

The system supports four user states:

```rust
pub enum UserStatus {
    Active,   // Actively interacting with the editor
    Idle,     // No interaction for 2-5 minutes
    Away,     // Extended inactivity (5+ minutes)
    Offline,  // Disconnected or session ended
}
```

Status transitions happen automatically based on:
- Mouse movement and clicks
- Keyboard input
- Heartbeat intervals
- Network connectivity

## Frontend Implementation

### React Hooks Architecture

#### `useCollaborationSubscriptions`

Central hook managing all real-time collaboration features:

```typescript
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

  return {
    getOnlineUsers: () => Array.from(usersRef.current.values()),
    getUserCursor: (userId: string) => usersRef.current.get(userId)?.cursorPosition,
  }
}
```

#### `useCursorBroadcast`

Handles efficient cursor position broadcasting with throttling:

```typescript
export const useCursorBroadcast = (planId: string, userId: string) => {
  const lastPositionRef = useRef<{ x: number; y: number } | null>(null)
  const broadcastTimeoutRef = useRef<number | null>(null)

  const broadcastCursorPosition = useCallback((x: number, y: number, selectedNodeId?: string) => {
    // Throttle cursor updates to avoid spam (10 updates per second)
    if (broadcastTimeoutRef.current) {
      clearTimeout(broadcastTimeoutRef.current)
    }

    broadcastTimeoutRef.current = window.setTimeout(() => {
      lastPositionRef.current = { x, y }

      // Publish cursor moved event via GraphQL mutation
      publishCursorPosition({ planId, userId, position: { x, y }, selectedNodeId })
    }, 100) // 100ms throttle
  }, [planId, userId])

  return { broadcastCursorPosition }
}
```

### UI Components

#### Collaborative Cursors (`CollaborativeCursors`)

Renders real-time cursor positions for all online users:

```typescript
export const CollaborativeCursors = memo(({ users, currentUserId }: CollaborativeCursorsProps) => {
  const viewport = useViewport()

  const visibleCursors = users.filter(user =>
    user.userId !== currentUserId &&
    user.isOnline &&
    user.cursorPosition
  )

  return (
    <>
      {visibleCursors.map((user) => (
        <CollaborativeCursor
          key={user.userId}
          userId={user.userId}
          userName={user.userName}
          avatarColor={user.avatarColor}
          position={user.cursorPosition}
          viewport={viewport}
          selectedNodeId={user.selectedNodeId}
        />
      ))}
    </>
  )
})
```

#### Individual Cursor (`CollaborativeCursor`)

Displays a single user's cursor with coordinate transformation:

```typescript
export const CollaborativeCursor = memo(({ userName, avatarColor, position, viewport }: CollaborativeCursorProps) => {
  // Convert world coordinates to screen coordinates within ReactFlow
  const screenX = (position.x * viewport.zoom) + viewport.x
  const screenY = (position.y * viewport.zoom) + viewport.y

  // Don't render if outside visible area
  if (screenX < -100 || screenY < -100 || screenX > 5000 || screenY > 5000) {
    return null
  }

  return (
    <Box style={{ position: 'absolute', left: screenX, top: screenY, pointerEvents: 'none', zIndex: 1000 }}>
      <IconPointer style={{ color: avatarColor, transform: 'rotate(-45deg)' }} />
      <Box style={{ backgroundColor: avatarColor, color: 'white', padding: '2px 6px', borderRadius: 4 }}>
        {userName}
        {selectedNodeId && <Text span>({selectedNodeId})</Text>}
      </Box>
    </Box>
  )
})
```

#### User Presence Indicator (`UserPresenceIndicator`)

Shows online collaborators with status information:

```typescript
export const UserPresenceIndicator = memo(({ users, maxVisible = 5 }: UserPresenceIndicatorProps) => {
  const onlineUsers = users.filter(user => user.isOnline)
  const visibleUsers = onlineUsers.slice(0, maxVisible)
  const hiddenCount = Math.max(0, onlineUsers.length - maxVisible)

  return (
    <Group gap={4} align="center">
      <IconWifi style={{ color: 'var(--mantine-color-green-6)' }} />

      <Group gap={-8}>
        {visibleUsers.map((user) => (
          <Tooltip key={user.userId} label={
            <Box>
              <Text>{user.userName}</Text>
              <Text size="xs">Last active: {new Date(user.lastActive).toLocaleTimeString()}</Text>
              {user.cursorPosition && (
                <Text size="xs">Cursor: ({Math.round(user.cursorPosition.x)}, {Math.round(user.cursorPosition.y)})</Text>
              )}
              {user.selectedNodeId && <Text size="xs" c="blue">Editing: {user.selectedNodeId}</Text>}
            </Box>
          }>
            <Avatar style={{ backgroundColor: user.avatarColor }}>
              <IconUser />
            </Avatar>
          </Tooltip>
        ))}

        {hiddenCount > 0 && (
          <Avatar color="gray">+{hiddenCount}</Avatar>
        )}
      </Group>

      <Text size="xs">{onlineUsers.length} online</Text>
    </Group>
  )
})
```

## Coordinate System & Transformations

### World vs Screen Coordinates

The system uses two coordinate systems:

1. **World Coordinates**: Absolute positions in the infinite canvas (stored in database)
2. **Screen Coordinates**: Pixel positions relative to viewport (used for rendering)

### Transformation Logic

```typescript
// World to Screen (for displaying cursors)
const screenX = (worldX * viewport.zoom) + viewport.x
const screenY = (worldY * viewport.zoom) + viewport.y

// Screen to World (for capturing mouse position)
const worldX = (screenX - viewport.x) / viewport.zoom
const worldY = (screenY - viewport.y) / viewport.zoom
```

### Cursor Position Capture

The Plan DAG Editor captures cursor positions during mouse movement:

```typescript
const handleMouseMove = useCallback((event: React.MouseEvent) => {
  if (readonly) return

  const rect = event.currentTarget.getBoundingClientRect()
  const screenX = event.clientX - rect.left
  const screenY = event.clientY - rect.top

  // Convert screen coordinates to world coordinates for broadcasting
  const viewport = viewportRef.current
  if (!viewport) return // Guard against null viewport

  const worldX = (screenX - viewport.x) / viewport.zoom
  const worldY = (screenY - viewport.y) / viewport.zoom

  broadcastCursorPosition(worldX, worldY, selectedNode || undefined)
}, [broadcastCursorPosition, selectedNode, readonly])
```

## Performance Optimizations

### Throttling and Debouncing

1. **Cursor Updates**: Throttled to 10Hz (100ms intervals) to prevent network spam
2. **Heartbeats**: Sent every 30 seconds for connection monitoring
3. **Status Updates**: Debounced with 2-second delay to avoid rapid state changes

### Memory Management

1. **User Map**: Uses `useRef` to maintain stable references across renders
2. **Event Cleanup**: Automatically removes offline users from client state
3. **Viewport Guards**: Prevents rendering cursors outside visible area

### Network Efficiency

1. **Delta Updates**: Only broadcast changes, not full state
2. **Batch Operations**: Group multiple presence updates when possible
3. **Connection Pooling**: Reuse GraphQL subscription connections

## Conflict Detection & Resolution

### Conflict Types

The system detects several types of editing conflicts:

```typescript
export interface ConflictEvent {
  type: 'node_conflict' | 'edge_conflict'
  nodeId?: string
  edgeId?: string
  conflictingUsers: string[]
  timestamp: string
}
```

### Detection Logic

```typescript
export const useConflictDetection = (planId: string) => {
  const detectConflict = useCallback((event: CollaborationEvent) => {
    if (event.eventType === 'NODE_UPDATED') {
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
          timestamp: new Date().toISOString(),
        }

        conflictsRef.current.push(conflict)
        console.warn('Node conflict detected:', conflict)
      }
    }
  }, [])
}
```

### Resolution Strategies

1. **Last-Write-Wins**: Most recent edit takes precedence
2. **User Notification**: Alert users about conflicts through UI indicators
3. **Automatic Merge**: Combine non-conflicting changes when possible
4. **Manual Resolution**: Provide conflict resolution interface for complex cases

## Security Considerations

### Authorization

1. **Project Access**: Users must be project collaborators to see presence data
2. **Role-Based Visibility**: Viewer/Editor/Owner permissions apply to presence features
3. **Session Validation**: All presence updates require valid authentication

### Privacy Controls

1. **Opt-Out Options**: Users can disable cursor broadcasting
2. **Anonymous Mode**: Option to hide detailed activity from other users
3. **Data Retention**: Presence data automatically expires after session ends

### Rate Limiting

1. **Cursor Updates**: Limited to 10Hz per user to prevent abuse
2. **Heartbeat Frequency**: Enforced 30-second minimum intervals
3. **Connection Limits**: Maximum concurrent connections per project

## Integration Points

### ReactFlow Integration

The collaboration system integrates seamlessly with ReactFlow:

```typescript
<ReactFlow
  nodes={nodes}
  edges={edges}
  onMouseMove={handleMouseMove}
  onViewportChange={handleViewportChange}
>
  {/* Standard ReactFlow components */}
  <Background />
  <Controls />
  <MiniMap />

  {/* Collaboration components */}
  <Panel position="top-right">
    <UserPresenceIndicator users={getOnlineUsers()} />
  </Panel>

  <CollaborativeCursors users={getOnlineUsers()} currentUserId={currentUserId} />
</ReactFlow>
```

### GraphQL Schema Integration

```graphql
type UserPresence {
  userId: ID!
  userName: String!
  avatarColor: String!
  isOnline: Boolean!
  cursorPosition: CursorPosition
  selectedNodeId: String
  lastActive: DateTime!
}

type CursorPosition {
  x: Float!
  y: Float!
}

type Subscription {
  userPresenceChanged(planId: ID!): UserPresence!
  planDagUpdated(planId: ID!): PlanDagUpdateEvent!
  collaborationEvents(planId: ID!): CollaborationEvent!
}

type Mutation {
  updateCursorPosition(planId: ID!, position: CursorPositionInput!, selectedNodeId: String): Boolean!
  updateUserPresence(planId: ID!, status: UserStatus!): Boolean!
}
```

## Future Enhancements

### Planned Features

1. **Voice Cursors**: Audio indicators for cursor movements
2. **Collaborative Selection**: Multi-user selection rectangles
3. **Live Editing Indicators**: Real-time typing indicators for text fields
4. **Presence History**: Timeline of user activity
5. **Smart Conflict Resolution**: AI-powered merge suggestions

### Technical Improvements

1. **WebRTC Integration**: Direct peer-to-peer cursor streaming
2. **Operational Transforms**: Advanced conflict resolution algorithms
3. **Offline Support**: Local presence tracking with sync on reconnection
4. **Mobile Optimization**: Touch-friendly collaboration features

## Troubleshooting

### Common Issues

**Cursors not appearing:**
- Check GraphQL subscription connection status
- Verify user authentication and project permissions
- Ensure viewport coordinates are properly calculated

**Performance degradation:**
- Monitor cursor update frequency (should be ≤10Hz)
- Check for memory leaks in user presence map
- Verify efficient coordinate transformations

**Synchronization problems:**
- Validate system clock synchronization across clients
- Check network latency and connection stability
- Monitor GraphQL subscription event delivery

### Debug Tools

Enable debug logging with:
```typescript
// Frontend debugging
localStorage.setItem('debug', 'collaboration:*')

// Backend debugging
RUST_LOG=layercake_core::collaboration=debug cargo run
```

## Conclusion

The Layercake Plan DAG Editor's collaboration system provides a robust foundation for real-time multi-user editing. Through careful attention to performance, security, and user experience, it enables seamless collaborative workflows while maintaining data integrity and system responsiveness.

The modular architecture allows for easy extension and customization, making it suitable for various collaborative editing scenarios beyond plan visualization.