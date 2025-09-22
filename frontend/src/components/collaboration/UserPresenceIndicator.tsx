import { memo } from 'react'
import { Group, Avatar, Tooltip, Badge, Box, Text } from '@mantine/core'
import { IconUser, IconWifi, IconWifiOff } from '@tabler/icons-react'
import { UserPresence } from '../../hooks/useCollaborationSubscriptions'

interface UserPresenceIndicatorProps {
  users: UserPresence[]
  maxVisible?: number
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
}

export const UserPresenceIndicator = memo(({
  users,
  maxVisible = 5,
  size = 'sm'
}: UserPresenceIndicatorProps) => {
  const onlineUsers = users.filter(user => user.isOnline)
  const visibleUsers = onlineUsers.slice(0, maxVisible)
  const hiddenCount = Math.max(0, onlineUsers.length - maxVisible)

  if (onlineUsers.length === 0) {
    return (
      <Group gap="xs" align="center">
        <IconWifiOff size={16} style={{ color: 'var(--mantine-color-gray-5)' }} />
        <Text size="xs" c="dimmed">No collaborators</Text>
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
                {user.cursorPosition && (
                  <Text size="xs" c="dimmed">
                    Cursor: ({Math.round(user.cursorPosition.x)}, {Math.round(user.cursorPosition.y)})
                  </Text>
                )}
                {user.selectedNodeId && (
                  <Text size="xs" c="blue">
                    Editing: {user.selectedNodeId}
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