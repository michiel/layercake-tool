import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Node,
  NodeChange,
  EdgeChange,
  ConnectionMode,
  Panel,
  OnMove,
  Viewport,
} from 'reactflow'
import { Stack, Title, Alert, Loader, Text, ActionIcon, Tooltip, Group } from '@mantine/core'
import { IconAlertCircle, IconEye, IconSettings, IconPlayerPlay, IconPlayerPause, IconRotate, IconCircleCheck, IconExclamationCircle } from '@tabler/icons-react'

import { usePlanDag, usePlanDagMutations, usePlanDagSubscription, useUserPresence, useCollaboration } from '../../../hooks/usePlanDag'
import { PlanDag, PlanDagNode, PlanDagEdge, ReactFlowNode, ReactFlowEdge, PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig } from '../../../types/plan-dag'
import { UserPresence } from '../../../hooks/useCollaborationSubscriptions'
import { validateConnection } from '../../../utils/planDagValidation'

// Import custom node types
import { DataSourceNode } from './nodes/DataSourceNode'
import { GraphNode } from './nodes/GraphNode'
import { TransformNode } from './nodes/TransformNode'
import { MergeNode } from './nodes/MergeNode'
import { CopyNode } from './nodes/CopyNode'
import { OutputNode } from './nodes/OutputNode'

// Import collaboration components
import { UserPresenceIndicator } from '../../collaboration/UserPresenceIndicator'
import { CollaborativeCursors } from '../../collaboration/CollaborativeCursors'

// Import dialogs
import { NodeConfigDialog } from './dialogs/NodeConfigDialog'

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
  [PlanDagNodeType.DATA_SOURCE]: DataSourceNode,
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
      nodeType: PlanDagNodeType.DATA_SOURCE,
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
      nodeType: PlanDagNodeType.TRANSFORM,
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
      nodeType: PlanDagNodeType.OUTPUT,
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
  planDag: PlanDag | any,
  onEdit?: (nodeId: string) => void,
  onDelete?: (nodeId: string) => void
): { nodes: ReactFlowNode[]; edges: ReactFlowEdge[] } => {
  const nodes: ReactFlowNode[] = planDag.nodes.map((node: any) => {
    // Convert string nodeType to enum if needed
    const nodeType = typeof node.nodeType === 'string' ?
      (Object.values(PlanDagNodeType) as string[]).includes(node.nodeType) ?
        node.nodeType as PlanDagNodeType : PlanDagNodeType.DATA_SOURCE
      : node.nodeType;

    return {
      ...node,
      nodeType,
      type: nodeType,
      data: {
        label: node.metadata.label,
        nodeType,
        config: typeof node.config === 'string' ? (() => {
          try {
            return JSON.parse(node.config)
          } catch (e) {
            console.warn('Failed to parse node config JSON:', node.config, e)
            return {}
          }
        })() : node.config,
        metadata: node.metadata,
        onEdit,
        onDelete,
      },
      draggable: true,
      selectable: true,
    }
  })

  const edges: ReactFlowEdge[] = planDag.edges.map((edge: any) => ({
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
    nodeType: node.data?.nodeType || node.nodeType || PlanDagNodeType.DATA_SOURCE,
    position: node.position,
    metadata: node.data?.metadata || node.metadata || { label: '' },
    config: typeof node.data?.config === 'string' ? node.data.config : JSON.stringify(node.data?.config || node.config || {}),
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

  // Use real GraphQL queries to fetch Plan DAG data
  const { planDag, loading, error } = usePlanDag(projectId)
  const { lastChange } = usePlanDagSubscription(projectId)
  const mutations = usePlanDagMutations(projectId)

  // Collaboration hooks - mock current user for frontend-only development
  // const currentUserId = 'user-123' // Unused for now
  const { users } = useUserPresence(projectId)
  const { broadcastCursorPosition, joinProject, leaveProject } = useCollaboration(projectId)

  // Real mutations are now available from usePlanDagMutations hook
  // mutations.moveNode, mutations.addEdge, mutations.deleteEdge, etc.

  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [selectedEdge, setSelectedEdge] = useState<string | null>(null)

  const [isDirty, setIsDirty] = useState(false)
  // const initializedRef = useRef(false) // Unused for now
  const viewportRef = useRef({ x: 0, y: 0, zoom: 1 })

  // Configuration dialog state
  const [configDialogOpen, setConfigDialogOpen] = useState(false)
  const [configNodeId, setConfigNodeId] = useState<string>('')
  const [configNodeType, setConfigNodeType] = useState<PlanDagNodeType>(PlanDagNodeType.DATA_SOURCE)
  const [configNodeConfig, setConfigNodeConfig] = useState<NodeConfig>({
    inputType: 'CSVNodesFromFile',
    source: '',
    dataType: 'Nodes',
    outputGraphRef: ''
  } as DataSourceNodeConfig)
  const [configNodeMetadata, setConfigNodeMetadata] = useState<NodeMetadata>({ label: '', description: '' })

  // Use real users from subscription, with fallback mock data for development
  const onlineUsers: UserPresence[] = users.length > 0 ? users : [
    {
      userId: 'user-456',
      userName: 'Alice Cooper',
      avatarColor: '#51cf66',
      isOnline: true,
      cursorPosition: { x: 250, y: 150 },
      selectedNodeId: 'transform_1',
      lastActive: new Date().toISOString()
    },
    {
      userId: 'user-789',
      userName: 'Bob Smith',
      avatarColor: '#339af0',
      isOnline: true,
      cursorPosition: { x: 450, y: 200 },
      selectedNodeId: undefined,
      lastActive: new Date().toISOString()
    }
  ]

  // Feature flag for dynamic data - can be controlled via environment or debug flag
  const useDynamicData = import.meta.env.VITE_USE_DYNAMIC_DATA === 'true' || false

  // Phase 2: Controlled update mechanisms
  const [updatesPaused, setUpdatesPaused] = useState(false)
  const [pendingUpdates, setPendingUpdates] = useState(0)
  const lastUpdateTimeRef = useRef<number>(0)
  const updateThrottleRef = useRef<NodeJS.Timeout | null>(null)
  const updateDebounceRef = useRef<NodeJS.Timeout | null>(null)

  // Update throttling configuration
  const UPDATE_THROTTLE_MS = 1000 // Minimum time between updates
  const UPDATE_DEBOUNCE_MS = 500  // Wait time after last change before applying
  const MAX_PENDING_UPDATES = 10  // Maximum queued updates

  // Stable reference pattern - only update when content actually changes
  const previousPlanDagRef = useRef<PlanDag | null>(null)
  const planDagStableRef = useRef<PlanDag>(staticMockPlanDag)

  // Controlled update functions
  const throttledUpdate = useCallback((updateFn: () => void) => {
    const now = Date.now()
    const timeSinceLastUpdate = now - lastUpdateTimeRef.current

    if (updatesPaused) {
      console.log('Updates paused, queuing update')
      setPendingUpdates(prev => Math.min(prev + 1, MAX_PENDING_UPDATES))
      return
    }

    if (timeSinceLastUpdate < UPDATE_THROTTLE_MS) {
      // Throttle: delay update until minimum time has passed
      if (updateThrottleRef.current) {
        clearTimeout(updateThrottleRef.current)
      }

      const delay = UPDATE_THROTTLE_MS - timeSinceLastUpdate
      updateThrottleRef.current = setTimeout(() => {
        lastUpdateTimeRef.current = Date.now()
        updateFn()
      }, delay)
    } else {
      // Can update immediately
      lastUpdateTimeRef.current = now
      updateFn()
    }
  }, [updatesPaused])

  const debouncedUpdate = useCallback((updateFn: () => void) => {
    if (updateDebounceRef.current) {
      clearTimeout(updateDebounceRef.current)
    }

    updateDebounceRef.current = setTimeout(() => {
      if (!updatesPaused) {
        updateFn()
      } else {
        setPendingUpdates(prev => Math.min(prev + 1, MAX_PENDING_UPDATES))
      }
    }, UPDATE_DEBOUNCE_MS)
  }, [updatesPaused])

  // Emergency pause function to stop all updates
  const pauseUpdates = useCallback(() => {
    console.log('Pausing all updates')
    setUpdatesPaused(true)
  }, [])

  const resumeUpdates = useCallback(() => {
    console.log('Resuming updates, processing', pendingUpdates, 'pending updates')
    setUpdatesPaused(false)
    setPendingUpdates(0)
  }, [pendingUpdates])

  // Deep equality check helper for plan DAG data
  const planDagEqual = useCallback((a: PlanDag | null, b: PlanDag | null): boolean => {
    if (a === b) return true
    if (!a || !b) return false

    return (
      a.version === b.version &&
      a.nodes.length === b.nodes.length &&
      a.edges.length === b.edges.length &&
      JSON.stringify(a.metadata) === JSON.stringify(b.metadata) &&
      a.nodes.every((nodeA, i) => {
        const nodeB = b.nodes[i]
        return nodeA && nodeB &&
               nodeA.id === nodeB.id &&
               nodeA.nodeType === nodeB.nodeType &&
               JSON.stringify(nodeA.position) === JSON.stringify(nodeB.position) &&
               JSON.stringify(nodeA.config) === JSON.stringify(nodeB.config)
      }) &&
      a.edges.every((edgeA, i) => {
        const edgeB = b.edges[i]
        return edgeA && edgeB &&
               edgeA.id === edgeB.id &&
               edgeA.source === edgeB.source &&
               edgeA.target === edgeB.target
      })
    )
  }, [])

  // Safe data selection with controlled updates and proper fallback
  const activePlanDag: PlanDag = useMemo(() => {
    let currentData: PlanDag

    if (!useDynamicData) {
      console.log('Using static mock data (dynamic data disabled)')
      currentData = staticMockPlanDag
    } else if (!planDag) {
      console.log('Using static mock data (no GraphQL data available)')
      currentData = staticMockPlanDag
    } else {
      console.log('Processing dynamic GraphQL data with controlled updates')
      currentData = {
        ...planDag,
        nodes: planDag.nodes.map((node: any) => ({
          ...node,
          nodeType: (typeof node.nodeType === 'string' &&
            (Object.values(PlanDagNodeType) as string[]).includes(node.nodeType)) ?
            node.nodeType as PlanDagNodeType : PlanDagNodeType.DATA_SOURCE
        })),
        edges: planDag.edges.map((edge: any) => ({
          ...edge,
          metadata: {
            ...edge.metadata,
            dataType: edge.metadata?.dataType || 'GraphData'
          }
        }))
      }
    }

    // Use controlled update mechanism for data changes
    const updateStableReference = () => {
      if (!planDagEqual(previousPlanDagRef.current, currentData)) {
        console.log('Plan DAG data changed, updating stable reference with controls')
        previousPlanDagRef.current = currentData
        planDagStableRef.current = currentData
      } else {
        console.log('Plan DAG data unchanged, using existing stable reference')
      }
    }

    // Apply throttling to prevent too frequent updates
    if (useDynamicData && planDag) {
      throttledUpdate(updateStableReference)
    } else {
      // For static data, update immediately without throttling
      updateStableReference()
    }

    return planDagStableRef.current
  }, [useDynamicData, planDag, planDagEqual, throttledUpdate])

  // Initialize ReactFlow state first with stable handlers
  const stableHandleEdit = useCallback((nodeId: string) => {
    console.log('Edit node:', nodeId)
    // Handle edit logic without causing re-renders
  }, [])

  const stableHandleDelete = useCallback((nodeId: string) => {
    console.log('Delete node:', nodeId)
    // Handle delete logic without causing re-renders
  }, [])

  // Use controlled data with stable conversion
  const reactFlowData = useMemo(() => {
    return convertPlanDagToReactFlow(activePlanDag, stableHandleEdit, stableHandleDelete)
  }, [activePlanDag, stableHandleEdit, stableHandleDelete])

  const [nodes, setNodes, onNodesChange] = useNodesState(reactFlowData.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(reactFlowData.edges)

  // Create handler functions that use the initialized state setters
  const handleNodeEdit = useCallback((nodeId: string) => {
    setNodes((currentNodes) => {
      const node = currentNodes.find(n => n.id === nodeId)
      if (!node) return currentNodes

      setConfigNodeId(nodeId)
      setConfigNodeType(node.data.nodeType)
      setConfigNodeConfig(typeof node.data.config === 'string' ?
        (() => {
          try {
            return JSON.parse(node.data.config)
          } catch (e) {
            console.warn('Failed to parse node config JSON:', node.data.config, e)
            return {
              inputType: 'CSVNodesFromFile',
              source: '',
              dataType: 'Nodes',
              outputGraphRef: ''
            } as DataSourceNodeConfig
          }
        })() : node.data.config)
      setConfigNodeMetadata(node.data.metadata)
      setConfigDialogOpen(true)

      return currentNodes
    })
  }, [])

  const handleNodeDelete = useCallback((nodeId: string) => {
    setNodes((nodes) => nodes.filter((node) => node.id !== nodeId))
    setEdges((edges) => edges.filter((edge) => edge.source !== nodeId && edge.target !== nodeId))
    setIsDirty(true)
    // Delete from backend
    mutations.deleteNode(nodeId)
    console.log('Node deleted:', nodeId)
  }, [mutations])

  // Removed problematic useEffect that was causing infinite loops

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

      // Only mark as dirty for non-position changes (position changes are handled separately)
      const hasNonPositionChanges = changes.some(change => change.type !== 'position')
      if (hasNonPositionChanges) {
        setIsDirty(true)
      }

      // Handle non-position changes
      changes.forEach((change) => {
        if (change.type === 'select') {
          const nodeId = change.selected ? change.id : null
          setSelectedNode(nodeId)
          onNodeSelect?.(nodeId)
        }
        // Note: Position changes are handled separately in onNodeDragStop
      })
    },
    [onNodesChange, onNodeSelect]
  )

  // Handle node drag end - save position only when dragging is complete
  const handleNodeDragStop = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      if (!readonly) {
        // Save the final position to backend
        mutations.moveNode(node.id, node.position)
        console.log('Node position saved:', node.id, node.position)
      }
    },
    [mutations, readonly]
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
    // Save changes to backend
    mutations.updateNode(nodeId, { config, metadata })
    console.log('Node configuration updated:', nodeId, config, metadata)
  }, [setNodes, handleNodeEdit, handleNodeDelete, mutations])

  // Handle viewport changes to track current zoom/pan state
  const handleViewportChange: OnMove = useCallback((_event, viewport: Viewport) => {
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
    if (!viewport) return // Guard against null viewport

    const worldX = (screenX - viewport.x) / viewport.zoom
    const worldY = (screenY - viewport.y) / viewport.zoom

    // Only broadcast if coordinates are valid numbers
    if (typeof worldX === 'number' && typeof worldY === 'number' &&
        !isNaN(worldX) && !isNaN(worldY) &&
        isFinite(worldX) && isFinite(worldY)) {
      broadcastCursorPosition(worldX, worldY, selectedNode || undefined)
    }
  }, [broadcastCursorPosition, selectedNode, readonly])

  // Use stable nodeTypes reference to prevent ReactFlow warnings
  const nodeTypes = NODE_TYPES

  // Save Plan DAG changes to backend
  const savePlanDag = useCallback(async () => {
    if (!activePlanDag || readonly) return

    const updatedPlanDag = convertReactFlowToPlanDag(nodes as ReactFlowNode[], edges as ReactFlowEdge[], activePlanDag.metadata)
    try {
      await mutations.updatePlanDag(updatedPlanDag)
      setIsDirty(false)
      console.log('Plan DAG saved successfully:', updatedPlanDag)
    } catch (error) {
      console.error('Failed to save Plan DAG:', error)
    }
  }, [activePlanDag, nodes, edges, readonly, mutations])

  // Auto-save on changes (debounced)
  useEffect(() => {
    if (!isDirty) return

    const timeoutId = setTimeout(savePlanDag, 2000) // Auto-save after 2 seconds
    return () => clearTimeout(timeoutId)
  }, [isDirty, savePlanDag])

  // Join/leave collaboration on mount/unmount
  useEffect(() => {
    if (!readonly) {
      joinProject().catch(err => {
        console.warn('Failed to join project collaboration:', err)
      })
    }

    return () => {
      if (!readonly) {
        leaveProject().catch(err => {
          console.warn('Failed to leave project collaboration:', err)
        })
      }

      // Phase 2: Cleanup update timers
      if (updateThrottleRef.current) {
        clearTimeout(updateThrottleRef.current)
      }
      if (updateDebounceRef.current) {
        clearTimeout(updateDebounceRef.current)
      }
    }
  }, [readonly, joinProject, leaveProject])

  const miniMapNodeColor = useCallback((node: Node) => {
    switch (node.data?.nodeType) {
      case 'DataSourceNode':
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
      <Stack align="center" justify="center" h="100%" gap="md">
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

  if (!activePlanDag) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} title="No Plan DAG found" color="yellow">
        This project doesn't have a Plan DAG configured yet.
      </Alert>
    )
  }

  return (
    <Stack h="100%" gap={0}>
      <Group justify="space-between" p="md" bg="gray.0">
        <Group gap="md">
          <Title order={3}>Plan DAG Editor</Title>
          <UserPresenceIndicator users={onlineUsers} maxVisible={5} size="sm" />
        </Group>
        <Group gap="xs">
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
          onNodeDragStop={handleNodeDragStop}
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
            <Stack gap="xs" p="xs" bg="white" style={{ borderRadius: 4, boxShadow: '0 2px 8px rgba(0,0,0,0.1)' }}>
              <Text size="sm" fw={500}>
                {activePlanDag.metadata.name || 'Untitled Plan'}
              </Text>
              <Text size="xs" c="dimmed">
                {nodes.length} nodes, {edges.length} connections
                {selectedEdge && ` - Edge ${selectedEdge} selected`}
              </Text>

              {/* Phase 2: Update Control Panel */}
              <Group gap="xs" mt="xs">
                <Text size="xs" fw={500} c="gray.6">Updates:</Text>
                <Group gap={4}>
                  <Tooltip label={useDynamicData ? "Dynamic data enabled" : "Static data mode"}>
                    <ActionIcon
                      size="xs"
                      variant="light"
                      color={useDynamicData ? "blue" : "gray"}
                    >
                      <IconRotate size="0.7rem" />
                    </ActionIcon>
                  </Tooltip>

                  <Tooltip label={updatesPaused ? "Resume updates" : "Pause updates"}>
                    <ActionIcon
                      size="xs"
                      variant="light"
                      color={updatesPaused ? "orange" : "green"}
                      onClick={updatesPaused ? resumeUpdates : pauseUpdates}
                    >
                      {updatesPaused ? <IconPlayerPlay size="0.7rem" /> : <IconPlayerPause size="0.7rem" />}
                    </ActionIcon>
                  </Tooltip>

                  {pendingUpdates > 0 && (
                    <Tooltip label={`${pendingUpdates} pending updates`}>
                      <ActionIcon size="xs" variant="light" color="orange">
                        <IconExclamationCircle size="0.7rem" />
                      </ActionIcon>
                    </Tooltip>
                  )}

                  {updatesPaused === false && pendingUpdates === 0 && (
                    <Tooltip label="Updates active">
                      <ActionIcon size="xs" variant="light" color="green">
                        <IconCircleCheck size="0.7rem" />
                      </ActionIcon>
                    </Tooltip>
                  )}
                </Group>
              </Group>
            </Stack>
          </Panel>

          {/* Collaborative cursors for real-time user presence */}
          <CollaborativeCursors users={onlineUsers} />

          {/* Collaboration features integration complete */}
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