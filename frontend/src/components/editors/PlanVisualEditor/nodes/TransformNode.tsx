import { memo, useMemo } from 'react'
import { NodeProps } from 'reactflow'
import { Stack, Text } from '@mantine/core'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType, GraphTransform } from '../../../../types/plan-dag'
import { usePlanDagCQRSMutations } from '../../../../hooks/usePlanDagCQRSMutations'

type TransformConfig = { transforms?: GraphTransform[] }

const FRIENDLY_NAMES: Record<string, string> = {
  PartitionDepthLimit: 'Limit depth',
  PartitionWidthLimit: 'Limit width',
  NodeLabelMaxLength: 'Truncate node labels',
  NodeLabelInsertNewlines: 'Wrap node labels',
  EdgeLabelMaxLength: 'Truncate edge labels',
  EdgeLabelInsertNewlines: 'Wrap edge labels',
  InvertGraph: 'Invert graph',
  GenerateHierarchy: 'Generate hierarchy',
  AggregateEdges: 'Aggregate edges',
}

const formatTransform = (transform: GraphTransform): string | null => {
  const { kind, params = {} } = transform

  switch (kind) {
    case 'PartitionDepthLimit':
      return params.maxPartitionDepth ? `Depth ≤ ${params.maxPartitionDepth}` : FRIENDLY_NAMES[kind]
    case 'PartitionWidthLimit':
      return params.maxPartitionWidth ? `Width ≤ ${params.maxPartitionWidth}` : FRIENDLY_NAMES[kind]
    case 'NodeLabelMaxLength':
      return params.nodeLabelMaxLength ? `Node labels ≤ ${params.nodeLabelMaxLength}` : FRIENDLY_NAMES[kind]
    case 'NodeLabelInsertNewlines':
      return params.nodeLabelInsertNewlinesAt ? `Wrap node labels @ ${params.nodeLabelInsertNewlinesAt}` : FRIENDLY_NAMES[kind]
    case 'EdgeLabelMaxLength':
      return params.edgeLabelMaxLength ? `Edge labels ≤ ${params.edgeLabelMaxLength}` : FRIENDLY_NAMES[kind]
    case 'EdgeLabelInsertNewlines':
      return params.edgeLabelInsertNewlinesAt ? `Wrap edge labels @ ${params.edgeLabelInsertNewlinesAt}` : FRIENDLY_NAMES[kind]
    case 'InvertGraph':
      return FRIENDLY_NAMES[kind]
    case 'GenerateHierarchy':
      return params.enabled === false ? null : FRIENDLY_NAMES[kind]
    case 'AggregateEdges':
      // Only surface when aggregation is explicitly disabled
      return params.enabled === false ? 'Keep duplicate edges' : null
    default:
      return kind
  }
}

const parseConfig = (config: unknown): TransformConfig => {
  if (!config) return {}
  if (typeof config === 'string') {
    try {
      return JSON.parse(config)
    } catch {
      return {}
    }
  }
  if (typeof config === 'object') {
    return config as TransformConfig
  }
  return {}
}

interface TransformNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const TransformNode = memo((props: TransformNodeProps) => {
  const { data, onEdit, onDelete } = props

  // Get project ID from context
  const projectId = data.projectId as number | undefined
  const { updateNode } = usePlanDagCQRSMutations({ projectId: projectId || 0 })

  const parsedConfig = useMemo(() => parseConfig(data.config), [data.config])
  const transformSummary = useMemo(() => {
    const transforms = Array.isArray(parsedConfig.transforms) ? parsedConfig.transforms : []
    if (!transforms.length) {
      return 'No transforms configured'
    }
    const parts = transforms
      .map(formatTransform)
      .filter((value): value is string => Boolean(value))

    return parts.length ? parts.join(' → ') : 'Transforms configured'
  }, [parsedConfig.transforms])

  const handleLabelChange = async (newLabel: string) => {
    try {
      await updateNode(props.id, {
        metadata: { ...data.metadata, label: newLabel }
      })
    } catch (error) {
      console.error('Failed to update node label:', error)
    }
  }

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.TRANSFORM}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      onLabelChange={handleLabelChange}
      readonly={data.readonly}
      edges={data.edges}
      hasValidConfig={data.hasValidConfig}
      editableLabel={false}
      children={
        <Stack gap={4}>
          <Text size="xs" c="dimmed">
            {transformSummary}
          </Text>
        </Stack>
      }
    />
  )
})

TransformNode.displayName = 'TransformNode'
