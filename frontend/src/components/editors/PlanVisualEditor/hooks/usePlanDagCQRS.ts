import { useState, useCallback, useRef, useMemo, useEffect } from 'react'
import { useNodesState, useEdgesState, Node, Edge } from 'reactflow'
import { useApolloClient } from '@apollo/client/react'
import { PlanDag, PlanDagNodeType } from '../../../../types/plan-dag'
import { PlanDagCQRSService } from '../../../../services/PlanDagCQRSService'
import { ReactFlowAdapter } from '../../../../adapters/ReactFlowAdapter'
import { useUnifiedUpdateManager } from './useUnifiedUpdateManager'
import { usePerformanceMonitor } from './usePerformanceMonitor'
import { useStableCallback, useExternalDataChangeDetector } from '../../../../hooks/useStableReference'
import { useSubscriptionFilter } from '../../../../hooks/useGraphQLSubscriptionFilter'

interface UsePlanDagCQRSOptions {
  projectId: number
  planId: number
  readonly?: boolean
  onNodeEdit?: (nodeId: string) => void
  onNodeDelete?: (nodeId: string) => void
}

interface PlanDagCQRSResult {
  // Data state
  planDag: PlanDag | null
  loading: boolean
  error: any

  // ReactFlow state
  nodes: Node[]
  edges: Edge[]
  setNodes: (nodes: Node[] | ((nodes: Node[]) => Node[])) => void
  setEdges: (edges: Edge[] | ((edges: Edge[]) => Edge[])) => void
  onNodesChange: (changes: any[]) => void
  onEdgesChange: (changes: any[]) => void

  // Validation state
  validationErrors: any[]
  validationLoading: boolean
  lastValidation: Date | null

  // Update management
  isDirty: boolean
  updateManager: ReturnType<typeof useUnifiedUpdateManager>

  // Performance monitoring
  performanceMonitor: ReturnType<typeof usePerformanceMonitor>

  // CQRS service
  cqrsService: PlanDagCQRSService

  // Actions
  savePlanDag: () => Promise<void>
  validatePlanDag: () => void
  refreshData: () => void
  setDragging: (dragging: boolean) => void
  updatePlanDagOptimistically: (updater: (current: PlanDag | null) => PlanDag | null) => void
}

// Deep equality check for Plan DAG objects with performance optimisation
const planDagEqual = (a: PlanDag | null, b: PlanDag | null): boolean => {
  if (a === b) return true
  if (!a || !b) return false

  // Quick checks first
  if (a.version !== b.version) return false
  if (a.nodes.length !== b.nodes.length) return false
  if (a.edges.length !== b.edges.length) return false

  // Only do expensive checks if quick checks pass
  try {
    return JSON.stringify(a) === JSON.stringify(b)
  } catch {
    return false
  }
}

export const usePlanDagCQRS = (options: UsePlanDagCQRSOptions): PlanDagCQRSResult => {
  const { projectId, planId, readonly = false, onNodeEdit, onNodeDelete } = options

  // Apollo client for CQRS service
  const apollo = useApolloClient()

  // Get client ID at top level to follow React hook rules
  const { clientId } = useSubscriptionFilter()

  // Initialize CQRS service once per planId/clientId
  const cqrsServiceRef = useRef<PlanDagCQRSService | null>(null)
  if (!cqrsServiceRef.current) {
    cqrsServiceRef.current = new PlanDagCQRSService(apollo, clientId)
  }
  const cqrsService = cqrsServiceRef.current

  // Local state
  const [planDag, setPlanDag] = useState<PlanDag | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<any>(null)
  const [isDirty, setIsDirty] = useState(false)

  // Performance monitoring
  const performanceMonitor = usePerformanceMonitor({
    enabled: !readonly,
    maxRenderTime: 16, // 60fps budget
    maxRendersPerSecond: 60,
    maxEventFrequency: 10,
    memoryWarningThreshold: 150, // MB
  })
  const trackPerformanceEvent = performanceMonitor.trackEvent

  // Consolidated state refs for better organisation and debugging
  const editorStateRef = useRef({
    planDag: {
      current: null as PlanDag | null,
      previous: null as PlanDag | null,
      stable: null as PlanDag | null,
    },
    sync: {
      isDragging: false,
      isExternalSync: false,
      isInitialized: false,
      previousChangeId: 0,
    },
    subscriptions: null as any,
  })

  // Stable plan DAG with change detection
  const stablePlanDag = useMemo(() => {
    if (!planDag) return null

    const isNewData = !editorStateRef.current.planDag.previous || !planDagEqual(editorStateRef.current.planDag.previous, planDag)
    if (isNewData) {
      console.log('[usePlanDagCQRS] Plan DAG data changed, updating stable reference')
      editorStateRef.current.planDag.previous = planDag
      editorStateRef.current.planDag.stable = planDag
    }

    return editorStateRef.current.planDag.stable
  }, [planDag])

  // TODO: Re-implement validation in Phase 3
  // Validation temporarily disabled after removing dead code dependencies

  // Stable callbacks for update manager to prevent reference instability
  const stableOnValidationNeeded = useStableCallback((_planDag: PlanDag) => {
    // Validation disabled for now
    console.log('[usePlanDagCQRS] Validation skipped (to be re-implemented in Phase 3)')
  })

  const stableOnPersistenceNeeded = useStableCallback(async (planDag: PlanDag) => {
    try {
      await cqrsService.commands.updatePlanDag({
        projectId,
        planId,
        planDag
      })
      setIsDirty(false)
      console.log('[usePlanDagCQRS] Plan DAG saved successfully via CQRS')
    } catch (error) {
      console.error('[usePlanDagCQRS] Failed to save Plan DAG:', error)
      throw error
    }
  })

  // Unified update manager
  const updateManager = useUnifiedUpdateManager({
    onValidationNeeded: stableOnValidationNeeded,
    onPersistenceNeeded: stableOnPersistenceNeeded,
    debounceMs: 500,
    throttleMs: 1000,
  })

  // ReactFlow conversion using new adapter with stable callback references
  const stableOnNodeEdit = useStableCallback(onNodeEdit || (() => {}))
  const stableOnNodeDelete = useStableCallback(onNodeDelete || (() => {}))

  const reactFlowData = useMemo(() => {
    if (!stablePlanDag) {
      return { nodes: [], edges: [] }
    }

    console.log('[usePlanDagCQRS] Converting Plan DAG to ReactFlow format via adapter')
    const converted = ReactFlowAdapter.planDagToReactFlow(stablePlanDag)

    // Add node callbacks and edge list to converted nodes
    const nodesWithCallbacks = converted.nodes.map(node => {
      const nodeType = node.data.nodeType
      const parsedConfig =
        node.data.config && typeof node.data.config === 'object'
          ? node.data.config
          : {}

      let hasValidConfig = false
      switch (nodeType) {
        case PlanDagNodeType.DATA_SOURCE: {
          const dataSetId = (parsedConfig as any)?.dataSetId
          const numericId =
            typeof dataSetId === 'number' ? dataSetId : Number(dataSetId)
          hasValidConfig = Number.isFinite(numericId) && numericId > 0
          break
        }
        case PlanDagNodeType.GRAPH:
          hasValidConfig = true
          break
        default:
          hasValidConfig = Object.keys(parsedConfig as Record<string, unknown>).length > 0
          break
      }

      return {
        ...node,
        data: {
          ...node.data,
          projectId, // Add projectId for preview queries
          planId,
          onEdit: () => stableOnNodeEdit(node.id),
          onDelete: () => stableOnNodeDelete(node.id),
          readonly,
          edges: converted.edges, // Add edges for node validation/display
          hasValidConfig
        }
      }
    })

    return { nodes: nodesWithCallbacks, edges: converted.edges }
  }, [stablePlanDag, stableOnNodeEdit, stableOnNodeDelete, readonly, projectId, planId])

  // ReactFlow state
  const [nodes, setNodes, onNodesChangeInternal] = useNodesState(reactFlowData.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(reactFlowData.edges)

  // Use the internal onNodesChange directly
  const onNodesChange = onNodesChangeInternal

  // External data change detection to prevent circular dependencies
  const reactFlowDataChange = useExternalDataChangeDetector(reactFlowData)

  // Sync ReactFlow state when external data changes - FIXED: prevent infinite loop
  useEffect(() => {
    // Skip if we're currently syncing from external changes (prevents React 18 double render issues)
    if (editorStateRef.current.sync.isExternalSync) {
      return
    }

    // Skip syncing during drag operations to prevent subscription echo interference
    // Position changes during drag are cosmetic only - actual save happens in handleNodeDragStop
    if (editorStateRef.current.sync.isDragging) {
      return
    }

    // Only sync when external reactFlowData has actually changed
    if (reactFlowDataChange.hasChanged) {
      const hasNewData = reactFlowData.nodes.length > 0 || reactFlowData.edges.length > 0
      const isCurrentEmpty = nodes.length === 0 && edges.length === 0

      // Check if node positions or data have changed (for real-time collaboration)
      const hasNodeChanges = reactFlowData.nodes.some((newNode, idx) => {
        const currentNode = nodes[idx]
        if (!currentNode) return true

        // Check position changes
        const posChanged = newNode.position.x !== currentNode.position.x ||
                          newNode.position.y !== currentNode.position.y

        // Check if node IDs are different (reordering/replacement)
        const idChanged = newNode.id !== currentNode.id

        return posChanged || idChanged
      })

      // Check if external data has MORE/FEWER items (additions/removals from other users)
      const hasMoreNodes = reactFlowData.nodes.length > nodes.length
      const hasMoreEdges = reactFlowData.edges.length > edges.length
      const hasFewerNodes = reactFlowData.nodes.length < nodes.length
      const hasFewerEdges = reactFlowData.edges.length < edges.length

      const buildEdgeSignature = (edge: Edge) => {
        const metadata = (edge.data as any)?.metadata
        return [
          edge.id,
          edge.source,
          edge.target,
          metadata?.dataType ?? '',
          metadata?.label ?? edge.label ?? ''
        ].join('|')
      }

      const currentEdgeSignatures = new Map(edges.map(edge => [edge.id, buildEdgeSignature(edge)]))
      const hasEdgeChanges = reactFlowData.edges.some(edge => currentEdgeSignatures.get(edge.id) !== buildEdgeSignature(edge))

      // Only sync when:
      // 1. First load (empty)
      // 2. External data has more/fewer items (additions/removals from other users)
      // 3. Same count but content changed (updates from other users)
      const shouldSync = hasNewData && (
        isCurrentEmpty ||
        hasMoreNodes ||
        hasMoreEdges ||
        hasFewerNodes ||
        hasFewerEdges ||
        hasNodeChanges ||
        hasEdgeChanges
      )

      if (shouldSync) {
        const reason = isCurrentEmpty ? 'empty' :
                      hasMoreNodes ? 'nodes-added' :
                      hasMoreEdges ? 'edges-added' :
                      hasFewerNodes ? 'nodes-removed' :
                      hasFewerEdges ? 'edges-removed' :
                      hasNodeChanges ? 'node-changed' :
                      hasEdgeChanges ? 'edge-changed' : 'unknown'

        console.log('[usePlanDagCQRS] Syncing ReactFlow state from external data change:', {
          changeId: reactFlowDataChange.changeId,
          newNodes: reactFlowData.nodes.length,
          newEdges: reactFlowData.edges.length,
          currentNodes: nodes.length,
          currentEdges: edges.length,
          reason
        })
        editorStateRef.current.sync.isExternalSync = true

        // BUGFIX: Merge nodes instead of replacing to preserve event handlers
        // When we replace the entire array, ReactFlow loses event listener bindings
        if (isCurrentEmpty) {
          // First load - set directly
          setNodes(reactFlowData.nodes)
        } else {
          // Merge updates: preserve existing nodes, update changed fields, add new nodes
          setNodes((currentNodes) => {
            const newNodesMap = new Map(reactFlowData.nodes.map(n => [n.id, n]))
            const currentNodesMap = new Map(currentNodes.map(n => [n.id, n]))

            // Merge existing nodes with updates
            const mergedNodes = currentNodes.map(currentNode => {
              const newNode = newNodesMap.get(currentNode.id)
              if (!newNode) {
                // Node was deleted
                return null
              }

              // Check if anything actually changed
              const positionChanged =
                newNode.position.x !== currentNode.position.x ||
                newNode.position.y !== currentNode.position.y
              const dataChanged = JSON.stringify(newNode.data) !== JSON.stringify(currentNode.data)

              if (!positionChanged && !dataChanged) {
                // Nothing changed, keep existing node object to preserve identity
                return currentNode
              }

              // Update changed fields while preserving node identity
              return {
                ...currentNode,
                position: newNode.position,
                data: {
                  ...currentNode.data,
                  ...newNode.data,
                  // Ensure event handlers are always present
                  onEdit: currentNode.data.onEdit || newNode.data.onEdit,
                  onDelete: currentNode.data.onDelete || newNode.data.onDelete,
                }
              }
            }).filter(Boolean) as Node[]

            // Add any new nodes that don't exist in current
            reactFlowData.nodes.forEach(newNode => {
              if (!currentNodesMap.has(newNode.id)) {
                mergedNodes.push(newNode)
              }
            })

            return mergedNodes
          })
        }

        setEdges(reactFlowData.edges)
        editorStateRef.current.sync.previousChangeId = reactFlowDataChange.changeId
        // Use setTimeout to clear the flag after state updates have propagated
        setTimeout(() => {
          editorStateRef.current.sync.isExternalSync = false
        }, 0)
      }
    }
    // Depend on reactFlowDataChange to detect all changes (positions, length, etc.)
  }, [reactFlowDataChange, reactFlowData, nodes, edges, setNodes, setEdges])

  // Load initial data and setup subscription - FIXED: prevent infinite loop
  useEffect(() => {
    // Only run setup once per project to prevent infinite loops
    if (editorStateRef.current.sync.isInitialized) {
      return
    }

    console.log('[usePlanDagCQRS] Setting up data loading and subscription for project:', projectId, 'plan:', planId)
    editorStateRef.current.sync.isInitialized = true

    const loadInitialData = async () => {
      try {
        setLoading(true)
        setError(null)

        // Load initial data using CQRS query service with fresh fetch to avoid stale cache
        const initialData = await cqrsService.queries.getPlanDag({ projectId, planId, fresh: true })

        if (initialData) {
          console.log('[usePlanDagCQRS] Initial Plan DAG loaded:', initialData)
          setPlanDag(initialData)
        } else {
          console.log('[usePlanDagCQRS] No Plan DAG found for project')
          setPlanDag(null)
        }
      } catch (err) {
        console.error('[usePlanDagCQRS] Error loading initial data:', err)
        setError(err)
      } finally {
        setLoading(false)
      }
    }

    // Setup delta-based subscription for efficient real-time updates
    const deltaSubscription = cqrsService.subscribeToDeltaUpdates(
      projectId,
      planId,
      () => editorStateRef.current.planDag.stable, // Get current Plan DAG for patch application
      (updatedPlanDag) => {
        console.log('[usePlanDagCQRS] Received delta update via JSON Patch subscription')
        trackPerformanceEvent('websocketMessages')
        setPlanDag(updatedPlanDag)
      },
      (error) => {
        console.error('[usePlanDagCQRS] Delta subscription error, triggering full refresh:', error)
        // On delta error, refresh full state
        cqrsService.queries.getPlanDag({ projectId, planId, fresh: true })
          .then(refreshedData => {
            if (refreshedData) {
              setPlanDag(refreshedData)
            }
          })
          .catch(err => setError(err))
      }
    )

    // Setup execution status subscription for real-time status updates
    const executionStatusSubscription = cqrsService.subscribeToExecutionStatusUpdates(
      projectId,
      planId,
      (nodeId, executionData) => {
        console.log('[usePlanDagCQRS] Received execution status update for node:', nodeId, executionData)
        trackPerformanceEvent('websocketMessages')

        // Update Plan DAG with execution status
        setPlanDag(current => {
          if (!current) return current

          const updated = {
            ...current,
            nodes: current.nodes.map(n =>
              n.id === nodeId
                ? { ...n, ...executionData }
                : n
            )
          }

          // Update stable ref immediately to prevent sync from using stale data
          editorStateRef.current.planDag.stable = updated
          editorStateRef.current.planDag.previous = updated

          return updated
        })
      },
      (error) => {
        console.error('[usePlanDagCQRS] Execution status subscription error:', error)
      }
    )

    editorStateRef.current.subscriptions = {
      delta: deltaSubscription,
      executionStatus: executionStatusSubscription,
    }
    loadInitialData()

    return () => {
      if (editorStateRef.current.subscriptions) {
        // Unsubscribe from both subscriptions
        if (editorStateRef.current.subscriptions.delta) {
          editorStateRef.current.subscriptions.delta.unsubscribe()
        }
        if (editorStateRef.current.subscriptions.executionStatus) {
          editorStateRef.current.subscriptions.executionStatus.unsubscribe()
        }
        editorStateRef.current.subscriptions = null
      }
      editorStateRef.current.sync.isInitialized = false
    }
  }, [projectId, planId, cqrsService, trackPerformanceEvent, readonly])  // Re-run when project or plan changes

  // Actions
  const savePlanDag = useCallback(async () => {
    if (!stablePlanDag || readonly) return

    console.log('[usePlanDagCQRS] Manually saving Plan DAG')
    updateManager.scheduleStructuralUpdate(stablePlanDag, 'manual-save')
    await updateManager.flushOperations()
  }, [stablePlanDag, readonly, updateManager])

  const validatePlanDag = useCallback(() => {
    if (stablePlanDag) {
      console.log('[usePlanDagCQRS] Validation not yet implemented (Phase 3)')
      performanceMonitor.trackEvent('validations')
      // TODO: Re-implement validation in Phase 3
    }
  }, [stablePlanDag, performanceMonitor])

  const refreshData = useCallback(async () => {
    console.log('[usePlanDagCQRS] Refreshing Plan DAG data')
    try {
      setLoading(true)
      const refreshedData = await cqrsService.queries.getPlanDag({ projectId, planId })
      setPlanDag(refreshedData)
    } catch (err) {
      console.error('[usePlanDagCQRS] Error refreshing data:', err)
      setError(err)
    } finally {
      setLoading(false)
    }
  }, [cqrsService, projectId])

  // Drag state control
  const setDragging = useCallback((dragging: boolean) => {
    editorStateRef.current.sync.isDragging = dragging
  }, [])

  // Optimistic update for planDag (used after mutations to prevent stale data syncs)
  const updatePlanDagOptimistically = useCallback((updater: (current: PlanDag | null) => PlanDag | null) => {
    console.log('[usePlanDagCQRS] Applying optimistic update to Plan DAG')
    setPlanDag(current => {
      const updated = updater(current)
      if (updated) {
        // Update stable ref immediately to prevent sync from using stale data
        editorStateRef.current.planDag.stable = updated
        editorStateRef.current.planDag.previous = updated
      }
      return updated
    })
  }, [])

  return {
    // Data state
    planDag: stablePlanDag,
    loading,
    error,

    // ReactFlow state
    nodes,
    edges,
    setNodes,
    setEdges,
    onNodesChange,
    onEdgesChange,

    // Validation state (temporarily disabled - to be re-implemented in Phase 3)
    validationErrors: [],
    validationLoading: false,
    lastValidation: null,

    // Update management
    isDirty,
    updateManager,

    // Performance monitoring
    performanceMonitor,

    // CQRS service
    cqrsService,

    // Actions
    savePlanDag,
    validatePlanDag,
    refreshData,
    setDragging, // Control drag state to suppress external syncs
    updatePlanDagOptimistically, // Optimistic updates to prevent sync overwrites
  }
}
