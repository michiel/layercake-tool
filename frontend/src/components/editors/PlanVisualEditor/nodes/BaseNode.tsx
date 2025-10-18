import { memo, ReactNode, useState } from 'react'
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
  editableLabel = false
}: BaseNodeProps) => {
  const color = getNodeColor(nodeType)
  const requiredInputs = getRequiredInputCount(nodeType)
  const canHaveOutputs = canHaveMultipleOutputs(nodeType)

  // Check if node is configured
  const isConfigured = isNodeConfigured(nodeType, id || '', edges, hasValidConfig)

  // Label editing state
  const [isEditingLabel, setIsEditingLabel] = useState(false)
  const [labelValue, setLabelValue] = useState(metadata.label || '')

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
              pointerEvents: 'none',
            }}
          />
          <Handle
            type="target"
            position={Position.Top}
            id="input-top"
            isConnectable={!readonly}
            style={{
              opacity: 0,
              pointerEvents: 'none',
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
        {/* Edge Creation Icon - Top Right */}
        {canHaveOutputs && !readonly && (
          <Tooltip label="Create edge" position="top">
            <div
              style={{
                position: 'absolute',
                top: 8,
                right: 8,
                zIndex: 10,
                cursor: 'pointer',
                color: color,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                width: 24,
                height: 24,
                borderRadius: '50%',
                background: '#f8f9fa',
                border: `1px solid ${color}`,
                transition: 'all 0.2s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = color;
                e.currentTarget.style.color = '#fff';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = '#f8f9fa';
                e.currentTarget.style.color = color;
              }}
              onMouseDown={(e) => {
                e.stopPropagation();
                // This will trigger ReactFlow's connection mode
                // The actual connection logic is handled by ReactFlow's onConnectStart
              }}
              className="nodrag"
            >
              <IconArrowRight size={14} />
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
              <Group gap={4}>
                {toolButtons || defaultToolButtons}
              </Group>
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
            style={{
              opacity: 0,
              pointerEvents: 'none',
            }}
          />
          <Handle
            type="source"
            position={Position.Bottom}
            id="output-bottom"
            isConnectable={!readonly}
            style={{
              opacity: 0,
              pointerEvents: 'none',
            }}
          />
        </>
      )}
    </>
  )
})

BaseNode.displayName = 'BaseNode'
