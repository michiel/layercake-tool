import { memo, useMemo } from 'react'
import { NodeProps } from 'reactflow'
import { Stack, Text } from '@mantine/core'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType, GraphFilter } from '../../../../types/plan-dag'
import { usePlanDagCQRSMutations } from '../../../../hooks/usePlanDagCQRSMutations'

type FilterConfig = { filters?: GraphFilter[] }

const FRIENDLY_NAMES: Record<string, string> = {
  RemoveUnconnectedNodes: 'Remove unconnected nodes',
  RemoveDanglingEdges: 'Remove dangling edges',
}

const formatFilter = (filter: GraphFilter): string | null => {
  const { kind, params = {} } = filter

  if (params.enabled === false) {
    return null
  }

  switch (kind) {
    case 'Preset':
      return params.preset ? FRIENDLY_NAMES[params.preset] || params.preset : 'Preset filter'
    case 'Query': {
      const targets = params.queryConfig?.targets?.join(', ') || 'nodes'
      return params.queryConfig ? `Query filter (${targets})` : 'Query filter'
    }
    default:
      return kind
  }
}

const parseConfig = (config: unknown): FilterConfig => {
  if (!config) return {}
  if (typeof config === 'string') {
    try {
      return JSON.parse(config)
    } catch {
      return {}
    }
  }
  if (typeof config === 'object') {
    return config as FilterConfig
  }
  return {}
}

interface FilterNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const FilterNode = memo((props: FilterNodeProps) => {
  const { data, onEdit, onDelete } = props

  // Get project ID from context
  const projectId = data.projectId as number | undefined
  const { updateNode } = usePlanDagCQRSMutations({ projectId: projectId || 0 })

  const parsedConfig = useMemo(() => parseConfig(data.config), [data.config])
  const filterSummary = useMemo(() => {
    const filters = Array.isArray(parsedConfig.filters) ? parsedConfig.filters : []
    if (!filters.length) {
      return 'No filters configured'
    }
    const parts = filters
      .map(formatFilter)
      .filter((value): value is string => Boolean(value))

    return parts.length ? parts.join(' → ') : 'Filters configured'
  }, [parsedConfig.filters])

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
      nodeType={PlanDagNodeType.FILTER}
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
            {filterSummary}
          </Text>
        </Stack>
      }
    />
  )
})

FilterNode.displayName = 'FilterNode'
