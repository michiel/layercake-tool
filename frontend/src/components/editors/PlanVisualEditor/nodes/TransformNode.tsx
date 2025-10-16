import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType } from '../../../../types/plan-dag'
import { usePlanDagCQRSMutations } from '../../../../hooks/usePlanDagCQRSMutations'

interface TransformNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const TransformNode = memo((props: TransformNodeProps) => {
  const { data, onEdit, onDelete } = props

  // Get project ID from context
  const projectId = data.projectId as number | undefined
  const { updateNode } = usePlanDagCQRSMutations({ projectId: projectId || 0 })

  const handleLabelChange = async (newLabel: string) => {
    try {
      await updateNode(props.id, {
        metadata: { ...data.metadata, label: newLabel }
      })
    } catch (error) {
      console.error('Failed to update node label:', error)
    }
  }

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.TRANSFORM}
      config={data.config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      onLabelChange={handleLabelChange}
      readonly={data.readonly}
      edges={data.edges}
      hasValidConfig={data.hasValidConfig}
      editableLabel={true}
    />
  )
})

TransformNode.displayName = 'TransformNode'