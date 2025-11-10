import { gql } from '@apollo/client'

export type ChatProviderOption = 'Ollama' | 'OpenAi' | 'Gemini' | 'Claude'

export const CHAT_PROVIDER_OPTIONS: Array<{ value: ChatProviderOption; label: string; description?: string }> = [
  { value: 'Ollama', label: 'Ollama (Local)' },
  { value: 'OpenAi', label: 'OpenAI (GPT)' },
  { value: 'Gemini', label: 'Google Gemini' },
  { value: 'Claude', label: 'Anthropic Claude' },
]

// Note: Backend sends SCREAMING_SNAKE_CASE but type is defined as PascalCase
export type ChatEventKind = 'AssistantMessage' | 'ToolInvocation' | 'ASSISTANT_MESSAGE' | 'TOOL_INVOCATION'

export interface StartChatSessionPayload {
  sessionId: string
  provider: ChatProviderOption
  model: string
}

export interface ChatEventPayload {
  kind: ChatEventKind
  message: string
  toolName?: string | null
}

export const START_CHAT_SESSION = gql`
  mutation StartChatSession($projectId: Int!, $provider: ChatProviderOption, $sessionId: String) {
    startChatSession(projectId: $projectId, provider: $provider, sessionId: $sessionId) {
      sessionId
      provider
      model
    }
  }
`

export const SEND_CHAT_MESSAGE = gql`
  mutation SendChatMessage($sessionId: String!, $message: String!) {
    sendChatMessage(sessionId: $sessionId, message: $message) {
      accepted
    }
  }
`

export const CHAT_EVENTS_SUBSCRIPTION = gql`
  subscription OnChatEvents($sessionId: String!) {
    chatEvents(sessionId: $sessionId) {
      kind
      message
      toolName
    }
  }
`

// Chat History Types
export interface ChatSession {
  id: number
  session_id: string
  project_id: number
  user_id: number
  title: string | null
  provider: string
  model_name: string
  is_archived: boolean
  created_at: string
  updated_at: string
  last_activity_at: string
  // RAG configuration
  enable_rag: boolean
  rag_top_k: number
  rag_threshold: number
  include_citations: boolean
}

export interface ChatMessage {
  id: number
  session_id: number
  message_id: string
  role: string
  content: string
  tool_name: string | null
  tool_call_id: string | null
  created_at: string
}

// Chat History Queries
export const GET_CHAT_SESSIONS = gql`
  query GetChatSessions($projectId: Int!, $includeArchived: Boolean = false, $limit: Int = 50, $offset: Int = 0) {
    chatSessions(projectId: $projectId, includeArchived: $includeArchived, limit: $limit, offset: $offset) {
      id
      session_id
      project_id
      user_id
      title
      provider
      model_name
      is_archived
      created_at
      updated_at
      last_activity_at
      enable_rag
      rag_top_k
      rag_threshold
      include_citations
    }
  }
`

export const GET_CHAT_HISTORY = gql`
  query GetChatHistory($sessionId: String!, $limit: Int = 100, $offset: Int = 0) {
    chatHistory(sessionId: $sessionId, limit: $limit, offset: $offset) {
      id
      session_id
      message_id
      role
      content
      tool_name
      tool_call_id
      created_at
    }
  }
`

export const GET_CHAT_MESSAGE_COUNT = gql`
  query GetChatMessageCount($sessionId: String!) {
    chatMessageCount(sessionId: $sessionId)
  }
`

// Chat History Mutations
export const UPDATE_CHAT_SESSION_TITLE = gql`
  mutation UpdateChatSessionTitle($sessionId: String!, $title: String!) {
    updateChatSessionTitle(sessionId: $sessionId, title: $title)
  }
`

export const ARCHIVE_CHAT_SESSION = gql`
  mutation ArchiveChatSession($sessionId: String!) {
    archiveChatSession(sessionId: $sessionId)
  }
`

export const UNARCHIVE_CHAT_SESSION = gql`
  mutation UnarchiveChatSession($sessionId: String!) {
    unarchiveChatSession(sessionId: $sessionId)
  }
`

export const DELETE_CHAT_SESSION = gql`
  mutation DeleteChatSession($sessionId: String!) {
    deleteChatSession(sessionId: $sessionId)
  }
`
