import { useMemo, useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useQuery, useMutation } from '@apollo/client/react'

import { AssistantRuntimeProvider } from '@assistant-ui/react'
import { IconMessageDots } from '@tabler/icons-react'

import { useChat } from '../components/chat/ChatProvider'
import { AssistantThread } from '../components/chat/AssistantThread'
import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent } from '../components/ui/card'
import { Label } from '../components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select'
import { Switch } from '../components/ui/switch'
import { Spinner } from '../components/ui/spinner'
import { CHAT_PROVIDER_OPTIONS, ChatProviderOption, GET_CHAT_SESSIONS, UPDATE_CHAT_SESSION_RAG, type ChatSession } from '../graphql/chat'
import { useRegisterChatContext } from '../hooks/useRegisterChatContext'
import { DEFAULT_CHAT_SUGGESTIONS } from '../config/chat'

const GET_PROJECT = gql`
  query GetProjectName($id: Int!) {
    project(id: $id) {
      id
      name
      description
    }
  }
`

export const ProjectChatPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const {
    provider,
    setProvider,
    runtime,
    session,
    messages,
    loading,
    error,
    isAwaitingAssistant,
    restart,
  } = useChat()

  useRegisterChatContext(
    numericProjectId
      ? `Viewing project chat for project ${numericProjectId}`
      : 'Viewing project chat',
    Number.isFinite(numericProjectId) ? numericProjectId : undefined,
  )

  const { data: projectData } = useQuery(GET_PROJECT, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })

  const project = useMemo(
    () => (projectData as any)?.project ?? null,
    [projectData],
  )

  // Fetch full session details to get RAG settings
  const { data: sessionsData, refetch: refetchSessions } = useQuery(GET_CHAT_SESSIONS, {
    variables: { projectId: numericProjectId, limit: 100 },
    skip: !Number.isFinite(numericProjectId) || !session?.sessionId,
  })

  const currentSession = useMemo(() => {
    if (!session?.sessionId || !(sessionsData as any)?.chatSessions) return null
    return ((sessionsData as any).chatSessions as ChatSession[]).find(
      (s) => s.session_id === session.sessionId
    )
  }, [session?.sessionId, sessionsData])

  const [ragEnabled, setRagEnabled] = useState(currentSession?.enable_rag ?? true)

  useEffect(() => {
    if (currentSession?.enable_rag !== undefined) {
      setRagEnabled(currentSession.enable_rag)
    }
  }, [currentSession])

  const [updateRagMutation] = useMutation(UPDATE_CHAT_SESSION_RAG)

  const handleRagToggle = async (enabled: boolean) => {
    if (!session?.sessionId) return

    setRagEnabled(enabled)
    try {
      await updateRagMutation({
        variables: {
          sessionId: session.sessionId,
          enableRag: enabled,
        },
      })
      await refetchSessions()
    } catch (err) {
      console.error('Failed to update RAG settings:', err)
      // Revert on error
      setRagEnabled(!enabled)
    }
  }

  const providerOptions = useMemo(
    () =>
      CHAT_PROVIDER_OPTIONS.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    [],
  )

  const statusLabel = loading
    ? 'Connecting…'
    : isAwaitingAssistant
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
            onClick={() => restart()}
            disabled={loading}
          >
            Restart Session
          </Button>
        </Group>
      </Group>

      {error && (
        <Alert variant="destructive">
          <AlertTitle>Chat unavailable</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <Card className="flex min-h-0 flex-1 flex-col">
        <CardContent className="flex min-h-0 flex-1 flex-col gap-4 pt-6">
          <Group justify="between" className="mb-2">
            <Group gap="xs">
              <Badge>{provider}</Badge>
              {session?.model && (
                <Badge variant="secondary">Model: {session.model}</Badge>
              )}
              <Badge variant="outline">Messages: {messages.length}</Badge>
              {session?.sessionId && (
                <Group gap="xs" className="items-center rounded-md border border-border px-3 py-1">
                  <span className="text-sm">📚 Knowledge Base</span>
                  <Switch
                    checked={ragEnabled}
                    onCheckedChange={handleRagToggle}
                    disabled={loading}
                  />
                </Group>
              )}
            </Group>
            <Group gap="xs">
              {loading ? (
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
            <AssistantThread
              suggestions={DEFAULT_CHAT_SUGGESTIONS}
              composerDisabled={isAwaitingAssistant}
              showSuggestions={!messages.length}
              composerPlaceholder={loading ? 'Connecting…' : undefined}
              layout="page"
            />
          </AssistantRuntimeProvider>
        </CardContent>
      </Card>
    </Stack>
  )
}
