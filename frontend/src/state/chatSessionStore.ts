import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'
import { shallow } from 'zustand/shallow'

import type { StartChatSessionPayload, ChatProviderOption } from '../graphql/chat'
import type { ChatMessageEntry } from '../types/chat'

interface ProviderSession {
  session?: StartChatSessionPayload
  messages: ChatMessageEntry[]
  isAwaitingAssistant: boolean
  projectId?: number
  lastUpdatedAt?: string
}

interface ChatSessionState {
  sessions: Record<ChatProviderOption, ProviderSession>
  updateProvider: (
    provider: ChatProviderOption,
    updater: (prev: ProviderSession) => ProviderSession,
  ) => void
  resetProvider: (provider: ChatProviderOption) => void
}

const DEFAULT_SESSION: ProviderSession = {
  session: undefined,
  messages: [],
  isAwaitingAssistant: false,
  projectId: undefined,
  lastUpdatedAt: undefined,
}

const storage =
  typeof window !== 'undefined'
    ? createJSONStorage<ChatSessionState>(() => window.localStorage)
    : undefined

export const useChatSessionStore = create(
  persist<ChatSessionState>(
    (set) => ({
      sessions: {} as Record<ChatProviderOption, ProviderSession>,
      updateProvider: (provider, updater) =>
        set((state) => {
          const previous = state.sessions[provider] ?? DEFAULT_SESSION
          return {
            sessions: {
              ...state.sessions,
              [provider]: updater(previous),
            },
          }
        }),
      resetProvider: (provider) =>
        set((state) => {
          const nextSessions = { ...state.sessions }
          delete nextSessions[provider]
          return { sessions: nextSessions }
        }),
    }),
    {
      name: 'layercake-chat-sessions',
      storage,
      version: 1,
    },
  ),
)

export const useProviderSession = (provider: ChatProviderOption) =>
  useChatSessionStore(
    (state) => state.sessions[provider] ?? DEFAULT_SESSION,
    shallow,
  )

export const appendMessageToProvider = (
  provider: ChatProviderOption,
  message: ChatMessageEntry,
) =>
  useChatSessionStore.getState().updateProvider(provider, (prev) => ({
    ...prev,
    messages: [...prev.messages, message],
    lastUpdatedAt: new Date().toISOString(),
  }))

export const setMessagesForProvider = (
  provider: ChatProviderOption,
  messages: ChatMessageEntry[],
) =>
  useChatSessionStore.getState().updateProvider(provider, (prev) => ({
    ...prev,
    messages,
    lastUpdatedAt: new Date().toISOString(),
  }))

export const setSessionForProvider = (
  provider: ChatProviderOption,
  session: StartChatSessionPayload | undefined,
  projectId?: number,
) =>
  useChatSessionStore.getState().updateProvider(provider, (prev) => ({
    ...prev,
    session,
    projectId: projectId ?? prev.projectId,
    lastUpdatedAt: new Date().toISOString(),
  }))

export const setAwaitingForProvider = (
  provider: ChatProviderOption,
  isAwaiting: boolean,
) =>
  useChatSessionStore.getState().updateProvider(provider, (prev) => ({
    ...prev,
    isAwaitingAssistant: isAwaiting,
  }))

export const resetProviderSession = (provider: ChatProviderOption) =>
  useChatSessionStore.getState().resetProvider(provider)
