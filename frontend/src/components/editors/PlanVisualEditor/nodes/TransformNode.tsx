import { memo, useMemo, useState } from 'react'
import { NodeProps } from 'reactflow'
import { Stack } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { IconFileText } from '@tabler/icons-react'
import { BaseNode } from './BaseNode'
import { PlanDagNodeType, GraphTransform } from '../../../../types/plan-dag'
import { usePlanDagCQRSMutations } from '../../../../hooks/usePlanDagCQRSMutations'
import { useGraphPreview } from '../../../../hooks/usePreview'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

type TransformConfig = { transforms?: GraphTransform[] }

const FRIENDLY_NAMES: Record<string, string> = {
  PartitionDepthLimit: 'Limit depth',
  PartitionWidthLimit: 'Limit width',
  NodeLabelMaxLength: 'Truncate node labels',
  NodeLabelInsertNewlines: 'Wrap node labels',
  EdgeLabelMaxLength: 'Truncate edge labels',
  EdgeLabelInsertNewlines: 'Wrap edge labels',
  InvertGraph: 'Invert graph',
  GenerateHierarchy: 'Generate hierarchy',
  AggregateEdges: 'Aggregate edges',
}

const formatTransform = (transform: GraphTransform): string | null => {
  const { kind, params = {} } = transform

  switch (kind) {
    case 'PartitionDepthLimit':
      return params.maxPartitionDepth ? `Depth ≤ ${params.maxPartitionDepth}` : FRIENDLY_NAMES[kind]
    case 'PartitionWidthLimit':
      return params.maxPartitionWidth ? `Width ≤ ${params.maxPartitionWidth}` : FRIENDLY_NAMES[kind]
    case 'NodeLabelMaxLength':
      return params.nodeLabelMaxLength ? `Node labels ≤ ${params.nodeLabelMaxLength}` : FRIENDLY_NAMES[kind]
    case 'NodeLabelInsertNewlines':
      return params.nodeLabelInsertNewlinesAt ? `Wrap node labels @ ${params.nodeLabelInsertNewlinesAt}` : FRIENDLY_NAMES[kind]
    case 'EdgeLabelMaxLength':
      return params.edgeLabelMaxLength ? `Edge labels ≤ ${params.edgeLabelMaxLength}` : FRIENDLY_NAMES[kind]
    case 'EdgeLabelInsertNewlines':
      return params.edgeLabelInsertNewlinesAt ? `Wrap edge labels @ ${params.edgeLabelInsertNewlinesAt}` : FRIENDLY_NAMES[kind]
    case 'InvertGraph':
      return FRIENDLY_NAMES[kind]
    case 'GenerateHierarchy':
      return params.enabled === false ? null : FRIENDLY_NAMES[kind]
    case 'AggregateEdges':
      // Only surface when aggregation is explicitly disabled
      return params.enabled === false ? 'Keep duplicate edges' : null
    default:
      return kind
  }
}

const parseConfig = (config: unknown): TransformConfig => {
  if (!config) return {}
  if (typeof config === 'string') {
    try {
      return JSON.parse(config)
    } catch {
      return {}
    }
  }
  if (typeof config === 'object') {
    return config as TransformConfig
  }
  return {}
}

interface TransformNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
}

export const TransformNode = memo((props: TransformNodeProps) => {
  const { data, onEdit, onDelete } = props
  const [showAnnotations, setShowAnnotations] = useState(false)

  // Get project ID from context
  const projectId = data.projectId as number | undefined
  const { updateNode } = usePlanDagCQRSMutations({ projectId: projectId || 0 })
  const { preview: graphPreview, loading: previewLoading } = useGraphPreview(projectId || 0, props.id, {
    skip: !showAnnotations,
  })

  const parsedConfig = useMemo(() => parseConfig(data.config), [data.config])
  const transformSummary = useMemo(() => {
    const transforms = Array.isArray(parsedConfig.transforms) ? parsedConfig.transforms : []
    if (!transforms.length) {
      return 'No transforms configured'
    }
    const parts = transforms
      .map(formatTransform)
      .filter((value): value is string => Boolean(value))

    return parts.length ? parts.join(' → ') : 'Transforms configured'
  }, [parsedConfig.transforms])

  const handleLabelChange = async (newLabel: string) => {
    try {
      await updateNode(props.id, {
        metadata: { ...data.metadata, label: newLabel }
      })
    } catch (error) {
      console.error('Failed to update node label:', error)
    }
  }

  const annotationText = graphPreview?.annotations || null

  return (
    <>
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
        editableLabel={false}
        footerContent={
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-7 w-7 p-0 text-indigo-600"
                  data-action-icon="annotations"
                  onClick={(e: React.MouseEvent) => {
                    e.stopPropagation()
                    e.preventDefault()
                    setShowAnnotations(true)
                  }}
                >
                  <IconFileText size={13} />
                </Button>
              </TooltipTrigger>
              <TooltipContent>View annotations</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        }
        children={
          <Stack gap="xs">
            <p className="text-xs text-muted-foreground">
              {transformSummary}
            </p>
          </Stack>
        }
      />
      <Dialog open={showAnnotations} onOpenChange={(open) => !open && setShowAnnotations(false)}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>Graph annotations</DialogTitle>
          </DialogHeader>
          <div className="max-h-[60vh] overflow-y-auto">
            {previewLoading ? (
              <p className="text-sm text-muted-foreground">Loading annotations…</p>
            ) : annotationText ? (
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                className="prose prose-sm dark:prose-invert"
              >
                {annotationText}
              </ReactMarkdown>
            ) : (
              <p className="text-sm text-muted-foreground">No annotations available yet.</p>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
})

TransformNode.displayName = 'TransformNode'
