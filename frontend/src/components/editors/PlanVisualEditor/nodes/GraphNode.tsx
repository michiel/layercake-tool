import { memo, useState } from 'react'
import { NodeProps } from 'reactflow'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Spinner } from '@/components/ui/spinner'
import { Group } from '@/components/layout-primitives'
import { IconSettings, IconTrash, IconPlayerPlayFilled, IconChartDots, IconTable, IconExternalLink } from '@tabler/icons-react'
import { useMutation } from '@apollo/client/react'
import { PlanDagNodeType, GraphNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { useGraphPreview } from '../../../../hooks/usePreview'
import { getExecutionStateLabel, getExecutionStateColor, isExecutionComplete, isExecutionInProgress, EXECUTE_NODE } from '../../../../graphql/preview'
import { GraphPreviewDialog } from '../../../visualization'
import type { GraphData } from '../../../visualization/GraphPreview'
import { GraphDataDialog } from '../dialogs/GraphDataDialog'
import { useNavigate } from 'react-router-dom'
import { BaseNode } from './BaseNode'
import { usePlanDagCQRSMutations } from '../../../../hooks/usePlanDagCQRSMutations'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'
import { UPDATE_GRAPH } from '../../../../graphql/graphs'

interface GraphNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const GraphNode = memo((props: GraphNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props
  const [showPreview, setShowPreview] = useState(false)
  const [showDataDialog, setShowDataDialog] = useState(false)
  const navigate = useNavigate()

  // Get project ID from context
  const projectId = data.projectId as number | undefined
  const { updateNode } = usePlanDagCQRSMutations({ projectId: projectId || 0 })
  const [updateGraphName] = useMutation(UPDATE_GRAPH)

  const config = data.config as GraphNodeConfig

  const handleLabelChange = async (newLabel: string) => {
    const trimmedLabel = newLabel.trim()
    const currentLabel = (data.metadata?.label || '').trim()

    if (trimmedLabel.length === 0 || trimmedLabel === currentLabel) {
      return
    }

    try {
      const resolvedGraphId = data.graphExecution?.graphId || null

      if (resolvedGraphId) {
        await updateGraphName({
          variables: {
            id: resolvedGraphId,
            input: { name: trimmedLabel }
          }
        })
      }

      await updateNode(props.id, {
        metadata: { ...data.metadata, label: trimmedLabel }
      })
    } catch (error) {
      console.error('Failed to update node label:', error)
    }
  }

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.GRAPH, props.id, edges, hasValidConfig)

  // Use inline execution metadata from PlanDAG query, only query if not available
  const graphExecution = data.graphExecution
  const needsExecutionQuery = !graphExecution && projectId
  const graphId = graphExecution?.graphId || null

  // Query pipeline graph preview (only for visualization dialog)
  const { preview: graphPreview } = useGraphPreview(
    projectId || 0,
    props.id,
    { skip: !showPreview || !projectId }
  )

  // Fallback query for execution state if not available inline
  const { preview: executionPreview, refetch: refetchExecutionState } = useGraphPreview(
    projectId || 0,
    props.id,
    { skip: !needsExecutionQuery }
  )
  const resolvedGraphId = graphId || executionPreview?.graphId || null

  // Execute node mutation
  const [executeNode, { loading: executing }] = useMutation(EXECUTE_NODE, {
    onCompleted: (data: any) => {
      showSuccessNotification('Execution Started', data.executeNode.message)
      // Refetch execution state to update badge
      refetchExecutionState()
    },
    onError: (error: any) => {
      showErrorNotification('Execution Failed', error.message)
    },
  })

  const handleExecute = () => {
    console.log('handleExecute called', { projectId, nodeId: props.id })
    if (!projectId) {
      showErrorNotification('Cannot Execute', 'Project ID is missing')
      return
    }
    executeNode({
      variables: {
        projectId,
        nodeId: props.id,
      },
    })
  }

  // Helper to get badge classes based on Mantine color
  const getBadgeClasses = (color: string, variant: 'filled' | 'light' | 'outline') => {
    const colorMap: Record<string, { filled: string; light: string; outline: string }> = {
      orange: {
        filled: 'bg-orange-600 text-white border-orange-600',
        light: 'bg-orange-100 text-orange-800 border-orange-200',
        outline: 'text-orange-600 border-orange-600',
      },
      blue: {
        filled: 'bg-blue-600 text-white border-blue-600',
        light: 'bg-blue-100 text-blue-800 border-blue-200',
        outline: 'text-blue-600 border-blue-600',
      },
      green: {
        filled: 'bg-green-600 text-white border-green-600',
        light: 'bg-green-100 text-green-800 border-green-200',
        outline: 'text-green-600 border-green-600',
      },
      yellow: {
        filled: 'bg-yellow-600 text-white border-yellow-600',
        light: 'bg-yellow-100 text-yellow-800 border-yellow-200',
        outline: 'text-yellow-600 border-yellow-600',
      },
      red: {
        filled: 'bg-red-600 text-white border-red-600',
        light: 'bg-red-100 text-red-800 border-red-200',
        outline: 'text-red-600 border-red-600',
      },
      gray: {
        filled: 'bg-gray-600 text-white border-gray-600',
        light: 'bg-gray-100 text-gray-800 border-gray-200',
        outline: 'text-gray-600 border-gray-600',
      },
    }
    return colorMap[color]?.[variant] || colorMap.gray[variant]
  }

  // Transform pipeline graph preview to force-graph format
  const getGraphPreviewData = (): GraphData | null => {
    if (!graphPreview) return null

    return {
      nodes: graphPreview.nodes.map((node) => ({
        id: node.id,
        name: node.label || node.id,
        layer: node.layer || 'default',
        attrs: {
          is_partition: node.isPartition ? 'true' : 'false',
          belongs_to: (node as any).belongsTo || node.attrs?.belongs_to || '',
          weight: (node.weight || 0).toString(),
          ...node.attrs,
        },
      })),
      links: graphPreview.edges.map((edge) => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        name: edge.label || '',
        layer: edge.layer || 'default',
        attrs: {
          weight: (edge.weight || 0).toString(),
          ...edge.attrs,
        },
      })),
      layers: graphPreview.layers?.map((layer) => ({
        layerId: layer.layerId,
        name: layer.name,
        backgroundColor: layer.backgroundColor,
        borderColor: layer.borderColor,
        textColor: layer.textColor,
      })),
    }
  }

  // Custom tool buttons for graph node
  const toolButtons = (
    <TooltipProvider>
      {/* Execute button - only show if configured */}
      {isConfigured && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="sm"
              variant="default"
              className="h-7 w-7 p-0 bg-green-600 hover:bg-green-700"
              data-action-icon="execute"
              disabled={executing}
              onMouseDown={(e: React.MouseEvent) => {
                e.stopPropagation()
                e.preventDefault()
                handleExecute()
              }}
            >
              <IconPlayerPlayFilled size={13} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Execute graph (build from upstream data sources)</TooltipContent>
        </Tooltip>
      )}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0"
            data-action-icon="edit"
            onMouseDown={(e: React.MouseEvent) => {
              e.stopPropagation()
              e.preventDefault()
              onEdit?.(props.id)
            }}
          >
            <IconSettings size={13} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Edit graph node</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0 text-red-600 hover:text-red-700 hover:bg-red-50"
            data-action-icon="delete"
            onMouseDown={(e: React.MouseEvent) => {
              e.stopPropagation()
              e.preventDefault()
              onDelete?.(props.id)
            }}
          >
            <IconTrash size={13} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Delete node</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )

  // Custom label badges for graph node
  const hasBadges =
    !isConfigured ||
    ((graphExecution || executionPreview) &&
      !isExecutionComplete((graphExecution || executionPreview)!.executionState))
  const labelBadges = hasBadges ? (
    <>
      {!isConfigured && (
        <Badge variant="outline" className={`text-xs ${getBadgeClasses('orange', 'outline')}`}>
          Not Configured
        </Badge>
      )}
      {(graphExecution || executionPreview) && !isExecutionComplete((graphExecution || executionPreview)!.executionState) && (
        <Badge
          variant={isExecutionComplete((graphExecution || executionPreview)!.executionState) ? 'secondary' : 'default'}
          className={`text-xs ${getBadgeClasses(
            getExecutionStateColor((graphExecution || executionPreview)!.executionState),
            isExecutionComplete((graphExecution || executionPreview)!.executionState) ? 'light' : 'filled'
          )}`}
        >
          <span className="flex items-center gap-1">
            {isExecutionInProgress((graphExecution || executionPreview)!.executionState) && <Spinner size="xs" />}
            {getExecutionStateLabel((graphExecution || executionPreview)!.executionState)}
          </span>
        </Badge>
      )}
    </>
  ) : null

  // Footer content with node/edge counts
  const footerContent = (graphExecution?.nodeCount !== undefined || config.metadata?.nodeCount !== undefined) ? (
    <p className="text-xs text-muted-foreground">
      Nodes: {graphExecution?.nodeCount || config.metadata.nodeCount}, Edges: {graphExecution?.edgeCount || config.metadata.edgeCount || 0}
    </p>
  ) : null

  return (
    <>
      <BaseNode
        {...props}
        nodeType={PlanDagNodeType.GRAPH}
        config={config}
        metadata={data.metadata}
        onEdit={() => onEdit?.(props.id)}
        onDelete={() => onDelete?.(props.id)}
        onLabelChange={handleLabelChange}
        readonly={readonly}
        edges={edges}
        hasValidConfig={hasValidConfig}
        toolButtons={toolButtons}
        labelBadges={labelBadges}
        footerContent={footerContent}
        editableLabel={true}
      >
        {/* Preview buttons */}
        {!readonly && (graphExecution ? isExecutionComplete(graphExecution.executionState) : executionPreview && isExecutionComplete(executionPreview.executionState)) && (
          <Group justify="center" gap="sm">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-9 w-9 rounded-full text-blue-600"
                    data-action-icon="preview"
                    onMouseDown={(e: React.MouseEvent) => {
                      e.stopPropagation()
                      e.preventDefault()
                      setShowPreview(true)
                    }}
                  >
                    <IconChartDots size={12} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Preview graph visualisation</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-9 w-9 rounded-full text-teal-600"
                    data-action-icon="data"
                    onMouseDown={(e: React.MouseEvent) => {
                      e.stopPropagation()
                      e.preventDefault()
                      setShowDataDialog(true)
                    }}
                  >
                    <IconTable size={12} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>View graph data (nodes, edges, layers)</TooltipContent>
              </Tooltip>
              {projectId && resolvedGraphId && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full text-purple-600"
                      data-action-icon="open-graph"
                      onMouseDown={(e: React.MouseEvent) => {
                        e.stopPropagation()
                        e.preventDefault()
                        navigate(`/projects/${projectId}/plan-nodes/${resolvedGraphId}/edit`)
                      }}
                    >
                      <IconExternalLink size={12} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Open graph editor</TooltipContent>
                </Tooltip>
              )}
            </TooltipProvider>
          </Group>
        )}
      </BaseNode>

      {/* Graph Preview Dialog */}
      <GraphPreviewDialog
        opened={showPreview}
        onClose={() => setShowPreview(false)}
        data={getGraphPreviewData()}
        title={`Graph Preview: ${data.metadata.label}`}
      />

      {/* Graph Data Dialog */}
      <GraphDataDialog
        opened={showDataDialog}
        onClose={() => setShowDataDialog(false)}
        graphId={resolvedGraphId}
        title={`Graph Data: ${data.metadata.label}`}
      />
    </>
  )
})

GraphNode.displayName = 'GraphNode'
