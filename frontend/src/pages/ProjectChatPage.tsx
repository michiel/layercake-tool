import '@assistant-ui/styles/index.css'

import {
  AssistantRuntimeProvider,
  ComposerPrimitive,
  MessagePrimitive,
  ThreadPrimitive,
  type MessageStatus,
  type ThreadMessageLike,
  useAssistantState,
  useExternalStoreRuntime,
} from '@assistant-ui/react'
import type {
  TextMessagePartComponent,
  ToolCallMessagePartComponent,
} from '@assistant-ui/react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import { IconMessageDots, IconRefresh } from '@tabler/icons-react'
import { useEffect, useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'

import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent } from '../components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select'
import { Spinner } from '../components/ui/spinner'
import { Label } from '../components/ui/label'
import { CHAT_PROVIDER_OPTIONS, type ChatProviderOption } from '../graphql/chat'
import { useChatSession, type ChatMessageEntry } from '../hooks/useChatSession'
import { cn } from '@/lib/utils'

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

const convertMessages = (
  entries: readonly ChatMessageEntry[],
): ThreadMessageLike[] =>
  entries.map((entry) => {
    const createdAt = new Date(entry.createdAt)

    if (entry.role === 'user') {
      return {
        role: 'user' as const,
        id: entry.id,
        createdAt,
        content: [{ type: 'text', text: entry.content }],
      }
    }

    if (entry.role === 'tool') {
      return {
        role: 'assistant' as const,
        id: entry.id,
        createdAt,
        status: COMPLETE_STATUS,
        content: [
          {
            type: 'tool-call' as const,
            toolName: entry.toolName ?? 'Tool',
            result: entry.content,
          },
        ],
      }
    }

    return {
      role: 'assistant' as const,
      id: entry.id,
      createdAt,
      status: COMPLETE_STATUS,
      content: [{ type: 'text', text: entry.content }],
    }
  })

const TextPart: TextMessagePartComponent = ({ text }) => (
  <p className="whitespace-pre-wrap break-words text-sm leading-relaxed">
    {text}
  </p>
)

const ToolCallPart: ToolCallMessagePartComponent<unknown, unknown> = ({
  toolName,
  result,
}) => (
  <div className="space-y-1 text-sm text-muted-foreground">
    <p className="font-medium text-xs uppercase tracking-wide">
      {toolName}
    </p>
    {typeof result === 'string' ? (
      <p className="whitespace-pre-wrap break-words">{result}</p>
    ) : (
      <pre className="whitespace-pre-wrap break-words rounded-md bg-muted/60 p-2 text-xs">
        {JSON.stringify(result, null, 2)}
      </pre>
    )}
  </div>
)

const ThreadMessageBubble = () => {
  const role = useAssistantState(({ message }) => message.role)

  const alignment =
    role === 'user' ? 'justify-end' : role === 'assistant' ? 'justify-start' : 'justify-start'
  const bubbleClasses = cn(
    'max-w-[75%] rounded-2xl px-4 py-3 text-sm shadow-sm border',
    role === 'user'
      ? 'bg-primary text-primary-foreground border-primary/80'
      : role === 'assistant'
        ? 'bg-muted text-foreground border-muted/80'
        : 'bg-secondary text-foreground border-secondary/80',
  )

  return (
    <div className={cn('flex w-full', alignment)}>
      <div className={bubbleClasses}>
        <MessagePrimitive.Parts
          components={{
            Text: TextPart,
            tools: { Override: ToolCallPart },
          }}
        />
      </div>
    </div>
  )
}

const ThreadComposer = () => (
  <ComposerPrimitive.Root className="flex items-end gap-3 rounded-xl border border-border bg-background px-4 py-3 shadow-sm">
    <ComposerPrimitive.Input
      aria-label="Ask the assistant"
      placeholder="Write a message…"
      className="flex-1 resize-none border-none bg-transparent p-0 text-sm outline-none focus-visible:ring-0"
    />
    <Group gap="sm">
      <ThreadPrimitive.If running>
        <ComposerPrimitive.Cancel asChild>
          <Button variant="ghost" size="sm">
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

const ThreadView = () => (
  <ThreadPrimitive.Root className="flex h-full flex-col gap-4">
    <ThreadPrimitive.Viewport className="flex-1 space-y-3 overflow-y-auto pr-1">
      <ThreadPrimitive.Empty>
        <Card className="border border-dashed">
          <CardContent className="pt-6">
            <Stack gap="xs">
              <h3 className="text-base font-semibold">
                Start a conversation
              </h3>
              <p className="text-sm text-muted-foreground">
                Ask a question about this project. The assistant can run Layercake
                tools whenever additional context is helpful.
              </p>
            </Stack>
          </CardContent>
        </Card>
      </ThreadPrimitive.Empty>

      <ThreadPrimitive.Messages components={{ Message: ThreadMessageBubble }} />
    </ThreadPrimitive.Viewport>

    <div className="border-t border-border pt-4">
      <ThreadComposer />
    </div>
  </ThreadPrimitive.Root>
)

export const ProjectChatPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [provider, setProvider] = useState<ChatProviderOption>('Gemini')

  const { data: projectData } = useQuery(GET_PROJECT, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })

  const project = useMemo(
    () => (projectData as any)?.project ?? null,
    [projectData],
  )

  const chat = useChatSession({
    projectId: Number.isFinite(numericProjectId) ? numericProjectId : undefined,
    provider,
  })

  const providerOptions = useMemo(
    () =>
      CHAT_PROVIDER_OPTIONS.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    [],
  )

  const [runtimeMessages, setRuntimeMessages] = useState<
    readonly ThreadMessageLike[]
  >([])

  useEffect(() => {
    setRuntimeMessages(convertMessages(chat.messages))
  }, [chat.messages])

  const runtime = useExternalStoreRuntime<ThreadMessageLike>({
    messages: runtimeMessages,
    setMessages: setRuntimeMessages,
    convertMessage: (message) => message,
    onNew: async (append) => {
      const content = append.content.find((part) => part.type === 'text')
      if (!content?.text?.trim()) {
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
        <Stack gap="xs">
          <h2 className="text-2xl font-bold">Project Chat</h2>
          <p className="text-sm text-muted-foreground">
            {project
              ? `Discuss project "${project.name}" with tool-assisted insights.`
              : 'Start a conversation powered by Layercake tools.'}
          </p>
        </Stack>

        <Group gap="md" align="end">
          <Stack gap="xs" className="min-w-[220px]">
            <Label htmlFor="provider-select">Provider</Label>
            <Select
              value={provider}
              onValueChange={(value) => {
                setProvider(value as ChatProviderOption)
                chat.restart()
              }}
            >
              <SelectTrigger id="provider-select">
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent>
                {providerOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    <div className="space-y-1">
                      <p className="font-medium">{option.label}</p>
                      <p className="text-xs text-muted-foreground">
                        {option.description}
                      </p>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Stack>

          <Button
            variant="secondary"
            onClick={chat.restart}
            disabled={chat.loading}
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

      <Card className="flex min-h-0 flex-1 flex-col">
        <CardContent className="flex min-h-0 flex-1 flex-col gap-4 pt-6">
          <Group justify="between" className="mb-2">
            <Group gap="xs">
              <Badge>{provider}</Badge>
              {chat.session?.model && (
                <Badge variant="secondary">Model: {chat.session.model}</Badge>
              )}
            </Group>
            <Group gap="xs">
              {chat.loading ? (
                <Spinner className="h-4 w-4" />
              ) : (
                <IconMessageDots className="h-5 w-5 text-primary" />
              )}
              <span className="text-sm text-muted-foreground">
                {statusLabel}
              </span>
            </Group>
          </Group>

          <AssistantRuntimeProvider runtime={runtime}>
            <ThreadView />
          </AssistantRuntimeProvider>
        </CardContent>
      </Card>
    </Stack>
  )
}
