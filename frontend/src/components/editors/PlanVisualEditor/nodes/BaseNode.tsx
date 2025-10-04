import { memo } from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Stack } from '@mantine/core'
import { IconSettings, IconTrash } from '@tabler/icons-react'
import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag'
import { getRequiredInputCount, canHaveMultipleOutputs, isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon, getNodeTypeLabel } from '../../../../utils/nodeStyles'

interface BaseNodeProps extends NodeProps {
  nodeType: PlanDagNodeType
  config: NodeConfig
  metadata: NodeMetadata
  onEdit?: () => void
  onDelete?: () => void
  readonly?: boolean
  edges?: any[]
  hasValidConfig?: boolean
  children?: React.ReactNode
}

export const BaseNode = memo(({
  nodeType,
  metadata,
  selected,
  onEdit,
  onDelete,
  readonly = false,
  edges = [],
  hasValidConfig = true,
  id,
  children
}: BaseNodeProps) => {
  const color = getNodeColor(nodeType)
  const requiredInputs = getRequiredInputCount(nodeType)
  const canHaveOutputs = canHaveMultipleOutputs(nodeType)

  // Check if node is configured
  const isConfigured = isNodeConfigured(nodeType, id || '', edges, hasValidConfig)

  return (
    <>
      {/* Input Handles - Left and Top sides */}
      {requiredInputs > 0 && (
        <>
          <Handle
            type="target"
            position={Position.Left}
            id="input-left"
            isConnectable={!readonly}
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '50%',
            }}
          />
          <Handle
            type="target"
            position={Position.Top}
            id="input-top"
            isConnectable={!readonly}
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '50%',
            }}
          />
        </>
      )}

      {/* Node Content */}
      <Paper
        shadow={selected ? "md" : "sm"}
        p="md"
        style={{
          border: `2px solid ${color}`,
          borderRadius: 8,
          minWidth: 200,
          maxWidth: 280,
          background: '#fff',
          cursor: 'default',
          pointerEvents: 'all',
        }}
      >
        {/* Top right: Edit and Delete icons */}
        {!readonly && (
          <Group gap={4} style={{ position: 'absolute', top: 8, right: 8, pointerEvents: 'auto', zIndex: 10 }}>
            <Tooltip label="Edit node">
              <ActionIcon
                size="sm"
                variant="subtle"
                color="gray"
                data-action-icon="edit"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  console.log('Edit icon mousedown, calling onEdit')
                  onEdit?.()
                }}
              >
                <IconSettings size="0.8rem" />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="Delete node">
              <ActionIcon
                size="sm"
                variant="subtle"
                color="red"
                data-action-icon="delete"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  console.log('Delete icon mousedown, calling onDelete')
                  onDelete?.()
                }}
              >
                <IconTrash size="0.8rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        )}

        {/* Middle: Icon and Label */}
        <Group gap="sm" mb="sm" wrap="nowrap" style={{ paddingRight: !readonly ? 60 : 0 }}>
          <div style={{
            color,
            display: 'flex',
            alignItems: 'center',
            flexShrink: 0
          }}>
            {getNodeIcon(nodeType, '1.4rem')}
          </div>
          <Text size="sm" fw={600} lineClamp={2} style={{ wordBreak: 'break-word', flex: 1, minWidth: 0 }}>
            {metadata.label}
          </Text>
        </Group>

        {/* Bottom: Labels and node-specific content */}
        <Stack gap="xs">
          <Group gap="xs" wrap="wrap">
            <Badge
              variant="light"
              color={color}
              size="xs"
              style={{ textTransform: 'none' }}
            >
              {getNodeTypeLabel(nodeType)}
            </Badge>
            {!isConfigured && (
              <Badge variant="outline" size="xs" color="orange">
                Not Configured
              </Badge>
            )}
          </Group>

          {/* Node-specific content */}
          {children}
        </Stack>
      </Paper>

      {/* Output Handles - Right and Bottom sides */}
      {canHaveOutputs && (
        <>
          <Handle
            type="source"
            position={Position.Right}
            id="output-right"
            isConnectable={!readonly}
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '3px',
            }}
          />
          <Handle
            type="source"
            position={Position.Bottom}
            id="output-bottom"
            isConnectable={!readonly}
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '3px',
            }}
          />
        </>
      )}
    </>
  )
})

BaseNode.displayName = 'BaseNode'
