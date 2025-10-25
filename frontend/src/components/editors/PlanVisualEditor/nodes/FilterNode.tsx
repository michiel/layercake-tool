import { memo, useMemo } from 'react'
import { NodeProps } from 'reactflow'
import { Stack, Text } from '@mantine/core'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType, QueryFilterConfig } from '../../../../types/plan-dag'
import { usePlanDagCQRSMutations } from '../../../../hooks/usePlanDagCQRSMutations'
import { extractQueryConfigFromRaw } from '../forms/filterConfigUtils'

type FilterConfig = { query?: QueryFilterConfig; filters?: unknown }

const formatQuerySummary = (queryConfig: QueryFilterConfig | null): string => {
  if (!queryConfig) {
    return 'No query configured'
  }
  const targets = queryConfig.targets?.length ? queryConfig.targets.join(', ') : 'nodes'
  const modeLabel = queryConfig.mode === 'exclude' ? 'Exclude' : 'Include'
  const rules = Array.isArray(queryConfig.ruleGroup?.rules) ? queryConfig.ruleGroup.rules.length : 0
  const ruleLabel = rules === 0 ? 'no rules' : `${rules} rule${rules === 1 ? '' : 's'}`
  return `${modeLabel} (${targets}, ${ruleLabel})`
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
  const queryConfig = useMemo(() => extractQueryConfigFromRaw(parsedConfig), [parsedConfig])
  const filterSummary = useMemo(() => formatQuerySummary(queryConfig), [queryConfig])

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
