import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import { Alert, Badge, Button, Card, Flex, Group, Loader, Paper, ScrollArea, Select, Stack, Text, Textarea } from '@mantine/core'
import { IconMessageDots, IconRefresh } from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import { CHAT_PROVIDER_OPTIONS, ChatProviderOption } from '../graphql/chat'
import { useChatSession } from '../hooks/useChatSession'
import { ChatSessionList } from '../components/chat/ChatSessionList'
import { ChatSessionHeader } from '../components/chat/ChatSessionHeader'

const GET_PROJECT = gql`
  query GetProjectName($id: Int!) {
    project(id: $id) {
      id
      name
      description
    }
  }
`

// Inner component that uses the chat session
// The key prop on this component forces it to remount when sessionId changes,
// which recreates the subscription hook with the new sessionId
const ChatInterface = ({
  projectId,
  provider,
  project,
  onProviderChange,
  selectedSessionId,
  currentSession,
  onNewSession,
  onSessionDeleted,
  onSessionArchived,
  onTitleUpdated,
}: {
  projectId: number | undefined
  provider: ChatProviderOption
  project: any
  onProviderChange: (provider: ChatProviderOption) => void
  selectedSessionId: string | null
  currentSession: any
  onNewSession: () => void
  onSessionDeleted: () => void
  onSessionArchived: () => void
  onTitleUpdated: () => void
}) => {
  const [input, setInput] = useState('')
  const chat = useChatSession({ projectId, provider, sessionId: selectedSessionId })

  // auto-scroll on new messages
  const viewportRef = useRef<HTMLDivElement | null>(null)
  useEffect(() => {
    if (viewportRef.current) {
      viewportRef.current.scrollTop = viewportRef.current.scrollHeight
    }
  }, [chat.messages])

  const handleSend = async () => {
    const trimmed = input.trim()
    if (!trimmed) return
    setInput('')
    await chat.sendMessage(trimmed)
  }

  const handleKeyDown: React.KeyboardEventHandler<HTMLTextAreaElement> = (event) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      handleSend()
    }
  }

  const providerSelectData = CHAT_PROVIDER_OPTIONS.map(option => ({
    value: option.value,
    label: option.label,
    description: option.description,
  }))

  const statusLabel = chat.loading
    ? 'Connecting…'
    : chat.isAwaitingAssistant
      ? 'Assistant thinking…'
      : 'Ready'

  return (
    <Stack h="100%" gap={0}>
      <ChatSessionHeader
        session={currentSession}
        onNewSession={onNewSession}
        onSessionDeleted={onSessionDeleted}
        onSessionArchived={onSessionArchived}
        onTitleUpdated={onTitleUpdated}
      />

      <Group justify="space-between" align="flex-start" p="md" style={{ borderBottom: '1px solid #e9ecef' }}>
        <div>
          <Text c="dimmed" size="sm">
            {project ? `Discuss project "${project.name}" with tool-assisted insights.` : 'Start a conversation powered by Layercake tools.'}
          </Text>
        </div>
        <Group gap="sm">
          <Select
            label="Provider"
            data={providerSelectData}
            value={provider}
            onChange={(value) => {
              if (value) {
                onProviderChange(value as ChatProviderOption)
              }
            }}
            styles={{ root: { minWidth: 200 } }}
          />
          <Button variant="light" leftSection={<IconRefresh size={16} />} onClick={chat.restart} disabled={chat.loading}>
            Restart Session
          </Button>
        </Group>
      </Group>

      {chat.error && (
        <Alert color="red" title="Chat unavailable" mt="xs">
          {chat.error}
        </Alert>
      )}

      <Paper withBorder={false} p="md" style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
        <Group justify="space-between" mb="sm">
          <Group gap="xs">
            <Badge color="blue" variant="light">{provider}</Badge>
            {chat.session?.model && <Badge color="gray" variant="light">Model: {chat.session.model}</Badge>}
          </Group>
          <Group gap="xs">
            {chat.loading ? <Loader size="sm" /> : <IconMessageDots size={18} style={{ color: '#4dabf7' }} />}
            <Text size="sm" c="dimmed">{statusLabel}</Text>
          </Group>
        </Group>

        <ScrollArea style={{ flex: 1 }} viewportRef={viewportRef} type="auto">
          <Stack gap="sm" pr="sm">
            {chat.messages.length === 0 && !chat.loading && (
              <Card withBorder radius="md" padding="lg">
                <Text c="dimmed" size="sm">
                  Start the conversation by asking about your project. The assistant can run Layercake tools like `list_projects`, `list_graphs`, or perform analysis via MCP.
                </Text>
              </Card>
            )}

            {chat.messages.map(message => (
              <Group key={message.id} justify={message.role === 'user' ? 'flex-end' : 'flex-start'}>
                <Card
                  radius="md"
                  padding="sm"
                  withBorder
                  style={{
                    maxWidth: '75%',
                    backgroundColor:
                      message.role === 'user'
                        ? '#edf2ff'
                        : message.role === 'tool'
                          ? '#fff4e6'
                          : '#f8f9fa',
                  }}
                >
                  <Stack gap={4}>
                    <Group gap="xs">
                      <Badge size="xs" color={message.role === 'user' ? 'blue' : message.role === 'tool' ? 'orange' : 'gray'}>
                        {message.role === 'user' ? 'You' : message.role === 'tool' ? message.toolName ?? 'Tool' : 'Assistant'}
                      </Badge>
                      <Text size="xs" c="dimmed">
                        {new Date(message.createdAt).toLocaleTimeString()}
                      </Text>
                    </Group>
                    <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
                      {message.content}
                    </Text>
                  </Stack>
                </Card>
              </Group>
            ))}
          </Stack>
        </ScrollArea>
      </Paper>

      <Stack gap="xs">
        <Textarea
          minRows={2}
          autosize
          placeholder="Ask a question about this project…"
          value={input}
          onChange={(event) => setInput(event.currentTarget.value)}
          onKeyDown={handleKeyDown}
          disabled={chat.loading}
        />
        <Group justify="flex-end">
          <Button onClick={handleSend} disabled={chat.loading || !input.trim()}>
            Send
          </Button>
        </Group>
      </Stack>
    </Stack>
  )
}

// Outer component that manages provider state and session selection
export const ProjectChatPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [provider, setProvider] = useState<ChatProviderOption>('Gemini')
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const [sessionListKey, setSessionListKey] = useState(0)

  const { data: projectData } = useQuery(GET_PROJECT, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })
  const project = useMemo(() => (projectData as any)?.project ?? null, [projectData])

  // Fetch current session details for header
  const { data: sessionsData, refetch: refetchSessions } = useQuery(gql`
    query GetSessions($projectId: Int!) {
      chatSessions(projectId: $projectId, includeArchived: false) {
        id
        session_id
        title
        provider
        model_name
      }
    }
  `, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })

  const currentSession = useMemo(() => {
    if (!selectedSessionId || !sessionsData) return null
    return (sessionsData as any)?.chatSessions?.find((s: any) => s.session_id === selectedSessionId) || null
  }, [selectedSessionId, sessionsData])

  // Use a combination of provider and timestamp as key to force remount on provider change
  // This ensures the ChatInterface (and its useChat session hook) completely remounts,
  // creating a fresh subscription with the new sessionId
  const [mountKey, setMountKey] = useState(0)
  const handleProviderChange = (newProvider: ChatProviderOption) => {
    setProvider(newProvider)
    setMountKey(prev => prev + 1) // Force remount
  }

  const handleNewSession = () => {
    setSelectedSessionId(null)
    setMountKey(prev => prev + 1)
  }

  const handleSessionDeleted = () => {
    setSelectedSessionId(null)
    setSessionListKey(prev => prev + 1)
    refetchSessions()
  }

  const handleSessionArchived = () => {
    setSelectedSessionId(null)
    setSessionListKey(prev => prev + 1)
    refetchSessions()
  }

  const handleTitleUpdated = () => {
    refetchSessions()
  }

  const handleSelectSession = (sessionId: string | null) => {
    setSelectedSessionId(sessionId)
    setMountKey(prev => prev + 1)
  }

  if (!Number.isFinite(numericProjectId)) {
    return <Text c="red">Invalid project ID</Text>
  }

  return (
    <Flex h="100%" gap={0}>
      {/* Session list sidebar */}
      <Paper
        withBorder
        style={{
          width: 300,
          borderTop: 'none',
          borderBottom: 'none',
          borderLeft: 'none',
          borderRadius: 0,
          display: 'flex',
          flexDirection: 'column',
        }}
        p="md"
      >
        <ChatSessionList
          key={sessionListKey}
          projectId={numericProjectId}
          selectedSessionId={selectedSessionId}
          onSelectSession={handleSelectSession}
        />
      </Paper>

      {/* Main chat interface */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        <ChatInterface
          key={`${provider}-${mountKey}-${selectedSessionId}`}
          projectId={numericProjectId}
          provider={provider}
          project={project}
          onProviderChange={handleProviderChange}
          selectedSessionId={selectedSessionId}
          currentSession={currentSession}
          onNewSession={handleNewSession}
          onSessionDeleted={handleSessionDeleted}
          onSessionArchived={handleSessionArchived}
          onTitleUpdated={handleTitleUpdated}
        />
      </div>
    </Flex>
  )
}
