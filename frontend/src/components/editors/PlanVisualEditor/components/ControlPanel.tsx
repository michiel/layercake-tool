import { Stack, Text, Group, ActionIcon, Tooltip } from '@mantine/core'
import {
  IconPlayerPlay,
  IconPlayerPause,
  IconRotate,
  IconCircleCheck,
  IconExclamationCircle,
  IconRefresh,
  IconNetwork,
  IconNetworkOff,
  IconDatabase,
  IconTransform,
  IconGitMerge,
  IconCopy,
  IconFileExport
} from '@tabler/icons-react'
import { Panel } from 'reactflow'
import { PlanDagNodeType } from '../../../../types/plan-dag'

interface ControlPanelProps {
  // Validation props
  validationLoading: boolean
  validationErrors: any[]
  lastValidation: Date | null
  onValidate: () => void

  // Update management props
  updatesPaused: boolean
  pendingUpdates: number
  onPauseUpdates: () => void
  onResumeUpdates: () => void

  // Collaboration props
  isConnected: boolean
  collaborationStatus: string
  hasError: boolean
  onlineUsers: any[]

  // Node drag props
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void
  readonly?: boolean
}

export const ControlPanel = ({
  validationLoading,
  validationErrors,
  lastValidation,
  onValidate,
  updatesPaused,
  pendingUpdates,
  onPauseUpdates,
  onResumeUpdates,
  isConnected,
  collaborationStatus,
  hasError,
  onlineUsers,
  onNodeDragStart,
  readonly = false
}: ControlPanelProps) => {
  const nodeTypes = [
    {
      type: PlanDagNodeType.DATA_SOURCE,
      label: 'Data Source',
      icon: <IconDatabase size="0.7rem" />,
      color: '#51cf66'
    },
    {
      type: PlanDagNodeType.GRAPH,
      label: 'Graph',
      icon: <IconNetwork size="0.7rem" />,
      color: '#339af0'
    },
    {
      type: PlanDagNodeType.TRANSFORM,
      label: 'Transform',
      icon: <IconTransform size="0.7rem" />,
      color: '#ff8cc8'
    },
    {
      type: PlanDagNodeType.MERGE,
      label: 'Merge',
      icon: <IconGitMerge size="0.7rem" />,
      color: '#ffd43b'
    },
    {
      type: PlanDagNodeType.COPY,
      label: 'Copy',
      icon: <IconCopy size="0.7rem" />,
      color: '#74c0fc'
    },
    {
      type: PlanDagNodeType.OUTPUT,
      label: 'Output',
      icon: <IconFileExport size="0.7rem" />,
      color: '#ff6b6b'
    }
  ];

  const handleNodeDragStart = (event: React.DragEvent, nodeType: PlanDagNodeType) => {
    if (!readonly) {
      onNodeDragStart(event, nodeType);
    }
  };

  return (
    <Panel position="top-left">
      <Stack gap="xs" style={{ backgroundColor: 'rgba(255, 255, 255, 0.95)', padding: '12px', borderRadius: '8px', boxShadow: '0 2px 8px rgba(0,0,0,0.1)' }}>
        {/* Node Creation */}
        {!readonly && (
          <Group gap="md">
            <Text size="xs" fw={500} c="gray.6">Nodes:</Text>
            <Group gap={4}>
              {nodeTypes.map((nodeType) => (
                <Tooltip key={nodeType.type} label={`Drag to add ${nodeType.label}`}>
                  <ActionIcon
                    size="xs"
                    variant="light"
                    style={{ backgroundColor: nodeType.color, color: 'white', cursor: 'grab' }}
                    draggable
                    onDragStart={(event) => handleNodeDragStart(event, nodeType.type)}
                  >
                    {nodeType.icon}
                  </ActionIcon>
                </Tooltip>
              ))}
            </Group>
          </Group>
        )}

        <Group gap="md">
          <Text size="xs" fw={500} c="gray.6">Controls:</Text>
          <Group gap={4}>
            {/* Update Control Buttons */}
            <Tooltip label={updatesPaused ? `Resume Updates (${pendingUpdates} pending)` : 'Pause Updates'}>
              <ActionIcon
                size="xs"
                variant="light"
                color={updatesPaused ? "orange" : "blue"}
                onClick={updatesPaused ? onResumeUpdates : onPauseUpdates}
              >
                {updatesPaused ? <IconPlayerPlay size="0.7rem" /> : <IconPlayerPause size="0.7rem" />}
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Trigger Validation">
              <ActionIcon size="xs" variant="light" color="gray" onClick={onValidate}>
                <IconRotate size="0.7rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        </Group>

        <Group gap="md">
          <Text size="xs" fw={500} c="gray.6">Status:</Text>
          <Group gap={4}>
            {/* Collaboration Status */}
            <Tooltip label={`Collaboration: ${collaborationStatus}${hasError ? ' (Error)' : ''}`}>
              <ActionIcon size="xs" variant="light" color={hasError ? "red" : isConnected ? "green" : "gray"}>
                {isConnected ? <IconNetwork size="0.7rem" /> : <IconNetworkOff size="0.7rem" />}
              </ActionIcon>
            </Tooltip>

            <Text size="xs" c="dimmed">{onlineUsers.length} online</Text>
          </Group>
        </Group>

        <Group gap="md">
          <Text size="xs" fw={500} c="gray.6">Validation:</Text>
          <Group gap={4}>
            {validationLoading && (
              <Tooltip label="Validating Plan DAG...">
                <ActionIcon size="xs" variant="light" color="blue">
                  <IconRefresh size="0.7rem" />
                </ActionIcon>
              </Tooltip>
            )}

            {validationErrors.length > 0 && !validationLoading && (
              <Tooltip label={`${validationErrors.length} validation error${validationErrors.length > 1 ? 's' : ''}`}>
                <ActionIcon size="xs" variant="light" color="red">
                  <IconExclamationCircle size="0.7rem" />
                </ActionIcon>
              </Tooltip>
            )}

            {validationErrors.length === 0 && !validationLoading && lastValidation && (
              <Tooltip label={`Validation passed (${lastValidation.toLocaleTimeString()})`}>
                <ActionIcon size="xs" variant="light" color="green">
                  <IconCircleCheck size="0.7rem" />
                </ActionIcon>
              </Tooltip>
            )}
          </Group>
        </Group>
      </Stack>
    </Panel>
  )
}