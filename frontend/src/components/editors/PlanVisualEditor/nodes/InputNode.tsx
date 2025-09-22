import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

interface InputNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const InputNode = memo((props: InputNodeProps) => {
  const { data, onEdit, onDelete } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.INPUT}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
    />
  )
})

InputNode.displayName = 'InputNode'