import { memo, useState } from 'react'
import { NodeProps, Handle, Position } from 'reactflow'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Stack } from '@mantine/core'
import { IconSettings, IconTrash, IconPlayerPlay } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { PlanDagNodeType, GraphNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon, getNodeTypeLabel } from '../../../../utils/nodeStyles'
import { GET_GRAPH_DATA, GraphDataResponse } from '../../../../graphql/graphs'
import { GraphPreviewDialog } from '../../../visualization/GraphPreviewDialog'
import { GraphData } from '../../../visualization/GraphPreview'

interface GraphNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const GraphNode = memo((props: GraphNodeProps) => {
  const { data, selected, onEdit, onDelete, readonly = false } = props
  const [showPreview, setShowPreview] = useState(false)

  const config = data.config as GraphNodeConfig
  const color = getNodeColor(PlanDagNodeType.GRAPH)

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.GRAPH, props.id, edges, hasValidConfig)

  // Get project ID from context (assuming it's available in data or from parent)
  const projectId = data.projectId as number | undefined

  // Query graph data for preview
  const { data: graphData } = useQuery<{ graphData: GraphDataResponse }>(GET_GRAPH_DATA, {
    variables: { projectId: projectId || 0 },
    skip: !projectId || !showPreview,
  })

  // Transform graph data for force-graph format
  const getGraphPreviewData = (): GraphData | null => {
    if (!graphData?.graphData) return null

    return {
      nodes: graphData.graphData.nodes.map((node: GraphDataResponse['nodes'][0]) => ({
        id: node.id,
        name: node.label,
        layer: node.layer,
        attrs: {
          is_partition: node.isPartition.toString(),
          weight: node.weight.toString(),
          type: node.layer,
        },
      })),
      links: graphData.graphData.edges.map((edge: GraphDataResponse['edges'][0]) => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        name: edge.label,
        layer: edge.layer,
        attrs: {
          weight: edge.weight.toString(),
          type: edge.layer,
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
        {/* Top right: Edit and Delete icons */}
        {!readonly && (
          <Group gap={4} style={{ position: 'absolute', top: 8, right: 8, pointerEvents: 'auto', zIndex: 10 }}>
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
        <Group gap="sm" mb="sm" wrap="nowrap" className="node-header" style={{ paddingRight: !readonly ? 60 : 0, cursor: 'grab' }}>
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

        {/* Center: Play button */}
        {!readonly && (
          <Group justify="center" mb="md">
            <Tooltip label={projectId ? "Preview graph" : "Preview graph (project ID required)"}>
              <ActionIcon
                size="xl"
                variant="light"
                color="blue"
                radius="xl"
                data-action-icon="preview"
                disabled={!projectId}
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  if (projectId) {
                    setShowPreview(true)
                  }
                }}
              >
                <IconPlayerPlay size="1.5rem" />
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
    </>
  )
})

GraphNode.displayName = 'GraphNode'