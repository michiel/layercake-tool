import { useCallback, useEffect, useState, useRef, useMemo } from 'react'
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
import { Stack, Alert, Loader, Text } from '@mantine/core'
import {
  IconAlertCircle
} from '@tabler/icons-react'

import { useCollaborationV2 } from '../../../hooks/useCollaborationV2'
import { PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig, ReactFlowEdge, PlanDagNode, PlanDag } from '../../../types/plan-dag'
import { validateConnectionWithCycleDetection, canAcceptMultipleInputs } from '../../../utils/planDagValidation'
import { ReactFlowAdapter } from '../../../adapters/ReactFlowAdapter'

// Import node types constant
import { NODE_TYPES } from './nodeTypes'

// Import collaboration components
import { CollaborativeCursors } from '../../collaboration/CollaborativeCursors'
import { UserPresenceData } from '../../../types/websocket'

// Import dialogs
import { NodeConfigDialog } from './NodeConfigDialog'
import { EdgeConfigDialog } from './EdgeConfigDialog'
import { NodeTypeSelector } from './dialogs/NodeTypeSelector'

// Import extracted components and hooks
import { AdvancedToolbar } from './components/AdvancedToolbar'
import { ContextMenu } from './components/ContextMenu'
import { ConnectionLine } from './components/ConnectionLine'
import { usePlanDagCQRS } from './hooks/usePlanDagCQRS'
import { useAdvancedOperations } from './hooks/useAdvancedOperations'
import { generateNodeId, getDefaultNodeConfig, getDefaultNodeMetadata } from './utils/nodeDefaults'
import { autoLayout } from './utils/autoLayout'

// Import ReactFlow styles
import 'reactflow/dist/style.css'

interface PlanVisualEditorProps {
  projectId: number
  onNodeSelect?: (nodeId: string | null) => void
  onEdgeSelect?: (edgeId: string | null) => void
  readonly?: boolean
}

const PlanVisualEditorInner = ({ projectId, onNodeSelect, onEdgeSelect, readonly = false }: PlanVisualEditorProps) => {
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
  } as DataSourceNodeConfig)
  const [configNodeMetadata, setConfigNodeMetadata] = useState<NodeMetadata>({ label: '', description: '' })

  // Node type selector for edge drop
  const [showNodeTypeMenu, setShowNodeTypeMenu] = useState(false)
  const [newNodePosition, setNewNodePosition] = useState<{ x: number; y: number } | null>(null)
  const connectionSourceRef = useRef<{ nodeId: string; handleId: string } | null>(null)

  // Ref to store nodes for handleNodeEdit (to avoid circular dependency)
  const nodesRef = useRef<Node[]>([])

  // Node action handlers (defined with stable references)
  const handleNodeEdit = useCallback((nodeId: string) => {
    console.log('Edit node triggered:', nodeId)
    setConfigNodeId(nodeId)

    // Find the node and populate config dialog state
    const node = nodesRef.current.find(n => n.id === nodeId)
    if (node) {
      console.log('Found node:', node)
      // Use node.type which is the ReactFlow type (DataSourceNode, OutputNode, etc.)
      // node.data.nodeType might be the backend format (DataSource, Output, etc.)
      setConfigNodeType(node.type as PlanDagNodeType || PlanDagNodeType.DATA_SOURCE)
      setConfigNodeConfig(node.data.config || {})
      setConfigNodeMetadata(node.data.metadata || { label: '', description: '' })
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
  } = planDagState

  // Keep nodesRef updated for handleNodeEdit
  useEffect(() => {
    nodesRef.current = nodes
  }, [nodes])

  // Get mutations from CQRS service (includes delta generation)
  // Adapt CQRS command interface to match old mutation interface
  const mutations = {
    addNode: (node: Partial<PlanDagNode>) =>
      cqrsService.commands.createNode({ projectId, nodeType: node.nodeType || 'DataSource', node }),
    addEdge: (edge: Partial<any>) =>
      cqrsService.commands.createEdge({ projectId, edge: edge as any }),
    updateNode: (nodeId: string, updates: Partial<PlanDagNode>) =>
      cqrsService.commands.updateNode({ projectId, nodeId, updates }),
    deleteNode: (nodeId: string) =>
      cqrsService.commands.deleteNode({ projectId, nodeId }),
    deleteEdge: (edgeId: string) =>
      cqrsService.commands.deleteEdge({ projectId, edgeId }),
    moveNode: (nodeId: string, position: { x: number; y: number }) =>
      cqrsService.commands.moveNode({ projectId, nodeId, position }),
    updatePlanDag: (planDag: any) =>
      cqrsService.commands.updatePlanDag({ projectId, planDag }),
  }

  // Setup delete handler with access to mutations
  useEffect(() => {
    deleteHandlerRef.current = (nodeId: string) => {
      console.log('Executing delete for node:', nodeId)

      // Suppress external syncs during delete operations
      setDragging(true)

      // Remove node from local state optimistically
      setNodes((nds) => nds.filter(node => node.id !== nodeId))

      // Remove edges connected to this node
      const edgesToDelete: string[] = []
      setEdges((eds) => {
        const filtered = eds.filter(edge => {
          if (edge.source === nodeId || edge.target === nodeId) {
            edgesToDelete.push(edge.id)
            return false
          }
          return true
        })
        return filtered
      })

      // Persist deletions to backend
      mutations.deleteNode(nodeId)
      edgesToDelete.forEach(edgeId => mutations.deleteEdge(edgeId))

      // Re-enable external syncs after a short delay to allow mutations to complete
      setTimeout(() => setDragging(false), 100)
    }
  }, [setNodes, setEdges, mutations, setDragging])

  // Collaboration setup
  const currentUserId: string | undefined = undefined
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

  // Other UI state
  const [selectedNode, setSelectedNode] = useState<string | null>(null)
  const [_selectedEdge, setSelectedEdge] = useState<string | null>(null)
  const viewportRef = useRef({ x: 0, y: 0, zoom: 1 })

  // Edge configuration dialog state
  const [edgeConfigDialogOpen, setEdgeConfigDialogOpen] = useState(false)
  const [configEdge, setConfigEdge] = useState<ReactFlowEdge | null>(null)

  // Use users directly from the collaboration hook
  const onlineUsers: UserPresenceData[] = collaboration.users || []



  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    opened: boolean;
    position: { x: number; y: number };
  }>({ opened: false, position: { x: 0, y: 0 } })

  // Advanced operations hook with delete callbacks
  const handleBulkDeleteNodes = useCallback((nodeIds: string[]) => {
    nodeIds.forEach(nodeId => mutations.deleteNode(nodeId))
  }, [mutations])

  const handleBulkDeleteEdges = useCallback((edgeIds: string[]) => {
    edgeIds.forEach(edgeId => mutations.deleteEdge(edgeId))
  }, [mutations])

  const advancedOps = useAdvancedOperations({
    nodes,
    edges,
    setNodes,
    setEdges,
    readonly,
    onDeleteNodes: handleBulkDeleteNodes,
    onDeleteEdges: handleBulkDeleteEdges,
  })

  // Handle node changes (position, selection, etc.)
  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      // Always apply changes to ReactFlow for visual updates
      onNodesChange(changes)

      // Skip performance tracking and side effects during drag
      // Position and dimension changes during drag are cosmetic only - actual save happens in handleNodeDragStop
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

        // Update the entire plan DAG (this is how edge updates work)
        await mutations.updatePlanDag(updatedPlanDag)
        console.log('Edge updated successfully:', edgeId)
      } catch (error) {
        console.error('Failed to update edge:', error)
      }
    },
    [planDag, mutations]
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

      // Generate new edge ID with source, target, and handle information
      const sourceHandle = newConnection.sourceHandle || 'output'
      const targetHandle = newConnection.targetHandle || 'input'
      const newEdgeId = `${newConnection.source}-${newConnection.target}-${sourceHandle}-${targetHandle}`

      // Create the new edge with the updated connection
      const newEdge: ReactFlowEdge = {
        id: newEdgeId,
        source: newConnection.source!,
        target: newConnection.target!,
        sourceHandle: newConnection.sourceHandle,
        targetHandle: newConnection.targetHandle,
        metadata: {
          label: isValid.dataType === 'GRAPH_REFERENCE' ? 'Graph Ref' : 'Data',
          dataType: isValid.dataType,
        }
      }

      // Update local state - remove old edge and add new one
      setEdges((els) => {
        const filtered = els.filter(e => e.id !== oldEdge.id)
        return addEdge({
          ...newEdge,
          type: 'smoothstep',
          animated: false,
          style: {
            stroke: isValid.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96',
            strokeWidth: 2,
          },
          data: { metadata: newEdge.metadata }
        }, filtered)
      })

      // Persist to backend: delete old edge and create new one
      // The edge ID in backend is stored in originalEdge.id
      const oldEdgeId = (oldEdge.data as any)?.originalEdge?.id || oldEdge.id
      mutations.deleteEdge(oldEdgeId)
      mutations.addEdge(newEdge)

      console.log('Edge reconnected - deleted:', oldEdgeId, 'created:', newEdge.id)
    },
    [readonly, setEdges, mutations, nodes, edges]
  )

  const handleReconnectEnd = useCallback(
    (_: MouseEvent | TouchEvent, edge: Edge) => {
      if (!edgeReconnectSuccessful.current && !readonly) {
        // Edge was dropped on empty space - delete it
        const edgeIdToDelete = (edge.data as any)?.originalEdge?.id || edge.id
        setEdges((eds) => eds.filter((e) => e.id !== edge.id))
        mutations.deleteEdge(edgeIdToDelete)
        console.log('Edge deleted on drop:', edgeIdToDelete)
      }
      edgeReconnectSuccessful.current = true
    },
    [readonly, setEdges, mutations]
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
          mutations.deleteEdge(change.id)
        }
        if (change.type === 'select') {
          const edgeId = change.selected ? change.id : null
          setSelectedEdge(edgeId)
          onEdgeSelect?.(edgeId)
        }
      })
    },
    [onEdgesChange, mutations, onEdgeSelect, readonly, planDagState.performanceMonitor, planDag, updateManager]
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

      console.log('[PlanVisualEditor] Creating edge with handles:', {
        sourceHandle: connection.sourceHandle,
        targetHandle: connection.targetHandle,
        source: connection.source,
        target: connection.target
      })

      const newEdge = {
        id: `edge-${Date.now()}`,
        source: connection.source!,
        target: connection.target!,
        sourceHandle: connection.sourceHandle || null, // Preserve specific handle used
        targetHandle: connection.targetHandle || null, // Preserve specific handle used
        type: 'smoothstep',
        animated: false,
        label: isValid.dataType === 'GRAPH_REFERENCE' ? 'Graph Ref' : 'Data',
        style: {
          stroke: isValid.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96',
          strokeWidth: 2,
        },
        labelStyle: {
          fontSize: 12,
          fontWeight: 500,
        },
        data: {
          metadata: {
            label: isValid.dataType === 'GRAPH_REFERENCE' ? 'Graph Ref' : 'Data',
            dataType: isValid.dataType,
          }
        }
      }

      // Optimistic update - add edge immediately for instant feedback
      setEdges((eds) => addEdge(newEdge, eds))

      // Persist to backend - subscription will confirm or correct
      const graphqlEdge: ReactFlowEdge = {
        id: newEdge.id,
        source: newEdge.source,
        target: newEdge.target,
        sourceHandle: newEdge.sourceHandle,
        targetHandle: newEdge.targetHandle,
        metadata: newEdge.data.metadata
      }

      // Don't suppress sync during edge creation - we need to see updates
      mutations.addEdge(graphqlEdge).catch(err => {
        console.error('[PlanVisualEditor] Failed to create edge:', err)
        // Remove optimistic edge on failure
        setEdges((eds) => eds.filter(e => e.id !== newEdge.id))
        alert(`Failed to create connection: ${err.message}`)
      })
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
                onEdit: () => handleNodeEdit(nodeId),
                onDelete: () => handleNodeDelete(nodeId),
              },
            }
          : node
      )
    )
    // Remove __typename from metadata before sending to backend
    const { __typename, ...cleanedMetadata } = metadata as any;

    // Save changes to backend
    mutations.updateNode(nodeId, {
      config: JSON.stringify(config),
      metadata: cleanedMetadata
    })
    console.log('Node configuration updated:', nodeId, config, cleanedMetadata)
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

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
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

      // Generate unique ID and default configuration
      const existingNodeIds = nodes.map(n => n.id);
      const nodeId = generateNodeId(nodeType, existingNodeIds);
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

      // Persist to database via GraphQL mutation
      const planDagNode: Partial<PlanDagNode> = {
        id: nodeId,
        nodeType,
        position,
        metadata,
        config: JSON.stringify(config)
      };

      // First try to add the node directly
      mutations.addNode(planDagNode).catch(async (err) => {
        console.error('Failed to add node to database:', err);

        // If the error is "Plan not found for project", initialize an empty plan first
        if (err.message?.includes('Plan not found for project') ||
            err.graphQLErrors?.some((e: any) => e.message?.includes('Plan not found for project'))) {
          console.log('Plan not found, initializing empty Plan DAG first...');

          try {
            // Initialize empty Plan DAG with the new node
            await mutations.updatePlanDag({
              version: '1.0.0',
              nodes: [{
                id: planDagNode.id!,
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
    [screenToFlowPosition, setNodes, handleNodeEdit, handleNodeDelete, mutations, projectId]
  );

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

    await cqrsService.commands.batchMoveNodes(projectId, nodePositions);

    // Persist edge handle changes to backend
    for (const edge of layoutedEdges) {
      await cqrsService.commands.updateEdge({
        projectId,
        edgeId: edge.id,
        updates: {
          sourceHandle: edge.sourceHandle || undefined,
          targetHandle: edge.targetHandle || undefined
        }
      });
    }

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

    await cqrsService.commands.batchMoveNodes(projectId, nodePositions);

    // Persist edge handle changes to backend
    for (const edge of layoutedEdges) {
      await cqrsService.commands.updateEdge({
        projectId,
        edgeId: edge.id,
        updates: {
          sourceHandle: edge.sourceHandle || undefined,
          targetHandle: edge.targetHandle || undefined
        }
      });
    }

    // Re-enable external syncs after layout completes
    setDragging(false);
  }, [nodes, edges, setNodes, setEdges, cqrsService, projectId, setDragging]);

  // Fit view - zoom to see all nodes
  const handleFitView = useCallback(() => {
    fitView({ padding: 0.2, includeHiddenNodes: false, duration: 300 });
  }, [fitView]);

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
        // Generate node ID and get defaults
        const existingNodeIds = nodes.map(n => n.id);
        const nodeId = generateNodeId(nodeType, existingNodeIds);
        const config = getDefaultNodeConfig(nodeType);
        const metadata = getDefaultNodeMetadata(nodeType);

        // Create the Plan DAG node at drop position
        const planDagNode: PlanDagNode = {
          id: nodeId,
          nodeType,
          position: newNodePosition,
          metadata,
          config: JSON.stringify(config) as any,
        };

        // Add via mutation (will trigger subscription update)
        const createdNode = await mutations.addNode(planDagNode);

        console.log('[PlanVisualEditor] Node created successfully:', nodeId);

        // Optimistically add node to local state (since subscription echo is suppressed)
        // Create a minimal Plan DAG with just the new node to convert
        const tempPlanDag: PlanDag = {
          version: String((parseInt(planDag?.version || '0') || 0) + 1),
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

        // Add node-specific data
        reactFlowNode.data = {
          ...reactFlowNode.data,
          onEdit: handleNodeEdit,
          onDelete: handleNodeDelete,
          readonly,
          edges: planDag?.edges || []
        };

        setNodes((nds) => [...nds, reactFlowNode]);

        // Create edge if we have source connection info
        if (sourceConnection) {
          const sourceNode = nodes.find(n => n.id === sourceConnection.nodeId);

          if (sourceNode) {
            // Check if target already has maximum inputs
            const targetInputs = edges.filter(e => e.target === nodeId)
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
                { source: sourceConnection.nodeId, target: nodeId }
              );

              if (validation.isValid) {
              // Determine target handle based on node type
              const targetHandle = 'input-left'; // Most nodes use input-left

              // Create edge
              const edgeId = `${sourceConnection.nodeId}-${nodeId}`;
              const edge: ReactFlowEdge = {
                id: edgeId,
                source: sourceConnection.nodeId,
                target: nodeId,
                sourceHandle: sourceConnection.handleId,
                targetHandle: targetHandle,
                metadata: {
                  label: validation.dataType === 'GRAPH_REFERENCE' ? 'Graph Ref' : 'Data',
                  dataType: validation.dataType,
                },
              };

              const createdEdge = await mutations.addEdge(edge);
              console.log('[PlanVisualEditor] Edge created successfully:', edgeId);

              // Optimistically add edge to local state (since subscription echo is suppressed)
              const tempEdgePlanDag: PlanDag = {
                version: String((parseInt(planDag?.version || '0') || 0) + 2),
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

              setEdges((eds) => [...eds, reactFlowEdge]);
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
    [newNodePosition, mutations, nodes, edges, planDag, readonly, handleNodeEdit, handleNodeDelete, setNodes, setEdges]
  );

  // Use stable nodeTypes reference directly


  // Join/leave collaboration on mount/unmount
  useEffect(() => {
    if (!readonly) {
      collaboration.joinProject()
    }

    return () => {
      if (!readonly) {
        collaboration.leaveProject()
      }
    }
    // Note: 'collaboration' intentionally omitted from deps to prevent infinite re-joins
    // Join/leave should only happen on component mount/unmount, not when collaboration object changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [readonly])

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
      // Check basic config validity
      const hasValidConfig = node.data?.hasValidConfig !== false
      if (!hasValidConfig) {
        map.set(node.id, false)
        return
      }

      // For DataSource nodes, also check if dataSourceId is set
      if (node.data?.nodeType === 'DATA_SOURCE') {
        const config = node.data?.config as any
        map.set(node.id, !!(config?.dataSourceId))
        return
      }

      map.set(node.id, true)
    })

    return map
  }, [nodeConfigKey])

  // Fast O(1) lookup helper using the memoized map
  const isNodeFullyConfigured = useCallback((nodeId: string): boolean => {
    return nodeConfigMap.get(nodeId) ?? false
  }, [nodeConfigMap])

  // Inject current edges into node data for configuration validation
  // Must be defined before any early returns to follow Rules of Hooks
  const nodesWithEdges = useMemo(() => {
    return nodes.map(node => ({
      ...node,
      data: {
        ...node.data,
        edges: edges, // Inject current edges for validation
        isUnconfigured: !isNodeFullyConfigured(node.id), // Add unconfigured flag
        projectId: projectId // Inject projectId for execute button
      }
    }))
  }, [nodes, edges, isNodeFullyConfigured, projectId])

  // Enhance edges with markers and styling based on source node configuration status
  // Must be defined before any early returns to follow Rules of Hooks
  const edgesWithMarkers = useMemo(() => {
    return edges.map(edge => {
      // Check if the source node is fully configured
      const sourceConfigured = isNodeFullyConfigured(edge.source)

      // Determine color: blue for configured source, orange for unconfigured source
      const edgeColor = sourceConfigured ? '#228be6' : '#fd7e14'

      return {
        ...edge,
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
  }, [edges, isNodeFullyConfigured])

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

  if (!planDag) {
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
    <Stack gap={0} style={{ height: '100%' }}>
      <style>{`
        .react-flow__node {
          border: none !important;
          padding: 0 !important;
          background: transparent !important;
        }
      `}</style>

      {/* Advanced Operations Toolbar */}
      <AdvancedToolbar
        selectedNodeCount={advancedOps.selectedNodes.length}
        hasClipboardData={advancedOps.hasClipboardData}
        clipboardInfo={advancedOps.clipboardInfo}
        canAlign={advancedOps.canAlign}
        canDistribute={advancedOps.canDistribute}
        readonly={readonly}
        onNodeDragStart={handleNodeDragStart}
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
        onAutoLayoutHorizontal={handleAutoLayoutHorizontal}
        onAutoLayoutVertical={handleAutoLayoutVertical}
        onFitView={handleFitView}
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
            console.log('onNodeClick - target:', target, 'isActionIcon:', isActionIcon, 'node:', node)
            if (isActionIcon) {
              event.stopPropagation()
              event.preventDefault()
              // Manually trigger the appropriate handler
              const actionType = isActionIcon.getAttribute('data-action-icon')
              console.log('Action icon clicked:', actionType, 'nodeId:', node.id)
              if (actionType === 'edit') {
                handleNodeEdit(node.id)
              } else if (actionType === 'delete') {
                handleNodeDelete(node.id)
              }
              return
            }
          }}
          nodeTypes={NODE_TYPES}
          connectionMode={ConnectionMode.Loose}
          connectionLineComponent={ConnectionLine}
          defaultEdgeOptions={{
            type: 'smoothstep',
            animated: false,
            style: { stroke: '#868e96', strokeWidth: 2 }
          }}
          multiSelectionKeyCode="Control"
          selectionKeyCode="Shift"
          deleteKeyCode={null}
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
                  Drag nodes from the toolbar above to begin creating your data transformation workflow.
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