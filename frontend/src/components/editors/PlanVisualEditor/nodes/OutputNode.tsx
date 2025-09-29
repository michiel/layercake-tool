import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

interface OutputNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const OutputNode = memo((props: OutputNodeProps) => {
  const { data, onEdit, onDelete } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.OUTPUT}
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

OutputNode.displayName = 'OutputNode'