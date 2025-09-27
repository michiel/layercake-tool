import { memo } from 'react'
import { Group, Avatar, Tooltip, Badge, Box, Text } from '@mantine/core'
import { IconUser, IconWifi, IconWifiOff, IconRefresh } from '@tabler/icons-react'
import { UserPresenceData, ConnectionState } from '../../types/websocket'

interface UserPresenceIndicatorProps {
  users: UserPresenceData[]
  connectionState: ConnectionState
  currentUserId?: string
  maxVisible?: number
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  onReconnect?: () => void
}

export const UserPresenceIndicator = memo(({
  users,
  connectionState,
  currentUserId,
  maxVisible = 5,
  size = 'sm',
  onReconnect
}: UserPresenceIndicatorProps) => {
  const onlineUsers = users.filter(user => user.userId !== currentUserId && user.isOnline)
  const visibleUsers = onlineUsers.slice(0, maxVisible)
  const hiddenCount = Math.max(0, onlineUsers.length - maxVisible)

  const getConnectionIcon = () => {
    switch (connectionState) {
      case ConnectionState.CONNECTED:
        return <IconWifi size={16} style={{ color: 'var(--mantine-color-green-6)' }} />
      case ConnectionState.CONNECTING:
      case ConnectionState.RECONNECTING:
        return <IconRefresh size={16} style={{ color: 'var(--mantine-color-yellow-6)' }} className="animate-spin" />
      default:
        return <IconWifiOff size={16} style={{ color: 'var(--mantine-color-red-6)' }} />
    }
  }

  const getConnectionText = () => {
    switch (connectionState) {
      case ConnectionState.CONNECTED:
        return onlineUsers.length === 0 ? 'No collaborators' : `${onlineUsers.length} online`
      case ConnectionState.CONNECTING:
        return 'Connecting...'
      case ConnectionState.RECONNECTING:
        return 'Reconnecting...'
      case ConnectionState.ERROR:
      case ConnectionState.DISCONNECTED:
        return 'Disconnected'
      default:
        return 'Unknown status'
    }
  }

  if (connectionState !== ConnectionState.CONNECTED || onlineUsers.length === 0) {
    return (
      <Group gap="xs" align="center">
        {getConnectionIcon()}
        <Text size="xs" c="dimmed">{getConnectionText()}</Text>
        {(connectionState === ConnectionState.ERROR || connectionState === ConnectionState.DISCONNECTED) && onReconnect && (
          <Text size="xs" c="blue" style={{ cursor: 'pointer' }} onClick={onReconnect}>
            Retry
          </Text>
        )}
      </Group>
    )
  }

  return (
    <Group gap={4} align="center">
      <IconWifi size={16} style={{ color: 'var(--mantine-color-green-6)' }} />

      <Group gap={-8}>
        {visibleUsers.map((user) => (
          <Tooltip
            key={user.userId}
            label={
              <Box>
                <Text size="sm" fw={500}>{user.userName}</Text>
                <Text size="xs" c="dimmed">
                  Last active: {new Date(user.lastActive).toLocaleTimeString()}
                </Text>
                {Object.keys(user.documents).length > 0 && (
                  <Text size="xs" c="dimmed">
                    Active in {Object.keys(user.documents).length} document{Object.keys(user.documents).length > 1 ? 's' : ''}
                  </Text>
                )}
                {/* Show cursor position from first active document */}
                {Object.values(user.documents)[0]?.position && (
                  <Text size="xs" c="dimmed">
                    Cursor: ({Math.round((Object.values(user.documents)[0].position as any).x || 0)}, {Math.round((Object.values(user.documents)[0].position as any).y || 0)})
                  </Text>
                )}
                {Object.values(user.documents)[0]?.selectedNodeId && (
                  <Text size="xs" c="blue">
                    Editing: {Object.values(user.documents)[0].selectedNodeId}
                  </Text>
                )}
              </Box>
            }
            withArrow
          >
            <Avatar
              size={size}
              radius="xl"
              style={{
                backgroundColor: user.avatarColor,
                border: '2px solid white',
                position: 'relative',
              }}
            >
              <IconUser size={size === 'xs' ? 12 : size === 'sm' ? 14 : 16} />

              {/* Online indicator */}
              <Badge
                size="xs"
                variant="filled"
                color="green"
                style={{
                  position: 'absolute',
                  bottom: -2,
                  right: -2,
                  minWidth: 8,
                  height: 8,
                  padding: 0,
                  border: '1px solid white',
                }}
              />
            </Avatar>
          </Tooltip>
        ))}

        {hiddenCount > 0 && (
          <Tooltip label={`${hiddenCount} more collaborator${hiddenCount > 1 ? 's' : ''}`}>
            <Avatar size={size} radius="xl" color="gray">
              <Text size={size === 'xs' ? '8px' : size === 'sm' ? '10px' : '12px'}>
                +{hiddenCount}
              </Text>
            </Avatar>
          </Tooltip>
        )}
      </Group>

      <Text size="xs" c="dimmed">
        {onlineUsers.length} online
      </Text>
    </Group>
  )
})

UserPresenceIndicator.displayName = 'UserPresenceIndicator'