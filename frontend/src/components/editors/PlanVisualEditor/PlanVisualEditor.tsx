import { useCallback, useEffect, useState, useRef } from 'react'
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
  ReactFlowProvider
} from 'reactflow'
import { Stack, Title, Alert, Loader, Text, Tooltip, Group, Badge, ActionIcon } from '@mantine/core'
import {
  IconAlertCircle,
  IconNetwork,
  IconNetworkOff
} from '@tabler/icons-react'

import { useCollaborationV2 } from '../../../hooks/useCollaborationV2'
import { PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig, ReactFlowEdge, PlanDagNode } from '../../../types/plan-dag'
import { validateConnectionWithCycleDetection } from '../../../utils/planDagValidation'
import { IconDownload } from '@tabler/icons-react'

// Import node types constant
import { NODE_TYPES } from './nodeTypes'

// Import collaboration components
import { CollaborativeCursors } from '../../collaboration/CollaborativeCursors'
import { UserPresenceData } from '../../../types/websocket'

// Import dialogs
import { NodeConfigDialog } from './NodeConfigDialog'
import { EdgeConfigDialog } from './EdgeConfigDialog'

// Import extracted components and hooks
import { AdvancedToolbar } from './components/AdvancedToolbar'
import { ContextMenu } from './components/ContextMenu'
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
    isDirty,
    updateManager,
    cqrsService,
    setDragging,
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
    }
  }, [setNodes, setEdges, mutations])

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
      setDragging(false) // Re-enable external syncs after drag

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
          } else {
            console.log('Node not moved significantly, skipping position save:', node.id)
          }

          // Clean up tracking
          delete dragStartPositions.current[node.id]
        }
      }
    },
    [mutations, readonly, updateManager, planDag, planDagState.performanceMonitor, setDragging]
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
    // Save changes to backend
    mutations.updateNode(nodeId, {
      config: JSON.stringify(config),
      metadata
    })
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

  // Auto-layout handlers
  const handleAutoLayoutHorizontal = useCallback(async () => {
    const layoutedNodes = await autoLayout(nodes, edges, {
      direction: 'horizontal',
      nodeSpacing: 50,
      rankSpacing: 250
    });

    setNodes(layoutedNodes);

    // Persist position changes to backend
    layoutedNodes.forEach(node => {
      mutations.moveNode(node.id, node.position);
    });
  }, [nodes, edges, setNodes, mutations]);

  const handleAutoLayoutVertical = useCallback(async () => {
    const layoutedNodes = await autoLayout(nodes, edges, {
      direction: 'vertical',
      nodeSpacing: 100,
      rankSpacing: 150
    });

    setNodes(layoutedNodes);

    // Persist position changes to backend
    layoutedNodes.forEach(node => {
      mutations.moveNode(node.id, node.position);
    });
  }, [nodes, edges, setNodes, mutations]);

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

  // Download Plan DAG as YAML
  const handleDownloadYAML = useCallback(() => {
    if (!planDag) return

    // Convert Plan DAG to YAML-like structure
    const yamlContent = convertPlanDagToYAML(planDag)

    // Create filename from plan name
    const planName = planDag.metadata?.name || 'plan'
    const escapedName = planName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
    const filename = `${escapedName}-plan.yaml`

    // Create and download file
    const blob = new Blob([yamlContent], { type: 'text/yaml;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)

    console.log(`Downloaded Plan DAG as ${filename}`)
  }, [planDag])

  // Simple YAML converter
  const convertPlanDagToYAML = (dag: any): string => {
    const indent = (level: number) => '  '.repeat(level)

    const serializeValue = (value: any, level: number): string => {
      if (value === null || value === undefined) return 'null'
      if (typeof value === 'string') return `"${value.replace(/"/g, '\\"')}"`
      if (typeof value === 'number' || typeof value === 'boolean') return String(value)
      if (Array.isArray(value)) {
        if (value.length === 0) return '[]'
        return '\n' + value.map(item =>
          `${indent(level)}- ${serializeValue(item, level + 1).trim()}`
        ).join('\n')
      }
      if (typeof value === 'object') {
        const entries = Object.entries(value)
        if (entries.length === 0) return '{}'
        return '\n' + entries.map(([key, val]) =>
          `${indent(level)}${key}: ${serializeValue(val, level + 1).trim()}`
        ).join('\n')
      }
      return String(value)
    }

    let yaml = '# Plan DAG Configuration\n'
    yaml += `# Generated on ${new Date().toISOString()}\n\n`
    yaml += `version: "${dag.version || '1.0.0'}"\n\n`

    if (dag.metadata) {
      yaml += 'metadata:\n'
      Object.entries(dag.metadata).forEach(([key, value]) => {
        yaml += `  ${key}: ${serializeValue(value, 2).trim()}\n`
      })
      yaml += '\n'
    }

    yaml += 'nodes:\n'
    dag.nodes.forEach((node: any) => {
      yaml += `  - id: "${node.id}"\n`
      yaml += `    nodeType: "${node.nodeType}"\n`
      if (node.position) {
        yaml += `    position:\n`
        yaml += `      x: ${node.position.x}\n`
        yaml += `      y: ${node.position.y}\n`
      }
      if (node.metadata) {
        yaml += `    metadata:\n`
        Object.entries(node.metadata).forEach(([key, value]) => {
          yaml += `      ${key}: ${serializeValue(value, 3).trim()}\n`
        })
      }
      if (node.config) {
        const config = typeof node.config === 'string' ? JSON.parse(node.config) : node.config
        yaml += `    config:\n`
        Object.entries(config).forEach(([key, value]) => {
          yaml += `      ${key}: ${serializeValue(value, 3).trim()}\n`
        })
      }
      yaml += '\n'
    })

    yaml += 'edges:\n'
    dag.edges.forEach((edge: any) => {
      yaml += `  - id: "${edge.id}"\n`
      yaml += `    source: "${edge.source}"\n`
      yaml += `    target: "${edge.target}"\n`
      if (edge.sourceHandle) yaml += `    sourceHandle: "${edge.sourceHandle}"\n`
      if (edge.targetHandle) yaml += `    targetHandle: "${edge.targetHandle}"\n`
      if (edge.metadata) {
        yaml += `    metadata:\n`
        Object.entries(edge.metadata).forEach(([key, value]) => {
          yaml += `      ${key}: ${serializeValue(value, 3).trim()}\n`
        })
      }
      yaml += '\n'
    })

    return yaml
  }

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
    <Stack h="100%" gap={0}>
      <style>{`
        .react-flow__node {
          border: none !important;
          padding: 0 !important;
          background: transparent !important;
        }
      `}</style>
      <Group justify="space-between" p="md" bg="gray.0">
        <Group gap="md">
          <Title order={3}>Plan DAG Editor</Title>
          {/* Online Users Status */}
          <Tooltip label={`Collaboration: ${collaboration.connectionState}`}>
            <Badge
              variant="light"
              color={collaboration.connected ? "green" : "gray"}
              leftSection={collaboration.connected ? <IconNetwork size="0.7rem" /> : <IconNetworkOff size="0.7rem" />}
            >
              {onlineUsers.length} online
            </Badge>
          </Tooltip>
        </Group>
        <Group gap="xs">
          {isDirty && (
            <Text size="sm" c="yellow.6">
              Unsaved changes
            </Text>
          )}
          <Tooltip label="Download Plan DAG as YAML">
            <ActionIcon
              variant="subtle"
              onClick={handleDownloadYAML}
              disabled={!planDag}
            >
              <IconDownload size="1.2rem" />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>


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
          connectionLineStyle={{ stroke: '#868e96', strokeWidth: 2 }}
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