import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

export const CopyNode = memo((props: NodeProps) => {
  const { data } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.COPY}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => console.log('Edit copy node:', props.id)}
      onDelete={() => console.log('Delete copy node:', props.id)}
    />
  )
})

CopyNode.displayName = 'CopyNode'