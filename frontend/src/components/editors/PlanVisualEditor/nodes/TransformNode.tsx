import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

interface TransformNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const TransformNode = memo((props: TransformNodeProps) => {
  const { data, onEdit, onDelete } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.TRANSFORM}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      readonly={data.readonly}
      edges={data.edges}
      hasValidConfig={data.hasValidConfig}
    />
  )
})

TransformNode.displayName = 'TransformNode'