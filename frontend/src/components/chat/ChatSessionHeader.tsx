import { useMutation } from '@apollo/client/react'
import { ActionIcon, Group, Menu, Modal, Text, TextInput, Tooltip } from '@mantine/core'
import { IconArchive, IconDotsVertical, IconEdit, IconPlus, IconTrash } from '@tabler/icons-react'
import { useState } from 'react'
import { ARCHIVE_CHAT_SESSION, ChatSession, DELETE_CHAT_SESSION, UPDATE_CHAT_SESSION_TITLE } from '../../graphql/chat'

interface ChatSessionHeaderProps {
  session: ChatSession | null
  onNewSession: () => void
  onSessionDeleted: () => void
  onSessionArchived: () => void
  onTitleUpdated: () => void
}

export const ChatSessionHeader = ({
  session,
  onNewSession,
  onSessionDeleted,
  onSessionArchived,
  onTitleUpdated,
}: ChatSessionHeaderProps) => {
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [editTitle, setEditTitle] = useState('')

  const [updateTitle] = useMutation(UPDATE_CHAT_SESSION_TITLE, {
    onCompleted: () => {
      setEditModalOpen(false)
      onTitleUpdated()
    },
  })

  const [archiveSession] = useMutation(ARCHIVE_CHAT_SESSION, {
    onCompleted: () => onSessionArchived(),
  })

  const [deleteSession] = useMutation(DELETE_CHAT_SESSION, {
    onCompleted: () => onSessionDeleted(),
  })

  const handleEditClick = () => {
    if (session) {
      setEditTitle(session.title || '')
      setEditModalOpen(true)
    }
  }

  const handleSaveTitle = async () => {
    if (session && editTitle.trim()) {
      try {
        await updateTitle({
          variables: {
            sessionId: session.session_id,
            title: editTitle.trim(),
          },
        })
      } catch (err) {
        console.error('Failed to update session title:', err)
      }
    }
  }

  const handleArchive = async () => {
    if (session) {
      try {
        await archiveSession({ variables: { sessionId: session.session_id } })
      } catch (err) {
        console.error('Failed to archive session:', err)
      }
    }
  }

  const handleDelete = async () => {
    if (session && window.confirm('Are you sure you want to delete this chat session? This action cannot be undone.')) {
      try {
        await deleteSession({ variables: { sessionId: session.session_id } })
      } catch (err) {
        console.error('Failed to delete session:', err)
      }
    }
  }

  return (
    <>
      <Group justify="space-between" p="md" style={{ borderBottom: '1px solid #e9ecef' }}>
        <Group gap="xs">
          {session ? (
            <>
              <Text size="lg" fw={600}>
                {session.title || 'Untitled Chat'}
              </Text>
              <Text size="sm" c="dimmed">
                {session.provider} - {session.model_name}
              </Text>
            </>
          ) : (
            <Text size="lg" fw={600} c="dimmed">
              Select a session or start a new chat
            </Text>
          )}
        </Group>

        <Group gap="xs">
          <Tooltip label="New Session">
            <ActionIcon size="lg" variant="subtle" onClick={onNewSession}>
              <IconPlus size={20} />
            </ActionIcon>
          </Tooltip>

          {session && (
            <>
              <Tooltip label="Edit Title">
                <ActionIcon size="lg" variant="subtle" onClick={handleEditClick}>
                  <IconEdit size={20} />
                </ActionIcon>
              </Tooltip>

              <Menu position="bottom-end" withinPortal>
                <Menu.Target>
                  <ActionIcon size="lg" variant="subtle">
                    <IconDotsVertical size={20} />
                  </ActionIcon>
                </Menu.Target>

                <Menu.Dropdown>
                  <Menu.Item leftSection={<IconArchive size={16} />} onClick={handleArchive}>
                    Archive
                  </Menu.Item>
                  <Menu.Item leftSection={<IconTrash size={16} />} color="red" onClick={handleDelete}>
                    Delete
                  </Menu.Item>
                </Menu.Dropdown>
              </Menu>
            </>
          )}
        </Group>
      </Group>

      <Modal opened={editModalOpen} onClose={() => setEditModalOpen(false)} title="Edit Session Title" centered>
        <TextInput
          label="Session Title"
          placeholder="Enter a title for this session"
          value={editTitle}
          onChange={(e) => setEditTitle(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              handleSaveTitle()
            }
          }}
          data-autofocus
        />
        <Group justify="flex-end" mt="md">
          <ActionIcon variant="subtle" onClick={() => setEditModalOpen(false)}>
            Cancel
          </ActionIcon>
          <ActionIcon variant="filled" onClick={handleSaveTitle}>
            Save
          </ActionIcon>
        </Group>
      </Modal>
    </>
  )
}
