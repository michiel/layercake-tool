import { useCallback, useEffect, useMemo, useState } from 'react'
import { useMutation, useSubscription } from '@apollo/client/react'
import {
  START_CHAT_SESSION,
  SEND_CHAT_MESSAGE,
  CHAT_EVENTS_SUBSCRIPTION,
  ChatProviderOption,
  ChatEventPayload,
  StartChatSessionPayload,
} from '../graphql/chat'

export type ChatMessageRole = 'user' | 'assistant' | 'tool'

export interface ChatMessageEntry {
  id: string
  role: ChatMessageRole
  content: string
  toolName?: string
  createdAt: string
}

interface UseChatSessionArgs {
  projectId?: number
  provider: ChatProviderOption
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

export function useChatSession({ projectId, provider }: UseChatSessionArgs): UseChatSessionResult {
  const [session, setSession] = useState<StartChatSessionPayload | undefined>()
  const [messages, setMessages] = useState<ChatMessageEntry[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isAwaitingAssistant, setAwaitingAssistant] = useState(false)
  const [restartKey, setRestartKey] = useState(0)
  const [subscriptionActive, setSubscriptionActive] = useState(false)

  const [startSession] = useMutation<{ startChatSession: StartChatSessionPayload }>(START_CHAT_SESSION)
  const [sendChat] = useMutation(SEND_CHAT_MESSAGE)

  const restart = useCallback(() => {
    setSession(undefined)
    setMessages([])
    setAwaitingAssistant(false)
    setError(null)
    setSubscriptionActive(false)
    setRestartKey(prev => prev + 1)
  }, [])

  useEffect(() => {
    if (!projectId) return

    let cancelled = false
    setLoading(true)
    setError(null)
    setMessages([])
    setSession(undefined)
    setAwaitingAssistant(false)
    setSubscriptionActive(false) // Ensure subscription is inactive during session creation

    ;(async () => {
      try {
        const { data } = await startSession({
          variables: { projectId, provider },
        })
        if (cancelled) return
        if (data?.startChatSession) {
          console.log('[Chat] Session started:', data.startChatSession)
          setSession(data.startChatSession)
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
  }, [projectId, provider, restartKey, startSession])

  // Activate subscription after session is created
  // This ensures Apollo Client treats each session change as a completely new subscription
  useEffect(() => {
    if (session?.sessionId && !loading) {
      console.log('[Chat] Activating subscription for session:', session.sessionId)
      // Use a longer delay to ensure Apollo Client fully processes the deactivation
      // and any React reconciliation has completed
      const timer = setTimeout(() => {
        console.log('[Chat] Subscription activated for session:', session.sessionId)
        setSubscriptionActive(true)
      }, 100)
      return () => {
        clearTimeout(timer)
        // Ensure subscription is deactivated on cleanup
        setSubscriptionActive(false)
      }
    } else {
      setSubscriptionActive(false)
    }
  }, [session?.sessionId, loading])

  // IMPORTANT: Include sessionId in the key to force subscription restart when session changes
  // Without this, Apollo reuses the old subscription even when sessionId changes
  const subscriptionKey = `chat-${session?.sessionId ?? 'none'}`
  console.log('[Chat] Subscription key:', subscriptionKey, 'skip:', !subscriptionActive, 'sessionId:', session?.sessionId)

  const subscriptionResult = useSubscription<{ chatEvents: ChatEventPayload }>(
    CHAT_EVENTS_SUBSCRIPTION,
    {
      variables: { sessionId: session?.sessionId ?? '' },
      skip: !subscriptionActive || !session?.sessionId, // Only subscribe when explicitly activated
      fetchPolicy: 'no-cache', // Don't cache subscription data
      shouldResubscribe: true, // Force resubscribe on variable changes
      onData: ({ data }) => {
        console.log('[Chat] Subscription data received:', data)
      },
      onError: (error) => {
        console.error('[Chat] Subscription error:', error)
      },
      onComplete: () => {
        console.log('[Chat] Subscription complete')
      },
      onSubscriptionData: (options) => {
        console.log('[Chat] onSubscriptionData callback:', options)
      },
    },
  )

  console.log('[Chat] Subscription state - data:', subscriptionResult.data, 'error:', subscriptionResult.error, 'loading:', subscriptionResult.loading)

  const subscriptionData = subscriptionResult.data
  const subscriptionError = subscriptionResult.error

  useEffect(() => {
    const payload = subscriptionData?.chatEvents
    console.log('[Chat] useEffect triggered, payload:', payload)
    if (!payload) return

    console.log('[Chat] Adding message to state:', payload)
    setMessages(prev => [
      ...prev,
      {
        id: makeId(),
        role: payload.kind === 'ToolInvocation' ? 'tool' : 'assistant',
        content: payload.message,
        toolName: payload.toolName ?? undefined,
        createdAt: nowIso(),
      },
    ])

    if (payload.kind === 'AssistantMessage') {
      setAwaitingAssistant(false)
    }
  }, [subscriptionData])

  useEffect(() => {
    if (!subscriptionError) return
    setError(getErrorMessage(subscriptionError))
    setAwaitingAssistant(false)
  }, [subscriptionError])

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim()
      if (!trimmed || !session?.sessionId) {
        return
      }

      const userMessage: ChatMessageEntry = {
        id: makeId(),
        role: 'user',
        content: trimmed,
        createdAt: nowIso(),
      }
      setMessages(prev => [...prev, userMessage])
      setAwaitingAssistant(true)

      try {
        await sendChat({
          variables: {
            sessionId: session.sessionId,
            message: trimmed,
          },
        })
      } catch (err) {
        const errorMessage = getErrorMessage(err)
        setAwaitingAssistant(false)
        setMessages(prev => [
          ...prev,
          {
            id: makeId(),
            role: 'assistant',
            content: `Error sending message: ${errorMessage}`,
            createdAt: nowIso(),
          },
        ])
      }
    },
    [sendChat, session?.sessionId],
  )

  const memoizedMessages = useMemo(() => messages, [messages])

  return {
    loading,
    session,
    messages: memoizedMessages,
    error,
    isAwaitingAssistant,
    sendMessage,
    restart,
  }
}
