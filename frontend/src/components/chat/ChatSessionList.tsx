import { useMutation, useQuery } from '@apollo/client/react'
import { ActionIcon, Badge, Card, Group, Loader, Menu, ScrollArea, Stack, Text, TextInput } from '@mantine/core'
import {  IconArchive, IconDotsVertical, IconMessage, IconTrash } from '@tabler/icons-react'
import { useState } from 'react'
import { ARCHIVE_CHAT_SESSION, ChatSession, DELETE_CHAT_SESSION, GET_CHAT_SESSIONS } from '../../graphql/chat'

// Simple relative time formatter (to avoid date-fns dependency for now)
const formatRelativeTime = (date: Date): string => {
  const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`
  if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`
  if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`
  return 'just now'
}

interface ChatSessionListProps {
  projectId: number
  selectedSessionId: string | null
  onSelectSession: (sessionId: string | null) => void
}

export const ChatSessionList = ({ projectId, selectedSessionId, onSelectSession }: ChatSessionListProps) => {
  const [searchQuery, setSearchQuery] = useState('')

  const { data, loading, error, refetch } = useQuery<{ chatSessions: ChatSession[] }>(GET_CHAT_SESSIONS, {
    variables: { projectId, includeArchived: false },
    fetchPolicy: 'cache-and-network',
  })

  const [archiveSession] = useMutation(ARCHIVE_CHAT_SESSION, {
    onCompleted: () => refetch(),
  })

  const [deleteSession] = useMutation(DELETE_CHAT_SESSION, {
    onCompleted: () => {
      refetch()
      if (selectedSessionId) {
        onSelectSession(null)
      }
    },
  })

  const sessions = data?.chatSessions || []

  const filteredSessions = sessions.filter(session => {
    if (!searchQuery) return true
    const query = searchQuery.toLowerCase()
    return (
      session.title?.toLowerCase().includes(query) ||
      session.provider.toLowerCase().includes(query) ||
      session.model_name.toLowerCase().includes(query)
    )
  })

  const handleArchive = async (sessionId: string) => {
    try {
      await archiveSession({ variables: { sessionId } })
    } catch (err) {
      console.error('Failed to archive session:', err)
    }
  }

  const handleDelete = async (sessionId: string) => {
    if (window.confirm('Are you sure you want to delete this chat session? This action cannot be undone.')) {
      try {
        await deleteSession({ variables: { sessionId } })
      } catch (err) {
        console.error('Failed to delete session:', err)
      }
    }
  }

  if (loading && !data) {
    return (
      <Stack align="center" justify="center" h="100%">
        <Loader size="md" />
        <Text size="sm" c="dimmed">Loading sessions...</Text>
      </Stack>
    )
  }

  if (error) {
    return (
      <Text c="red" size="sm">
        Error loading sessions: {error.message}
      </Text>
    )
  }

  return (
    <Stack h="100%" gap="md">
      <TextInput
        placeholder="Search sessions..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.currentTarget.value)}
        size="sm"
      />

      <ScrollArea style={{ flex: 1 }}>
        <Stack gap="xs">
          {filteredSessions.length === 0 && (
            <Text c="dimmed" size="sm" ta="center" mt="md">
              {searchQuery ? 'No sessions match your search' : 'No chat sessions yet'}
            </Text>
          )}

          {filteredSessions.map((session) => (
            <Card
              key={session.session_id}
              padding="sm"
              radius="md"
              withBorder
              style={{
                cursor: 'pointer',
                backgroundColor: selectedSessionId === session.session_id ? '#e7f5ff' : undefined,
                borderColor: selectedSessionId === session.session_id ? '#228be6' : undefined,
              }}
              onClick={() => onSelectSession(session.session_id)}
            >
              <Group justify="space-between" wrap="nowrap">
                <Stack gap={4} style={{ flex: 1, minWidth: 0 }}>
                  <Group gap="xs" wrap="nowrap">
                    <IconMessage size={14} />
                    <Text size="sm" fw={500} truncate>
                      {session.title || 'Untitled Chat'}
                    </Text>
                  </Group>

                  <Group gap="xs">
                    <Badge size="xs" variant="light">
                      {session.provider}
                    </Badge>
                    <Text size="xs" c="dimmed" truncate>
                      {session.model_name}
                    </Text>
                  </Group>

                  <Text size="xs" c="dimmed">
                    {formatRelativeTime(new Date(session.last_activity_at))}
                  </Text>
                </Stack>

                <Menu position="bottom-end" withinPortal>
                  <Menu.Target>
                    <ActionIcon
                      size="sm"
                      variant="subtle"
                      onClick={(e) => {
                        e.stopPropagation()
                      }}
                    >
                      <IconDotsVertical size={16} />
                    </ActionIcon>
                  </Menu.Target>

                  <Menu.Dropdown>
                    <Menu.Item
                      leftSection={<IconArchive size={14} />}
                      onClick={(e) => {
                        e.stopPropagation()
                        handleArchive(session.session_id)
                      }}
                    >
                      Archive
                    </Menu.Item>
                    <Menu.Item
                      leftSection={<IconTrash size={14} />}
                      color="red"
                      onClick={(e) => {
                        e.stopPropagation()
                        handleDelete(session.session_id)
                      }}
                    >
                      Delete
                    </Menu.Item>
                  </Menu.Dropdown>
                </Menu>
              </Group>
            </Card>
          ))}
        </Stack>
      </ScrollArea>
    </Stack>
  )
}
