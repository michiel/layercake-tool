import { useCallback, useEffect, useRef, useState } from 'react'
import { useApolloClient, useMutation } from '@apollo/client/react'
import {
  START_CHAT_SESSION,
  SEND_CHAT_MESSAGE,
  ChatProviderOption,
  ChatEventPayload,
  StartChatSessionPayload,
  CHAT_EVENTS_SUBSCRIPTION,
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
  const [restartNonce, setRestartNonce] = useState(0)

  const client = useApolloClient()
  const [startSession] = useMutation<{ startChatSession: StartChatSessionPayload }>(START_CHAT_SESSION)
  const [sendChat] = useMutation(SEND_CHAT_MESSAGE)

  const subscriptionRef = useRef<{ unsubscribe: () => void } | null>(null)

  const teardownSubscription = useCallback(() => {
    if (subscriptionRef.current) {
      subscriptionRef.current.unsubscribe()
      subscriptionRef.current = null
    }
  }, [])

  const restart = useCallback(() => {
    teardownSubscription()
    setSession(undefined)
    setMessages([])
    setAwaitingAssistant(false)
    setError(null)
    setRestartNonce(prev => prev + 1)
  }, [teardownSubscription])

  useEffect(() => {
    if (!projectId) {
      return
    }

    let cancelled = false
    setLoading(true)
    setError(null)
    setMessages([])
    setSession(undefined)
    setAwaitingAssistant(false)
    teardownSubscription()

    ;(async () => {
      try {
        const { data } = await startSession({
          variables: { projectId, provider },
        })
        if (cancelled) return
        if (data?.startChatSession) {
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
  }, [projectId, provider, restartNonce, startSession, teardownSubscription])

  useEffect(() => {
    if (!session?.sessionId) {
      return
    }

    const observable = client.subscribe<{ chatEvents: ChatEventPayload }>({
      query: CHAT_EVENTS_SUBSCRIPTION,
      variables: { sessionId: session.sessionId },
      fetchPolicy: 'no-cache',
    })

    subscriptionRef.current = observable.subscribe({
      next: ({ data }) => {
        const payload = data?.chatEvents
        if (!payload) {
          return
        }

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
      },
      error: (subscriptionErr) => {
        setError(getErrorMessage(subscriptionErr))
        setAwaitingAssistant(false)
      },
    })

    return () => {
      teardownSubscription()
    }
  }, [client, session?.sessionId, teardownSubscription])

  useEffect(() => {
    return () => {
      teardownSubscription()
    }
  }, [teardownSubscription])

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

  return {
    loading,
    session,
    messages,
    error,
    isAwaitingAssistant,
    sendMessage,
    restart,
  }
}
