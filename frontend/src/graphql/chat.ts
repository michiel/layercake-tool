import { gql } from '@apollo/client'

export type ChatProviderOption = 'Ollama' | 'OpenAi' | 'Gemini' | 'Claude'

export const CHAT_PROVIDER_OPTIONS: Array<{ value: ChatProviderOption; label: string; description?: string }> = [
  { value: 'Ollama', label: 'Ollama (Local)' },
  { value: 'OpenAi', label: 'OpenAI (GPT)' },
  { value: 'Gemini', label: 'Google Gemini' },
  { value: 'Claude', label: 'Anthropic Claude' },
]

export type ChatEventKind = 'AssistantMessage' | 'ToolInvocation'

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
  mutation StartChatSession($projectId: Int!, $provider: ChatProviderOption) {
    startChatSession(projectId: $projectId, provider: $provider) {
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
