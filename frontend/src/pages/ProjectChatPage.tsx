import '@assistant-ui/styles/index.css'

import {
  AssistantRuntimeProvider,
  ComposerPrimitive,
  MessagePrimitive,
  ThreadPrimitive,
  type MessageStatus,
  type ThreadMessageLike,
  useExternalStoreRuntime,
} from '@assistant-ui/react'
import type {
  ToolCallMessagePartComponent,
} from '@assistant-ui/react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import {
  IconArrowDown,
  IconCircleX,
  IconMessageDots,
  IconRefresh,
  IconSend,
} from '@tabler/icons-react'
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

const UserMessage = () => (
  <MessagePrimitive.Root className="grid w-full max-w-3xl auto-rows-auto grid-cols-[minmax(48px,1fr)_auto] gap-y-2 self-end py-3 [&>*]:col-start-2">
    <div className="rounded-2xl bg-primary px-4 py-3 text-sm text-primary-foreground shadow">
      <MessagePrimitive.Parts />
    </div>
  </MessagePrimitive.Root>
)

const AssistantMessage = () => (
  <MessagePrimitive.Root className="grid w-full max-w-3xl auto-rows-auto grid-cols-[auto_minmax(48px,1fr)] gap-y-2 self-start py-3 [&>*]:col-start-1">
    <div className="rounded-2xl border border-border bg-muted px-4 py-3 text-sm shadow-sm">
      <MessagePrimitive.Parts components={{ tools: { Override: ToolCallPart } }} />
    </div>
  </MessagePrimitive.Root>
)

const SystemMessage = () => (
  <MessagePrimitive.Root className="mx-auto max-w-2xl py-2 text-xs text-muted-foreground">
    <MessagePrimitive.Parts />
  </MessagePrimitive.Root>
)

const ScrollToBottomButton = () => (
  <ThreadPrimitive.ScrollToBottom asChild>
    <Button
      size="icon"
      variant="secondary"
      className="absolute -top-12 right-4 rounded-full shadow"
    >
      <IconArrowDown className="h-4 w-4" />
    </Button>
  </ThreadPrimitive.ScrollToBottom>
)

const Suggestion = ({ prompt }: { prompt: string }) => (
  <ThreadPrimitive.Suggestion
    prompt={prompt}
    method="replace"
    autoSend
    className="flex flex-col gap-1 rounded-xl border border-dashed border-border bg-muted/50 p-3 text-left text-sm transition-colors hover:bg-muted"
  >
    {prompt}
  </ThreadPrimitive.Suggestion>
)

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
          <Button variant="ghost" size="icon">
            <IconCircleX className="h-4 w-4" />
          </Button>
        </ComposerPrimitive.Cancel>
      </ThreadPrimitive.If>
      <ThreadPrimitive.If running={false}>
        <ComposerPrimitive.Send asChild>
          <Button size="icon">
            <IconSend className="h-4 w-4" />
          </Button>
        </ComposerPrimitive.Send>
      </ThreadPrimitive.If>
    </Group>
  </ComposerPrimitive.Root>
)

const ThreadView = () => (
  <ThreadPrimitive.Root className="relative flex h-full flex-col">
    <ThreadPrimitive.Viewport className="flex-1 space-y-3 overflow-y-auto px-1 pb-6">
      <ThreadPrimitive.Empty>
        <Stack gap="lg" className="items-center py-12">
          <Card className="w-full max-w-3xl border border-dashed">
            <CardContent className="pt-6">
              <Stack gap="xs">
                <h3 className="text-base font-semibold">
                  Start a conversation
                </h3>
                <p className="text-sm text-muted-foreground">
                  Ask a question about this project. The assistant can run Layercake tools whenever additional context is helpful.
                </p>
              </Stack>
            </CardContent>
          </Card>
          <Group gap="sm" wrap className="max-w-3xl">
            <Suggestion prompt="Summarize the latest project updates." />
            <Suggestion prompt="List recent tool invocations for this project." />
            <Suggestion prompt="What are the open tasks for this project?" />
          </Group>
        </Stack>
      </ThreadPrimitive.Empty>

      <ThreadPrimitive.Messages
        components={{
          UserMessage,
          AssistantMessage,
          SystemMessage,
        }}
      />

      <ScrollToBottomButton />
    </ThreadPrimitive.Viewport>

    <div className="border-t border-border bg-background px-1 pt-4">
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
