import { useCallback, useEffect, useMemo, useState } from 'react'
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  Node,
  NodeChange,
  EdgeChange,
  ConnectionMode,
  Panel,
} from 'reactflow'
import { Stack, Title, Alert, Loader, Text, ActionIcon, Tooltip, Group } from '@mantine/core'
import { IconAlertCircle, IconEye, IconSettings, IconPlay } from '@tabler/icons-react'

import { usePlanDag, usePlanDagMutations, usePlanDagSubscription } from '../../../hooks/usePlanDag'
import { PlanDag, PlanDagNode, PlanDagEdge, ReactFlowNode, ReactFlowEdge } from '../../../types/plan-dag'
import { validateConnection } from '../../../utils/planDagValidation'

// Import custom node types
import { InputNode } from './nodes/InputNode'
import { GraphNode } from './nodes/GraphNode'
import { TransformNode } from './nodes/TransformNode'
import { MergeNode } from './nodes/MergeNode'
import { CopyNode } from './nodes/CopyNode'
import { OutputNode } from './nodes/OutputNode'

// Import ReactFlow styles
import 'reactflow/dist/style.css'

interface PlanVisualEditorProps {
  projectId: number
  onNodeSelect?: (nodeId: string | null) => void
  onEdgeSelect?: (edgeId: string | null) => void
  readonly?: boolean
}

// Custom node types for ReactFlow
const nodeTypes = {
  InputNode,
  GraphNode,
  TransformNode,
  MergeNode,
  CopyNode,
  OutputNode,
}

// Convert Plan DAG to ReactFlow format
const convertPlanDagToReactFlow = (planDag: PlanDag): { nodes: ReactFlowNode[]; edges: ReactFlowEdge[] } => {
  const nodes: ReactFlowNode[] = planDag.nodes.map((node) => ({
    ...node,
    type: node.type,
    data: {
      label: node.metadata.label,
      nodeType: node.type,
      config: node.config,
      metadata: node.metadata,
    },
    draggable: true,
    selectable: true,
  }))

  const edges: ReactFlowEdge[] = planDag.edges.map((edge) => ({
    ...edge,
    type: 'smoothstep',
    animated: false,
    label: edge.metadata.label,
    style: {
      stroke: edge.metadata.dataType === 'GraphReference' ? '#228be6' : '#868e96',
      strokeWidth: 2,
    },
    labelStyle: {
      fontSize: 12,
      fontWeight: 500,
    },
  }))

  return { nodes, edges }
}

// Convert ReactFlow back to Plan DAG format
const convertReactFlowToPlanDag = (
  nodes: ReactFlowNode[],
  edges: ReactFlowEdge[],
  metadata: PlanDag['metadata']
): PlanDag => {
  const planDagNodes: PlanDagNode[] = nodes.map((node) => ({
    id: node.id,
    type: node.data.nodeType,
    position: node.position,
    metadata: node.data.metadata,
    config: node.data.config,
  }))

  const planDagEdges: PlanDagEdge[] = edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    metadata: {
      label: edge.label as string,
      dataType: edge.metadata?.dataType || 'GraphData',
    },
  }))

  return {
    version: metadata.version,
    nodes: planDagNodes,
    edges: planDagEdges,
    metadata,
  }
}

export const PlanVisualEditor = ({ projectId, onNodeSelect, onEdgeSelect, readonly = false }: PlanVisualEditorProps) => {
  const { planDag, loading, error } = usePlanDag(projectId)
  const mutations = usePlanDagMutations(projectId)
  const { lastChange } = usePlanDagSubscription(projectId)

  const [nodes, setNodes, onNodesChange] = useNodesState<ReactFlowNode>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<ReactFlowEdge>([])
  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [selectedEdge, setSelectedEdge] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)

  // Initialize ReactFlow from Plan DAG data
  useEffect(() => {
    if (planDag) {
      const { nodes: rfNodes, edges: rfEdges } = convertPlanDagToReactFlow(planDag)
      setNodes(rfNodes)
      setEdges(rfEdges)
    }
  }, [planDag, setNodes, setEdges])

  // Handle real-time changes from other users
  useEffect(() => {
    if (lastChange && planDag) {
      // Apply real-time changes from collaborators
      console.log('Real-time change received:', lastChange)
      // This would update the local state with remote changes
    }
  }, [lastChange, planDag])

  // Handle node changes (position, selection, etc.)
  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      onNodesChange(changes)
      setIsDirty(true)

      // Handle node position changes
      changes.forEach((change) => {
        if (change.type === 'position' && change.position && !readonly) {
          mutations.moveNode(change.id, change.position)
        }
        if (change.type === 'select') {
          const nodeId = change.selected ? change.id : null
          setSelectedNode(nodeId)
          onNodeSelect?.(nodeId)
        }
      })
    },
    [onNodesChange, mutations, onNodeSelect, readonly]
  )

  // Handle edge changes
  const handleEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      onEdgesChange(changes)
      setIsDirty(true)

      changes.forEach((change) => {
        if (change.type === 'remove' && !readonly) {
          mutations.deleteEdge(change.id)
        }
        if (change.type === 'select') {
          const edgeId = change.selected ? change.id : null
          setSelectedEdge(edgeId)
          onEdgeSelect?.(edgeId)
        }
      })
    },
    [onEdgesChange, mutations, onEdgeSelect, readonly]
  )

  // Handle new connections
  const onConnect = useCallback(
    (connection: Connection) => {
      if (readonly) return

      // Validate the connection
      const sourceNode = nodes.find((n) => n.id === connection.source)
      const targetNode = nodes.find((n) => n.id === connection.target)

      if (!sourceNode || !targetNode) return

      const isValid = validateConnection(sourceNode.data.nodeType, targetNode.data.nodeType)
      if (!isValid.isValid) {
        console.error('Invalid connection:', isValid.errorMessage)
        return
      }

      const newEdge: ReactFlowEdge = {
        id: `edge-${Date.now()}`,
        source: connection.source!,
        target: connection.target!,
        type: 'smoothstep',
        animated: false,
        metadata: {
          label: isValid.dataType === 'GraphReference' ? 'Graph Ref' : 'Data',
          dataType: isValid.dataType,
        },
        label: isValid.dataType === 'GraphReference' ? 'Graph Ref' : 'Data',
        style: {
          stroke: isValid.dataType === 'GraphReference' ? '#228be6' : '#868e96',
          strokeWidth: 2,
        },
        labelStyle: {
          fontSize: 12,
          fontWeight: 500,
        },
      }

      setEdges((eds) => addEdge(newEdge, eds))
      mutations.addEdge(newEdge)
    },
    [nodes, readonly, mutations, setEdges]
  )

  // Save Plan DAG changes
  const savePlanDag = useCallback(async () => {
    if (!planDag || readonly) return

    const updatedPlanDag = convertReactFlowToPlanDag(nodes, edges, planDag.metadata)
    await mutations.updatePlanDag(updatedPlanDag)
    setIsDirty(false)
  }, [planDag, nodes, edges, mutations, readonly])

  // Auto-save on changes (debounced)
  useEffect(() => {
    if (!isDirty) return

    const timeoutId = setTimeout(savePlanDag, 2000) // Auto-save after 2 seconds
    return () => clearTimeout(timeoutId)
  }, [isDirty, savePlanDag])

  const miniMapNodeColor = useCallback((node: Node) => {
    switch (node.data?.nodeType) {
      case 'InputNode':
        return '#51cf66'
      case 'GraphNode':
        return '#339af0'
      case 'TransformNode':
        return '#ff8cc8'
      case 'MergeNode':
        return '#ffd43b'
      case 'CopyNode':
        return '#74c0fc'
      case 'OutputNode':
        return '#ff6b6b'
      default:
        return '#868e96'
    }
  }, [])

  if (loading) {
    return (
      <Stack align="center" justify="center" h="100%" spacing="md">
        <Loader size="lg" />
        <Text>Loading Plan DAG...</Text>
      </Stack>
    )
  }

  if (error) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} title="Error loading Plan DAG" color="red">
        {error.message}
      </Alert>
    )
  }

  if (!planDag) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} title="No Plan DAG found" color="yellow">
        This project doesn't have a Plan DAG configured yet.
      </Alert>
    )
  }

  return (
    <Stack h="100%" spacing={0}>
      <Group justify="space-between" p="md" bg="gray.0">
        <Title order={3}>Plan DAG Editor</Title>
        <Group spacing="xs">
          {isDirty && (
            <Text size="sm" c="yellow.6">
              Unsaved changes
            </Text>
          )}
          <Tooltip label="Preview execution">
            <ActionIcon variant="light" color="blue">
              <IconEye size="1rem" />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Run plan">
            <ActionIcon variant="light" color="green">
              <IconPlay size="1rem" />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Settings">
            <ActionIcon variant="light" color="gray">
              <IconSettings size="1rem" />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>

      <div style={{ flex: 1, position: 'relative' }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onConnect={onConnect}
          nodeTypes={nodeTypes}
          connectionMode={ConnectionMode.Loose}
          fitView
          attributionPosition="top-right"
          proOptions={{ hideAttribution: true }}
        >
          <Background />
          <Controls />
          <MiniMap nodeColor={miniMapNodeColor} />

          <Panel position="top-left">
            <Stack spacing="xs" p="xs" bg="white" style={{ borderRadius: 4, boxShadow: '0 2px 8px rgba(0,0,0,0.1)' }}>
              <Text size="sm" fw={500}>
                {planDag.metadata.name || 'Untitled Plan'}
              </Text>
              <Text size="xs" c="dimmed">
                {nodes.length} nodes, {edges.length} connections
              </Text>
            </Stack>
          </Panel>
        </ReactFlow>
      </div>
    </Stack>
  )
}