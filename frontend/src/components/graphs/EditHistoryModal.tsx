import React, { useState } from 'react'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  Modal,
  Text,
  Group,
  Badge,
  Stack,
  ScrollArea,
  Button,
  Alert,
  Timeline,
  Code,
  ActionIcon,
  Tooltip,
  Box,
  Loader
} from '@mantine/core'
import {
  IconHistory,
  IconPlayerPlay,
  IconTrash,
  IconX,
  IconAlertCircle,
  IconClock
} from '@tabler/icons-react'
import {
  GET_GRAPH_EDITS,
  REPLAY_GRAPH_EDITS,
  CLEAR_GRAPH_EDITS,
  GraphEdit
} from '../../graphql/graphs'

interface EditHistoryModalProps {
  opened: boolean
  onClose: () => void
  graphId: number
  graphName: string
}

const EditHistoryModal: React.FC<EditHistoryModalProps> = ({
  opened,
  onClose,
  graphId,
  graphName
}) => {
  const [showAppliedEdits, setShowAppliedEdits] = useState(false)

  const { data, loading, error, refetch } = useQuery(GET_GRAPH_EDITS, {
    variables: {
      graphId,
      unappliedOnly: !showAppliedEdits
    },
    skip: !opened
  })

  const [replayEdits, { loading: replayLoading }] = useMutation(REPLAY_GRAPH_EDITS, {
    onCompleted: (data: any) => {
      const summary = data.replayGraphEdits
      alert(`Replay Complete: Applied ${summary.applied}, Skipped ${summary.skipped}, Failed ${summary.failed}`)
      refetch()
    },
    onError: (error: any) => {
      alert(`Replay Failed: ${error.message}`)
    }
  })

  const [clearEdits, { loading: clearLoading }] = useMutation(CLEAR_GRAPH_EDITS, {
    onCompleted: () => {
      alert('Edits Cleared: All edit history has been removed')
      refetch()
      onClose()
    },
    onError: (error: any) => {
      alert(`Clear Failed: ${error.message}`)
    }
  })

  const handleReplay = () => {
    if (window.confirm('This will replay all unapplied edits on the current graph data. Edits that can\'t be applied will be skipped. Continue?')) {
      replayEdits({ variables: { graphId } })
    }
  }

  const handleClear = () => {
    if (window.confirm('This will permanently delete all edit history for this graph. This action cannot be undone. Continue?')) {
      clearEdits({ variables: { graphId } })
    }
  }

  const formatValue = (value: any): string => {
    if (value === null || value === undefined) return 'null'
    if (typeof value === 'object') return JSON.stringify(value, null, 2)
    return String(value)
  }

  const getOperationColor = (operation: string): string => {
    switch (operation) {
      case 'create': return 'green'
      case 'update': return 'blue'
      case 'delete': return 'red'
      default: return 'gray'
    }
  }

  const getTargetTypeIcon = (targetType: string) => {
    switch (targetType) {
      case 'node': return '⬢'
      case 'edge': return '→'
      case 'layer': return '▦'
      default: return '•'
    }
  }

  const edits = (data as any)?.graphEdits || []

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Group gap="xs">
          <IconHistory size={20} />
          <Text fw={600}>Edit History: {graphName}</Text>
        </Group>
      }
      size="xl"
    >
      <Stack gap="md">
        {/* Controls */}
        <Group justify="space-between">
          <Group gap="xs">
            <Button
              size="xs"
              variant={showAppliedEdits ? 'light' : 'filled'}
              onClick={() => setShowAppliedEdits(!showAppliedEdits)}
            >
              {showAppliedEdits ? 'Show Unapplied Only' : 'Show All Edits'}
            </Button>
            <Badge color="gray" variant="light">
              {edits.length} {edits.length === 1 ? 'edit' : 'edits'}
            </Badge>
          </Group>

          <Group gap="xs">
            <Tooltip label="Replay unapplied edits">
              <ActionIcon
                variant="light"
                color="blue"
                onClick={handleReplay}
                loading={replayLoading}
                disabled={edits.filter((e: GraphEdit) => !e.applied).length === 0}
              >
                <IconPlayerPlay size={16} />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="Clear all edits">
              <ActionIcon
                variant="light"
                color="red"
                onClick={handleClear}
                loading={clearLoading}
                disabled={edits.length === 0}
              >
                <IconTrash size={16} />
              </ActionIcon>
            </Tooltip>
          </Group>
        </Group>

        {/* Loading state */}
        {loading && (
          <Group justify="center" py="xl">
            <Loader size="sm" />
            <Text size="sm" c="dimmed">Loading edit history...</Text>
          </Group>
        )}

        {/* Error state */}
        {error && (
          <Alert color="red" icon={<IconX />}>
            Failed to load edit history: {error.message}
          </Alert>
        )}

        {/* Empty state */}
        {!loading && !error && edits.length === 0 && (
          <Alert color="blue" icon={<IconAlertCircle />}>
            {showAppliedEdits
              ? 'No edit history found for this graph.'
              : 'No unapplied edits. All changes have been applied.'}
          </Alert>
        )}

        {/* Edit timeline */}
        {!loading && !error && edits.length > 0 && (
          <ScrollArea h={500}>
            <Timeline active={-1} bulletSize={24} lineWidth={2}>
              {edits.map((edit: GraphEdit) => (
                <Timeline.Item
                  key={edit.id}
                  bullet={getTargetTypeIcon(edit.targetType)}
                  title={
                    <Group gap="xs">
                      <Badge color={getOperationColor(edit.operation)} size="sm">
                        {edit.operation}
                      </Badge>
                      <Badge variant="light" color="gray" size="sm">
                        {edit.targetType}
                      </Badge>
                      <Text size="sm" fw={500}>
                        {edit.targetId}
                      </Text>
                      {edit.applied && (
                        <Badge color="green" size="xs" variant="dot">
                          Applied
                        </Badge>
                      )}
                    </Group>
                  }
                >
                  <Stack gap="xs" mt="xs">
                    {edit.fieldName && (
                      <Text size="xs" c="dimmed">
                        Field: <Code>{edit.fieldName}</Code>
                      </Text>
                    )}

                    {edit.operation === 'update' && (
                      <Box>
                        <Group gap="xs" mb={4}>
                          <Text size="xs" c="dimmed">Old value:</Text>
                        </Group>
                        <Code block style={{ fontSize: '11px', maxHeight: '100px', overflow: 'auto' }}>
                          {formatValue(edit.oldValue)}
                        </Code>
                        <Group gap="xs" mt={4} mb={4}>
                          <Text size="xs" c="dimmed">New value:</Text>
                        </Group>
                        <Code block style={{ fontSize: '11px', maxHeight: '100px', overflow: 'auto' }}>
                          {formatValue(edit.newValue)}
                        </Code>
                      </Box>
                    )}

                    {edit.operation === 'create' && edit.newValue && (
                      <Box>
                        <Text size="xs" c="dimmed" mb={4}>Created with:</Text>
                        <Code block style={{ fontSize: '11px', maxHeight: '150px', overflow: 'auto' }}>
                          {formatValue(edit.newValue)}
                        </Code>
                      </Box>
                    )}

                    {edit.operation === 'delete' && edit.oldValue && (
                      <Box>
                        <Text size="xs" c="dimmed" mb={4}>Deleted:</Text>
                        <Code block style={{ fontSize: '11px', maxHeight: '150px', overflow: 'auto' }}>
                          {formatValue(edit.oldValue)}
                        </Code>
                      </Box>
                    )}

                    <Group gap="xs">
                      <IconClock size={12} />
                      <Text size="xs" c="dimmed">
                        {new Date(edit.createdAt).toLocaleString()}
                      </Text>
                      <Text size="xs" c="dimmed">
                        • Sequence #{edit.sequenceNumber}
                      </Text>
                    </Group>
                  </Stack>
                </Timeline.Item>
              ))}
            </Timeline>
          </ScrollArea>
        )}
      </Stack>
    </Modal>
  )
}

export default EditHistoryModal
