import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

export const TransformNode = memo((props: NodeProps) => {
  const { data } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.TRANSFORM}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => console.log('Edit transform node:', props.id)}
      onDelete={() => console.log('Delete transform node:', props.id)}
    />
  )
})

TransformNode.displayName = 'TransformNode'