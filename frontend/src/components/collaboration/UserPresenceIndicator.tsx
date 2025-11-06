import { memo } from 'react'
import { IconUser, IconUsers } from '@tabler/icons-react'
import { UserPresenceData, ConnectionState } from '../../types/websocket'
import { Group, Stack } from '../layout-primitives'
import { Avatar, AvatarFallback } from '../ui/avatar'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { HoverCard, HoverCardTrigger, HoverCardContent } from '../ui/hover-card'

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
    <HoverCard>
      <HoverCardTrigger asChild>
        <Button variant="ghost" size="icon" className="relative">
          <IconUsers className="h-5 w-5" />
          {onlineUsers.length > 0 && (
            <Badge
              className="absolute -top-0.5 -right-0.5 min-w-[18px] h-[18px] px-1 text-[10px] leading-tight"
              style={{
                backgroundColor: '#22c55e',
                color: 'white',
              }}
            >
              {onlineUsers.length}
            </Badge>
          )}
        </Button>
      </HoverCardTrigger>

      <HoverCardContent className="w-[300px]">
        <Stack gap="xs">
          <p className="text-sm font-medium">Active Users ({onlineUsers.length})</p>

          {onlineUsers.length === 0 ? (
            <p className="text-xs text-muted-foreground">No other users online</p>
          ) : (
            onlineUsers.map((user) => (
              <Group key={user.userId} gap="sm" justify="between">
                <Group gap="xs">
                  <Avatar className="h-8 w-8" style={{ backgroundColor: user.avatarColor }}>
                    <AvatarFallback style={{ backgroundColor: user.avatarColor }}>
                      <IconUser className="h-3.5 w-3.5 text-white" />
                    </AvatarFallback>
                  </Avatar>
                  <div>
                    <p className="text-sm font-medium">{user.userName}</p>
                    <p className="text-xs text-muted-foreground">
                      Last active: {new Date(user.lastActive).toLocaleTimeString()}
                    </p>
                    {/* Show document activity if available */}
                    {Object.keys(user.documents).length > 0 && (
                      <p className="text-xs text-muted-foreground">
                        Active in {Object.keys(user.documents).length} document{Object.keys(user.documents).length > 1 ? 's' : ''}
                      </p>
                    )}
                  </div>
                </Group>

                <Badge
                  className="text-xs"
                  style={{
                    backgroundColor: '#22c55e',
                    color: 'white',
                  }}
                >
                  Online
                </Badge>
              </Group>
            ))
          )}
        </Stack>
      </HoverCardContent>
    </HoverCard>
  )
})

UserPresenceIndicator.displayName = 'UserPresenceIndicator'