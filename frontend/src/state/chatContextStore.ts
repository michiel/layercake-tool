import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

import type { ChatProviderOption } from '../graphql/chat'

interface ChatContextState {
  provider: ChatProviderOption
  projectId?: number
  pageContext: string | null
  setProvider: (provider: ChatProviderOption) => void
  setProjectId: (projectId?: number) => void
  setPageContext: (context: string | null) => void
  clearPageContext: () => void
}

const storage =
  typeof window !== 'undefined'
    ? createJSONStorage<ChatContextState>(() => window.localStorage)
    : undefined

export const useChatContextStore = create(
  persist<ChatContextState>(
    (set) => ({
      provider: 'Gemini',
      projectId: undefined,
      pageContext: null,
      setProvider: (provider) => set({ provider }),
      setProjectId: (projectId) => set({ projectId }),
      setPageContext: (context) => set({ pageContext: context }),
      clearPageContext: () => set({ pageContext: null }),
    }),
    {
      name: 'layercake-chat-context',
      storage,
      version: 1,
    },
  ),
)
