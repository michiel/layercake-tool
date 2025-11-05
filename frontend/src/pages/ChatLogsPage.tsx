import { useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  ARCHIVE_CHAT_SESSION,
  DELETE_CHAT_SESSION,
  GET_CHAT_HISTORY,
  GET_CHAT_SESSIONS,
  UNARCHIVE_CHAT_SESSION,
  type ChatMessage,
  type ChatSession,
} from '../graphql/chat'
import { Stack, Group } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Card, CardContent } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Spinner } from '@/components/ui/spinner'

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
    return <p className="text-red-600">Invalid project ID</p>
  }

  return (
    <Stack className="h-full p-4" gap="md">
      <div>
        <h2 className="text-2xl font-semibold">Chat History</h2>
        <p className="text-sm text-muted-foreground">
          Browse recorded sessions for this project. Select a row to inspect the conversation.
        </p>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertTitle>Unable to load sessions</AlertTitle>
          <AlertDescription>{error.message}</AlertDescription>
        </Alert>
      )}

      <Group align="start" gap="md" className="flex-1" style={{ minHeight: 0 }}>
        <Card className="border rounded-lg flex-[2]" style={{ minHeight: 0 }}>
          <CardContent className="p-4">
            <Group justify="between" className="mb-2">
              <h5 className="text-lg font-semibold">Sessions</h5>
              <Group gap="xs">
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={page === 0 || loading}
                  onClick={() => setPage((prev) => Math.max(prev - 1, 0))}
                >
                  Previous
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={!hasNext || loading}
                  onClick={() => setPage((prev) => prev + 1)}
                >
                  Next
                </Button>
              </Group>
            </Group>

            <ScrollArea className="max-h-full">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead style={{ width: '35%' }}>Title</TableHead>
                    <TableHead>Provider</TableHead>
                    <TableHead>Model</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead>Last Activity</TableHead>
                    <TableHead>Archived</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loading && displaySessions.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={6}>
                        <Group justify="center">
                          <Spinner size="sm" />
                          <p className="text-sm text-muted-foreground">
                            Loading sessions…
                          </p>
                        </Group>
                      </TableCell>
                    </TableRow>
                  ) : displaySessions.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={6}>
                        <p className="text-sm text-muted-foreground text-center">
                          No sessions recorded yet.
                        </p>
                      </TableCell>
                    </TableRow>
                  ) : (
                    displaySessions.map((session) => (
                      <TableRow
                        key={session.session_id}
                        onClick={() => setSelectedSession(session)}
                        className={`cursor-pointer ${selectedSession?.session_id === session.session_id ? 'bg-muted' : ''}`}
                      >
                        <TableCell>
                          <p className="font-medium">{session.title || 'Untitled Chat'}</p>
                        </TableCell>
                        <TableCell>{session.provider}</TableCell>
                        <TableCell>{session.model_name}</TableCell>
                        <TableCell>{new Date(session.created_at).toLocaleString()}</TableCell>
                        <TableCell>{new Date(session.last_activity_at).toLocaleString()}</TableCell>
                        <TableCell>{session.is_archived ? 'Yes' : 'No'}</TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </ScrollArea>
          </CardContent>
        </Card>

        <Card className="border rounded-lg flex-[3]" style={{ minHeight: 0 }}>
          <CardContent className="p-4">
            {selectedSession ? (
              <Stack gap="md" className="h-full" style={{ minHeight: 0 }}>
                <Group justify="between" align="start">
                  <div>
                    <h5 className="text-lg font-semibold">{selectedSession.title || 'Untitled Chat'}</h5>
                    <Group gap="xs">
                      <Badge variant="secondary" className="bg-blue-100 text-blue-800">
                        {selectedSession.provider}
                      </Badge>
                      <Badge variant="secondary">
                        {selectedSession.model_name}
                      </Badge>
                    </Group>
                    <p className="text-xs text-muted-foreground mt-1">
                      Session ID: {selectedSession.session_id}
                    </p>
                  </div>
                  <Group gap="xs">
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => void handleToggleArchive(selectedSession)}
                    >
                      {selectedSession.is_archived ? 'Unarchive' : 'Archive'}
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => void handleDelete(selectedSession)}
                    >
                      Delete
                    </Button>
                  </Group>
                </Group>

                {historyError && (
                  <Alert variant="destructive">
                    <AlertTitle>Unable to load messages</AlertTitle>
                    <AlertDescription>{historyError.message}</AlertDescription>
                  </Alert>
                )}

                <ScrollArea className="flex-1">
                  <Stack gap="sm">
                    {historyLoading ? (
                      <Group justify="center" className="mt-4">
                        <Spinner size="sm" />
                        <p className="text-sm text-muted-foreground">
                          Loading messages…
                        </p>
                      </Group>
                    ) : historyData?.chatHistory.length ? (
                      historyData.chatHistory.map((message) => (
                        <Card key={message.message_id} className="border rounded-lg">
                          <CardContent className="p-3">
                            <Stack gap="xs">
                              <Group gap="xs">
                                <Badge
                                  className={`text-xs ${
                                    message.role === 'user'
                                      ? 'bg-blue-100 text-blue-800'
                                      : message.role === 'tool'
                                        ? 'bg-orange-100 text-orange-800'
                                        : 'bg-gray-100 text-gray-800'
                                  }`}
                                >
                                  {message.role === 'user'
                                    ? 'User'
                                    : message.role === 'tool'
                                      ? message.tool_name ?? 'Tool'
                                      : 'Assistant'}
                                </Badge>
                                <p className="text-xs text-muted-foreground">
                                  {new Date(message.created_at).toLocaleString()}
                                </p>
                              </Group>
                              <p className="text-sm whitespace-pre-wrap">
                                {message.content}
                              </p>
                            </Stack>
                          </CardContent>
                        </Card>
                      ))
                    ) : (
                      <p className="text-sm text-muted-foreground">
                        No messages recorded for this session.
                      </p>
                    )}
                  </Stack>
                </ScrollArea>
              </Stack>
            ) : (
              <Stack align="center" justify="center" className="h-full">
                <p className="text-sm text-muted-foreground">
                  Select a session to view its messages.
                </p>
              </Stack>
            )}
          </CardContent>
        </Card>
      </Group>
    </Stack>
  )
}
