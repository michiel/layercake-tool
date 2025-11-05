import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import { IconMessageDots, IconRefresh } from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import { CHAT_PROVIDER_OPTIONS, ChatProviderOption } from '../graphql/chat'
import { useChatSession } from '../hooks/useChatSession'
import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent } from '../components/ui/card'
import { Label } from '../components/ui/label'
import { ScrollArea } from '../components/ui/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select'
import { Spinner } from '../components/ui/spinner'
import { Textarea } from '../components/ui/textarea'

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
    <Stack gap="md" className="h-full p-4">
      <Group justify="between" align="start">
        <div>
          <h2 className="text-2xl font-bold">Project Chat</h2>
          <p className="text-sm text-muted-foreground">
            {project
              ? `Discuss project "${project.name}" with tool-assisted insights.`
              : 'Start a conversation powered by Layercake tools.'}
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

          <ScrollArea className="flex-1" viewportRef={viewportRef}>
            <Stack gap="sm" className="pr-2">
              {chat.messages.length === 0 && !chat.loading && (
                <Card className="border">
                  <CardContent className="pt-6">
                    <p className="text-sm text-muted-foreground">
                      Ask a question about this project. The assistant can run project-scoped tools for fresh data.
                    </p>
                  </CardContent>
                </Card>
              )}

              {chat.messages.map((message) => (
                <Group key={message.id} justify={message.role === 'user' ? 'end' : 'start'}>
                  <Card
                    className="border max-w-[75%]"
                    style={{
                      backgroundColor:
                        message.role === 'user'
                          ? '#edf2ff'
                          : message.role === 'tool'
                            ? '#fff4e6'
                            : '#f8f9fa',
                    }}
                  >
                    <CardContent className="pt-4 pb-4">
                      <Stack gap="xs">
                        <Group gap="xs">
                          <Badge
                            variant={
                              message.role === 'user'
                                ? 'default'
                                : message.role === 'tool'
                                  ? 'secondary'
                                  : 'outline'
                            }
                            className="text-xs"
                          >
                            {message.role === 'user'
                              ? 'You'
                              : message.role === 'tool'
                                ? message.toolName ?? 'Tool'
                                : 'Assistant'}
                          </Badge>
                          <span className="text-xs text-muted-foreground">
                            {new Date(message.createdAt).toLocaleTimeString()}
                          </span>
                        </Group>
                        <p className="text-sm whitespace-pre-wrap">
                          {message.content}
                        </p>
                      </Stack>
                    </CardContent>
                  </Card>
                </Group>
              ))}
            </Stack>
          </ScrollArea>
        </CardContent>
      </Card>

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
        rows={2}
        placeholder="Ask a question about this project…"
        value={input}
        onChange={(event) => setInput(event.currentTarget.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
      />
      <Group justify="end">
        <Button onClick={() => void handleSend()} disabled={disabled || !input.trim()}>
          Send
        </Button>
      </Group>
    </Stack>
  )
}
