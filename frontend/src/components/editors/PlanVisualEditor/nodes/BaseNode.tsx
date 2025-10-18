import {
  memo,
  ReactNode,
  useState,
  useCallback,
  useRef,
  type MouseEvent as ReactMouseEvent,
  type TouchEvent as ReactTouchEvent
} from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Box, TextInput } from '@mantine/core'
import { IconSettings, IconTrash, IconCheck, IconX, IconArrowRight } from '@tabler/icons-react'
import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag'
import { getRequiredInputCount, canHaveMultipleOutputs, isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon } from '../../../../utils/nodeStyles'

interface BaseNodeProps extends NodeProps {
  nodeType: PlanDagNodeType
  config: NodeConfig
  metadata: NodeMetadata
  onEdit?: () => void
  onDelete?: () => void
  onLabelChange?: (newLabel: string) => void
  readonly?: boolean
  edges?: any[]
  hasValidConfig?: boolean
  children?: ReactNode
  toolButtons?: ReactNode
  labelBadges?: ReactNode
  footerContent?: ReactNode
  editableLabel?: boolean
}

export const BaseNode = memo(({
  nodeType,
  metadata,
  selected,
  onEdit,
  onDelete,
  onLabelChange,
  readonly = false,
  edges = [],
  hasValidConfig = true,
  id,
  children,
  toolButtons,
  labelBadges,
  footerContent,
  editableLabel = false,
  sourcePosition
}: BaseNodeProps) => {
  const color = getNodeColor(nodeType)
  const requiredInputs = getRequiredInputCount(nodeType)
  const canHaveOutputs = canHaveMultipleOutputs(nodeType)

  // Check if node is configured
  const isConfigured = isNodeConfigured(nodeType, id || '', edges, hasValidConfig)

  // Label editing state
  const [isEditingLabel, setIsEditingLabel] = useState(false)
  const [labelValue, setLabelValue] = useState(metadata.label || '')
  const [edgeTriggerHovered, setEdgeTriggerHovered] = useState(false)
  const [edgeDropTargetHovered, setEdgeDropTargetHovered] = useState(false)

  const handleRefs = useRef<Record<string, HTMLDivElement | null>>({})

  const registerHandleRef = useCallback(
    (id: string) => (el: HTMLDivElement | null) => {
      if (el) {
        handleRefs.current[id] = el
      } else {
        delete handleRefs.current[id]
      }
    },
    []
  )

  type ConnectionStartEvent = ReactMouseEvent<HTMLDivElement> | ReactTouchEvent<HTMLDivElement>

  const getPointerDetails = useCallback((event: ConnectionStartEvent) => {
    const nativeEvent = event.nativeEvent as MouseEvent | TouchEvent

    if ('touches' in nativeEvent && nativeEvent.touches.length > 0) {
      const touch = nativeEvent.touches[0]
      return {
        clientX: touch.clientX,
        clientY: touch.clientY,
        pointerType: 'touch' as const
      }
    }

    if ('changedTouches' in nativeEvent && nativeEvent.changedTouches.length > 0) {
      const touch = nativeEvent.changedTouches[0]
      return {
        clientX: touch.clientX,
        clientY: touch.clientY,
        pointerType: 'touch' as const
      }
    }

    return {
      clientX: (nativeEvent as MouseEvent).clientX,
      clientY: (nativeEvent as MouseEvent).clientY,
      pointerType: 'mouse' as const
    }
  }, [])

  const dispatchHandlePointerDown = useCallback(
    (handleElement: HTMLDivElement, clientX: number, clientY: number, pointerType: 'mouse' | 'touch') => {
      const commonInit = {
        bubbles: true,
        cancelable: true,
        button: 0,
        buttons: 1,
        clientX,
        clientY
      }

      if (typeof PointerEvent !== 'undefined') {
        try {
          handleElement.dispatchEvent(
            new PointerEvent('pointerdown', {
              ...commonInit,
              pointerId: Date.now(),
              pointerType
            })
          )
        } catch {
          // Safari might not support PointerEvent; fall back to MouseEvent
        }
      }

      handleElement.dispatchEvent(
        new MouseEvent('mousedown', {
          ...commonInit,
          view: window
        })
      )
    },
    []
  )

  const handleStartConnection = useCallback(
    (event: ConnectionStartEvent) => {
      if (readonly) return

      const preferredHandleId = sourcePosition === Position.Bottom ? 'output-bottom' : 'output-right'
      const handleElement =
        handleRefs.current[preferredHandleId] ??
        handleRefs.current['output-right'] ??
        handleRefs.current['output-bottom']

      if (!handleElement) {
        console.warn('[BaseNode] Unable to start connection - no output handle available for node', id)
        return
      }

      event.stopPropagation()
      event.preventDefault()

      const { clientX, clientY, pointerType } = getPointerDetails(event)
      const handleRect = handleElement.getBoundingClientRect()
      const resolvedClientX = Number.isFinite(clientX) ? clientX : handleRect.left + handleRect.width / 2
      const resolvedClientY = Number.isFinite(clientY) ? clientY : handleRect.top + handleRect.height / 2

      dispatchHandlePointerDown(handleElement, resolvedClientX, resolvedClientY, pointerType)
    },
    [readonly, sourcePosition, getPointerDetails, dispatchHandlePointerDown, id]
  )

  const handleLabelSave = () => {
    if (labelValue.trim() && labelValue !== metadata.label) {
      onLabelChange?.(labelValue.trim())
    }
    setIsEditingLabel(false)
  }

  const handleLabelCancel = () => {
    setLabelValue(metadata.label || '')
    setIsEditingLabel(false)
  }

  // Default tool buttons if not provided
  const defaultToolButtons = !readonly && (
    <>
      {editableLabel && (
        <Tooltip label="Edit label">
          <ActionIcon
            size="sm"
            variant="subtle"
            color="blue"
            data-action-icon="edit-label"
            onMouseDown={(e) => {
              e.stopPropagation()
              e.preventDefault()
              setIsEditingLabel(true)
            }}
          >
            <IconSettings size="0.8rem" />
          </ActionIcon>
        </Tooltip>
      )}
      {!editableLabel && (
        <Tooltip label="Edit node">
          <ActionIcon
            size="sm"
            variant="subtle"
            color="gray"
            data-action-icon="edit"
            onMouseDown={(e) => {
              e.stopPropagation()
              e.preventDefault()
              onEdit?.()
            }}
          >
            <IconSettings size="0.8rem" />
          </ActionIcon>
        </Tooltip>
      )}
      <Tooltip label="Delete node">
        <ActionIcon
          size="sm"
          variant="subtle"
          color="red"
          data-action-icon="delete"
          onMouseDown={(e) => {
            e.stopPropagation()
            e.preventDefault()
            onDelete?.()
          }}
        >
          <IconTrash size="0.8rem" />
        </ActionIcon>
      </Tooltip>
    </>
  )

  // Default label badges if not provided
  const defaultLabelBadges = (
    <>
      {!isConfigured && (
        <Badge variant="outline" size="xs" color="orange">
          Not Configured
        </Badge>
      )}
    </>
  )

  return (
    <>
      {/* Input Handles - Hidden for floating edges but functional */}
      {requiredInputs > 0 && (
        <>
          <Handle
            type="target"
            position={Position.Left}
            id="input-left"
            isConnectable={!readonly}
            style={{
              opacity: 0,
              pointerEvents: 'auto',
              width: 36,
              height: 36,
              transform: 'translate(-18px, -50%)',
            }}
          />
          <Handle
            type="target"
            position={Position.Top}
            id="input-top"
            isConnectable={!readonly}
            style={{
              opacity: 0,
              pointerEvents: 'auto',
              width: 36,
              height: 36,
              transform: 'translate(-50%, -18px)',
            }}
          />
        </>
      )}

      {/* Node Content */}
      <Paper
        shadow={selected ? "md" : "sm"}
        p={0}
        style={{
          border: `2px solid ${color}`,
          borderRadius: 8,
          minWidth: 200,
          maxWidth: 280,
          background: '#fff',
          cursor: 'default',
          pointerEvents: 'all',
          display: 'flex',
          flexDirection: 'column',
          position: 'relative',
        }}
      >
        {requiredInputs > 0 && !readonly && (
          <Tooltip label="Edge drop target" position="right">
            <div
              style={{
                position: 'absolute',
                top: '50%',
                left: -14,
                width: 32,
                height: 32,
                transform: 'translateY(-50%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: '50%',
                background: edgeDropTargetHovered ? color : '#f8f9fa',
                color: edgeDropTargetHovered ? '#fff' : color,
                border: `1px solid ${color}`,
                boxShadow: '0 1px 2px rgba(0,0,0,0.05)',
              }}
              onMouseEnter={() => setEdgeDropTargetHovered(true)}
              onMouseLeave={() => setEdgeDropTargetHovered(false)}
              className="nodrag"
            >
              <IconArrowRight size={16} style={{ pointerEvents: 'none' }} />
            </div>
          </Tooltip>
        )}
        {canHaveOutputs && !readonly && (
          <Tooltip label="Create edge" position="left">
            <div
              style={{
                position: 'absolute',
                top: '50%',
                right: -14,
                width: 32,
                height: 32,
                transform: 'translateY(-50%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                cursor: 'pointer',
                borderRadius: '50%',
                background: edgeTriggerHovered ? color : '#f8f9fa',
                color: edgeTriggerHovered ? '#fff' : color,
                border: `1px solid ${color}`,
                boxShadow: '0 1px 2px rgba(0,0,0,0.05)',
              }}
              onMouseEnter={() => setEdgeTriggerHovered(true)}
              onMouseLeave={() => setEdgeTriggerHovered(false)}
              onMouseDown={(event) => handleStartConnection(event)}
              onTouchStart={(event) => handleStartConnection(event)}
              className="nodrag"
            >
              <IconArrowRight size={16} style={{ pointerEvents: 'none' }} />
            </div>
          </Tooltip>
        )}
        {/* Top Row: Icon and Label */}
        <Group
          gap="sm"
          p="md"
          pb="xs"
          wrap="nowrap"
          className="node-header"
          style={{ cursor: isEditingLabel ? 'default' : 'grab', flex: '0 0 auto' }}
        >
          <div style={{
            color,
            display: 'flex',
            alignItems: 'center',
            flexShrink: 0
          }}>
            {getNodeIcon(nodeType, '1.4rem')}
          </div>
          {isEditingLabel ? (
            <Group gap="xs" style={{ flex: 1, minWidth: 0 }} wrap="nowrap">
              <TextInput
                size="sm"
                value={labelValue}
                onChange={(e) => setLabelValue(e.currentTarget.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    handleLabelSave()
                  } else if (e.key === 'Escape') {
                    handleLabelCancel()
                  }
                }}
                style={{ flex: 1, minWidth: 0 }}
                autoFocus
                onMouseDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
              />
              <ActionIcon
                size="sm"
                color="green"
                variant="filled"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  handleLabelSave()
                }}
              >
                <IconCheck size="0.8rem" />
              </ActionIcon>
              <ActionIcon
                size="sm"
                color="red"
                variant="filled"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  handleLabelCancel()
                }}
              >
                <IconX size="0.8rem" />
              </ActionIcon>
            </Group>
          ) : (
            <Text size="sm" fw={600} lineClamp={2} style={{ wordBreak: 'break-word', flex: 1, minWidth: 0 }}>
              {metadata.label}
            </Text>
          )}
        </Group>

        {/* Middle: Node-specific content */}
        {children && (
          <Box px="md" pb="xs" style={{ flex: '1 1 auto' }}>
            {children}
          </Box>
        )}

        {/* Bottom Section 1: Labels (narrow horizontal section) */}
        {(labelBadges !== undefined ? labelBadges !== null : !isConfigured) && (
          <Box
            px="md"
            py="xs"
            style={{
              borderTop: `1px solid #e9ecef`,
              flex: '0 0 auto',
            }}
          >
            <Group gap="xs" wrap="wrap">
              {labelBadges ?? defaultLabelBadges}
            </Group>
          </Box>
        )}

        {/* Bottom Section 2: Tool buttons and footer content (narrow horizontal section) */}
        {(!readonly || footerContent) && (
          <Group
            gap="sm"
            px="md"
            py="xs"
            justify="space-between"
            style={{
              borderTop: `1px solid #e9ecef`,
              flex: '0 0 auto',
              pointerEvents: 'auto',
              minHeight: 36,
            }}
          >
            {footerContent && <div style={{ flex: 1, minWidth: 0 }}>{footerContent}</div>}
            {!readonly && (
              <>
                <Group gap={4} wrap="nowrap">
                  {toolButtons || defaultToolButtons}
                </Group>
                <div style={{ flex: 1 }} />
              </>
            )}
          </Group>
        )}
      </Paper>

      {/* Output Handles - Hidden for floating edges but functional */}
      {canHaveOutputs && (
        <>
          <Handle
            type="source"
            position={Position.Right}
            id="output-right"
            isConnectable={!readonly}
            ref={registerHandleRef('output-right')}
            className="nodrag"
            style={{
              opacity: 0,
              pointerEvents: 'auto',
              width: 28,
              height: 28,
              transform: 'translate(14px, -50%)',
            }}
          />
          <Handle
            type="source"
            position={Position.Bottom}
            id="output-bottom"
            isConnectable={!readonly}
            ref={registerHandleRef('output-bottom')}
            style={{
              opacity: 0,
              pointerEvents: 'auto',
            }}
          />
        </>
      )}
    </>
  )
})

BaseNode.displayName = 'BaseNode'
