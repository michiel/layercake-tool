import { useCallback, useEffect, useState, useRef, useMemo } from 'react'
import { gql } from '@apollo/client'
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  addEdge,
  Connection,
  Node,
  NodeChange,
  EdgeChange,
  ConnectionMode,
  OnMove,
  Viewport,
  useReactFlow,
  ReactFlowProvider,
  Edge,
  MarkerType
} from 'reactflow'
import {
  IconAlertCircle
} from '@tabler/icons-react'
import { Alert, AlertDescription } from '../../ui/alert'
import { Spinner } from '../../ui/spinner'
import { Card } from '../../ui/card'

import { PlanDagNodeType, NodeConfig, NodeMetadata, DataSetNodeConfig, ReactFlowEdge, PlanDagNode, PlanDag, StoryNodeConfig } from '../../../types/plan-dag'
import { validateConnectionWithCycleDetection, canAcceptMultipleInputs, isNodeConfigured, getValidTargetNodeTypes } from '../../../utils/planDagValidation'
import { ReactFlowAdapter } from '../../../adapters/ReactFlowAdapter'
import { getEdgeColor, getEdgeLabel } from '../../../utils/edgeStyles'
import type { EdgeDataType } from '../../../utils/edgeStyles'

// Import node types constant
import { NODE_TYPES } from './nodeTypes'

// Import collaboration components
import { CollaborativeCursors } from '../../collaboration/CollaborativeCursors'
import { UserPresenceData } from '../../../types/websocket'
import { PlanVisualEditorContext } from './context'

// Import dialogs
import { NodeConfigDialog } from './NodeConfigDialog'
import { EdgeConfigDialog } from './EdgeConfigDialog'
import { NodeTypeSelector, NODE_TYPE_SELECTOR_DEFAULTS } from './dialogs/NodeTypeSelector'

// Import extracted components and hooks
import { AdvancedToolbar } from './components/AdvancedToolbar'
import { usePlanDagCQRS } from './hooks/usePlanDagCQRS'
import { getDefaultNodeConfig, getDefaultNodeMetadata } from './utils/nodeDefaults'
import { autoLayout } from './utils/autoLayout'
import { useMutation } from '@apollo/client/react'
import { UPDATE_GRAPH } from '../../../graphql/graphs'
import { EXECUTE_PLAN, CLEAR_PROJECT_EXECUTION, STOP_PLAN_EXECUTION } from '../../../graphql/preview'
import { showSuccessNotification, showErrorNotification } from '../../../utils/notifications'
import { createApolloClientForEndpoint } from '@/graphql/client'

const DELETE_PROJECTION = gql`
  mutation DeleteProjection($id: ID!) {
    deleteProjection(id: $id)
  }
`

// Import floating edge components
import { FloatingEdge } from './edges/FloatingEdge'
import { FloatingConnectionLine } from './edges/FloatingConnectionLine'

// Import ReactFlow styles
import 'reactflow/dist/style.css'

interface PlanVisualEditorProps {
  projectId: number
  planId: number
  onNodeSelect?: (nodeId: string | null) => void
  onEdgeSelect?: (edgeId: string | null) => void
  readonly?: boolean
  focusNodeId?: string
  collaboration?: any // Project-level collaboration instance from App.tsx
}

const isTauri = !!(window as any).__TAURI__;

const parseNodeConfigValue = (value: any) => {
  if (typeof value === 'string') {
    try {
      return JSON.parse(value)
    } catch (error) {
      console.warn('[PlanVisualEditor] Failed to parse node config JSON', error)
      return value
    }
  }

  return value
}

const sanitizeNodeMetadata = (raw: any, fallback?: NodeMetadata): NodeMetadata => {
  if (raw && typeof raw === 'object') {
    const { label, description } = raw as any
    const metadata: NodeMetadata = {
      label: typeof label === 'string' ? label : fallback?.label ?? ''
    }

    const resolvedDescription =
      typeof description === 'string' ? description : fallback?.description
    if (resolvedDescription && resolvedDescription.length > 0) {
      metadata.description = resolvedDescription
    }

    return metadata
  }

  return fallback ? { ...fallback } : { label: '' }
}

const PlanVisualEditorInner = ({ projectId, planId, onNodeSelect, onEdgeSelect, readonly = false, focusNodeId, collaboration }: PlanVisualEditorProps) => {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [updateGraphNameMutation] = useMutation(UPDATE_GRAPH)
  const projectionsClient = useMemo(
    () =>
      createApolloClientForEndpoint({
        httpPath: '/projections/graphql',
        wsPath: '/projections/graphql/ws',
      }),
    []
  )
  const [deleteProjectionMutation] = useMutation(DELETE_PROJECTION, { client: projectionsClient })
  // Get ReactFlow instance for fit view and screen position conversion
  const { fitView, screenToFlowPosition } = useReactFlow();

  // Configuration dialog state - needs to be defined early
  const [configDialogOpen, setConfigDialogOpen] = useState(false)
  const [configNodeId, setConfigNodeId] = useState<string>('')
  const [configNodeType, setConfigNodeType] = useState<PlanDagNodeType>(PlanDagNodeType.DATA_SOURCE)
  const [configNodeConfig, setConfigNodeConfig] = useState<NodeConfig>({
    inputType: 'CSVNodesFromFile',
    source: '',
    dataType: 'Nodes',
    outputGraphRef: ''
  } as DataSetNodeConfig)
  const [configNodeMetadata, setConfigNodeMetadata] = useState<NodeMetadata>({ label: '', description: '' })
  const [configGraphIdHint, setConfigGraphIdHint] = useState<number | null>(null)
  const [configGraphSourceNodeId, setConfigGraphSourceNodeId] = useState<string | null>(null)
  const [draggingNode, setDraggingNode] = useState<{ type: PlanDagNodeType, position: { x: number, y: number } } | null>(null);

  // Node type selector for edge drop
  const [showNodeTypeMenu, setShowNodeTypeMenu] = useState(false)
  const [newNodePosition, setNewNodePosition] = useState<{ x: number; y: number } | null>(null)
  const connectionSourceRef = useRef<{ nodeId: string; handleId: string } | null>(null)
  const [allowedNodeTypes, setAllowedNodeTypes] = useState<PlanDagNodeType[]>(NODE_TYPE_SELECTOR_DEFAULTS)

  // Ref to store nodes for handleNodeEdit (to avoid circular dependency)
  const nodesRef = useRef<Node[]>([])
  const edgesRef = useRef<Edge[]>([])

  // Node action handlers (defined with stable references)
  const handleNodeEdit = useCallback((nodeId: string) => {
    console.log('Edit node triggered:', nodeId)
    setConfigNodeId(nodeId)

    // Find the node and populate config dialog state
    const node = nodesRef.current.find(n => n.id === nodeId)
    if (node) {
      console.log('Found node:', node)
      const resolvedType =
        (node.data?.nodeType as PlanDagNodeType | undefined) ||
        (node.type as PlanDagNodeType | undefined) ||
        PlanDagNodeType.DATA_SOURCE
      setConfigNodeType(resolvedType)
      setConfigNodeConfig(node.data.config || {})
      setConfigNodeMetadata(node.data.metadata || { label: '', description: '' })

      // Infer upstream graph ID for projection nodes
      const incoming = edgesRef.current.find(e => e.target === nodeId)
      if (incoming) {
        const sourceNode = nodesRef.current.find(n => n.id === incoming.source)
        const graphExec = (sourceNode as any)?.data?.graphExecution
        // Prefer graphDataId (graph_data tables) and fall back to legacy graphId
        const graphExecId = graphExec?.graphDataId ?? graphExec?.graphId
        setConfigGraphIdHint(typeof graphExecId === 'number' ? graphExecId : null)
        setConfigGraphSourceNodeId(sourceNode?.id ?? null)
      } else {
        setConfigGraphIdHint(null)
        setConfigGraphSourceNodeId(null)
      }
    } else {
      console.warn('Node not found:', nodeId)
    }

    setConfigDialogOpen(true)
  }, [])

  // Use ref for delete handler since it needs access to state/mutations defined later
  const deleteHandlerRef = useRef<((nodeId: string) => void) | null>(null)

  const handleNodeDelete = useCallback((nodeId: string) => {
    console.log('Node delete triggered:', nodeId)
    deleteHandlerRef.current?.(nodeId)
  }, [])

  // Use Plan DAG CQRS state management with delta subscriptions
  const planDagState = usePlanDagCQRS({
    projectId,
    planId,
    readonly,
    onNodeEdit: handleNodeEdit,
    onNodeDelete: handleNodeDelete,
  })

  // Extract state for easier access
  const {
    planDag,
    loading,
    error,
    nodes,
    edges,
    setNodes,
    setEdges,
    onNodesChange,
    onEdgesChange,
    updateManager,
    cqrsService,
    setDragging,
    updatePlanDagOptimistically,
    refreshData,
  } = planDagState

  // Keep nodesRef updated for handleNodeEdit
  useEffect(() => {
    nodesRef.current = nodes
  }, [nodes])

  useEffect(() => {
    edgesRef.current = edges
  }, [edges])

  const edgeSignature = useMemo(() => {
    return edges
      .map(edge => {
        const metadata = (edge.data as any)?.metadata
        return [
          edge.id,
          edge.source,
          edge.target,
          metadata?.dataType ?? '',
          metadata?.label ?? edge.label ?? ''
        ].join('|')
      })
      .join('::')
  }, [edges])

  useEffect(() => {
    setNodes((currentNodes) =>
      currentNodes.map(node => ({
        ...node,
        data: {
          ...node.data,
          edges
        }
      }))
    )
  }, [edgeSignature, edges, setNodes])

  useEffect(() => {
    if (!focusNodeId) return
    if (lastFocusedNodeIdRef.current === focusNodeId) return

    const targetNode = nodes.find(node => node.id === focusNodeId)
    if (!targetNode) return

    lastFocusedNodeIdRef.current = focusNodeId

    setNodes((currentNodes) => currentNodes.map(node => ({
      ...node,
      selected: node.id === focusNodeId
    })))
    setSelectedNode(focusNodeId)
    onNodeSelect?.(focusNodeId)

    requestAnimationFrame(() => {
      fitView({ nodes: [targetNode], padding: 0.25 })
    })
  }, [focusNodeId, nodes, setNodes, fitView, onNodeSelect])

  // Get mutations from CQRS service (includes delta generation)
  // Adapt CQRS command interface to match old mutation interface
  const mutations = {
    addNode: (node: Partial<PlanDagNode>) =>
      cqrsService.commands.createNode({ projectId, planId, nodeType: node.nodeType || 'DataSet', node }),
    addEdge: (edge: Partial<any>) =>
      cqrsService.commands.createEdge({ projectId, planId, edge: edge as any }),
    updateNode: (nodeId: string, updates: Partial<PlanDagNode>) => {
      const serializedUpdates = { ...updates } as any
      if (serializedUpdates.config !== undefined) {
        serializedUpdates.config =
          typeof serializedUpdates.config === 'string'
            ? serializedUpdates.config
            : JSON.stringify(serializedUpdates.config ?? {})
      }
      return cqrsService.commands.updateNode({ projectId, planId, nodeId, updates: serializedUpdates })
    },
    deleteNode: (nodeId: string) =>
      cqrsService.commands.deleteNode({ projectId, planId, nodeId }),
    deleteEdge: (edgeId: string) =>
      cqrsService.commands.deleteEdge({ projectId, planId, edgeId }),
    moveNode: (nodeId: string, position: { x: number; y: number }) =>
      cqrsService.commands.moveNode({ projectId, planId, nodeId, position }),
    updatePlanDag: (planDag: any) =>
      cqrsService.commands.updatePlanDag({ projectId, planId, planDag }),
  }

  // Setup delete handler with access to mutations
  useEffect(() => {
    deleteHandlerRef.current = (nodeId: string) => {
      console.log('Executing delete for node:', nodeId)

      // Suppress external syncs during delete operations
      setDragging(true)

      // If deleting a projection node, also remove its backing projection
      const targetNode = nodesRef.current.find(node => node.id === nodeId)
      const nodeType = targetNode?.data?.nodeType || targetNode?.type
      if (nodeType === PlanDagNodeType.PROJECTION) {
        const projectionId = (targetNode as any)?.data?.config?.projectionId
        if (projectionId) {
          deleteProjectionMutation({
            variables: { id: projectionId.toString() },
          }).catch(err => console.error('Failed to delete projection for node', nodeId, err))
        }
      }

      // Collect edge IDs to delete BEFORE modifying state
      // (Must access edges state directly to avoid closure issues)
      const edgesToDelete = edges.filter(edge => edge.source === nodeId || edge.target === nodeId)
      const backendEdgeIdsToDelete = edgesToDelete.map(edge =>
        (edge.data as any)?.originalEdge?.id || edge.id
      )

      // Remove node from local state optimistically
      setNodes((nds) => nds.filter(node => node.id !== nodeId))

      // Remove edges connected to this node
      setEdges((eds) => eds.filter(edge => edge.source !== nodeId && edge.target !== nodeId))

      // Keep Plan DAG state in sync to prevent stale external sync overwrites
      updatePlanDagOptimistically((current) => {
        if (!current) return current
        return {
          ...current,
          nodes: current.nodes.filter(node => node.id !== nodeId),
          edges: current.edges.filter(edge => edge.source !== nodeId && edge.target !== nodeId),
          metadata: {
            ...current.metadata,
            lastModified: new Date().toISOString()
          }
        }
      })

      // Persist deletions to backend
      mutations.deleteNode(nodeId)
      backendEdgeIdsToDelete.forEach(edgeId => mutations.deleteEdge(edgeId))

      // Re-enable external syncs after a short delay to allow mutations to complete
      setTimeout(() => setDragging(false), 100)
    }
  }, [setNodes, setEdges, mutations, setDragging, edges, deleteProjectionMutation, updatePlanDagOptimistically])

  // Switch to plan-dag-canvas document when component mounts
  useEffect(() => {
    if (collaboration?.switchDocument) {
      collaboration.switchDocument('plan-dag-canvas', 'canvas')
    }
  }, [collaboration])

  // Other UI state
  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [_selectedEdge, setSelectedEdge] = useState<string | null>(null)
  const viewportRef = useRef({ x: 0, y: 0, zoom: 1 })
  const lastFocusedNodeIdRef = useRef<string | null>(null)

  // Edge configuration dialog state
  const [edgeConfigDialogOpen, setEdgeConfigDialogOpen] = useState(false)
  const [configEdge, setConfigEdge] = useState<ReactFlowEdge | null>(null)

  // Use users directly from the collaboration prop
  const onlineUsers: UserPresenceData[] = collaboration?.users || []



  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    opened: boolean;
    position: { x: number; y: number };
  }>({ opened: false, position: { x: 0, y: 0 } })

  // Advanced operations removed - keeping only auto-layout and fit view

  // Handle node changes (position, selection, etc.)
  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      // PERFORMANCE: Always apply changes to ReactFlow for visual updates (fast)
      onNodesChange(changes)

      // PERFORMANCE FIX (Phase 1.2): Skip expensive processing during drag
      // Position and dimension changes during drag are cosmetic only - actual save happens in handleNodeDragStop
      // This prevents triggering validation, sync, and other side effects on every mouse move
      if (isDragging.current) {
        const hasSignificantChanges = changes.some(change =>
          change.type !== 'position' && change.type !== 'dimensions'
        )
        if (!hasSignificantChanges) {
          // All changes are position/dimension updates during drag - skip side effects
          return
        }
      }

      // Track performance for significant changes only (not position updates during drag)
      planDagState.performanceMonitor.trackEvent('nodeChanges')

      // DELTA MIGRATION: Disabled bulk update - granular mutations generate deltas
      // Handle structural changes with unified update manager
      // const hasStructuralChanges = changes.some(change =>
      //   change.type !== 'position' && change.type !== 'select'
      // )

      // if (hasStructuralChanges && planDag) {
      //   updateManager.scheduleStructuralUpdate(planDag, 'node-structural-change')
      // }

      // Handle selection changes
      changes.forEach((change) => {
        if (change.type === 'select') {
          const nodeId = change.selected ? change.id : null
          setSelectedNode(nodeId)
          onNodeSelect?.(nodeId)
        }
      })
    },
    [onNodesChange, onNodeSelect, updateManager, planDag, planDagState.performanceMonitor]
  )

  // Track initial positions when drag starts
  const dragStartPositions = useRef<Record<string, { x: number; y: number }>>({})
  const isDragging = useRef(false)

  // Handle flow node drag start - track initial position
  const handleFlowNodeDragStart = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      isDragging.current = true
      setDragging(true) // Suppress external syncs during drag
      dragStartPositions.current[node.id] = { ...node.position }
    },
    [setDragging]
  )

  // Handle node drag end - save position only when position actually changed
  const handleNodeDragStop = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      isDragging.current = false

      if (!readonly) {
        const initialPosition = dragStartPositions.current[node.id]
        if (initialPosition) {
          // Increased threshold to reduce excessive position updates (5-10px as recommended)
          const threshold = 8 // pixels
          const hasMovedX = Math.abs(node.position.x - initialPosition.x) > threshold
          const hasMovedY = Math.abs(node.position.y - initialPosition.y) > threshold

          if (hasMovedX || hasMovedY) {
            // Track performance for position updates
            planDagState.performanceMonitor.trackEvent('positionUpdates')

            // DELTA MIGRATION: Disabled bulk update - granular moveNode mutation generates deltas
            // updateManager.scheduleCosmeticUpdate(planDag!, 'node-position-change')

            // Update via granular mutation for delta-based sync
            mutations.moveNode(node.id, node.position)
            console.log('Node position saved:', node.id, node.position)

            // Optimistically update both ReactFlow nodes AND planDag (prevents stale sync)
            setNodes((nds) =>
              nds.map((n) =>
                n.id === node.id
                  ? { ...n, position: node.position }
                  : n
              )
            )

            // Also update planDag to prevent sync from overwriting with stale positions
            updatePlanDagOptimistically((current) => {
              if (!current) return current

              return {
                ...current,
                nodes: current.nodes.map((n) =>
                  n.id === node.id
                    ? { ...n, position: node.position }
                    : n
                )
              }
            })

            // Re-enable external syncs after a short delay to allow mutation to complete
            setTimeout(() => setDragging(false), 100)
          } else {
            console.log('Node not moved significantly, skipping position save:', node.id)
            // Re-enable external syncs immediately if no mutation was sent
            setDragging(false)
          }

          // Clean up tracking
          delete dragStartPositions.current[node.id]
        } else {
          // Re-enable external syncs if no initial position was tracked
          setDragging(false)
        }
      } else {
        // Re-enable external syncs if readonly
        setDragging(false)
      }
    },
    [mutations, readonly, updateManager, planDag, planDagState.performanceMonitor, setDragging, setNodes, updatePlanDagOptimistically]
  )

  // Note: Node editing is handled by individual icon clicks within node components

  // Handle edge double-click - open configuration dialog
  const handleEdgeDoubleClick = useCallback(
    (_event: React.MouseEvent, edge: any) => {
      if (!readonly) {
        setConfigEdge(edge as ReactFlowEdge)
        setEdgeConfigDialogOpen(true)
      }
    },
    [readonly]
  )

  // Handle edge update
  const handleEdgeUpdate = useCallback(
    async (edgeId: string, updates: Partial<ReactFlowEdge>) => {
      if (!planDag) return

      try {
        // Create updated plan DAG with modified edge
        const updatedPlanDag = {
          ...planDag,
          edges: planDag.edges.map((edge: any) =>
            edge.id === edgeId
              ? {
                  ...edge,
                  metadata: {
                    ...edge.metadata,
                    ...updates.metadata
                  }
                }
              : edge
          )
        }

        updatePlanDagOptimistically(() => ({
          ...updatedPlanDag,
          metadata: {
            ...updatedPlanDag.metadata,
            lastModified: new Date().toISOString()
          }
        }))

        // Update the entire plan DAG (this is how edge updates work)
        await mutations.updatePlanDag(updatedPlanDag)
        console.log('Edge updated successfully:', edgeId)
      } catch (error) {
        console.error('Failed to update edge:', error)
      }
    },
    [planDag, mutations, updatePlanDagOptimistically]
  )

  // Edge reconnection handlers
  const edgeReconnectSuccessful = useRef(true)

  const handleReconnectStart = useCallback(() => {
    edgeReconnectSuccessful.current = false
  }, [])

  const handleReconnect = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      if (readonly) return

      edgeReconnectSuccessful.current = true

      // Validate the new connection
      const sourceNode = nodes.find((n) => n.id === newConnection.source)
      const targetNode = nodes.find((n) => n.id === newConnection.target)

      if (!sourceNode || !targetNode) {
        console.error('[handleReconnect] Source or target node not found')
        return
      }

      const isValid = validateConnectionWithCycleDetection(
        sourceNode.data.nodeType,
        targetNode.data.nodeType,
        nodes,
        edges.filter(e => e.id !== oldEdge.id), // Exclude old edge from cycle check
        { source: newConnection.source!, target: newConnection.target! }
      )

      if (!isValid.isValid) {
        console.error('[handleReconnect] Invalid connection:', isValid.errorMessage)
        alert(`Connection Error: ${isValid.errorMessage}`)
        return
      }

      // Generate new edge ID (simplified since no handles)
      const newEdgeId = `${newConnection.source}-${newConnection.target}-${Date.now()}`

      // Create the new edge with the updated connection (floating edges don't use handles)
      const resolvedLabel = getEdgeLabel(isValid.dataType)
      const newEdge: ReactFlowEdge = {
        id: newEdgeId,
        source: newConnection.source!,
        target: newConnection.target!,
        metadata: {
          label: resolvedLabel,
          dataType: isValid.dataType,
        }
      }
      const resolvedColor = getEdgeColor(isValid.dataType)

      // Update local state - remove old edge and add new one
      setEdges((els) => {
        const filtered = els.filter(e => e.id !== oldEdge.id)
        return addEdge({
          ...newEdge,
          type: 'floating',
          animated: false,
          style: {
            stroke: resolvedColor,
            strokeWidth: 2,
          },
          data: { metadata: newEdge.metadata }
        }, filtered)
      })

      // Persist to backend: delete old edge and create new one
      // The edge ID in backend is stored in originalEdge.id
      const oldEdgeId = (oldEdge.data as any)?.originalEdge?.id || oldEdge.id
      mutations.deleteEdge(oldEdgeId)
      updatePlanDagOptimistically((current) => {
        if (!current) return current
        return {
          ...current,
          edges: current.edges.filter(edge => edge.id !== oldEdgeId && edge.id !== oldEdge.id),
          metadata: {
            ...current.metadata,
            lastModified: new Date().toISOString()
          }
        }
      })
      mutations.addEdge(newEdge).then((createdEdge) => {
        updatePlanDagOptimistically((current) => {
          if (!current) return current
          if (current.edges.some(edge => edge.id === createdEdge.id)) {
            return current
          }
          return {
            ...current,
            edges: [...current.edges, createdEdge],
            metadata: {
              ...current.metadata,
              lastModified: new Date().toISOString()
            }
          }
        })
      })

      console.log('Edge reconnected - deleted:', oldEdgeId, 'created:', newEdge.id)
    },
    [readonly, setEdges, mutations, nodes, edges, updatePlanDagOptimistically]
  )

  const handleReconnectEnd = useCallback(
    (_: MouseEvent | TouchEvent, edge: Edge) => {
      if (!edgeReconnectSuccessful.current && !readonly) {
        // Edge was dropped on empty space - delete it
        const edgeIdToDelete = (edge.data as any)?.originalEdge?.id || edge.id
        setEdges((eds) => eds.filter((e) => e.id !== edge.id))
        updatePlanDagOptimistically((current) => {
          if (!current) return current
          return {
            ...current,
            edges: current.edges.filter(e => e.id !== edgeIdToDelete && e.id !== edge.id),
            metadata: {
              ...current.metadata,
              lastModified: new Date().toISOString()
            }
          }
        })
        mutations.deleteEdge(edgeIdToDelete)
        console.log('Edge deleted on drop:', edgeIdToDelete)
      }
      edgeReconnectSuccessful.current = true
    },
    [readonly, setEdges, mutations, updatePlanDagOptimistically]
  )

  // Handle edge changes
  const handleEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      // Track performance for edge changes
      planDagState.performanceMonitor.trackEvent('edgeChanges')

      onEdgesChange(changes)

      // DELTA MIGRATION: Disabled bulk update - granular mutations generate deltas
      // Use unified update manager for edge changes
      // if (planDag) {
      //   updateManager.scheduleStructuralUpdate(planDag, 'edge-change')
      // }

      changes.forEach((change) => {
        if (change.type === 'remove' && !readonly) {
          // Find the edge to get the backend ID
          const edge = edges.find(e => e.id === change.id)
          if (edge) {
            const backendEdgeId = (edge.data as any)?.originalEdge?.id || edge.id

            // Remove from local state immediately for instant visual feedback
            setEdges((eds) => eds.filter((e) => e.id !== change.id))

            // Keep Plan DAG state in sync to prevent stale external sync overwrites
            updatePlanDagOptimistically((current) => {
              if (!current) return current
              return {
                ...current,
                edges: current.edges.filter(e => e.id !== backendEdgeId && e.id !== change.id),
                metadata: {
                  ...current.metadata,
                  lastModified: new Date().toISOString()
                }
              }
            })

            // Persist deletion to backend
            mutations.deleteEdge(backendEdgeId)
            console.log('Edge deleted via DEL key:', backendEdgeId)
          }
        }
        if (change.type === 'select') {
          const edgeId = change.selected ? change.id : null
          setSelectedEdge(edgeId)
          onEdgeSelect?.(edgeId)
        }
      })
    },
    [onEdgesChange, mutations, onEdgeSelect, readonly, planDagState.performanceMonitor, planDag, updateManager, edges, setEdges, updatePlanDagOptimistically]
  )

  // Handle new connections
  const onConnect = useCallback(
    (connection: Connection) => {
      console.log('[PlanVisualEditor] onConnect called:', connection)

      if (readonly) {
        console.log('[PlanVisualEditor] Connection blocked: readonly mode')
        return
      }

      // Validate the connection
      const sourceNode = nodes.find((n) => n.id === connection.source)
      const targetNode = nodes.find((n) => n.id === connection.target)

      if (!sourceNode || !targetNode) {
        console.error('[PlanVisualEditor] Connection failed: source or target node not found', {
          source: connection.source,
          target: connection.target,
          sourceNode: !!sourceNode,
          targetNode: !!targetNode
        })
        return
      }

      // Check if target already has maximum inputs (e.g., GraphNodes can only have one input)
      const targetInputs = edges.filter(e => e.target === connection.target)
      const targetCanAcceptMultiple = canAcceptMultipleInputs(targetNode.data.nodeType)

      if (!targetCanAcceptMultiple && targetInputs.length >= 1) {
        const errorMsg = `${targetNode.data.nodeType} nodes can only have one input connection. Disconnect the existing input first, or use a Merge node to combine multiple sources.`
        console.error('Invalid connection:', errorMsg)
        alert(`Connection Error: ${errorMsg}`)
        return
      }

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

      const newEdge = {
        id: `edge-${Date.now()}`,
        source: connection.source!,
        target: connection.target!,
        // Floating edges don't use sourceHandle/targetHandle
        type: 'floating',
        animated: false,
        style: {
          stroke: getEdgeColor(isValid.dataType),
          strokeWidth: 2,
        },
        data: {
          metadata: {
            label: getEdgeLabel(isValid.dataType),
            dataType: isValid.dataType,
          }
        }
      }

      // Optimistic update - add edge immediately for instant feedback
      setEdges((eds) => addEdge(newEdge, eds))

      // Persist to backend - subscription will confirm or correct
      // Do not send ID - backend will generate it
      const graphqlEdge: Partial<ReactFlowEdge> = {
        // Omit id - backend will generate it
        source: newEdge.source,
        target: newEdge.target,
        // Floating edges don't use sourceHandle/targetHandle
        metadata: newEdge.data.metadata
      }

      // Don't suppress sync during edge creation - we need to see updates
      mutations.addEdge(graphqlEdge).then((createdEdge) => {
        updatePlanDagOptimistically((current) => {
          if (!current) return current
          if (current.edges.some(edge => edge.id === createdEdge.id)) {
            return current
          }
          return {
            ...current,
            edges: [...current.edges, createdEdge],
            metadata: {
              ...current.metadata,
              lastModified: new Date().toISOString()
            }
          }
        })
      }).catch(err => {
        console.error('[PlanVisualEditor] Failed to create edge:', err)
        // Remove optimistic edge on failure
        setEdges((eds) => eds.filter(e => e.id !== newEdge.id))
        alert(`Failed to create connection: ${err.message}`)
      })
    },
    [nodes, edges, readonly, mutations, setEdges, updatePlanDagOptimistically]
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

  const handleNodeConfigSave = useCallback(
    async (nodeId: string, config: NodeConfig, metadata: NodeMetadata) => {
      const preparedConfig = parseNodeConfigValue(config)
      const sanitizedMetadata = sanitizeNodeMetadata(metadata)

      setNodes((nodes) =>
        nodes.map((node) =>
          node.id === nodeId
            ? {
                ...node,
                data: {
                  ...node.data,
                  config: preparedConfig,
                  metadata: sanitizedMetadata,
                  label: sanitizedMetadata.label,
                  onEdit: () => handleNodeEdit(nodeId),
                  onDelete: () => handleNodeDelete(nodeId)
                }
              }
            : node
        )
      )

      updatePlanDagOptimistically((current) => {
        if (!current) return current
        return {
          ...current,
          nodes: current.nodes.map(node =>
            node.id === nodeId
              ? {
                  ...node,
                  config: preparedConfig,
                  metadata: sanitizedMetadata
                }
              : node
          ),
          metadata: {
            ...current.metadata,
            lastModified: new Date().toISOString()
          }
        }
      })

      const nodeRecord = nodesRef.current.find((node) => node.id === nodeId)
      const nodeType = nodeRecord?.data?.nodeType as PlanDagNodeType | undefined
      const existingLabel = (nodeRecord?.data?.metadata?.label || '').trim()
      const nextLabel = sanitizedMetadata.label?.trim()
      const graphIdForNode =
        nodeRecord?.data?.graphExecution?.graphDataId ??
        nodeRecord?.data?.graphExecution?.graphId

      try {
        if (
          nodeType === PlanDagNodeType.GRAPH &&
          graphIdForNode &&
          nextLabel &&
          nextLabel.length > 0 &&
          nextLabel !== existingLabel
        ) {
          await updateGraphNameMutation({
            variables: {
              id: graphIdForNode,
              input: { name: nextLabel }
            }
          })
        }

        const updatedNode = await mutations.updateNode(nodeId, {
          config: preparedConfig,
          metadata: sanitizedMetadata
        })

        if (updatedNode) {
          const serverConfig = parseNodeConfigValue(updatedNode.config)
          const serverMetadata = sanitizeNodeMetadata(updatedNode.metadata, sanitizedMetadata)

          setNodes((nodes) =>
            nodes.map((node) =>
              node.id === nodeId
                ? {
                    ...node,
                    data: {
                      ...node.data,
                      config: serverConfig,
                      metadata: serverMetadata,
                      label: serverMetadata.label,
                      onEdit: () => handleNodeEdit(nodeId),
                      onDelete: () => handleNodeDelete(nodeId)
                    }
                  }
                : node
            )
          )

          updatePlanDagOptimistically((current) => {
            if (!current) return current
            return {
              ...current,
              nodes: current.nodes.map(node =>
                node.id === nodeId
                  ? {
                      ...node,
                      config: serverConfig,
                      metadata: serverMetadata
                    }
                  : node
              ),
              metadata: {
                ...current.metadata,
                lastModified: new Date().toISOString()
              }
            }
          })

          console.log('[PlanVisualEditor] Node configuration synchronized with backend:', nodeId)
        }
      } catch (error: any) {
        console.error('[PlanVisualEditor] Failed to update node configuration:', error)
        alert(`Failed to update node configuration: ${error.message || error}`)
      }
    },
    [handleNodeDelete, handleNodeEdit, mutations, setNodes, updateGraphNameMutation, updatePlanDagOptimistically]
  )

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
      collaboration?.broadcastCursorPosition(worldX, worldY, selectedNode || undefined)
    }
  }, [collaboration, selectedNode, readonly])

  // Drag and drop handlers for creating new nodes
  const handleNodeDragStart = useCallback((event: React.DragEvent, nodeType: PlanDagNodeType) => {
    if (isTauri) return;
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  }, []);

  const handleDragOver = useCallback((event: React.DragEvent) => {
    if (isTauri) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
      if (isTauri) return;
      event.preventDefault();

      const nodeType = event.dataTransfer.getData('application/reactflow') as PlanDagNodeType;

      console.log('[PlanVisualEditor] Node dropped:', { nodeType, allTypes: Object.values(PlanDagNodeType) });

      // Check if the dropped element is a valid node type
      if (!nodeType || !Object.values(PlanDagNodeType).includes(nodeType)) {
        console.warn('[PlanVisualEditor] Invalid node type dropped:', nodeType);
        return;
      }

      // Calculate the position where the node was dropped
      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      // Get default configuration and metadata
      const config = getDefaultNodeConfig(nodeType);
      const metadata = getDefaultNodeMetadata(nodeType);

      // Persist to database via GraphQL mutation - backend will generate ID
      const planDagNode: Partial<PlanDagNode> = {
        // Do not set id - backend will generate it
        nodeType,
        position,
        metadata,
        config: JSON.stringify(config)
      };

      // First try to add the node directly, then add to ReactFlow with backend-generated ID
      mutations.addNode(planDagNode).then((createdNode) => {
        // Use the ID generated by the backend
        const backendNodeId = createdNode.id;

        // Create new node with backend-generated ID
        const newNode: Node = {
          id: backendNodeId,
          type: nodeType,
          position,
          data: {
            nodeType,
            label: metadata.label,
            config,
            metadata,
            isUnconfigured: true, // Mark as unconfigured (will show orange highlight)
            onEdit: () => handleNodeEdit(backendNodeId),
            onDelete: () => handleNodeDelete(backendNodeId),
            readonly: false,
            edges: edges, // Required by nodes that check edge configuration
            projectId: projectId // Required by artefact nodes for export mutations
          }
        };

        // Add the node to the ReactFlow state with backend ID
        setNodes((nds) => nds.concat(newNode));

        updatePlanDagOptimistically((current) => {
          if (!current) return current
          if (current.nodes.some(node => node.id === backendNodeId)) {
            return current
          }
          return {
            ...current,
            nodes: [...current.nodes, createdNode],
            metadata: {
              ...current.metadata,
              lastModified: new Date().toISOString()
            }
          }
        })
      }).catch(async (err) => {
        console.error('Failed to add node to database:', err);

        // If the error is "Plan not found for project", initialize an empty plan first
        if (err.message?.includes('Plan not found for project') ||
            err.graphQLErrors?.some((e: any) => e.message?.includes('Plan not found for project'))) {
          console.log('Plan not found, initializing empty Plan DAG first...');

          try {
            // Initialize empty Plan DAG with the new node (backend will generate ID)
            await mutations.updatePlanDag({
              version: '1.0.0',
              nodes: [{
                // Omit id - backend will generate it
                nodeType: planDagNode.nodeType!,
                position: planDagNode.position!,
                metadata: planDagNode.metadata!,
                config: planDagNode.config!
              }],
              edges: [],
              metadata: {
                version: '1.0.0',
                name: `Plan DAG for Project ${projectId}`,
                description: 'Auto-generated Plan DAG',
                created: new Date().toISOString(),
                lastModified: new Date().toISOString(),
                author: 'user'
              }
            });
            console.log('Plan DAG initialized successfully');
          } catch (initError) {
            console.error('Failed to initialize Plan DAG:', initError);
            // TODO: Show user-friendly error message
          }
        }
      });
    },
    [screenToFlowPosition, setNodes, handleNodeEdit, handleNodeDelete, mutations, projectId, updatePlanDagOptimistically]
  );

  const handlePointerDrop = (nodeType: PlanDagNodeType, position: { x: number, y: number }) => {
    // Get default configuration and metadata
    const config = getDefaultNodeConfig(nodeType);
    const metadata = getDefaultNodeMetadata(nodeType);

    // Persist to database via GraphQL mutation - backend will generate ID
    const planDagNode: Partial<PlanDagNode> = {
      nodeType,
      position,
      metadata,
      config: JSON.stringify(config)
    };

    mutations.addNode(planDagNode).then((createdNode) => {
      const backendNodeId = createdNode.id;
      const newNode: Node = {
        id: backendNodeId,
        type: nodeType,
        position,
        data: {
          nodeType,
          label: metadata.label,
          config,
          metadata,
          isUnconfigured: true,
          onEdit: () => handleNodeEdit(backendNodeId),
          onDelete: () => handleNodeDelete(backendNodeId),
          readonly: false,
          edges: edges,
          projectId: projectId
        }
      };
      setNodes((nds) => nds.concat(newNode));

      updatePlanDagOptimistically((current) => {
        if (!current) return current
        if (current.nodes.some(node => node.id === backendNodeId)) {
          return current
        }
        return {
          ...current,
          nodes: [...current.nodes, createdNode],
          metadata: {
            ...current.metadata,
            lastModified: new Date().toISOString()
          }
        }
      })
    }).catch(async (err) => {
      console.error('Failed to add node to database:', err);
      if (err.message?.includes('Plan not found for project') ||
          err.graphQLErrors?.some((e: any) => e.message?.includes('Plan not found for project'))) {
        console.log('Plan not found, initializing empty Plan DAG first...');
        try {
          await mutations.updatePlanDag({
            version: '1.0.0',
            nodes: [{
              nodeType: planDagNode.nodeType!,
              position: planDagNode.position!,
              metadata: planDagNode.metadata!,
              config: planDagNode.config!
            }],
            edges: [],
            metadata: {
              version: '1.0.0',
              name: `Plan DAG for Project ${projectId}`,
              description: 'Auto-generated Plan DAG',
              created: new Date().toISOString(),
              lastModified: new Date().toISOString(),
              author: 'user'
            }
          });
          console.log('Plan DAG initialized successfully');
        } catch (initError) {
          console.error('Failed to initialize Plan DAG:', initError);
        }
      }
    });
  };

  const handleNodePointerDragStart = (event: React.MouseEvent, nodeType: PlanDagNodeType) => {
    if (!isTauri || !reactFlowWrapper.current) return;

    const reactFlowBounds = reactFlowWrapper.current.getBoundingClientRect();
    const position = screenToFlowPosition({
      x: event.clientX - reactFlowBounds.left,
      y: event.clientY - reactFlowBounds.top,
    });

    setDraggingNode({ type: nodeType, position });

    const onMouseMove = (moveEvent: MouseEvent) => {
      const position = screenToFlowPosition({
        x: moveEvent.clientX - reactFlowBounds.left,
        y: moveEvent.clientY - reactFlowBounds.top,
      });
      setDraggingNode(prev => prev ? { ...prev, position } : null);
    };

    const onMouseUp = (upEvent: MouseEvent) => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);

      const finalPosition = screenToFlowPosition({
        x: upEvent.clientX - reactFlowBounds.left,
        y: upEvent.clientY - reactFlowBounds.top,
      });

      handlePointerDrop(nodeType, finalPosition);
      setDraggingNode(null);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  };

  // Context menu handlers
  const handleContextMenu = useCallback((event: React.MouseEvent) => {
    event.preventDefault();
    // Context menu disabled - just prevent default browser menu
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

  // Auto-layout handlers
  const handleAutoLayoutHorizontal = useCallback(async () => {
    // Suppress external syncs during layout operations
    setDragging(true);

    const { nodes: layoutedNodes, edges: layoutedEdges } = await autoLayout(nodes, edges, {
      direction: 'horizontal'
    });

    setNodes(layoutedNodes);
    setEdges(layoutedEdges);

    // Persist position changes to backend in a single batch operation
    const nodePositions = layoutedNodes.map(node => ({
      nodeId: node.id,
      position: node.position,
      sourcePosition: node.sourcePosition,
      targetPosition: node.targetPosition
    }));

    await cqrsService.commands.batchMoveNodes({ projectId, planId, nodePositions });

    // Floating edges don't use handles, no edge updates needed

    // Re-enable external syncs after layout completes
    setDragging(false);
  }, [nodes, edges, setNodes, setEdges, cqrsService, projectId, setDragging]);

  const handleAutoLayoutVertical = useCallback(async () => {
    // Suppress external syncs during layout operations
    setDragging(true);

    const { nodes: layoutedNodes, edges: layoutedEdges } = await autoLayout(nodes, edges, {
      direction: 'vertical'
    });

    setNodes(layoutedNodes);
    setEdges(layoutedEdges);

    // Persist position changes to backend in a single batch operation
    const nodePositions = layoutedNodes.map(node => ({
      nodeId: node.id,
      position: node.position,
      sourcePosition: node.sourcePosition,
      targetPosition: node.targetPosition
    }));

    await cqrsService.commands.batchMoveNodes({ projectId, planId, nodePositions });

    // Floating edges don't use handles, no edge updates needed

    // Re-enable external syncs after layout completes
    setDragging(false);
  }, [nodes, edges, setNodes, setEdges, cqrsService, projectId, setDragging]);

  // Fit view - zoom to see all nodes
  const handleFitView = useCallback(() => {
    fitView({ padding: 0.2, includeHiddenNodes: false, duration: 300 });
  }, [fitView]);

  // Execution control mutations
  const [executePlan] = useMutation(EXECUTE_PLAN, {
    onCompleted: (data: any) => {
      showSuccessNotification('Execution Started', data.executePlan.message);
      // Refresh the plan DAG data to update node execution states on the canvas
      refreshData();
    },
    onError: (error: any) => {
      showErrorNotification('Execution Failed', error.message);
    },
  });

  const [stopExecution] = useMutation(STOP_PLAN_EXECUTION, {
    onCompleted: (data: any) => {
      showSuccessNotification('Execution Stopped', data.stopPlanExecution.message);
      // Refresh the plan DAG data to update node execution states on the canvas
      refreshData();
    },
    onError: (error: any) => {
      showErrorNotification('Stop Failed', error.message);
    },
  });

  const [clearExecution] = useMutation(CLEAR_PROJECT_EXECUTION, {
    onCompleted: (data: any) => {
      showSuccessNotification('Execution State Cleared', data.clearProjectExecution.message);
      // Refresh the plan DAG data to update node execution states on the canvas
      refreshData();
    },
    onError: (error: any) => {
      showErrorNotification('Clear Failed', error.message);
    },
  });

  // Execution control handlers
  const handlePlay = useCallback(() => {
    if (!projectId || !planId) {
      showErrorNotification('Error', 'Cannot execute without a project and plan selected.');
      return;
    }

    console.log('[PlanVisualEditor] Execute DAG requested for project:', projectId);

    executePlan({
      variables: {
        projectId,
        planId,
      }
    });
  }, [projectId, planId, executePlan]);

  const handleStop = useCallback(() => {
    if (!projectId) {
      showErrorNotification('Error', 'No project ID available');
      return;
    }

    console.log('[PlanVisualEditor] Stop execution requested for project:', projectId);

    stopExecution({
      variables: {
        projectId: projectId
      }
    });
  }, [projectId, stopExecution]);

  const handleClear = useCallback(() => {
    if (!projectId) {
      showErrorNotification('Error', 'No project ID available');
      return;
    }

    // Confirm before clearing
    if (!confirm('This will reset all execution state for all nodes (graph data will be cleared, but configuration and datasets will be kept). Continue?')) {
      return;
    }

    console.log('[PlanVisualEditor] Clear execution state requested for project:', projectId);

    clearExecution({
      variables: {
        projectId: projectId
      }
    });
  }, [projectId, clearExecution]);

  // Handle connection start - track source node
  const handleConnectStart = useCallback(
    (_event: any, params: { nodeId: string | null; handleId: string | null }) => {
      if (params.nodeId && params.handleId) {
        connectionSourceRef.current = {
          nodeId: params.nodeId,
          handleId: params.handleId,
        };
      }
    },
    []
  );

  // Handle connection end - create node on edge drop
  const handleConnectEnd = useCallback(
    (event: MouseEvent | TouchEvent) => {
      if (readonly) return;

      const targetIsPane = (event.target as Element).classList.contains('react-flow__pane');

      if (targetIsPane && connectionSourceRef.current) {
        // Calculate position where user dropped
        const position = screenToFlowPosition({
          x: (event as MouseEvent).clientX,
          y: (event as MouseEvent).clientY,
        });

        // Store connection info for when user selects node type
        setNewNodePosition(position);
        const sourceNode = nodesRef.current.find((node) => node.id === connectionSourceRef.current?.nodeId)
        const nextAllowed = sourceNode
          ? getValidTargetNodeTypes(sourceNode.data?.nodeType as PlanDagNodeType)
          : NODE_TYPE_SELECTOR_DEFAULTS

        if (!nextAllowed.length) {
          alert('The selected node type cannot connect to any downstream nodes.')
          return
        }

        setAllowedNodeTypes(nextAllowed)
        setShowNodeTypeMenu(true);
      }
    },
    [readonly, screenToFlowPosition]
  );

  // Handle node type selection after dropping on canvas
  const handleNodeTypeSelect = useCallback(
    async (nodeType: PlanDagNodeType) => {
      if (!newNodePosition) return;

      setShowNodeTypeMenu(false);

      const sourceConnection = connectionSourceRef.current;

      try {
        // Get defaults (backend will generate ID)
        const config = getDefaultNodeConfig(nodeType);
        const metadata = getDefaultNodeMetadata(nodeType);

        // Create the Plan DAG node at drop position (without ID)
        const planDagNode: Partial<PlanDagNode> = {
          // Omit id - backend will generate it
          nodeType,
          position: newNodePosition,
          metadata,
          config: JSON.stringify(config) as any,
        };

        // Add via mutation (will trigger subscription update and return backend-generated ID)
        const createdNode = await mutations.addNode(planDagNode);

        console.log('[PlanVisualEditor] Node created successfully:', createdNode.id);

        // Optimistically add node to local state (since subscription echo is suppressed)
        // Create a minimal Plan DAG with just the new node to convert
        // Use unique version to avoid cache collisions (don't rely on stale planDag.version)
        const tempPlanDag: PlanDag = {
          version: `temp-node-${createdNode.id}-${Date.now()}`,
          nodes: [createdNode],
          edges: [],
          metadata: planDag?.metadata || {
            version: '1',
            name: 'Untitled',
            description: '',
            created: new Date().toISOString(),
            lastModified: new Date().toISOString(),
            author: 'Unknown'
          }
        };
        const converted = ReactFlowAdapter.planDagToReactFlow(tempPlanDag);
        const reactFlowNode = converted.nodes[0];

        // Add node-specific data with current edges state (not stale planDag.edges)
        reactFlowNode.data = {
          ...reactFlowNode.data,
          onEdit: handleNodeEdit,
          onDelete: handleNodeDelete,
          readonly,
          edges: edges, // Use current edges state to avoid stale closure
          projectId: projectId
        };

        setNodes((nds) => [...nds, reactFlowNode]);

        updatePlanDagOptimistically((current) => {
          if (!current) return current
          if (current.nodes.some(node => node.id === createdNode.id)) {
            return current
          }
          return {
            ...current,
            nodes: [...current.nodes, createdNode],
            metadata: {
              ...current.metadata,
              lastModified: new Date().toISOString()
            }
          }
        })

        // Create edge if we have source connection info
        if (sourceConnection) {
          const sourceNode = nodes.find(n => n.id === sourceConnection.nodeId);

          if (sourceNode) {
            // Check if target already has maximum inputs
            const targetInputs = edges.filter(e => e.target === createdNode.id)
            const targetCanAcceptMultiple = canAcceptMultipleInputs(nodeType)

            if (!targetCanAcceptMultiple && targetInputs.length >= 1) {
              const errorMsg = `${nodeType} nodes can only have one input connection. Use a Merge node to combine multiple sources.`
              console.error('[PlanVisualEditor] Connection blocked:', errorMsg)
              alert(`Connection Error: ${errorMsg}`)
            } else {
              // Validate connection
              const validation = validateConnectionWithCycleDetection(
                sourceNode.data.nodeType,
                nodeType,
                nodes,
                edges,
                { source: sourceConnection.nodeId, target: createdNode.id }
              );

              if (validation.isValid) {
              // Create edge (backend will generate ID)
              // Floating edges don't use sourceHandle/targetHandle
              const edge: Partial<ReactFlowEdge> = {
                // Omit id - backend will generate it
                source: sourceConnection.nodeId,
                target: createdNode.id,  // Use backend-generated node ID
                metadata: {
                  label: getEdgeLabel(validation.dataType),
                  dataType: validation.dataType,
                },
              };

              const createdEdge = await mutations.addEdge(edge);
              console.log('[PlanVisualEditor] Edge created successfully:', createdEdge.id);

              // Optimistically add edge to local state (since subscription echo is suppressed)
              // Use unique version to avoid cache collisions (don't rely on stale planDag.version)
              const tempEdgePlanDag: PlanDag = {
                version: `temp-edge-${createdEdge.id}-${Date.now()}`,
                nodes: [],
                edges: [createdEdge],
                metadata: planDag?.metadata || {
                  version: '1',
                  name: 'Untitled',
                  description: '',
                  created: new Date().toISOString(),
                  lastModified: new Date().toISOString(),
                  author: 'Unknown'
                }
              };
              const convertedEdge = ReactFlowAdapter.planDagToReactFlow(tempEdgePlanDag);
              const reactFlowEdge = convertedEdge.edges[0];

              // Only add if not already present (avoid duplicates from subscription updates)
              setEdges((eds) => {
                const exists = eds.some(e => e.id === reactFlowEdge.id)
                return exists ? eds : [...eds, reactFlowEdge]
              });

              updatePlanDagOptimistically((current) => {
                if (!current) return current
                if (current.edges.some(existing => existing.id === createdEdge.id)) {
                  return current
                }
                return {
                  ...current,
                  edges: [...current.edges, createdEdge],
                  metadata: {
                    ...current.metadata,
                    lastModified: new Date().toISOString()
                  }
                }
              })
              } else {
                console.warn('[PlanVisualEditor] Invalid connection:', validation.errorMessage);
              }
            }
          }
        }
      } catch (error) {
        console.error('[PlanVisualEditor] Failed to create node/edge:', error);
        alert(`Failed to create node: ${error instanceof Error ? error.message : 'Unknown error'}`);
      } finally {
        // Clear state
        setNewNodePosition(null);
        connectionSourceRef.current = null;
      }
    },
    [newNodePosition, mutations, nodes, edges, planDag, readonly, handleNodeEdit, handleNodeDelete, setNodes, setEdges, updatePlanDagOptimistically]
  );

  // Use stable nodeTypes reference directly
  // Note: Project join/leave is now handled at App level

  // Register edge types for floating edges
  const edgeTypes = useMemo(() => ({
    floating: FloatingEdge,
  }), []);

  const miniMapNodeColor = useCallback((node: Node) => {
    switch (node.data?.nodeType) {
      case 'DataSetNode':
        return '#51cf66'
      case 'GraphNode':
        return '#339af0'
      case 'TransformNode':
        return '#ff8cc8'
      case 'MergeNode':
        return '#ffd43b'
      case 'GraphArtefactNode':
        return '#ff6b6b'
      case 'TreeArtefactNode':
        return '#845ef7'
      default:
        return '#868e96'
    }
  }, [])

  // Create a stable serialization key for node configuration (excludes position)
  // This ensures the map only recalculates when configuration changes, not on every drag
  const nodeConfigKey = useMemo(() => {
    return nodes.map(node => {
      // Include only configuration-relevant data, exclude position
      return `${node.id}:${node.data?.nodeType}:${node.data?.hasValidConfig}:${JSON.stringify(node.data?.config || {})}`
    }).join('|')
  }, [nodes])

  // Memoized map of node configuration status for O(1) lookups
  // Only recalculates when node configuration changes, not on position updates
  const nodeConfigMap = useMemo(() => {
    const map = new Map<string, boolean>()

    nodes.forEach(node => {
      const nodeType = node.data?.nodeType || PlanDagNodeType.DATA_SOURCE
      const config = node.data?.config
      const hasValidConfig =
        nodeType === PlanDagNodeType.PROJECTION
          ? Boolean((config as any)?.projectionId)
          : node.data?.hasValidConfig !== false

      // Use the comprehensive isNodeConfigured validation that checks edges
      const configured = isNodeConfigured(nodeType, node.id, edges, hasValidConfig)
      map.set(node.id, configured)
    })

    return map
  }, [nodeConfigKey, edges])

  // Fast O(1) lookup helper using the memoized map
  const isNodeFullyConfigured = useCallback((nodeId: string): boolean => {
    return nodeConfigMap.get(nodeId) ?? false
  }, [nodeConfigMap])

  // PERFORMANCE FIX: Separate node data from positions to prevent re-renders during drag
  // Create a stable map of node data that only changes when actual data changes, not positions
  const nodeDataMap = useMemo(() => {
    const map = new Map<string, any>()
    nodes.forEach(node => {
      const nodeType = node.data?.nodeType || PlanDagNodeType.DATA_SOURCE
      const config = node.data?.config
      const hasValidConfig =
        nodeType === PlanDagNodeType.PROJECTION
          ? Boolean((config as any)?.projectionId)
          : node.data?.hasValidConfig !== false

      map.set(node.id, {
        // Preserve original data first
        ...node.data,
        // Then override with calculated/injected values
        edges: edges,
        isUnconfigured: !isNodeFullyConfigured(node.id),
        hasValidConfig,
        projectId: projectId
      })
    })
    return map
  }, [edges, isNodeFullyConfigured, projectId, nodeConfigKey]) // Uses nodeConfigKey instead of nodes!

  // Inject enriched data into nodes - only recreates node objects when data actually changes
  // Position changes don't trigger this memo because nodeDataMap is stable during drag
  const nodesWithEdges = useMemo(() => {
    return nodes.map(node => ({
      ...node,
      data: nodeDataMap.get(node.id) || node.data
    }))
  }, [nodes, nodeDataMap])

  // Enhance edges with markers and styling based on source node configuration status
  // Must be defined before any early returns to follow Rules of Hooks
  const edgesWithMarkers = useMemo(() => {
    return edges.map(edge => {
      // Check if the source node is fully configured
      const sourceConfigured = isNodeFullyConfigured(edge.source)

      // Determine base color from edge data type; fall back to warning orange while source is unconfigured
      const edgeDataType = ((edge.data as any)?.metadata?.dataType ||
        (edge as any).metadata?.dataType) as EdgeDataType
      const configuredColor = getEdgeColor(edgeDataType)
      const edgeColor = sourceConfigured ? configuredColor : '#fd7e14'

      return {
        ...edge,
        reconnectable: !readonly,
        style: {
          ...(edge.style || {}),
          stroke: edgeColor,
          strokeWidth: 2
        },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: edgeColor,
          width: 20,
          height: 20
        }
      }
    })
  }, [edges, isNodeFullyConfigured, readonly])

  const sequenceStoryId = useMemo(() => {
    if (configNodeType !== PlanDagNodeType.SEQUENCE_ARTEFACT || !configNodeId) {
      return undefined
    }

    const incomingStoryEdge = edges.find(edge => edge.target === configNodeId)
    if (!incomingStoryEdge) return undefined

    const sourceNode = nodes.find(node => node.id === incomingStoryEdge.source)
    if (!sourceNode || sourceNode.data?.nodeType !== PlanDagNodeType.STORY) {
      return undefined
    }

    const storyConfig = sourceNode.data?.config as StoryNodeConfig | undefined
    const rawStoryId = storyConfig?.storyId
    if (typeof rawStoryId === 'number') {
      return rawStoryId > 0 ? rawStoryId : undefined
    }
    const parsed = Number(rawStoryId)
    return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined
  }, [configNodeId, configNodeType, edges, nodes])

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <Spinner className="h-8 w-8" />
        <p>Loading Plan DAG...</p>
      </div>
    )
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <IconAlertCircle className="h-4 w-4" />
        <AlertDescription>
          <p className="font-semibold mb-1">Error loading Plan DAG</p>
          <p className="text-sm">{error.message}</p>
        </AlertDescription>
      </Alert>
    )
  }

  if (!planDag) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <Alert className="w-full max-w-[500px]">
          <IconAlertCircle className="h-4 w-4" />
          <AlertDescription>
            <p className="font-semibold mb-1">No Plan DAG found</p>
            <p className="text-sm">This project doesn't have a Plan DAG configured yet. Create one by adding nodes to the canvas using the toolbar.</p>
          </AlertDescription>
        </Alert>
        <p className="text-sm text-muted-foreground text-center">
          Plan DAGs define data transformation workflows through connected nodes and edges.
        </p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full gap-0">
      <style>{`
        .react-flow__node {
          border: none !important;
          padding: 0 !important;
          background: transparent !important;
        }
        .react-flow__pane {
          cursor: default !important;
        }
        .react-flow__edge-interaction {
          cursor: grab !important;
        }
        .react-flow__edge-interaction:active {
          cursor: grabbing !important;
        }
      `}</style>

      {/* Simplified Toolbar - Node palette, auto-layout, fit view, and execution controls */}
      <AdvancedToolbar
        readonly={readonly}
        onNodeDragStart={handleNodeDragStart}
        onNodePointerDragStart={handleNodePointerDragStart}
        onAutoLayoutHorizontal={handleAutoLayoutHorizontal}
        onAutoLayoutVertical={handleAutoLayoutVertical}
        onFitView={handleFitView}
        onPlay={handlePlay}
        onStop={handleStop}
        onClear={handleClear}
      />

      <div
        ref={reactFlowWrapper}
        style={{ flex: 1, position: 'relative' }}
        onMouseMove={handleMouseMove}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onContextMenu={handleContextMenu}
        onClick={handleCanvasClick}
      >
        {draggingNode && (
          <Card
            className="absolute z-[1000] pointer-events-none shadow-sm p-3 rounded-md"
            style={{
              left: draggingNode.position.x,
              top: draggingNode.position.y,
              transform: 'translate(-50%, -50%)',
            }}
          >
            <p>{draggingNode.type}</p>
          </Card>
        )}
        <PlanVisualEditorContext.Provider value={{ projectId, planId }}>
          <ReactFlow
          nodes={nodesWithEdges}
          edges={edgesWithMarkers}
          onNodesChange={handleNodesChange}
          onEdgesChange={(changes) => {
            console.log('ReactFlow onEdgesChange called:', changes)
            handleEdgesChange(changes)
          }}
          onConnect={onConnect}
          isValidConnection={isValidConnection}
          onMove={handleViewportChange}
          onNodeDragStart={handleFlowNodeDragStart}
          onNodeDragStop={handleNodeDragStop}
          onEdgeDoubleClick={handleEdgeDoubleClick}
          onReconnect={handleReconnect}
          onReconnectStart={handleReconnectStart}
          onReconnectEnd={handleReconnectEnd}
          onConnectStart={handleConnectStart}
          onConnectEnd={handleConnectEnd}
          onNodeClick={(event, node) => {
            // Prevent selection when clicking on action icons (edit/delete)
            const target = event.target as HTMLElement
            const isActionIcon = target.closest('[data-action-icon]')
            if (isActionIcon) {
              // Manually trigger the appropriate handler
              const actionType = isActionIcon.getAttribute('data-action-icon')
              if (actionType === 'edit') {
                event.stopPropagation()
                event.preventDefault()
                handleNodeEdit(node.id)
                return
              } else if (actionType === 'delete') {
                event.stopPropagation()
                event.preventDefault()
                handleNodeDelete(node.id)
                return
              }
              // For other actions like 'preview' and 'download', let the node's own handlers work
              // Don't stop propagation - the action buttons handle their own events
            }
          }}
          nodeTypes={NODE_TYPES}
          edgeTypes={edgeTypes}
          connectionMode={ConnectionMode.Loose}
          connectionLineComponent={FloatingConnectionLine}
          defaultEdgeOptions={{
            type: 'floating',
            animated: false,
            style: { stroke: '#868e96', strokeWidth: 2 }
          }}
          multiSelectionKeyCode="Control"
          selectionKeyCode="Shift"
          deleteKeyCode={readonly ? null : ["Delete", "Backspace"]}
          panOnDrag={[1, 2]}
          selectionOnDrag={true}
          minZoom={0.1}
          maxZoom={4}
          fitView
          attributionPosition="top-right"
          proOptions={{ hideAttribution: true }}
        >
          <Background />
          <Controls />
          <MiniMap nodeColor={miniMapNodeColor} />

          {/* Empty state overlay when no nodes exist */}
          {nodes.length === 0 && (
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-10 pointer-events-none text-center">
              <div className="flex flex-col items-center gap-4 p-6">
                <Alert className="max-w-[400px] pointer-events-auto">
                  <IconAlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    <p className="font-semibold mb-1">Start building your Plan DAG</p>
                    <p className="text-sm">Drag nodes from the toolbar above to begin creating your data transformation workflow.</p>
                  </AlertDescription>
                </Alert>
                <p className="text-sm text-muted-foreground text-center max-w-[350px]">
                  Create connections between nodes to define how data flows through your pipeline.
                </p>
              </div>
            </div>
          )}

          {/* Phase 3: Collaborative cursors for real-time user presence */}
          <CollaborativeCursors users={onlineUsers} currentUserId={undefined} />

          {/* Collaboration features integration complete */}
          </ReactFlow>
        </PlanVisualEditorContext.Provider>
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
        storyIdHint={sequenceStoryId}
        graphIdHint={configGraphIdHint}
        graphSourceNodeIdHint={configGraphSourceNodeId}
        onSave={handleNodeConfigSave}
      />

      {/* Edge Configuration Dialog */}
      <EdgeConfigDialog
        edge={configEdge}
        opened={edgeConfigDialogOpen}
        onClose={() => setEdgeConfigDialogOpen(false)}
        onSave={handleEdgeUpdate}
        readonly={readonly}
      />

      {/* Node Type Selector for Edge Drop */}
      <NodeTypeSelector
        opened={showNodeTypeMenu}
        onClose={() => setShowNodeTypeMenu(false)}
        onSelect={handleNodeTypeSelect}
        allowedNodeTypes={allowedNodeTypes}
      />

      {/* Context Menu removed - advanced operations no longer available */}
    </div>
  )
}

export const PlanVisualEditor = (props: PlanVisualEditorProps) => {
  return (
    <ReactFlowProvider>
      <PlanVisualEditorInner {...props} />
    </ReactFlowProvider>
  );
};
