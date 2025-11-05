import '@assistant-ui/styles/index.css'

import {
  AssistantRuntimeProvider,
  type MessageStatus,
  type ThreadMessageLike,
  useAssistantState,
  useExternalStoreRuntime,
} from '@assistant-ui/react'
import { ThreadPrimitive, MessagePrimitive, ComposerPrimitive } from '@assistant-ui/react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import {
  Alert,
  Badge,
  Button,
  Card,
  Group,
  Loader,
  Paper,
  ScrollArea,
  Select,
  Stack,
  Text,
  Title,
} from '@mantine/core'
import { IconMessageDots, IconRefresh } from '@tabler/icons-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'

import { CHAT_PROVIDER_OPTIONS, type ChatProviderOption } from '../graphql/chat'
import { useChatSession, type ChatMessageEntry } from '../hooks/useChatSession'

const GET_PROJECT = gql`
  query GetProjectName($id: Int!) {
    project(id: $id) {
      id
      name
      description
    }
  }
`

const COMPLETE_STATUS: MessageStatus = { type: 'complete', reason: 'stop' }

const convertMessages = (entries: readonly ChatMessageEntry[]): ThreadMessageLike[] => {
  return entries.map((entry) => {
    const createdAt = new Date(entry.createdAt)
    if (entry.role === 'user') {
      return {
        role: 'user',
        id: entry.id,
        createdAt,
        content: [{ type: 'text', text: entry.content }],
      }
    }

    if (entry.role === 'tool') {
      return {
        role: 'assistant',
        id: entry.id,
        createdAt,
        status: COMPLETE_STATUS,
        content: [
          {
            type: 'tool-call',
            toolName: entry.toolName ?? 'Tool',
            result: entry.content,
          },
        ],
      }
    }

    return {
      role: 'assistant',
      id: entry.id,
      createdAt,
      status: COMPLETE_STATUS,
      content: [{ type: 'text', text: entry.content }],
    }
  })
}

const SimpleThreadMessage = () => {
  const role = useAssistantState(({ message }) => message.role)
  const align = role === 'user' ? 'flex-end' : 'flex-start'
  const background =
    role === 'user' ? 'var(--mantine-color-blue-1)' : role === 'assistant' ? 'var(--mantine-color-gray-1)' : 'transparent'

  return (
    <Group justify={align} w="100%" py="xs">
      <Paper
        radius="md"
        p="md"
        withBorder
        style={{
          maxWidth: '70%',
          background,
        }}
      >
        <MessagePrimitive.Parts />
      </Paper>
    </Group>
  )
}

const SimpleComposer = () => (
  <ComposerPrimitive.Root
    style={{
      display: 'flex',
      alignItems: 'flex-end',
      gap: 'var(--mantine-spacing-sm)',
      border: '1px solid var(--mantine-color-gray-3)',
      borderRadius: 'var(--mantine-radius-md)',
      padding: 'var(--mantine-spacing-sm)',
      background: 'var(--mantine-color-body)',
    }}
  >
    <ComposerPrimitive.Input
      aria-label="Ask the assistant"
      placeholder="Write a message…"
      style={{
        flex: 1,
        border: 'none',
        background: 'transparent',
        resize: 'none',
        outline: 'none',
        font: 'inherit',
        padding: '0',
      }}
    />
    <Group gap="xs" justify="flex-end" wrap="nowrap">
      <ThreadPrimitive.If running>
        <ComposerPrimitive.Cancel asChild>
          <Button variant="light" color="red" size="sm">
            Cancel
          </Button>
        </ComposerPrimitive.Cancel>
      </ThreadPrimitive.If>
      <ThreadPrimitive.If running={false}>
        <ComposerPrimitive.Send asChild>
          <Button size="sm">Send</Button>
        </ComposerPrimitive.Send>
      </ThreadPrimitive.If>
    </Group>
  </ComposerPrimitive.Root>
)

const ThreadView = () => {
  const viewportRef = useRef<HTMLDivElement | null>(null)

  return (
    <ThreadPrimitive.Root
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--mantine-spacing-md)',
        minHeight: '100%',
      }}
    >
      <ThreadPrimitive.Viewport
        ref={viewportRef}
        style={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <ThreadPrimitive.Empty>
          <Card withBorder radius="md" shadow="sm" p="lg">
            <Stack gap="xs">
              <Title order={4}>Start a conversation</Title>
              <Text c="dimmed" size="sm">
                Ask the assistant about this project. It can invoke Layercake tools where appropriate.
              </Text>
            </Stack>
          </Card>
        </ThreadPrimitive.Empty>

        <ThreadPrimitive.Messages components={{ Message: SimpleThreadMessage }} />
      </ThreadPrimitive.Viewport>
      <div style={{ position: 'sticky', bottom: 0 }}>
        <SimpleComposer />
      </div>
    </ThreadPrimitive.Root>
  )
}

export const AssistantUiChatPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [provider, setProvider] = useState<ChatProviderOption>('Gemini')

  const { data: projectData } = useQuery(GET_PROJECT, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })
  const project = useMemo(() => (projectData as any)?.project ?? null, [projectData])

  const chat = useChatSession({
    projectId: Number.isFinite(numericProjectId) ? numericProjectId : undefined,
    provider,
  })

  const providerSelectData = useMemo(
    () =>
      CHAT_PROVIDER_OPTIONS.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    [],
  )

  const [runtimeMessages, setRuntimeMessages] = useState<readonly ThreadMessageLike[]>([])

  useEffect(() => {
    setRuntimeMessages(convertMessages(chat.messages))
  }, [chat.messages])

  const runtime = useExternalStoreRuntime<ThreadMessageLike>({
    messages: runtimeMessages,
    setMessages: setRuntimeMessages,
    convertMessage: (message) => message,
    onNew: async (append) => {
      const content = append.content.find((part) => part.type === 'text')
      if (!content?.text) {
        return
      }
      await chat.sendMessage(content.text)
    },
  })

  const statusLabel = chat.loading
    ? 'Connecting…'
    : chat.isAwaitingAssistant
      ? 'Assistant thinking…'
      : 'Ready'

  return (
    <Stack h="100%" p="md" gap="md">
      <Group justify="space-between" align="flex-start">
        <div>
          <Title order={2}>Assistant UI Preview</Title>
          <Text c="dimmed" size="sm">
            {project
              ? `Exploring project "${project.name}" with the assistant-ui prototype.`
              : 'Prototype chat interface powered by assistant-ui.'}
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

      <Paper
        withBorder
        p="md"
        radius="md"
        style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}
      >
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

        <ScrollArea style={{ flex: 1 }} type="auto">
          <div style={{ minHeight: '100%', display: 'flex', flexDirection: 'column' }}>
            <AssistantRuntimeProvider runtime={runtime}>
              <ThreadView />
            </AssistantRuntimeProvider>
          </div>
        </ScrollArea>
      </Paper>
    </Stack>
  )
}
