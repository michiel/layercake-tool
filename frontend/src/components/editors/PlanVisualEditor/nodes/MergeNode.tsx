import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { Text } from '@mantine/core'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType, MergeNodeConfig } from '../../../../types/plan-dag'

interface MergeNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const MergeNode = memo((props: MergeNodeProps) => {
  const { data, onEdit, onDelete } = props
  const config = data.config as MergeNodeConfig

  // Footer content with merge strategy
  const footerContent = config.mergeStrategy ? (
    <Text size="xs" c="dimmed">
      {config.mergeStrategy}
    </Text>
  ) : null

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.MERGE}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      readonly={data.readonly}
      edges={data.edges}
      hasValidConfig={data.hasValidConfig}
      footerContent={footerContent}
    />
  )
})

MergeNode.displayName = 'MergeNode'