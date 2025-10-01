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
import { Stack, Title, Alert, Loader, Text, ActionIcon, Tooltip, Group } from '@mantine/core'
import {
  IconAlertCircle,
  IconEye,
  IconSettings,
  IconPlayerPlay
} from '@tabler/icons-react'

import { useCollaborationV2 } from '../../../hooks/useCollaborationV2'
import { PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig, ReactFlowEdge, PlanDagNode } from '../../../types/plan-dag'
import { validateConnectionWithCycleDetection } from '../../../utils/planDagValidation'

// Import node types constant
import { NODE_TYPES } from './nodeTypes'

// Import collaboration components
import { CollaborativeCursors } from '../../collaboration/CollaborativeCursors'
import { UserPresenceData } from '../../../types/websocket'

// Import dialogs
import { NodeConfigDialog } from './NodeConfigDialog'
import { EdgeConfigDialog } from './EdgeConfigDialog'

// Import extracted components and hooks
import { ControlPanel } from './components/ControlPanel'
import { AdvancedToolbar } from './components/AdvancedToolbar'
import { ContextMenu } from './components/ContextMenu'
// import { CollaborationManager } from './components/CollaborationManager'
import { usePlanDagCQRS } from './hooks/usePlanDagCQRS'
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

const PlanVisualEditorInner = ({ projectId, onNodeSelect, onEdgeSelect, readonly = false }: PlanVisualEditorProps) => {

  // Configuration dialog state - needs to be defined early
  const [configDialogOpen, setConfigDialogOpen] = useState(false)
  const [configNodeId, setConfigNodeId] = useState<string>('')
  const [configNodeType] = useState<PlanDagNodeType>(PlanDagNodeType.DATA_SOURCE)
  const [configNodeConfig] = useState<NodeConfig>({
    inputType: 'CSVNodesFromFile',
    source: '',
    dataType: 'Nodes',
    outputGraphRef: ''
  } as DataSourceNodeConfig)
  const [configNodeMetadata] = useState<NodeMetadata>({ label: '', description: '' })

  // Node action handlers (defined with stable references)
  const handleNodeEdit = useCallback((nodeId: string) => {
    console.log('Edit node triggered:', nodeId)
    setConfigNodeId(nodeId)
    // Will access nodes via callback to avoid dependency
    setConfigDialogOpen(true)
  }, [])

  const handleNodeDelete = useCallback((nodeId: string) => {
    console.log('Node delete triggered:', nodeId)
    // Will be implemented via callback from planDagState
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
    validationErrors,
    validationLoading,
    lastValidation,
    isDirty,
    updateManager,
    validatePlanDag: handleValidate,
    cqrsService,
  } = planDagState

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

  // Advanced operations hook
  const advancedOps = useAdvancedOperations({
    nodes,
    edges,
    setNodes,
    setEdges,
    readonly,
  })

  // Handle node changes (position, selection, etc.)
  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      // Track performance for node changes
      planDagState.performanceMonitor.trackEvent('nodeChanges')

      onNodesChange(changes)

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

  // Handle flow node drag start - track initial position
  const handleFlowNodeDragStart = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      dragStartPositions.current[node.id] = { ...node.position }
    },
    []
  )

  // Handle node drag end - save position only when position actually changed
  const handleNodeDragStop = useCallback(
    (_event: React.MouseEvent, node: Node) => {
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
    [mutations, readonly, updateManager, planDag]
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

      setEdges((eds) => addEdge(newEdge, eds))

      // Convert to GraphQL edge format for mutation
      const graphqlEdge: ReactFlowEdge = {
        id: newEdge.id,
        source: newEdge.source,
        target: newEdge.target,
        sourceHandle: newEdge.sourceHandle,
        targetHandle: newEdge.targetHandle,
        metadata: newEdge.data.metadata
      }
      mutations.addEdge(graphqlEdge)
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
      <Group justify="space-between" p="md" bg="gray.0">
        <Group gap="md">
          <Title order={3}>Plan DAG Editor</Title>
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
          nodeTypes={NODE_TYPES}
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
            updatesPaused={false}
            pendingUpdates={0}
            onPauseUpdates={() => {}}
            onResumeUpdates={() => {}}
            isConnected={collaboration.connected}
            collaborationStatus={collaboration.connectionState}
            hasError={!!collaboration.error}
            onlineUsers={onlineUsers}
            onNodeDragStart={handleNodeDragStart}
            readonly={readonly}
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