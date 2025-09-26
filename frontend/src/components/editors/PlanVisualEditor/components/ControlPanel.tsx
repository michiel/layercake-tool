import { Stack, Text, Group, ActionIcon, Tooltip } from '@mantine/core'
import {
  IconPlayerPlay,
  IconPlayerPause,
  IconRotate,
  IconCircleCheck,
  IconExclamationCircle,
  IconRefresh,
  IconNetwork,
  IconNetworkOff
} from '@tabler/icons-react'
import { Panel } from 'reactflow'

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
  onlineUsers
}: ControlPanelProps) => {
  return (
    <Panel position="top-left">
      <Stack gap="xs" style={{ backgroundColor: 'rgba(255, 255, 255, 0.95)', padding: '12px', borderRadius: '8px', boxShadow: '0 2px 8px rgba(0,0,0,0.1)' }}>
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