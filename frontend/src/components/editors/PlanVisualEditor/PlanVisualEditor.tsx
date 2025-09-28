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
  OnMove,
  Viewport,
  useReactFlow,
  ReactFlowProvider
} from 'reactflow'
import { Stack, Title, Alert, Loader, Text, ActionIcon, Tooltip, Group } from '@mantine/core'
import {
  IconAlertCircle,
  IconEye,
  IconSettings,
  IconPlayerPlay
} from '@tabler/icons-react'

import { usePlanDag, usePlanDagMutations, usePlanDagValidation, usePlanDagSubscription } from '../../../hooks/usePlanDag'
import { useCollaborationV2 } from '../../../hooks/useCollaborationV2'
import { PlanDag, PlanDagNode, PlanDagEdge, ReactFlowNode, ReactFlowEdge, PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig } from '../../../types/plan-dag'
import { validateConnectionWithCycleDetection } from '../../../utils/planDagValidation'

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
import { UserPresenceData } from '../../../types/websocket'

// Import dialogs
import { NodeConfigDialog } from './NodeConfigDialog'

// Import extracted components and hooks
import { ControlPanel } from './components/ControlPanel'
import { NodeToolbar } from './components/NodeToolbar'
import { AdvancedToolbar } from './components/AdvancedToolbar'
import { ContextMenu } from './components/ContextMenu'
// import { CollaborationManager } from './components/CollaborationManager'
import { useUpdateManagement } from './hooks/useUpdateManagement'
import { useAdvancedOperations } from './hooks/useAdvancedOperations'
import { generateNodeId, getDefaultNodeConfig, getDefaultNodeMetadata } from './utils/nodeDefaults'

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

const PlanVisualEditorInner = ({ projectId, onNodeSelect, onEdgeSelect, readonly = false }: PlanVisualEditorProps) => {

  // Use real GraphQL queries to fetch Plan DAG data
  const { planDag, loading, error } = usePlanDag(projectId)
  const { lastChange } = usePlanDagSubscription(projectId)
  const mutations = usePlanDagMutations(projectId)

  // Phase 4: Plan DAG validation integration
  const { validate, validationResult, loading: validationLoading } = usePlanDagValidation()

  // Phase 3: Advanced collaboration hooks integration
  // TODO: Implement proper authentication and get current user ID
  const currentUserId: string | undefined = undefined

  // New WebSocket collaboration hook
  const collaboration = useCollaborationV2({
    projectId,
    documentId: 'plan-dag-canvas',
    documentType: 'canvas',
    enableWebSocket: true,
    userInfo: {
      id: currentUserId || 'anonymous',
      name: currentUserId ? `User ${currentUserId}` : 'Anonymous User',
      avatarColor: '#3b82f6'
    }
  })

  // Collaboration is now handled via WebSocket through CollaborationManager

  // Phase 4: Validation state and error tracking
  const [validationErrors, setValidationErrors] = useState<any[]>([])
  const [lastValidation, setLastValidation] = useState<Date | null>(null)
  const validationTimeoutRef = useRef<number | null>(null)

  // Phase 2: Update management using custom hook
  const {
    updatesPaused,
    pendingUpdates,
    throttledUpdate,
    debouncedUpdate,
    pauseUpdates,
    resumeUpdates,
    cleanup: cleanupUpdateManagement
  } = useUpdateManagement({
    throttleMs: 1000,
    debounceMs: 500,
    maxPendingUpdates: 10
  })

  // Real-time collaboration events are now handled via WebSocket

  // Real mutations are now available from usePlanDagMutations hook
  // mutations.moveNode, mutations.addEdge, mutations.deleteEdge, etc.

  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [_selectedEdge, setSelectedEdge] = useState<string | null>(null)

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

  // Use users directly from the new collaboration hook
  const onlineUsers: UserPresenceData[] = collaboration.users || []

  // Stable reference pattern - only update when content actually changes
  const previousPlanDagRef = useRef<PlanDag | null>(null)
  const planDagStableRef = useRef<PlanDag | null>(null)


  // Phase 3: Conflict detection will be handled via WebSocket events
  // Future implementation will leverage real-time collaboration events

  // Handle remote changes from subscriptions
  useEffect(() => {
    if (!lastChange || !collaboration.connected) return

    console.log('Real-time subscription change detected:', lastChange)

    // Conflict detection will be handled via WebSocket events in the future

    // Integrate remote change with controlled update system
    debouncedUpdate(() => {
      console.log('Processing real-time subscription update')
      // GraphQL subscription data is automatically merged by Apollo Client
      // This just ensures we don't miss any updates during controlled update periods
    })
  }, [lastChange, collaboration.connected, pauseUpdates, debouncedUpdate])

  // Phase 4: Validation integration with controlled updates
  const runValidation = useCallback(async (planDagToValidate: PlanDag) => {
    if (validationLoading) return

    console.log('Running Plan DAG validation')
    setLastValidation(new Date())

    try {
      const result = await validate(planDagToValidate)
      if (result.data?.validatePlanDag) {
        const validation = result.data.validatePlanDag
        setValidationErrors(validation.errors || [])

        if (!validation.isValid && validation.errors.length > 0) {
          console.warn('Plan DAG validation failed:', validation.errors)
          // Pause updates temporarily if there are critical validation errors
          const criticalErrors = validation.errors.filter(err =>
            err.message.includes('cycle') || err.message.includes('unreachable')
          )
          if (criticalErrors.length > 0) {
            pauseUpdates()
          }
        } else {
          console.log('Plan DAG validation passed')
        }
      }
    } catch (error) {
      console.error('Validation failed:', error)
      setValidationErrors([{ message: 'Validation service unavailable' }])
    }
  }, [validate, validationLoading, pauseUpdates])

  // Auto-validate after changes (debounced)
  const scheduleValidation = useCallback((planDagToValidate: PlanDag) => {
    if (validationTimeoutRef.current) {
      clearTimeout(validationTimeoutRef.current)
    }

    validationTimeoutRef.current = setTimeout(() => {
      runValidation(planDagToValidate)
    }, 2000) // Validate 2 seconds after last change
  }, [runValidation])

  // Update validation results when validationResult changes
  useEffect(() => {
    if (validationResult) {
      setValidationErrors(validationResult.errors || [])
    }
  }, [validationResult])

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
  const activePlanDag: PlanDag | null = useMemo(() => {
    if (!planDag) {
      console.log('No GraphQL data available - showing empty state')
      return null
    }

    console.log('Processing live GraphQL data with controlled updates')
    const currentData: PlanDag = {
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

    // Use controlled update mechanism for data changes
    const updateStableReference = () => {
      if (!planDagEqual(previousPlanDagRef.current, currentData)) {
        console.log('Plan DAG data changed, updating stable reference with controls and scheduling validation')
        previousPlanDagRef.current = currentData
        planDagStableRef.current = currentData

        // Phase 4: Schedule validation after data changes
        scheduleValidation(currentData)
      } else {
        console.log('Plan DAG data unchanged, using existing stable reference')
      }
    }

    // Apply throttling to prevent too frequent updates for live data
    throttledUpdate(updateStableReference)

    return planDagStableRef.current
  }, [planDag, planDagEqual, throttledUpdate, scheduleValidation])

  // Handle validation trigger
  const handleValidate = useCallback(() => {
    if (validate && activePlanDag) {
      validate(activePlanDag)
    }
  }, [validate, activePlanDag])

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
    if (!activePlanDag) {
      return { nodes: [], edges: [] }
    }
    return convertPlanDagToReactFlow(activePlanDag, stableHandleEdit, stableHandleDelete)
  }, [activePlanDag, stableHandleEdit, stableHandleDelete])

  const [nodes, setNodes, onNodesChange] = useNodesState(reactFlowData.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(reactFlowData.edges)

  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    opened: boolean;
    position: { x: number; y: number };
  }>({ opened: false, position: { x: 0, y: 0 } })

  // Advanced operations hook
  const advancedOps = useAdvancedOperations({
    nodes,
    edges,
    setNodes,
    setEdges,
    readonly,
  })

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

      // Use enhanced validation with cycle detection
      const isValid = validateConnectionWithCycleDetection(
        sourceNode.data.nodeType,
        targetNode.data.nodeType,
        nodes,
        edges,
        { source: connection.source!, target: connection.target! }
      )

      if (!isValid.isValid) {
        console.error('Invalid connection:', isValid.errorMessage)

        // Show user-friendly error notification (temporary alert)
        alert(`Connection Error: ${isValid.errorMessage}`)
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
[nodes, edges, readonly, mutations, setEdges]
  )

  // Validate connections in real-time during drag for visual feedback
  const isValidConnection = useCallback(
    (connection: Connection) => {
      const sourceNode = nodes.find((n) => n.id === connection.source)
      const targetNode = nodes.find((n) => n.id === connection.target)

      if (!sourceNode || !targetNode) return false

      const validation = validateConnectionWithCycleDetection(
        sourceNode.data.nodeType,
        targetNode.data.nodeType,
        nodes,
        edges,
        { source: connection.source!, target: connection.target! }
      )

      return validation.isValid
    },
    [nodes, edges]
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
      collaboration.broadcastCursorPosition(worldX, worldY, selectedNode || undefined)
    }
  }, [collaboration, selectedNode, readonly])

  // Drag and drop handlers for creating new nodes
  const handleNodeDragStart = useCallback((event: React.DragEvent, nodeType: PlanDagNodeType) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  }, []);

  const handleDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const { screenToFlowPosition } = useReactFlow();

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const nodeType = event.dataTransfer.getData('application/reactflow') as PlanDagNodeType;

      // Check if the dropped element is a valid node type
      if (!nodeType || !Object.values(PlanDagNodeType).includes(nodeType)) {
        return;
      }

      // Calculate the position where the node was dropped
      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      // Generate unique ID and default configuration
      const nodeId = generateNodeId(nodeType);
      const config = getDefaultNodeConfig(nodeType);
      const metadata = getDefaultNodeMetadata(nodeType);

      // Create new node with temporary data structure for ReactFlow
      const newNode: Node = {
        id: nodeId,
        type: nodeType,
        position,
        data: {
          nodeType,
          label: metadata.label,
          config,
          metadata,
          isUnconfigured: true, // Mark as unconfigured (will show orange highlight)
          onEdit: () => handleNodeEdit(nodeId),
          onDelete: () => handleNodeDelete(nodeId),
          readonly: false
        }
      };

      // Add the node to the ReactFlow state
      setNodes((nds) => nds.concat(newNode));
      setIsDirty(true);

      // Persist to database via GraphQL mutation
      const planDagNode: Partial<PlanDagNode> = {
        id: nodeId,
        nodeType,
        position,
        metadata,
        config: JSON.stringify(config)
      };

      mutations.addNode(planDagNode).catch(err => {
        console.error('Failed to add node to database:', err);
        // TODO: Show user-friendly error message
      });
    },
    [screenToFlowPosition, setNodes, setIsDirty, handleNodeEdit, handleNodeDelete, mutations]
  );

  // Context menu handlers
  const handleContextMenu = useCallback((event: React.MouseEvent) => {
    event.preventDefault();
    setContextMenu({
      opened: true,
      position: { x: event.clientX, y: event.clientY }
    });
  }, []);

  const handleCloseContextMenu = useCallback(() => {
    setContextMenu(prev => ({ ...prev, opened: false }));
  }, []);

  // Close context menu when clicking elsewhere
  const handleCanvasClick = useCallback(() => {
    if (contextMenu.opened) {
      handleCloseContextMenu();
    }
  }, [contextMenu.opened, handleCloseContextMenu]);

  // Memoize nodeTypes to prevent recreation warnings
  const nodeTypes = useMemo(() => NODE_TYPES, []);

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
      collaboration.joinProject()
    }

    return () => {
      if (!readonly) {
        collaboration.leaveProject()
      }

      // Phase 2: Cleanup update timers
      cleanupUpdateManagement()

      // Phase 4: Cleanup validation timer
      if (validationTimeoutRef.current) {
        clearTimeout(validationTimeoutRef.current)
      }
    }
  }, [readonly, collaboration, cleanupUpdateManagement])

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
      <Stack align="center" justify="center" h="100%" gap="md">
        <Alert icon={<IconAlertCircle size="1rem" />} title="No Plan DAG found" color="blue" w="100%" maw="500px">
          This project doesn't have a Plan DAG configured yet. Create one by adding nodes to the canvas using the toolbar.
        </Alert>
        <Text size="sm" c="dimmed" ta="center">
          Plan DAGs define data transformation workflows through connected nodes and edges.
        </Text>
      </Stack>
    )
  }

  return (
    <Stack h="100%" gap={0}>
      <Group justify="space-between" p="md" bg="gray.0">
        <Group gap="md">
          <Title order={3}>Plan DAG Editor</Title>
          <UserPresenceIndicator users={onlineUsers} connectionState={collaboration.connectionState} maxVisible={5} size="sm" />
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

      {/* Node Toolbar for drag-and-drop */}
      <NodeToolbar onNodeDragStart={handleNodeDragStart} readonly={readonly} />

      {/* Advanced Operations Toolbar */}
      <AdvancedToolbar
        selectedNodeCount={advancedOps.selectedNodes.length}
        hasClipboardData={advancedOps.hasClipboardData}
        clipboardInfo={advancedOps.clipboardInfo}
        canAlign={advancedOps.canAlign}
        canDistribute={advancedOps.canDistribute}
        readonly={readonly}
        onDuplicate={advancedOps.handleDuplicate}
        onCopy={advancedOps.handleCopy}
        onPaste={advancedOps.handlePaste}
        onCut={advancedOps.handleCut}
        onDelete={advancedOps.handleDelete}
        onSelectAll={advancedOps.handleSelectAll}
        onDeselectAll={advancedOps.handleDeselectAll}
        onAlignLeft={advancedOps.handleAlignLeft}
        onAlignRight={advancedOps.handleAlignRight}
        onAlignTop={advancedOps.handleAlignTop}
        onAlignBottom={advancedOps.handleAlignBottom}
        onAlignCenterHorizontal={() => advancedOps.handleAlignCenter('horizontal')}
        onAlignCenterVertical={() => advancedOps.handleAlignCenter('vertical')}
        onDistributeHorizontal={() => advancedOps.handleDistribute('horizontal')}
        onDistributeVertical={() => advancedOps.handleDistribute('vertical')}
      />

      <div
        style={{ flex: 1, position: 'relative' }}
        onMouseMove={handleMouseMove}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onContextMenu={handleContextMenu}
        onClick={handleCanvasClick}
      >
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onConnect={onConnect}
          isValidConnection={isValidConnection}
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

          <ControlPanel
            validationLoading={validationLoading}
            validationErrors={validationErrors}
            lastValidation={lastValidation}
            onValidate={handleValidate}
            updatesPaused={updatesPaused}
            pendingUpdates={pendingUpdates}
            onPauseUpdates={pauseUpdates}
            onResumeUpdates={resumeUpdates}
            isConnected={collaboration.connected}
            collaborationStatus={collaboration.connectionState}
            hasError={!!collaboration.error}
            onlineUsers={onlineUsers}
          />

          {/* Empty state overlay when no nodes exist */}
          {nodes.length === 0 && (
            <div
              style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                zIndex: 10,
                pointerEvents: 'none',
                textAlign: 'center',
              }}
            >
              <Stack align="center" gap="md" p="xl">
                <Alert
                  icon={<IconAlertCircle size="1rem" />}
                  title="Start building your Plan DAG"
                  color="blue"
                  style={{ maxWidth: '400px', pointerEvents: 'auto' }}
                >
                  Drag nodes from the toolbar on the left to begin creating your data transformation workflow.
                </Alert>
                <Text size="sm" c="dimmed" ta="center" style={{ maxWidth: '350px' }}>
                  Create connections between nodes to define how data flows through your pipeline.
                </Text>
              </Stack>
            </div>
          )}

          {/* Phase 3: Collaborative cursors for real-time user presence */}
          <CollaborativeCursors users={onlineUsers} currentUserId={currentUserId || undefined} />

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
        projectId={projectId}
        onSave={handleNodeConfigSave}
      />

      {/* Context Menu for advanced operations */}
      <ContextMenu
        opened={contextMenu.opened}
        onClose={handleCloseContextMenu}
        position={contextMenu.position}
        selectedNodeCount={advancedOps.selectedNodes.length}
        hasClipboardData={advancedOps.hasClipboardData}
        canAlign={advancedOps.canAlign}
        canDistribute={advancedOps.canDistribute}
        readonly={readonly}
        onDuplicate={advancedOps.handleDuplicate}
        onCopy={advancedOps.handleCopy}
        onPaste={advancedOps.handlePaste}
        onCut={advancedOps.handleCut}
        onDelete={advancedOps.handleDelete}
        onSelectAll={advancedOps.handleSelectAll}
        onDeselectAll={advancedOps.handleDeselectAll}
        onAlignLeft={advancedOps.handleAlignLeft}
        onAlignRight={advancedOps.handleAlignRight}
        onAlignTop={advancedOps.handleAlignTop}
        onAlignBottom={advancedOps.handleAlignBottom}
        onAlignCenterHorizontal={() => advancedOps.handleAlignCenter('horizontal')}
        onAlignCenterVertical={() => advancedOps.handleAlignCenter('vertical')}
        onDistributeHorizontal={() => advancedOps.handleDistribute('horizontal')}
        onDistributeVertical={() => advancedOps.handleDistribute('vertical')}
      />
    </Stack>
  )
}

export const PlanVisualEditor = (props: PlanVisualEditorProps) => {
  return (
    <ReactFlowProvider>
      <PlanVisualEditorInner {...props} />
    </ReactFlowProvider>
  );
};