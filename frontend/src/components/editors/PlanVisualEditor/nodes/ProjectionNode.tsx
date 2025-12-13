import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { IconPresentation } from '@tabler/icons-react'
import { PlanDagNodeType, ProjectionNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { BaseNode } from './BaseNode'
import { Badge } from '@/components/ui/badge'
import { Stack } from '@/components/layout-primitives'

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const ProjectionNode = memo((props: ExtendedNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props

  const config = data.config as ProjectionNodeConfig
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false

  const isConfigured = isNodeConfigured(
    PlanDagNodeType.PROJECTION,
    props.id,
    edges,
    hasValidConfig
  )

  const labelBadges = !isConfigured ? (
    <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
      Not Configured
    </Badge>
  ) : null

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.PROJECTION}
      config={config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      readonly={readonly}
      edges={edges}
      hasValidConfig={hasValidConfig}
      labelBadges={labelBadges}
    >
      <Stack gap="xs">
        {config.projectionId && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <IconPresentation size={14} />
            <span>Projection #{config.projectionId}</span>
          </div>
        )}
        {!config.projectionId && (
          <p className="text-xs text-muted-foreground">
            No projection selected
          </p>
        )}
      </Stack>
    </BaseNode>
  )
})

ProjectionNode.displayName = 'ProjectionNode'
