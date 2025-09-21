import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

export const GraphNode = memo((props: NodeProps) => {
  const { data } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.GRAPH}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => console.log('Edit graph node:', props.id)}
      onDelete={() => console.log('Delete graph node:', props.id)}
    />
  )
})

GraphNode.displayName = 'GraphNode'