import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
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
import { IconAlertCircle, IconEye, IconSettings, IconPlayerPlay } from '@tabler/icons-react'

// import { usePlanDag, usePlanDagMutations, usePlanDagSubscription } from '../../../hooks/usePlanDag'
import { PlanDag, PlanDagNode, PlanDagEdge, ReactFlowNode, ReactFlowEdge, PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../types/plan-dag'
import { validateConnection } from '../../../utils/planDagValidation'
import { UserPresence } from '../../../hooks/useCollaborationSubscriptions'

// Import custom node types
import { InputNode } from './nodes/InputNode'
import { GraphNode } from './nodes/GraphNode'
import { TransformNode } from './nodes/TransformNode'
import { MergeNode } from './nodes/MergeNode'
import { CopyNode } from './nodes/CopyNode'
import { OutputNode } from './nodes/OutputNode'

// Import dialogs
import { NodeConfigDialog } from './dialogs/NodeConfigDialog'

// Import collaboration components
import { UserPresenceIndicator } from '../../collaboration/UserPresenceIndicator'
import { CollaborativeCursors } from '../../collaboration/CollaborativeCursors'
import { useUserPresence, useCursorBroadcast } from '../../../hooks/useCollaborationSubscriptions'

// Import ReactFlow styles
import 'reactflow/dist/style.css'

interface PlanVisualEditorProps {
  projectId: number
  onNodeSelect?: (nodeId: string | null) => void
  onEdgeSelect?: (edgeId: string | null) => void
  readonly?: boolean
}

// Define stable nodeTypes outside component to prevent recreation warning
const NODE_TYPES = {
  [PlanDagNodeType.INPUT]: InputNode,
  [PlanDagNodeType.GRAPH]: GraphNode,
  [PlanDagNodeType.TRANSFORM]: TransformNode,
  [PlanDagNodeType.MERGE]: MergeNode,
  [PlanDagNodeType.COPY]: CopyNode,
  [PlanDagNodeType.OUTPUT]: OutputNode,
}


// Static mock Plan DAG for frontend-only development - prevents recreation issues
const staticMockPlanDag: PlanDag = {
  version: "1.0",
  nodes: [
    {
      id: 'input_1',
      type: PlanDagNodeType.INPUT,
      position: { x: 100, y: 100 },
      metadata: { label: 'CSV Import', description: 'Import nodes from CSV file' },
      config: {
        inputType: 'CSVNodesFromFile',
        source: 'import/nodes.csv',
        dataType: 'Nodes',
        outputGraphRef: 'graph_main'
      }
    },
    {
      id: 'transform_1',
      type: PlanDagNodeType.TRANSFORM,
      position: { x: 300, y: 100 },
      metadata: { label: 'Filter Nodes', description: 'Apply node filtering' },
      config: {
        inputGraphRef: 'graph_main',
        outputGraphRef: 'graph_filtered',
        transformType: 'FilterNodes',
        transformConfig: { nodeFilter: 'type = "important"' }
      }
    },
    {
      id: 'output_1',
      type: PlanDagNodeType.OUTPUT,
      position: { x: 500, y: 100 },
      metadata: { label: 'Export DOT', description: 'Generate Graphviz output' },
      config: {
        sourceGraphRef: 'graph_filtered',
        renderTarget: 'DOT',
        outputPath: 'output/result.dot',
        renderConfig: { containNodes: true, orientation: 'TB' }
      }
    }
  ],
  edges: [
    {
      id: 'edge_1',
      source: 'input_1',
      target: 'transform_1',
      metadata: { label: 'Data', dataType: 'GraphData' }
    },
    {
      id: 'edge_2',
      source: 'transform_1',
      target: 'output_1',
      metadata: { label: 'Filtered', dataType: 'GraphData' }
    }
  ],
  metadata: {
    version: "1.0",
    name: "Demo Plan DAG",
    description: "Frontend development demonstration"
  }
}

// Convert Plan DAG to ReactFlow format
const convertPlanDagToReactFlow = (
  planDag: PlanDag,
  onEdit?: (nodeId: string) => void,
  onDelete?: (nodeId: string) => void
): { nodes: ReactFlowNode[]; edges: ReactFlowEdge[] } => {
  const nodes: ReactFlowNode[] = planDag.nodes.map((node) => ({
    ...node,
    type: node.type,
    data: {
      label: node.metadata.label,
      nodeType: node.type,
      config: node.config,
      metadata: node.metadata,
      onEdit,
      onDelete,
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

  // Skip backend calls for frontend-only development
  const { planDag, loading, error } = { planDag: null, loading: false, error: null } // usePlanDag(projectId)
  const { lastChange } = { lastChange: null } // usePlanDagSubscription(projectId)

  // Collaboration hooks - mock current user for frontend-only development
  const currentUserId = 'user-123'
  const { getOnlineUsers } = useUserPresence(projectId.toString(), currentUserId)
  const { broadcastCursorPosition } = useCursorBroadcast(projectId.toString(), currentUserId)

  // Mock mutations for frontend-only development
  const mutations = {
    moveNode: (nodeId: string, position: { x: number; y: number }) => {
      console.log('Mock moveNode:', nodeId, position)
    },
    addEdge: (edge: any) => {
      console.log('Mock addEdge:', edge)
    },
    deleteEdge: (edgeId: string) => {
      console.log('Mock deleteEdge:', edgeId)
    }
  }

  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [selectedEdge, setSelectedEdge] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)
  const initializedRef = useRef(false)
  const viewportRef = useRef({ x: 0, y: 0, zoom: 1 })

  // Configuration dialog state
  const [configDialogOpen, setConfigDialogOpen] = useState(false)
  const [configNodeId, setConfigNodeId] = useState<string>('')
  const [configNodeType, setConfigNodeType] = useState<PlanDagNodeType>(PlanDagNodeType.INPUT)
  const [configNodeConfig, setConfigNodeConfig] = useState<NodeConfig>({})
  const [configNodeMetadata, setConfigNodeMetadata] = useState<NodeMetadata>({ label: '', description: '' })

  // Mock online users for frontend-only development
  const [mockUsers] = useState<UserPresence[]>([
    {
      userId: 'user-456',
      userName: 'Alice Cooper',
      avatarColor: '#228be6',
      isOnline: true,
      cursorPosition: { x: 250, y: 150 },
      selectedNodeId: 'transform_1',
      lastActive: new Date().toISOString()
    },
    {
      userId: 'user-789',
      userName: 'Bob Smith',
      avatarColor: '#51cf66',
      isOnline: true,
      cursorPosition: { x: 450, y: 200 },
      lastActive: new Date().toISOString()
    }
  ])

  // Use static mock Plan DAG for frontend-only development
  const mockPlanDag = planDag || staticMockPlanDag

  // Create stable handler functions
  const handleNodeEdit = useCallback((nodeId: string) => {
    setNodes((currentNodes) => {
      const node = currentNodes.find(n => n.id === nodeId)
      if (!node) return currentNodes

      setConfigNodeId(nodeId)
      setConfigNodeType(node.data.nodeType)
      setConfigNodeConfig(node.data.config)
      setConfigNodeMetadata(node.data.metadata)
      setConfigDialogOpen(true)

      return currentNodes
    })
  }, [])

  const handleNodeDelete = useCallback((nodeId: string) => {
    setNodes((nodes) => nodes.filter((node) => node.id !== nodeId))
    setEdges((edges) => edges.filter((edge) => edge.source !== nodeId && edge.target !== nodeId))
    setIsDirty(true)
    console.log('Node deleted:', nodeId)
  }, [])

  // Initialize with ReactFlow data including handlers - only run once since handlers are stable
  const basicReactFlowData = useMemo(() => {
    return convertPlanDagToReactFlow(staticMockPlanDag, handleNodeEdit, handleNodeDelete)
  }, []) // Empty deps since handlers are stable and staticMockPlanDag is constant

  const [nodes, setNodes, onNodesChange] = useNodesState<ReactFlowNode>(basicReactFlowData.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState<ReactFlowEdge>(basicReactFlowData.edges)

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

  const handleNodeConfigSave = useCallback((nodeId: string, config: NodeConfig, metadata: NodeMetadata) => {
    setNodes((nodes) =>
      nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              data: {
                ...node.data,
                config,
                metadata,
                label: metadata.label, // Update the label for ReactFlow
                onEdit: handleNodeEdit,
                onDelete: handleNodeDelete,
              },
            }
          : node
      )
    )
    setIsDirty(true)
    console.log('Node configuration updated:', nodeId, config, metadata)
  }, [setNodes, handleNodeEdit, handleNodeDelete])

  // Handle viewport changes to track current zoom/pan state
  const handleViewportChange = useCallback((viewport: { x: number; y: number; zoom: number }) => {
    viewportRef.current = viewport
  }, [])

  // Handle mouse movement for cursor broadcasting
  const handleMouseMove = useCallback((event: React.MouseEvent) => {
    if (readonly) return

    const rect = event.currentTarget.getBoundingClientRect()
    const screenX = event.clientX - rect.left
    const screenY = event.clientY - rect.top

    // Convert screen coordinates to world coordinates for broadcasting
    const viewport = viewportRef.current
    const worldX = (screenX - viewport.x) / viewport.zoom
    const worldY = (screenY - viewport.y) / viewport.zoom

    broadcastCursorPosition(worldX, worldY, selectedNode || undefined)
  }, [broadcastCursorPosition, selectedNode, readonly])

  // Use stable nodeTypes reference to prevent ReactFlow warnings
  const nodeTypes = NODE_TYPES

  // Save Plan DAG changes (mock for frontend-only development)
  const savePlanDag = useCallback(async () => {
    if (!mockPlanDag || readonly) return

    const updatedPlanDag = convertReactFlowToPlanDag(nodes, edges, mockPlanDag.metadata)
    console.log('Would save Plan DAG:', updatedPlanDag)
    setIsDirty(false)
  }, [mockPlanDag, nodes, edges, readonly])

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

  if (!mockPlanDag) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} title="No Plan DAG found" color="yellow">
        This project doesn't have a Plan DAG configured yet.
      </Alert>
    )
  }

  return (
    <Stack h="100%" spacing={0}>
      <Group justify="space-between" p="md" bg="gray.0">
        <Group spacing="md">
          <Title order={3}>Plan DAG Editor</Title>
          <UserPresenceIndicator users={mockUsers} maxVisible={3} size="sm" />
        </Group>
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
              <IconPlayerPlay size="1rem" />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Settings">
            <ActionIcon variant="light" color="gray">
              <IconSettings size="1rem" />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>

      <div
        style={{ flex: 1, position: 'relative' }}
        onMouseMove={handleMouseMove}
      >
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onConnect={onConnect}
          onMove={handleViewportChange}
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
                {mockPlanDag.metadata.name || 'Untitled Plan'}
              </Text>
              <Text size="xs" c="dimmed">
                {nodes.length} nodes, {edges.length} connections
              </Text>
            </Stack>
          </Panel>
          {/* Collaborative Cursors Overlay - Must be inside ReactFlow */}
          <CollaborativeCursors
            users={mockUsers}
            currentUserId={currentUserId}
          />
        </ReactFlow>
      </div>

      {/* Node Configuration Dialog */}
      <NodeConfigDialog
        opened={configDialogOpen}
        onClose={() => setConfigDialogOpen(false)}
        nodeId={configNodeId}
        nodeType={configNodeType}
        config={configNodeConfig}
        metadata={configNodeMetadata}
        onSave={handleNodeConfigSave}
      />
    </Stack>
  )
}