import { useParams } from 'react-router-dom'
import { Stack, Title, Text } from '@mantine/core'
import { ChatSessionList } from '../components/chat/ChatSessionList'

export const ChatLogsPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  if (!Number.isFinite(numericProjectId)) {
    return <Text c="red">Invalid project ID</Text>
  }

  return (
    <Stack h="100%" p="md" gap="md">
      <div>
        <Title order={2}>Chat Logs</Title>
        <Text c="dimmed" size="sm">
          Browse and view your chat history. Click on a session to view its conversation.
        </Text>
      </div>

      <div style={{ flex: 1, minHeight: 0 }}>
        <ChatSessionList
          projectId={numericProjectId}
          selectedSessionId={null}
          onSelectSession={(sessionId) => {
            if (sessionId) {
              // Open the session in a read-only view (future implementation)
              console.log('View session:', sessionId)
            }
          }}
        />
      </div>
    </Stack>
  )
}
