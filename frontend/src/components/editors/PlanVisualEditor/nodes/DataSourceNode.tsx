import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'

interface DataSourceNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const DataSourceNode = memo((props: DataSourceNodeProps) => {
  const { data, onEdit, onDelete } = props

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.DATA_SOURCE}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
    />
  )
})

DataSourceNode.displayName = 'DataSourceNode'