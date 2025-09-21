import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

export const MergeNode = memo((props: NodeProps) => {
  const { data } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.MERGE}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => console.log('Edit merge node:', props.id)}
      onDelete={() => console.log('Delete merge node:', props.id)}
    />
  )
})

MergeNode.displayName = 'MergeNode'