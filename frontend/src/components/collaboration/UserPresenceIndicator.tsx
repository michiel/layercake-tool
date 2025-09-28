import { memo } from 'react'
import { Group, Avatar, Badge, Box, Text, HoverCard, ActionIcon, Stack } from '@mantine/core'
import { IconUser, IconUsers } from '@tabler/icons-react'
import { UserPresenceData, ConnectionState } from '../../types/websocket'

interface UserPresenceIndicatorProps {
  users: UserPresenceData[]
  connectionState: ConnectionState
  currentUserId?: string
}

export const UserPresenceIndicator = memo(({
  users,
  connectionState,
  currentUserId
}: UserPresenceIndicatorProps) => {
  const onlineUsers = users.filter(user => user.userId !== currentUserId && user.isOnline)

  // Only show presence indicator when connected and there are users
  // Connection status is handled by TopBar
  if (connectionState !== ConnectionState.CONNECTED || onlineUsers.length === 0) {
    return null
  }

  return (
    <HoverCard width={300} shadow="md">
      <HoverCard.Target>
        <ActionIcon variant="subtle" size="lg" style={{ position: 'relative' }}>
          <IconUsers size="1.2rem" />
          {onlineUsers.length > 0 && (
            <Badge
              size="xs"
              color="green"
              variant="filled"
              style={{
                position: 'absolute',
                top: -2,
                right: -2,
                minWidth: '18px',
                height: '18px',
                padding: '0 4px',
                fontSize: '10px'
              }}
            >
              {onlineUsers.length}
            </Badge>
          )}
        </ActionIcon>
      </HoverCard.Target>

      <HoverCard.Dropdown>
        <Stack gap="xs">
          <Text fw={500} size="sm">Active Users ({onlineUsers.length})</Text>

          {onlineUsers.length === 0 ? (
            <Text size="xs" c="dimmed">No other users online</Text>
          ) : (
            onlineUsers.map((user) => (
              <Group key={user.userId} gap="sm" justify="space-between">
                <Group gap="xs">
                  <Avatar
                    size="sm"
                    radius="xl"
                    style={{ backgroundColor: user.avatarColor }}
                  >
                    <IconUser size={14} />
                  </Avatar>
                  <Box>
                    <Text size="sm" fw={500}>{user.userName}</Text>
                    <Text size="xs" c="dimmed">
                      Last active: {new Date(user.lastActive).toLocaleTimeString()}
                    </Text>
                    {/* Show document activity if available */}
                    {Object.keys(user.documents).length > 0 && (
                      <Text size="xs" c="dimmed">
                        Active in {Object.keys(user.documents).length} document{Object.keys(user.documents).length > 1 ? 's' : ''}
                      </Text>
                    )}
                  </Box>
                </Group>

                <Badge
                  size="xs"
                  color="green"
                  variant="filled"
                >
                  Online
                </Badge>
              </Group>
            ))
          )}
        </Stack>
      </HoverCard.Dropdown>
    </HoverCard>
  )
})

UserPresenceIndicator.displayName = 'UserPresenceIndicator'