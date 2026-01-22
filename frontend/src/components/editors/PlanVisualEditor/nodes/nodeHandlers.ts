import type { NodeProps } from 'reactflow'

interface NodeDataHandlers {
  onEdit?: (nodeId?: string) => void
  onDelete?: (nodeId?: string) => void
}

interface NodeActionProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const resolveNodeHandlers = (props: NodeProps & NodeActionProps) => {
  const { onEdit, onDelete, data } = props
  const handlers = (data as NodeDataHandlers) ?? {}
  return {
    onEdit: onEdit ?? handlers.onEdit,
    onDelete: onDelete ?? handlers.onDelete,
  }
}
