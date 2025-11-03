import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import { Alert, Badge, Button, Card, Group, Loader, Paper, ScrollArea, Select, Stack, Text, Textarea, Title } from '@mantine/core'
import { IconMessageDots, IconRefresh } from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import { CHAT_PROVIDER_OPTIONS, ChatProviderOption } from '../graphql/chat'
import { useChatSession } from '../hooks/useChatSession'

const GET_PROJECT = gql`
  query GetProjectName($id: Int!) {
    project(id: $id) {
      id
      name
      description
    }
  }
`

const DEFAULT_PROVIDER: ChatProviderOption = 'Gemini'

export const ProjectChatPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [provider, setProvider] = useState<ChatProviderOption>(DEFAULT_PROVIDER)

  const { data: projectData } = useQuery(GET_PROJECT, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })
  const project = useMemo(() => (projectData as any)?.project ?? null, [projectData])

  const chat = useChatSession({
    projectId: Number.isFinite(numericProjectId) ? numericProjectId : undefined,
    provider,
  })

  const viewportRef = useRef<HTMLDivElement | null>(null)
  useEffect(() => {
    if (viewportRef.current) {
      viewportRef.current.scrollTop = viewportRef.current.scrollHeight
    }
  }, [chat.messages])

  const handleSend = async (input: string, clear: () => void) => {
    const trimmed = input.trim()
    if (!trimmed) return
    await chat.sendMessage(trimmed)
    clear()
  }

  const providerSelectData = useMemo(
    () =>
      CHAT_PROVIDER_OPTIONS.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    [],
  )

  const statusLabel = chat.loading
    ? 'Connecting…'
    : chat.isAwaitingAssistant
      ? 'Assistant thinking…'
      : 'Ready'

  return (
    <Stack h="100%" p="md" gap="md">
      <Group justify="space-between" align="flex-start">
        <div>
          <Title order={2}>Project Chat</Title>
          <Text c="dimmed" size="sm">
            {project
              ? `Discuss project "${project.name}" with tool-assisted insights.`
              : 'Start a conversation powered by Layercake tools.'}
          </Text>
        </div>
        <Group gap="sm">
          <Select
            label="Provider"
            data={providerSelectData}
            value={provider}
            onChange={(value) => {
              if (value) {
                setProvider(value as ChatProviderOption)
                chat.restart()
              }
            }}
            styles={{ root: { minWidth: 220 } }}
          />
          <Button
            variant="light"
            leftSection={<IconRefresh size={16} />}
            onClick={chat.restart}
            disabled={chat.loading}
          >
            Restart Session
          </Button>
        </Group>
      </Group>

      {chat.error && (
        <Alert color="red" title="Chat unavailable" mt="xs">
          {chat.error}
        </Alert>
      )}

      <Paper withBorder p="md" radius="md" style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
        <Group justify="space-between" mb="sm">
          <Group gap="xs">
            <Badge color="blue" variant="light">
              {provider}
            </Badge>
            {chat.session?.model && (
              <Badge color="gray" variant="light">
                Model: {chat.session.model}
              </Badge>
            )}
          </Group>
          <Group gap="xs">
            {chat.loading ? <Loader size="sm" /> : <IconMessageDots size={18} style={{ color: '#4dabf7' }} />}
            <Text size="sm" c="dimmed">
              {statusLabel}
            </Text>
          </Group>
        </Group>

        <ScrollArea style={{ flex: 1 }} viewportRef={viewportRef} type="auto">
          <Stack gap="sm" pr="sm">
            {chat.messages.length === 0 && !chat.loading && (
              <Card withBorder radius="md" padding="lg">
                <Text c="dimmed" size="sm">
                  Ask a question about this project. The assistant can run project-scoped tools for fresh data.
                </Text>
              </Card>
            )}

            {chat.messages.map((message) => (
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
                          ? 'You'
                          : message.role === 'tool'
                            ? message.toolName ?? 'Tool'
                            : 'Assistant'}
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

      <ChatInput onSend={handleSend} disabled={chat.loading} />
    </Stack>
  )
}

const ChatInput = ({
  onSend,
  disabled,
}: {
  onSend: (input: string, clear: () => void) => Promise<void>
  disabled: boolean
}) => {
  const [input, setInput] = useState('')

  const clear = () => setInput('')

  const handleSend = async () => {
    await onSend(input, clear)
  }

  const handleKeyDown: React.KeyboardEventHandler<HTMLTextAreaElement> = (event) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      void handleSend()
    }
  }

  return (
    <Stack gap="xs">
      <Textarea
        minRows={2}
        autosize
        placeholder="Ask a question about this project…"
        value={input}
        onChange={(event) => setInput(event.currentTarget.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
      />
      <Group justify="flex-end">
        <Button onClick={() => void handleSend()} disabled={disabled || !input.trim()}>
          Send
        </Button>
      </Group>
    </Stack>
  )
}
