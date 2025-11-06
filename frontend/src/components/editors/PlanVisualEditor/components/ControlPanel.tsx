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
import { Stack, Group } from '../../../layout-primitives'
import { Button } from '../../../ui/button'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '../../../ui/tooltip'

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
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  onNodePointerDragStart: (event: React.MouseEvent, nodeType: PlanDagNodeType) => void;
  readonly?: boolean;
}

const isTauri = !!(window as any).__TAURI__;

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
  onNodePointerDragStart,
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

  const handleNodePointerDragStart = (event: React.MouseEvent, nodeType: PlanDagNodeType) => {
    if (!readonly) {
      onNodePointerDragStart(event, nodeType);
    }
  };

  return (
    <Panel position="top-left">
      <TooltipProvider>
        <Stack gap="xs" className="bg-white/95 p-3 rounded-lg shadow-md">
          {/* Node Creation */}
          {!readonly && (
            <Group gap="md">
              <p className="text-xs font-medium text-gray-600">Nodes:</p>
              <Group gap="xs">
                {nodeTypes.map((nodeType) => (
                  <Tooltip key={nodeType.type}>
                    <TooltipTrigger asChild>
                      <Button
                        size="icon"
                        variant="secondary"
                        className="h-6 w-6 cursor-grab text-white hover:opacity-80"
                        style={{ backgroundColor: nodeType.color }}
                        draggable={!isTauri}
                        onDragStart={(event) => handleNodeDragStart(event, nodeType.type)}
                        onMouseDown={(event) => handleNodePointerDragStart(event, nodeType.type)}
                      >
                        {nodeType.icon}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Drag to add {nodeType.label}
                    </TooltipContent>
                  </Tooltip>
                ))}
              </Group>
            </Group>
          )}

          <Group gap="md">
            <p className="text-xs font-medium text-gray-600">Controls:</p>
            <Group gap="xs">
              {/* Update Control Buttons */}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="secondary"
                    className={`h-6 w-6 ${updatesPaused ? 'text-orange-600' : 'text-blue-600'}`}
                    onClick={updatesPaused ? onResumeUpdates : onPauseUpdates}
                  >
                    {updatesPaused ? <IconPlayerPlay size="0.7rem" /> : <IconPlayerPause size="0.7rem" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {updatesPaused ? `Resume Updates (${pendingUpdates} pending)` : 'Pause Updates'}
                </TooltipContent>
              </Tooltip>

              <Tooltip>
                <TooltipTrigger asChild>
                  <Button size="icon" variant="secondary" className="h-6 w-6" onClick={onValidate}>
                    <IconRotate size="0.7rem" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Trigger Validation</TooltipContent>
              </Tooltip>
            </Group>
          </Group>

          <Group gap="md">
            <p className="text-xs font-medium text-gray-600">Status:</p>
            <Group gap="xs">
              {/* Collaboration Status */}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="secondary"
                    className={`h-6 w-6 ${hasError ? 'text-red-600' : isConnected ? 'text-green-600' : 'text-gray-600'}`}
                  >
                    {isConnected ? <IconNetwork size="0.7rem" /> : <IconNetworkOff size="0.7rem" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  Collaboration: {collaborationStatus}{hasError ? ' (Error)' : ''}
                </TooltipContent>
              </Tooltip>

              <p className="text-xs text-muted-foreground">{onlineUsers.length} online</p>
            </Group>
          </Group>

          <Group gap="md">
            <p className="text-xs font-medium text-gray-600">Validation:</p>
            <Group gap="xs">
              {validationLoading && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button size="icon" variant="secondary" className="h-6 w-6 text-blue-600">
                      <IconRefresh size="0.7rem" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Validating Plan DAG...</TooltipContent>
                </Tooltip>
              )}

              {validationErrors.length > 0 && !validationLoading && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button size="icon" variant="secondary" className="h-6 w-6 text-red-600">
                      <IconExclamationCircle size="0.7rem" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    {validationErrors.length} validation error{validationErrors.length > 1 ? 's' : ''}
                  </TooltipContent>
                </Tooltip>
              )}

              {validationErrors.length === 0 && !validationLoading && lastValidation && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button size="icon" variant="secondary" className="h-6 w-6 text-green-600">
                      <IconCircleCheck size="0.7rem" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    Validation passed ({lastValidation.toLocaleTimeString()})
                  </TooltipContent>
                </Tooltip>
              )}
            </Group>
          </Group>
        </Stack>
      </TooltipProvider>
    </Panel>
  )
}