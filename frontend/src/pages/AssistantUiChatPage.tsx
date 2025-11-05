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
import { IconMessageDots, IconRefresh } from '@tabler/icons-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'

import { CHAT_PROVIDER_OPTIONS, type ChatProviderOption } from '../graphql/chat'
import { useChatSession, type ChatMessageEntry } from '../hooks/useChatSession'
import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent } from '../components/ui/card'
import { Label } from '../components/ui/label'
import { ScrollArea } from '../components/ui/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select'
import { Spinner } from '../components/ui/spinner'

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
  const align = role === 'user' ? 'end' : 'start'
  const background =
    role === 'user' ? '#edf2ff' : role === 'assistant' ? '#f8f9fa' : 'transparent'

  return (
    <Group justify={align} className="w-full py-2">
      <Card
        className="border max-w-[70%]"
        style={{ background }}
      >
        <CardContent className="pt-4 pb-4">
          <MessagePrimitive.Parts />
        </CardContent>
      </Card>
    </Group>
  )
}

const SimpleComposer = () => (
  <ComposerPrimitive.Root
    className="flex items-end gap-2 border rounded-md p-2 bg-background"
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
    <Group gap="xs" justify="end" className="flex-shrink-0">
      <ThreadPrimitive.If running>
        <ComposerPrimitive.Cancel asChild>
          <Button variant="destructive" size="sm">
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
      className="flex flex-col gap-4 min-h-full"
    >
      <ThreadPrimitive.Viewport
        ref={viewportRef}
        className="flex-1 flex flex-col"
      >
        <ThreadPrimitive.Empty>
          <Card className="border shadow-sm">
            <CardContent className="pt-6">
              <Stack gap="xs">
                <h4 className="text-lg font-semibold">Start a conversation</h4>
                <p className="text-sm text-muted-foreground">
                  Ask the assistant about this project. It can invoke Layercake tools where appropriate.
                </p>
              </Stack>
            </CardContent>
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
    <Stack gap="md" className="h-full p-4">
      <Group justify="between" align="start">
        <div>
          <h2 className="text-2xl font-bold">Assistant UI Preview</h2>
          <p className="text-sm text-muted-foreground">
            {project
              ? `Exploring project "${project.name}" with the assistant-ui prototype.`
              : 'Prototype chat interface powered by assistant-ui.'}
          </p>
        </div>
        <Group gap="sm">
          <div className="space-y-2" style={{ minWidth: 220 }}>
            <Label htmlFor="provider-select">Provider</Label>
            <Select
              value={provider}
              onValueChange={(value) => {
                if (value) {
                  setProvider(value as ChatProviderOption)
                  chat.restart()
                }
              }}
            >
              <SelectTrigger id="provider-select">
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent>
                {providerSelectData.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    <div>
                      <div>{option.label}</div>
                      <div className="text-xs text-muted-foreground">{option.description}</div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <Button
            variant="secondary"
            onClick={chat.restart}
            disabled={chat.loading}
            className="mt-8"
          >
            <IconRefresh className="mr-2 h-4 w-4" />
            Restart Session
          </Button>
        </Group>
      </Group>

      {chat.error && (
        <Alert variant="destructive">
          <AlertTitle>Chat unavailable</AlertTitle>
          <AlertDescription>{chat.error}</AlertDescription>
        </Alert>
      )}

      <Card className="border flex-1 flex flex-col min-h-0">
        <CardContent className="pt-6 flex flex-col flex-1 min-h-0">
          <Group justify="between" className="mb-4">
            <Group gap="xs">
              <Badge>
                {provider}
              </Badge>
              {chat.session?.model && (
                <Badge variant="secondary">
                  Model: {chat.session.model}
                </Badge>
              )}
            </Group>
            <Group gap="xs">
              {chat.loading ? <Spinner className="h-4 w-4" /> : <IconMessageDots size={18} className="text-blue-400" />}
              <p className="text-sm text-muted-foreground">
                {statusLabel}
              </p>
            </Group>
          </Group>

          <ScrollArea className="flex-1">
            <div className="min-h-full flex flex-col">
              <AssistantRuntimeProvider runtime={runtime}>
                <ThreadView />
              </AssistantRuntimeProvider>
            </div>
          </ScrollArea>
        </CardContent>
      </Card>
    </Stack>
  )
}
