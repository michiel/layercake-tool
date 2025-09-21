import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

export const OutputNode = memo((props: NodeProps) => {
  const { data } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.OUTPUT}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => console.log('Edit output node:', props.id)}
      onDelete={() => console.log('Delete output node:', props.id)}
    />
  )
})

OutputNode.displayName = 'OutputNode'