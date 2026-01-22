import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { resolveNodeHandlers } from './nodeHandlers'
import { PlanDagNodeType, MergeNodeConfig } from '../../../../types/plan-dag'

interface MergeNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const MergeNode = memo((props: MergeNodeProps) => {
  const { data } = props
  const { onEdit: resolvedOnEdit, onDelete: resolvedOnDelete } = resolveNodeHandlers(props)
  const config = data.config as MergeNodeConfig

  // Footer content with merge strategy
  const footerContent = config.mergeStrategy ? (
    <p className="text-xs text-muted-foreground">
      {config.mergeStrategy}
    </p>
  ) : null

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.MERGE}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => resolvedOnEdit?.(props.id)}
      onDelete={() => resolvedOnDelete?.(props.id)}
      readonly={data.readonly}
      edges={data.edges}
      hasValidConfig={data.hasValidConfig}
      footerContent={footerContent}
    />
  )
})

MergeNode.displayName = 'MergeNode'
