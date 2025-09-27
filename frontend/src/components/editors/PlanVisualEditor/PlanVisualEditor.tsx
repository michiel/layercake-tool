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

import { usePlanDag, usePlanDagMutations, usePlanDagValidation, usePlanDagSubscription, useUserPresence, useCollaboration } from '../../../hooks/usePlanDag'
import { PlanDag, PlanDagNode, PlanDagEdge, ReactFlowNode, ReactFlowEdge, PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig } from '../../../types/plan-dag'
import {
  UserPresence,
  useCollaborationEventsSubscription,
  useConflictDetection,
  useCollaborationConnection,
  type ConflictEvent
} from '../../../hooks/useCollaborationSubscriptions'
import { CollaborationEvent } from '../../../graphql/subscriptions'
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

// Import dialogs
import { NodeConfigDialog } from './NodeConfigDialog'

// Import extracted components and hooks
import { ControlPanel } from './components/ControlPanel'
import { NodeToolbar } from './components/NodeToolbar'
// import { CollaborationManager } from './components/CollaborationManager'
import { useUpdateManagement } from './hooks/useUpdateManagement'
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

const PlanVisualEditorInner = ({ projectId, onNodeSelect, onEdgeSelect, readonly = false }: PlanVisualEditorProps) => {

  // Use real GraphQL queries to fetch Plan DAG data
  const { planDag, loading, error } = usePlanDag(projectId)
  const { lastChange } = usePlanDagSubscription(projectId)
  const mutations = usePlanDagMutations(projectId)

  // Phase 4: Plan DAG validation integration
  const { validate, validationResult, loading: validationLoading } = usePlanDagValidation()

  // Phase 3: Advanced collaboration hooks integration
  const currentUserId = 'user-123' // Mock current user for frontend-only development
  const { users } = useUserPresence(projectId, currentUserId)
  const { broadcastCursorPosition, joinProject, leaveProject } = useCollaboration(projectId)

  // Advanced collaboration features
  const { status: collaborationStatus, isConnected, hasError } = useCollaborationConnection(projectId.toString())
  const { getActiveConflicts } = useConflictDetection(projectId.toString())

  // Collaboration events state
  const [_collaborationEvents, setCollaborationEvents] = useState<CollaborationEvent[]>([])
  const [_activeConflicts, setActiveConflicts] = useState<ConflictEvent[]>([])
  const collaborationEventsRef = useRef<CollaborationEvent[]>([])

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

  // Handle collaboration events integration with controlled updates
  const handleCollaborationEvent = useCallback((event: CollaborationEvent) => {
    console.log('Collaboration event received:', event)

    // Add to events log (keep last 100 events)
    collaborationEventsRef.current = [event, ...collaborationEventsRef.current.slice(0, 99)]
    setCollaborationEvents([...collaborationEventsRef.current])

    // Handle different event types with controlled updates
    if (event.eventType === 'NODE_UPDATED' && event.data.nodeEvent) {
      // Throttled update for node changes from other users
      throttledUpdate(() => {
        console.log('Processing remote node update:', event.data.nodeEvent)
        // Node updates are handled via GraphQL subscriptions automatically
      })
    } else if (event.eventType === 'EDGE_CREATED' || event.eventType === 'EDGE_DELETED') {
      // Immediate update for edge changes (less frequent, more critical)
      debouncedUpdate(() => {
        console.log('Processing remote edge change:', event.eventType)
        // Edge updates are handled via GraphQL subscriptions automatically
      })
    }
  }, [throttledUpdate, debouncedUpdate])

  // Subscribe to collaboration events
  useCollaborationEventsSubscription(projectId.toString(), handleCollaborationEvent)

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


  // Stable reference pattern - only update when content actually changes
  const previousPlanDagRef = useRef<PlanDag | null>(null)
  const planDagStableRef = useRef<PlanDag>(staticMockPlanDag)


  // Phase 3: Conflict detection and resolution
  const checkForConflicts = useCallback(() => {
    const conflicts = getActiveConflicts()
    setActiveConflicts(conflicts)
    return conflicts
  }, [getActiveConflicts])

  // Auto-check for conflicts every 5 seconds
  useEffect(() => {
    if (!isConnected) return

    const intervalId = setInterval(() => {
      checkForConflicts()
    }, 5000)

    return () => clearInterval(intervalId)
  }, [isConnected, checkForConflicts])

  // Handle remote changes from subscriptions
  useEffect(() => {
    if (!lastChange || !isConnected) return

    console.log('Real-time subscription change detected:', lastChange)

    // Check for conflicts when remote changes come in
    const conflicts = checkForConflicts()
    if (conflicts.length > 0) {
      console.warn('Conflicts detected with remote changes:', conflicts)
      // Pause updates temporarily to handle conflicts
      pauseUpdates()
    }

    // Integrate remote change with controlled update system
    debouncedUpdate(() => {
      console.log('Processing real-time subscription update')
      // GraphQL subscription data is automatically merged by Apollo Client
      // This just ensures we don't miss any updates during controlled update periods
    })
  }, [lastChange, isConnected, checkForConflicts, pauseUpdates, debouncedUpdate])

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
        console.log('Plan DAG data changed, updating stable reference with controls and scheduling validation')
        previousPlanDagRef.current = currentData
        planDagStableRef.current = currentData

        // Phase 4: Schedule validation after data changes
        scheduleValidation(currentData)
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
  }, [useDynamicData, planDag, planDagEqual, throttledUpdate, scheduleValidation])

  // Handle validation trigger
  const handleValidate = useCallback(() => {
    if (validate) {
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
      broadcastCursorPosition(worldX, worldY, selectedNode || undefined)
    }
  }, [broadcastCursorPosition, selectedNode, readonly])

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
        type: 'dagNode',
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

      // TODO: The backend sync will be handled when the user saves or the component unmounts
      // For now, we just add to the local ReactFlow state
    },
    [screenToFlowPosition, setNodes, setIsDirty, handleNodeEdit, handleNodeDelete]
  );

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
      cleanupUpdateManagement()

      // Phase 4: Cleanup validation timer
      if (validationTimeoutRef.current) {
        clearTimeout(validationTimeoutRef.current)
      }
    }
  }, [readonly, joinProject, leaveProject, cleanupUpdateManagement])

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

      {/* Node Toolbar for drag-and-drop */}
      <NodeToolbar onNodeDragStart={handleNodeDragStart} readonly={readonly} />

      <div
        style={{ flex: 1, position: 'relative' }}
        onMouseMove={handleMouseMove}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
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
            isConnected={isConnected}
            collaborationStatus={collaborationStatus}
            hasError={hasError}
            onlineUsers={onlineUsers}
          />

          {/* Phase 3: Collaborative cursors for real-time user presence */}
          <CollaborativeCursors users={onlineUsers} currentUserId={currentUserId} />

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