import { useCallback, useEffect, useRef, useState } from 'react'
import { useApolloClient, useMutation } from '@apollo/client/react'
import {
  START_CHAT_SESSION,
  SEND_CHAT_MESSAGE,
  ChatProviderOption,
  ChatEventPayload,
  StartChatSessionPayload,
  CHAT_EVENTS_SUBSCRIPTION,
  GET_CHAT_HISTORY,
  ChatMessage,
} from '../graphql/chat'
import { ChatMessageEntry, ChatMessageRole } from '../types/chat'
import {
  appendMessageToProvider,
  resetProviderSession,
  setAwaitingForProvider,
  setMessagesForProvider,
  setSessionForProvider,
  useProviderSession,
} from '../state/chatSessionStore'

interface UseChatSessionArgs {
  projectId?: number
  provider: ChatProviderOption
  sessionId?: string | null
  context?: string | null
}

interface UseChatSessionResult {
  loading: boolean
  session?: StartChatSessionPayload
  messages: ChatMessageEntry[]
  error?: string | null
  isAwaitingAssistant: boolean
  sendMessage: (content: string) => Promise<void>
  restart: () => void
}

const nowIso = () => new Date().toISOString()
const makeId = () => `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`

const getErrorMessage = (error: unknown) => {
  if (!error) return 'Unknown error'
  const maybeApollo = error as any
  if (Array.isArray(maybeApollo?.graphQLErrors) && maybeApollo.graphQLErrors.length > 0) {
    return maybeApollo.graphQLErrors.map((e: any) => e.message).join(', ')
  }
  if (maybeApollo?.networkError?.message) {
    return maybeApollo.networkError.message
  }
  if (maybeApollo?.message) {
    return maybeApollo.message
  }
  if (error instanceof Error) return error.message
  return String(error)
}

export function useChatSession({
  projectId,
  provider,
  sessionId,
  context,
}: UseChatSessionArgs): UseChatSessionResult {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const providerState = useProviderSession(provider)
  const [startSession] = useMutation<{ startChatSession: StartChatSessionPayload }>(START_CHAT_SESSION)
  const [sendChat] = useMutation(SEND_CHAT_MESSAGE)
  const client = useApolloClient()
  const subscriptionRef = useRef<{ unsubscribe: () => void } | null>(null)

  const teardownSubscription = useCallback(() => {
    if (subscriptionRef.current) {
      subscriptionRef.current.unsubscribe()
      subscriptionRef.current = null
    }
  }, [])

  const activeSessionId =
    providerState.session?.sessionId ?? sessionId ?? undefined

  const restart = useCallback(() => {
    teardownSubscription()
    resetProviderSession(provider)
    setLoading(false)
    setError(null)
  }, [provider, teardownSubscription])

  // Ensure project id is tracked alongside the provider session
  useEffect(() => {
    if (providerState.session && projectId !== undefined) {
      setSessionForProvider(provider, providerState.session, projectId)
    }
  }, [provider, providerState.session, projectId])

  // Establish or reuse chat session
  useEffect(() => {
    if (!projectId) {
      return
    }

    if (providerState.session?.sessionId) {
      setLoading(false)
      setAwaitingForProvider(provider, false)
      return
    }

    let cancelled = false
    setLoading(true)
    setAwaitingForProvider(provider, false)

    ;(async () => {
      try {
        const { data } = await startSession({
          variables: { projectId, provider, sessionId: sessionId ?? providerState.session?.sessionId ?? null },
        })
        if (cancelled) return
        if (data?.startChatSession) {
          setSessionForProvider(provider, data.startChatSession, projectId)
        } else {
          setError('Failed to establish chat session.')
        }
      } catch (err) {
        if (!cancelled) {
          setError(getErrorMessage(err))
        }
      } finally {
        if (!cancelled) {
          setLoading(false)
        }
      }
    })()

    return () => {
      cancelled = true
    }
  }, [
    projectId,
    provider,
    sessionId,
    providerState.session?.sessionId,
    startSession,
  ])

  // Load history if needed (e.g., after reload)
  useEffect(() => {
    if (!activeSessionId || providerState.messages.length > 0) {
      return
    }

    let cancelled = false
    ;(async () => {
      try {
        const { data } = await client.query<{ chatHistory: ChatMessage[] }>({
          query: GET_CHAT_HISTORY,
          variables: { sessionId: activeSessionId },
          fetchPolicy: 'network-only',
        })
        if (cancelled) return
        if (data?.chatHistory) {
          const loadedMessages: ChatMessageEntry[] = data.chatHistory.map((msg) => ({
            id: msg.message_id,
            role: msg.role as ChatMessageRole,
            content: msg.content,
            toolName: msg.tool_name || undefined,
            createdAt: msg.created_at,
          }))
          setMessagesForProvider(provider, loadedMessages)
        }
      } catch (err) {
        if (!cancelled) {
          setError(getErrorMessage(err))
        }
      }
    })()

    return () => {
      cancelled = true
    }
  }, [activeSessionId, provider, providerState.messages.length, client])

  // Subscribe to real-time chat events from the active session
  useEffect(() => {
    if (!providerState.session?.sessionId) {
      return
    }

    const observable = client.subscribe<{ chatEvents: ChatEventPayload }>({
      query: CHAT_EVENTS_SUBSCRIPTION,
      variables: { sessionId: providerState.session.sessionId },
      fetchPolicy: 'no-cache',
    })

    subscriptionRef.current = observable.subscribe({
      next: ({ data }) => {
        const payload = data?.chatEvents
        if (!payload) {
          return
        }

        appendMessageToProvider(provider, {
          id: makeId(),
          role: payload.kind === 'ToolInvocation' ? 'tool' : 'assistant',
          content: payload.message,
          toolName: payload.toolName ?? undefined,
          createdAt: nowIso(),
        })

        if (payload.kind === 'AssistantMessage') {
          setAwaitingForProvider(provider, false)
        }
      },
      error: (subscriptionErr) => {
        setError(getErrorMessage(subscriptionErr))
        setAwaitingForProvider(provider, false)
      },
    })

    return () => {
      teardownSubscription()
    }
  }, [client, provider, providerState.session?.sessionId, teardownSubscription])

  useEffect(() => () => teardownSubscription(), [teardownSubscription])

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim()
      const sessionIdentifier = providerState.session?.sessionId
      if (!trimmed || !sessionIdentifier) {
        return
      }

      const enriched =
        context && context.trim().length > 0
          ? `Context: ${context.trim()}\n\n${trimmed}`
          : trimmed

      const userMessage: ChatMessageEntry = {
        id: makeId(),
        role: 'user',
        content: trimmed,
        createdAt: nowIso(),
      }
      appendMessageToProvider(provider, userMessage)
      setAwaitingForProvider(provider, true)

      try {
        await sendChat({
          variables: {
            sessionId: sessionIdentifier,
            message: enriched,
          },
        })
      } catch (err) {
        const errorMessage = getErrorMessage(err)
        setAwaitingForProvider(provider, false)
        appendMessageToProvider(provider, {
          id: makeId(),
          role: 'assistant',
          content: `Error sending message: ${errorMessage}`,
          createdAt: nowIso(),
        })
      }
    },
    [context, provider, providerState.session?.sessionId, sendChat],
  )

  return {
    loading,
    session: providerState.session,
    messages: providerState.messages,
    error,
    isAwaitingAssistant: providerState.isAwaitingAssistant,
    sendMessage,
    restart,
  }
}
