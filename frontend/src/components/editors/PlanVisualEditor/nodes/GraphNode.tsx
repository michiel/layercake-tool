import { memo, useState } from 'react'
import { NodeProps } from 'reactflow'
import { Text, Group, ActionIcon, Tooltip, Badge, Loader } from '@mantine/core'
import { IconSettings, IconTrash, IconPlayerPlayFilled, IconChartDots, IconTable, IconExternalLink } from '@tabler/icons-react'
import { useMutation } from '@apollo/client/react'
import { PlanDagNodeType, GraphNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { useGraphPreview } from '../../../../hooks/usePreview'
import { getExecutionStateLabel, getExecutionStateColor, isExecutionComplete, isExecutionInProgress, EXECUTE_NODE } from '../../../../graphql/preview'
import { GraphPreviewDialog } from '../../../visualization/GraphPreviewDialog'
import { GraphData } from '../../../visualization/GraphPreview'
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
        backgroundColor: layer.properties?.background_color,
        borderColor: layer.properties?.border_color,
        textColor: layer.properties?.text_color,
      })),
    }
  }

  // Custom tool buttons for graph node
  const toolButtons = (
    <>
      {/* Execute button - only show if configured */}
      {isConfigured && (
        <Tooltip label="Execute graph (build from upstream data sources)">
          <ActionIcon
            size="sm"
            variant="filled"
            color="green"
            data-action-icon="execute"
            loading={executing}
            onMouseDown={(e) => {
              e.stopPropagation()
              e.preventDefault()
              handleExecute()
            }}
          >
            <IconPlayerPlayFilled size="0.8rem" />
          </ActionIcon>
        </Tooltip>
      )}
      <Tooltip label="Edit graph node">
        <ActionIcon
          size="sm"
          variant="subtle"
          color="gray"
          data-action-icon="edit"
          onMouseDown={(e) => {
            e.stopPropagation()
            e.preventDefault()
            onEdit?.(props.id)
          }}
        >
          <IconSettings size="0.8rem" />
        </ActionIcon>
      </Tooltip>
      <Tooltip label="Delete node">
        <ActionIcon
          size="sm"
          variant="subtle"
          color="red"
          data-action-icon="delete"
          onMouseDown={(e) => {
            e.stopPropagation()
            e.preventDefault()
            onDelete?.(props.id)
          }}
        >
          <IconTrash size="0.8rem" />
        </ActionIcon>
      </Tooltip>
    </>
  )

  // Custom label badges for graph node
  const hasBadges = config.isReference || !isConfigured || ((graphExecution || executionPreview) && !isExecutionComplete((graphExecution || executionPreview)!.executionState))
  const labelBadges = hasBadges ? (
    <>
      {config.isReference && (
        <Badge variant="outline" size="xs" color="blue">
          Reference
        </Badge>
      )}
      {!isConfigured && (
        <Badge variant="outline" size="xs" color="orange">
          Not Configured
        </Badge>
      )}
      {(graphExecution || executionPreview) && !isExecutionComplete((graphExecution || executionPreview)!.executionState) && (
        <Badge
          variant={isExecutionComplete((graphExecution || executionPreview)!.executionState) ? 'light' : 'filled'}
          color={getExecutionStateColor((graphExecution || executionPreview)!.executionState)}
          size="xs"
          leftSection={isExecutionInProgress((graphExecution || executionPreview)!.executionState) ? <Loader size={10} /> : undefined}
        >
          {getExecutionStateLabel((graphExecution || executionPreview)!.executionState)}
        </Badge>
      )}
    </>
  ) : null

  // Footer content with node/edge counts
  const footerContent = (graphExecution?.nodeCount !== undefined || config.metadata?.nodeCount !== undefined) ? (
    <Text size="xs" c="dimmed">
      Nodes: {graphExecution?.nodeCount || config.metadata.nodeCount}, Edges: {graphExecution?.edgeCount || config.metadata.edgeCount || 0}
    </Text>
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
            <Tooltip label="Preview graph visualisation">
              <ActionIcon
                size="lg"
                variant="light"
                color="blue"
                radius="xl"
                data-action-icon="preview"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  setShowPreview(true)
                }}
              >
                <IconChartDots size="0.75rem" />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="View graph data (nodes, edges, layers)">
              <ActionIcon
                size="lg"
                variant="light"
                color="teal"
                radius="xl"
                data-action-icon="data"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  setShowDataDialog(true)
                }}
              >
                <IconTable size="0.75rem" />
              </ActionIcon>
            </Tooltip>
            {projectId && resolvedGraphId && (
              <Tooltip label="Open graph editor">
                <ActionIcon
                  size="lg"
                  variant="light"
                  color="grape"
                  radius="xl"
                  data-action-icon="open-graph"
                  onMouseDown={(e) => {
                    e.stopPropagation()
                    e.preventDefault()
                    navigate(`/projects/${projectId}/plan-nodes/${resolvedGraphId}/edit`)
                  }}
                >
                  <IconExternalLink size="0.75rem" />
                </ActionIcon>
              </Tooltip>
            )}
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
