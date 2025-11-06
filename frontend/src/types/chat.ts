export type ChatMessageRole = 'user' | 'assistant' | 'tool'

export interface ChatMessageEntry {
  id: string
  role: ChatMessageRole
  content: string
  toolName?: string
  createdAt: string
}
