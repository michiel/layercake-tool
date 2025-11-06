import '@assistant-ui/styles/index.css'

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react'

import {
  AssistantRuntimeProvider,
  useExternalStoreRuntime,
  type ThreadMessageLike,
} from '@assistant-ui/react'

import type { ChatProviderOption } from '../../graphql/chat'
import { useChatSession } from '../../hooks/useChatSession'
import { ChatMessageEntry } from '../../types/chat'
import { useChatContextStore } from '../../state/chatContextStore'
import { resetProviderSession } from '../../state/chatSessionStore'
import { ChatModal } from './ChatModal'

interface ChatContextValue extends ReturnType<typeof useChatSession> {
  provider: ChatProviderOption
  setProvider: (provider: ChatProviderOption) => void
  projectId?: number
  setProjectId: (projectId?: number) => void
  setPageContext: (context: string | null) => void
  pageContext: string | null
  runtime: ReturnType<typeof useExternalStoreRuntime<ThreadMessageLike>>
}

const ChatContext = createContext<ChatContextValue | undefined>(undefined)

const convertMessages = (entries: readonly ChatMessageEntry[]): ThreadMessageLike[] =>
  entries.map((entry) => {
    if (entry.role === 'user') {
      return {
        role: 'user' as const,
        id: entry.id,
        createdAt: new Date(entry.createdAt),
        content: [{ type: 'text', text: entry.content }],
      }
    }

    if (entry.role === 'tool') {
      return {
        role: 'assistant' as const,
        id: entry.id,
        createdAt: new Date(entry.createdAt),
        status: { type: 'complete', reason: 'stop' } as const,
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
      createdAt: new Date(entry.createdAt),
      status: { type: 'complete', reason: 'stop' } as const,
      content: [{ type: 'text', text: entry.content }],
    }
  })

export const ChatProvider = ({ children }: { children: React.ReactNode }) => {
  const provider = useChatContextStore((state) => state.provider)
  const projectId = useChatContextStore((state) => state.projectId)
  const pageContext = useChatContextStore((state) => state.pageContext)
  const setProviderInStore = useChatContextStore((state) => state.setProvider)
  const setProjectId = useChatContextStore((state) => state.setProjectId)
  const setPageContext = useChatContextStore((state) => state.setPageContext)

  const contextSummary = useMemo(() => {
    const details: string[] = []
    if (pageContext) {
      details.push(pageContext)
    }
    if (projectId !== undefined) {
      details.push(`Project ID: ${projectId}`)
    }
    return details.length > 0 ? details.join(' | ') : null
  }, [pageContext, projectId])

  const chat = useChatSession({
    projectId,
    provider,
    context: contextSummary,
  })

  const [runtimeMessages, setRuntimeMessages] = useState<readonly ThreadMessageLike[]>(() =>
    convertMessages(chat.messages),
  )

  useEffect(() => {
    setRuntimeMessages(convertMessages(chat.messages))
  }, [chat.messages])

  const runtime = useExternalStoreRuntime<ThreadMessageLike>({
    messages: runtimeMessages,
    setMessages: setRuntimeMessages,
    convertMessage: (message) => message,
    onNew: async (append) => {
      const textPart = append.content.find((part) => part.type === 'text')
      if (textPart?.text) {
        await chat.sendMessage(textPart.text)
      }
    },
  })

  const handleProviderChange = useCallback(
    (nextProvider: ChatProviderOption) => {
      if (nextProvider !== provider) {
        resetProviderSession(nextProvider)
      }
      setProviderInStore(nextProvider)
    },
    [provider, setProviderInStore],
  )

  const value = useMemo<ChatContextValue>(
    () => ({
      ...chat,
      provider,
      setProvider: handleProviderChange,
      projectId,
      setProjectId,
      pageContext,
      setPageContext,
      runtime,
    }),
    [
      chat,
      provider,
      handleProviderChange,
      projectId,
      setProjectId,
      pageContext,
      setPageContext,
      runtime,
    ],
  )

  return (
    <ChatContext.Provider value={value}>
      {children}
      <AssistantRuntimeProvider runtime={runtime}>
        <ChatModal />
      </AssistantRuntimeProvider>
    </ChatContext.Provider>
  )
}

export const useChat = () => {
  const context = useContext(ChatContext)
  if (!context) {
    throw new Error('useChat must be used within ChatProvider')
  }
  return context
}
