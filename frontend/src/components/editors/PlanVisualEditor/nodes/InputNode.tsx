import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

export const InputNode = memo((props: NodeProps) => {
  const { data } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.INPUT}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => console.log('Edit input node:', props.id)}
      onDelete={() => console.log('Delete input node:', props.id)}
    />
  )
})

InputNode.displayName = 'InputNode'