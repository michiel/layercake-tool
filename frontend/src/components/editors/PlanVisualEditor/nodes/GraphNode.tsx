import { memo, useState } from 'react'
import { NodeProps, Handle, Position } from 'reactflow'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Stack, Loader } from '@mantine/core'
import { IconSettings, IconTrash, IconPlayerPlayFilled, IconChartDots, IconTable } from '@tabler/icons-react'
import { useMutation } from '@apollo/client/react'
import { PlanDagNodeType, GraphNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon, getNodeTypeLabel } from '../../../../utils/nodeStyles'
import { useGraphPreview } from '../../../../hooks/usePreview'
import { getExecutionStateLabel, getExecutionStateColor, isExecutionComplete, isExecutionInProgress, EXECUTE_NODE } from '../../../../graphql/preview'
import { GraphPreviewDialog } from '../../../visualization/GraphPreviewDialog'
import { GraphData } from '../../../visualization/GraphPreview'
import { GraphDataDialog } from '../dialogs/GraphDataDialog'

interface GraphNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const GraphNode = memo((props: GraphNodeProps) => {
  const { data, selected, onEdit, onDelete, readonly = false } = props
  const [showPreview, setShowPreview] = useState(false)
  const [showDataDialog, setShowDataDialog] = useState(false)

  const config = data.config as GraphNodeConfig
  const color = getNodeColor(PlanDagNodeType.GRAPH)

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.GRAPH, props.id, edges, hasValidConfig)

  // Get project ID from context
  const projectId = data.projectId as number | undefined

  // Query pipeline graph preview
  const { preview: graphPreview } = useGraphPreview(
    projectId || 0,
    props.id,
    { skip: !showPreview || !projectId }
  )

  // Query execution state (always fetch to show status)
  const { preview: executionPreview, refetch: refetchExecutionState } = useGraphPreview(
    projectId || 0,
    props.id,
    { skip: !projectId }
  )

  // Execute node mutation
  const [executeNode, { loading: executing }] = useMutation(EXECUTE_NODE, {
    onCompleted: (data: any) => {
      console.log('Success:', data.executeNode.message)
      // Refetch execution state to update badge
      refetchExecutionState()
    },
    onError: (error: any) => {
      console.error('Execution Failed:', error.message)
    },
  })

  const handleExecute = () => {
    if (!projectId) return
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
          is_partition: node.isPartition.toString(),
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
    }
  }

  return (
    <>
      {/* Input Handles */}
      <Handle
        type="target"
        position={Position.Left}
        id="input-left"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />
      <Handle
        type="target"
        position={Position.Top}
        id="input-top"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />

      {/* Output Handles */}
      <Handle
        type="source"
        position={Position.Right}
        id="output-right"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="output-bottom"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />

      {/* Node Content */}
      <Paper
        shadow={selected ? "md" : "sm"}
        p="md"
        style={{
          border: `2px solid ${color}`,
          borderRadius: 8,
          minWidth: 200,
          maxWidth: 280,
          background: '#fff',
          cursor: 'default',
          pointerEvents: 'all',
        }}
      >
        {/* Top right: Execute, Edit and Delete icons */}
        {!readonly && (
          <Group gap={4} style={{ position: 'absolute', top: 8, right: 8, pointerEvents: 'auto', zIndex: 10 }}>
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
          </Group>
        )}

        {/* Middle: Icon and Label */}
        <Group gap="sm" mb="sm" wrap="nowrap" className="node-header" style={{ paddingRight: !readonly ? 100 : 0, cursor: 'grab' }}>
          <div style={{
            color,
            display: 'flex',
            alignItems: 'center',
            flexShrink: 0
          }}>
            {getNodeIcon(PlanDagNodeType.GRAPH, '1.4rem')}
          </div>
          <Text size="sm" fw={600} lineClamp={2} style={{ wordBreak: 'break-word', flex: 1, minWidth: 0 }}>
            {data.metadata.label}
          </Text>
        </Group>

        {/* Center: Preview buttons */}
        {!readonly && executionPreview && isExecutionComplete(executionPreview.executionState) && (
          <Group justify="center" mb="md" gap="sm">
            <Tooltip label="Preview graph visualization">
              <ActionIcon
                size="xl"
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
                <IconChartDots size="1.5rem" />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="View graph data (nodes, edges, layers)">
              <ActionIcon
                size="xl"
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
                <IconTable size="1.5rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        )}

        {/* Bottom: Labels and metadata */}
        <Stack gap="xs">
          <Group gap="xs" wrap="wrap">
            <Badge
              variant="light"
              color={color}
              size="xs"
              style={{ textTransform: 'none' }}
            >
              {getNodeTypeLabel(PlanDagNodeType.GRAPH)}
            </Badge>
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
            {executionPreview && (
              <Badge
                variant={isExecutionComplete(executionPreview.executionState) ? 'light' : 'filled'}
                color={getExecutionStateColor(executionPreview.executionState)}
                size="xs"
                leftSection={isExecutionInProgress(executionPreview.executionState) ? <Loader size={10} /> : undefined}
              >
                {getExecutionStateLabel(executionPreview.executionState)}
              </Badge>
            )}
          </Group>

          {config.metadata?.nodeCount !== undefined && (
            <Text size="xs" c="dimmed">
              Nodes: {config.metadata.nodeCount}, Edges: {config.metadata.edgeCount || 0}
            </Text>
          )}
        </Stack>
      </Paper>

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
        graphId={executionPreview?.graphId || null}
        title={`Graph Data: ${data.metadata.label}`}
      />
    </>
  )
})

GraphNode.displayName = 'GraphNode'