import { memo } from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge } from '@mantine/core'
import { IconSettings, IconTrash } from '@tabler/icons-react'
import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag'
import { getNodeTypeColor, getRequiredInputCount, canHaveMultipleOutputs, isNodeConfigured } from '../../../../utils/planDagValidation'

interface BaseNodeProps extends NodeProps {
  nodeType: PlanDagNodeType
  config: NodeConfig
  metadata: NodeMetadata
  onEdit?: () => void
  onDelete?: () => void
  readonly?: boolean
  edges?: any[] // Add edges for configuration validation
  hasValidConfig?: boolean // Add configuration validation flag
}

export const BaseNode = memo(({
  nodeType,
  config,
  metadata,
  selected,
  onEdit,
  onDelete,
  readonly = false,
  edges = [],
  hasValidConfig = true,
  id
}: BaseNodeProps) => {
  const color = getNodeTypeColor(nodeType)
  const requiredInputs = getRequiredInputCount(nodeType)
  const canHaveOutputs = canHaveMultipleOutputs(nodeType)

  // Check if node is configured according to SPECIFICATION.md
  const isConfigured = isNodeConfigured(nodeType, id || '', edges, hasValidConfig)

  const getTypeLabel = (type: PlanDagNodeType): string => {
    switch (type) {
      case PlanDagNodeType.DATA_SOURCE: return 'Data Source'
      case PlanDagNodeType.GRAPH: return 'Graph'
      case PlanDagNodeType.TRANSFORM: return 'Transform'
      case PlanDagNodeType.MERGE: return 'Merge'
      case PlanDagNodeType.COPY: return 'Copy'
      case PlanDagNodeType.OUTPUT: return 'Output'
      default: return 'Unknown'
    }
  }

  return (
    <>
      {/* Input Handles - Left and Top sides */}
      {requiredInputs > 0 && (
        <>
          {/* Left Input Handle */}
          <Handle
            type="target"
            position={Position.Left}
            id="input-left"
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '50%', // Round for inputs
            }}
          />

          {/* Top Input Handle */}
          <Handle
            type="target"
            position={Position.Top}
            id="input-top"
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '50%', // Round for inputs
            }}
          />
        </>
      )}

      {/* Node Content */}
      <Paper
        shadow={selected ? "md" : "sm"}
        p="sm"
        style={{
          border: selected
            ? `2px solid ${color}`
            : !isConfigured
              ? `2px solid #fd7e14` // Orange outline for unconfigured nodes
              : `1px solid #e9ecef`,
          borderRadius: 8,
          minWidth: 180,
          maxWidth: 250,
          background: '#fff',
          cursor: 'default', // Remove pointer cursor since we don't want click-to-edit
        }}
      >
        <Group justify="space-between" mb="xs">
          <Group gap="xs">
            <Badge
              color={color}
              variant="light"
              size="sm"
            >
              {getTypeLabel(nodeType)}
            </Badge>
            {!isConfigured && (
              <Badge variant="outline" size="xs" color="orange">
                Not Configured
              </Badge>
            )}
          </Group>

          {!readonly && (
            <Group gap="xs">
              <Tooltip label="Edit node">
                <ActionIcon
                  size="sm"
                  variant="subtle"
                  color="gray"
                  onClick={(e) => {
                    e.stopPropagation()
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
                  onClick={(e) => {
                    e.stopPropagation()
                    onDelete?.()
                  }}
                >
                  <IconTrash size="0.8rem" />
                </ActionIcon>
              </Tooltip>
            </Group>
          )}
        </Group>

        <Text size="sm" fw={500} mb="xs">
          {metadata.label}
        </Text>

        {metadata.description && (
          <Text size="xs" c="dimmed" lineClamp={2}>
            {metadata.description}
          </Text>
        )}

        {/* Node-specific content */}
        <div style={{ marginTop: 8 }}>
          {renderNodeSpecificContent(nodeType, config)}
        </div>
      </Paper>

      {/* Output Handles - Right and Bottom sides */}
      {canHaveOutputs && (
        <>
          {/* Right Output Handle */}
          <Handle
            type="source"
            position={Position.Right}
            id="output-right"
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '0', // Square for outputs
            }}
          />

          {/* Bottom Output Handle */}
          <Handle
            type="source"
            position={Position.Bottom}
            id="output-bottom"
            style={{
              background: '#fff',
              border: `2px solid ${color}`,
              width: 12,
              height: 12,
              borderRadius: '0', // Square for outputs
            }}
          />
        </>
      )}
    </>
  )
})

BaseNode.displayName = 'BaseNode'

// Render node-specific configuration details
const renderNodeSpecificContent = (nodeType: PlanDagNodeType, config: NodeConfig) => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      const dataSourceConfig = config as any
      return (
        <Text size="xs" c="dimmed">
          {dataSourceConfig.dataType}: {dataSourceConfig.source}
        </Text>
      )

    case PlanDagNodeType.GRAPH:
      const graphConfig = config as any
      return (
        <Text size="xs" c="dimmed">
          Graph ID: {graphConfig.graphId}
        </Text>
      )

    case PlanDagNodeType.TRANSFORM:
      const transformConfig = config as any
      return (
        <Text size="xs" c="dimmed">
          {transformConfig.transformType}
        </Text>
      )

    case PlanDagNodeType.MERGE:
      const mergeConfig = config as any
      return (
        <Text size="xs" c="dimmed">
          Strategy: {mergeConfig.mergeStrategy}
        </Text>
      )

    case PlanDagNodeType.COPY:
      const copyConfig = config as any
      return (
        <Text size="xs" c="dimmed">
          Type: {copyConfig.copyType}
        </Text>
      )

    case PlanDagNodeType.OUTPUT:
      const outputConfig = config as any
      return (
        <Text size="xs" c="dimmed">
          Format: {outputConfig.renderTarget}
        </Text>
      )

    default:
      return null
  }
}