import { useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery } from '@apollo/client/react'
import { Alert, Badge, Button, Card, Group, Loader, Paper, ScrollArea, Stack, Table, Text, Title } from '@mantine/core'
import {
  ARCHIVE_CHAT_SESSION,
  DELETE_CHAT_SESSION,
  GET_CHAT_HISTORY,
  GET_CHAT_SESSIONS,
  UNARCHIVE_CHAT_SESSION,
  type ChatMessage,
  type ChatSession,
} from '../graphql/chat'

const PAGE_SIZE = 10

export const ChatLogsPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [page, setPage] = useState(0)
  const [selectedSession, setSelectedSession] = useState<ChatSession | null>(null)

  const { data, loading, error, refetch } = useQuery<{ chatSessions: ChatSession[] }>(GET_CHAT_SESSIONS, {
    variables: {
      projectId: Number.isFinite(numericProjectId) ? numericProjectId : 0,
      includeArchived: true,
      limit: PAGE_SIZE + 1,
      offset: page * PAGE_SIZE,
    },
    skip: !Number.isFinite(numericProjectId),
    fetchPolicy: 'network-only',
  })

  const sessions = data?.chatSessions ?? []
  const displaySessions = useMemo(() => sessions.slice(0, PAGE_SIZE), [sessions])
  const hasNext = sessions.length > PAGE_SIZE

  const { data: historyData, loading: historyLoading, error: historyError, refetch: refetchHistory } = useQuery<{
    chatHistory: ChatMessage[]
  }>(GET_CHAT_HISTORY, {
    variables: { sessionId: selectedSession?.session_id ?? '' },
    skip: !selectedSession,
    fetchPolicy: 'network-only',
  })

  const [archiveSession] = useMutation(ARCHIVE_CHAT_SESSION)
  const [unarchiveSession] = useMutation(UNARCHIVE_CHAT_SESSION)
  const [deleteSession] = useMutation(DELETE_CHAT_SESSION)

  const syncSelection = (sessionId: string | null, dataset?: ChatSession[]) => {
    if (!sessionId) {
      setSelectedSession(null)
      return
    }

    const updated = dataset?.find((session) => session.session_id === sessionId)
    if (updated) {
      setSelectedSession(updated)
    }
  }

  const handleToggleArchive = async (session: ChatSession) => {
    const shouldRefreshHistory = selectedSession?.session_id === session.session_id

    if (session.is_archived) {
      await unarchiveSession({ variables: { sessionId: session.session_id } })
    } else {
      await archiveSession({ variables: { sessionId: session.session_id } })
    }

    const result = await refetch()
    syncSelection(session.session_id, result.data?.chatSessions)
    if (shouldRefreshHistory) {
      await refetchHistory()
    }
  }

  const handleDelete = async (session: ChatSession) => {
    const wasSelected = selectedSession?.session_id === session.session_id

    await deleteSession({ variables: { sessionId: session.session_id } })
    const result = await refetch()
    const remaining = result.data?.chatSessions ?? []
    if (remaining.length === 0 && page > 0) {
      setPage((prev) => Math.max(prev - 1, 0))
    }
    if (wasSelected) {
      setSelectedSession(null)
    }
  }

  if (!Number.isFinite(numericProjectId)) {
    return <Text c="red">Invalid project ID</Text>
  }

  return (
    <Stack h="100%" p="md" gap="md">
      <div>
        <Title order={2}>Chat History</Title>
        <Text c="dimmed" size="sm">
          Browse recorded sessions for this project. Select a row to inspect the conversation.
        </Text>
      </div>

      {error && (
        <Alert color="red" title="Unable to load sessions">
          {error.message}
        </Alert>
      )}

      <Group align="flex-start" gap="md" grow style={{ flex: 1, minHeight: 0 }}>
        <Paper withBorder radius="md" p="md" style={{ flex: 2, minHeight: 0 }}>
          <Group justify="space-between" mb="sm">
            <Title order={5}>Sessions</Title>
            <Group gap="xs">
              <Button
                variant="subtle"
                size="xs"
                disabled={page === 0 || loading}
                onClick={() => setPage((prev) => Math.max(prev - 1, 0))}
              >
                Previous
              </Button>
              <Button
                variant="subtle"
                size="xs"
                disabled={!hasNext || loading}
                onClick={() => setPage((prev) => prev + 1)}
              >
                Next
              </Button>
            </Group>
          </Group>

          <ScrollArea style={{ maxHeight: '100%' }}>
            <Table highlightOnHover stickyHeader>
              <Table.Thead>
                <Table.Tr>
                  <Table.Th style={{ width: '35%' }}>Title</Table.Th>
                  <Table.Th>Provider</Table.Th>
                  <Table.Th>Model</Table.Th>
                  <Table.Th>Created</Table.Th>
                  <Table.Th>Last Activity</Table.Th>
                  <Table.Th>Archived</Table.Th>
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {loading && displaySessions.length === 0 ? (
                  <Table.Tr>
                    <Table.Td colSpan={6}>
                      <Group justify="center">
                        <Loader size="sm" />
                        <Text size="sm" c="dimmed">
                          Loading sessions…
                        </Text>
                      </Group>
                    </Table.Td>
                  </Table.Tr>
                ) : displaySessions.length === 0 ? (
                  <Table.Tr>
                    <Table.Td colSpan={6}>
                      <Text size="sm" c="dimmed" ta="center">
                        No sessions recorded yet.
                      </Text>
                    </Table.Td>
                  </Table.Tr>
                ) : (
                  displaySessions.map((session) => (
                    <Table.Tr
                      key={session.session_id}
                      onClick={() => setSelectedSession(session)}
                      style={{
                        cursor: 'pointer',
                        backgroundColor:
                          selectedSession?.session_id === session.session_id ? '#f1f3f5' : undefined,
                      }}
                    >
                      <Table.Td>
                        <Text fw={500}>{session.title || 'Untitled Chat'}</Text>
                      </Table.Td>
                      <Table.Td>{session.provider}</Table.Td>
                      <Table.Td>{session.model_name}</Table.Td>
                      <Table.Td>{new Date(session.created_at).toLocaleString()}</Table.Td>
                      <Table.Td>{new Date(session.last_activity_at).toLocaleString()}</Table.Td>
                      <Table.Td>{session.is_archived ? 'Yes' : 'No'}</Table.Td>
                    </Table.Tr>
                  ))
                )}
              </Table.Tbody>
            </Table>
          </ScrollArea>
        </Paper>

        <Paper withBorder radius="md" p="md" style={{ flex: 3, minHeight: 0 }}>
          {selectedSession ? (
            <Stack gap="md" h="100%" style={{ minHeight: 0 }}>
              <Group justify="space-between" align="flex-start">
                <div>
                  <Title order={5}>{selectedSession.title || 'Untitled Chat'}</Title>
                  <Group gap="xs">
                    <Badge color="blue" variant="light">
                      {selectedSession.provider}
                    </Badge>
                    <Badge color="gray" variant="light">
                      {selectedSession.model_name}
                    </Badge>
                  </Group>
                  <Text size="xs" c="dimmed" mt={4}>
                    Session ID: {selectedSession.session_id}
                  </Text>
                </div>
                <Group gap="xs">
                  <Button
                    variant="light"
                    size="xs"
                    onClick={() => void handleToggleArchive(selectedSession)}
                  >
                    {selectedSession.is_archived ? 'Unarchive' : 'Archive'}
                  </Button>
                  <Button
                    variant="outline"
                    color="red"
                    size="xs"
                    onClick={() => void handleDelete(selectedSession)}
                  >
                    Delete
                  </Button>
                </Group>
              </Group>

              {historyError && (
                <Alert color="red" title="Unable to load messages">
                  {historyError.message}
                </Alert>
              )}

              <ScrollArea style={{ flex: 1 }}>
                <Stack gap="sm">
                  {historyLoading ? (
                    <Group justify="center" mt="md">
                      <Loader size="sm" />
                      <Text size="sm" c="dimmed">
                        Loading messages…
                      </Text>
                    </Group>
                  ) : historyData?.chatHistory.length ? (
                    historyData.chatHistory.map((message) => (
                      <Card key={message.message_id} radius="md" withBorder padding="sm">
                        <Stack gap={4}>
                          <Group gap="xs">
                            <Badge
                              size="xs"
                              color={
                                message.role === 'user'
                                  ? 'blue'
                                  : message.role === 'tool'
                                    ? 'orange'
                                    : 'gray'
                              }
                            >
                              {message.role === 'user'
                                ? 'User'
                                : message.role === 'tool'
                                  ? message.tool_name ?? 'Tool'
                                  : 'Assistant'}
                            </Badge>
                            <Text size="xs" c="dimmed">
                              {new Date(message.created_at).toLocaleString()}
                            </Text>
                          </Group>
                          <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
                            {message.content}
                          </Text>
                        </Stack>
                      </Card>
                    ))
                  ) : (
                    <Text size="sm" c="dimmed">
                      No messages recorded for this session.
                    </Text>
                  )}
                </Stack>
              </ScrollArea>
            </Stack>
          ) : (
            <Stack align="center" justify="center" h="100%">
              <Text c="dimmed" size="sm">
                Select a session to view its messages.
              </Text>
            </Stack>
          )}
        </Paper>
      </Group>
    </Stack>
  )
}
