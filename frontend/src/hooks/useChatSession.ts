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
  useChatSessionsHydrationStatus,
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

const DEBUG = import.meta.env.DEV

export function useChatSession({
  projectId,
  provider,
  sessionId,
  context,
}: UseChatSessionArgs): UseChatSessionResult {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const providerState = useProviderSession(provider)
  const hydrationStatus = useChatSessionsHydrationStatus()
  const isHydrated = hydrationStatus === 'ready'
  const [startSession] = useMutation<{ startChatSession: StartChatSessionPayload }>(START_CHAT_SESSION)
  const [sendChat] = useMutation(SEND_CHAT_MESSAGE)
  const client = useApolloClient()
  const subscriptionRef = useRef<{ unsubscribe: () => void } | null>(null)
  const pendingMessagesRef = useRef<string[]>([])
  const awaitingTimeoutRef = useRef<number | null>(null)

  const teardownSubscription = useCallback(() => {
    if (subscriptionRef.current) {
      subscriptionRef.current.unsubscribe()
      subscriptionRef.current = null
    }
  }, [])

  const activeSessionId =
    providerState.session?.sessionId ?? sessionId ?? undefined

  const restart = useCallback(() => {
    if (DEBUG) {
      console.log('[useChatSession] Restarting session', { provider })
    }
    teardownSubscription()
    resetProviderSession(provider)
    setLoading(false)
    setError(null)
    if (awaitingTimeoutRef.current) {
      clearTimeout(awaitingTimeoutRef.current)
      awaitingTimeoutRef.current = null
    }
  }, [provider, teardownSubscription])

  // Debug logging effect
  useEffect(() => {
    if (DEBUG) {
      console.log('[useChatSession] State update', {
        provider,
        isHydrated,
        hydrationStatus,
        hasSession: !!providerState.session?.sessionId,
        sessionId: providerState.session?.sessionId,
        loading,
        isAwaitingAssistant: providerState.isAwaitingAssistant,
        messagesCount: providerState.messages.length,
        pendingCount: pendingMessagesRef.current.length,
        projectId,
        error,
      })
    }
  }, [
    provider,
    isHydrated,
    hydrationStatus,
    providerState.session?.sessionId,
    providerState.isAwaitingAssistant,
    providerState.messages.length,
    loading,
    projectId,
    error,
  ])

  // Ensure project id is tracked alongside the provider session
  useEffect(() => {
    if (!isHydrated) {
      return
    }
    if (providerState.session && projectId !== undefined) {
      setSessionForProvider(provider, providerState.session, projectId)
    }
  }, [provider, providerState.session, projectId, isHydrated])

  // Establish or reuse chat session
  useEffect(() => {
    if (!projectId || !isHydrated) {
      if (DEBUG && !isHydrated) {
        console.log('[useChatSession] Waiting for hydration before starting session')
      }
      if (DEBUG && !projectId) {
        console.log('[useChatSession] No projectId provided, skipping session start')
      }
      return
    }

    if (providerState.session?.sessionId) {
      if (DEBUG) {
        console.log('[useChatSession] Reusing existing session', { sessionId: providerState.session.sessionId })
      }
      setLoading(false)
      setAwaitingForProvider(provider, false)
      return
    }

    let cancelled = false
    setLoading(true)
    setAwaitingForProvider(provider, false)

    if (DEBUG) {
      console.log('[useChatSession] Starting new session', { projectId, provider })
    }

    ;(async () => {
      try {
        const { data } = await startSession({
          variables: { projectId, provider, sessionId: sessionId ?? providerState.session?.sessionId ?? null },
        })
        if (cancelled) return
        if (data?.startChatSession) {
          if (DEBUG) {
            console.log('[useChatSession] Session started', data.startChatSession)
          }
          setSessionForProvider(provider, data.startChatSession, projectId)
        } else {
          const errorMsg = 'Failed to establish chat session.'
          if (DEBUG) {
            console.error('[useChatSession] Session start failed - no data returned')
          }
          setError(errorMsg)
        }
      } catch (err) {
        if (!cancelled) {
          const errorMsg = getErrorMessage(err)
          if (DEBUG) {
            console.error('[useChatSession] Session start error', errorMsg, err)
          }
          setError(errorMsg)
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
  }, [projectId, provider, sessionId, providerState.session?.sessionId, startSession, isHydrated])

  // Load history if needed (e.g., after reload)
  useEffect(() => {
    if (!isHydrated || !activeSessionId || providerState.messages.length > 0) {
      return
    }

    if (DEBUG) {
      console.log('[useChatSession] Loading chat history', { sessionId: activeSessionId })
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
          if (DEBUG) {
            console.log('[useChatSession] Loaded chat history', { count: data.chatHistory.length })
          }
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
          const errorMsg = getErrorMessage(err)
          if (DEBUG) {
            console.error('[useChatSession] Failed to load history', errorMsg, err)
          }
          setError(errorMsg)
        }
      }
    })()

    return () => {
      cancelled = true
    }
  }, [activeSessionId, provider, providerState.messages.length, client, isHydrated])

  // Subscribe to real-time chat events from the active session
  useEffect(() => {
    if (!isHydrated || !providerState.session?.sessionId) {
      return
    }

    if (DEBUG) {
      console.log('[useChatSession] Setting up subscription', { sessionId: providerState.session.sessionId })
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

        if (DEBUG) {
          console.log('[useChatSession] Received chat event', payload)
        }

        const isToolInvocation = payload.kind === 'ToolInvocation' || payload.kind === 'TOOL_INVOCATION'
        const isAssistantMessage = payload.kind === 'AssistantMessage' || payload.kind === 'ASSISTANT_MESSAGE'

        appendMessageToProvider(provider, {
          id: makeId(),
          role: isToolInvocation ? 'tool' : 'assistant',
          content: payload.message,
          toolName: payload.toolName ?? undefined,
          createdAt: nowIso(),
        })

        if (isAssistantMessage) {
          if (DEBUG) {
            console.log('[useChatSession] Assistant response complete, clearing awaiting state')
          }
          setAwaitingForProvider(provider, false)
          // Clear timeout when response arrives
          if (awaitingTimeoutRef.current) {
            clearTimeout(awaitingTimeoutRef.current)
            awaitingTimeoutRef.current = null
          }
        }
      },
      error: (subscriptionErr) => {
        const errorMsg = getErrorMessage(subscriptionErr)
        if (DEBUG) {
          console.error('[useChatSession] Subscription error', errorMsg, subscriptionErr)
        }
        setError(errorMsg)
        setAwaitingForProvider(provider, false)
        // Clear timeout on error
        if (awaitingTimeoutRef.current) {
          clearTimeout(awaitingTimeoutRef.current)
          awaitingTimeoutRef.current = null
        }
      },
    })

    return () => {
      teardownSubscription()
    }
  }, [client, provider, providerState.session?.sessionId, teardownSubscription, isHydrated])

  useEffect(() => () => teardownSubscription(), [teardownSubscription])

  const deliverMessage = useCallback(
    async (sessionIdentifier: string, message: string) => {
      if (DEBUG) {
        console.log('[useChatSession] Delivering message', { sessionId: sessionIdentifier, messageLength: message.length })
      }
      try {
        await sendChat({
          variables: {
            sessionId: sessionIdentifier,
            message,
          },
        })
        if (DEBUG) {
          console.log('[useChatSession] Message sent successfully')
        }
      } catch (err) {
        const errorMessage = getErrorMessage(err)
        if (DEBUG) {
          console.error('[useChatSession] Failed to send message', errorMessage, err)
        }
        appendMessageToProvider(provider, {
          id: makeId(),
          role: 'assistant',
          content: `Error sending message: ${errorMessage}`,
          createdAt: nowIso(),
        })
        setAwaitingForProvider(provider, false)
        setError(errorMessage)
        // Clear timeout on error
        if (awaitingTimeoutRef.current) {
          clearTimeout(awaitingTimeoutRef.current)
          awaitingTimeoutRef.current = null
        }
        throw err // Re-throw to let pending message handler know it failed
      }
    },
    [provider, sendChat],
  )

  useEffect(() => {
    if (!providerState.session?.sessionId) {
      return
    }
    if (pendingMessagesRef.current.length === 0) {
      return
    }

    const sessionIdentifier = providerState.session.sessionId
    const queue = [...pendingMessagesRef.current]
    pendingMessagesRef.current = []

    if (DEBUG) {
      console.log('[useChatSession] Processing pending messages', { count: queue.length })
    }

    // Process pending messages sequentially with proper error handling
    ;(async () => {
      for (const message of queue) {
        try {
          await deliverMessage(sessionIdentifier, message)
        } catch (err) {
          if (DEBUG) {
            console.error('[useChatSession] Failed to deliver pending message', err)
          }
          // Stop processing on first error - user can retry
          break
        }
      }
    })()
  }, [providerState.session?.sessionId, deliverMessage, provider])

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim()
      if (!trimmed) {
        return
      }

      if (DEBUG) {
        console.log('[useChatSession] Sending message', { hasSession: !!providerState.session?.sessionId })
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

      // Set timeout to automatically reset awaiting state after 30 seconds
      if (awaitingTimeoutRef.current) {
        clearTimeout(awaitingTimeoutRef.current)
      }
      awaitingTimeoutRef.current = setTimeout(() => {
        if (DEBUG) {
          console.warn('[useChatSession] Message timeout - no response received')
        }
        setAwaitingForProvider(provider, false)
        setError('Response timeout - please try again')
        awaitingTimeoutRef.current = null
      }, 30000) // 30 second timeout

      const sessionIdentifier = providerState.session?.sessionId
      if (!sessionIdentifier) {
        if (DEBUG) {
          console.log('[useChatSession] No session yet, queuing message')
        }
        pendingMessagesRef.current.push(enriched)
        return
      }

      deliverMessage(sessionIdentifier, enriched)
    },
    [context, provider, providerState.session?.sessionId, deliverMessage],
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
