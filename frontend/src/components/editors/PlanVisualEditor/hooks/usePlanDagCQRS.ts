import { useState, useCallback, useRef, useMemo, useEffect } from 'react'
import { useNodesState, useEdgesState, Node, Edge } from 'reactflow'
import { useApolloClient } from '@apollo/client/react'
import { PlanDag } from '../../../../types/plan-dag'
import { PlanDagCQRSService } from '../../../../services/PlanDagCQRSService'
import { ReactFlowAdapter } from '../../../../adapters/ReactFlowAdapter'
import { useUnifiedUpdateManager } from './useUnifiedUpdateManager'
import { usePerformanceMonitor } from './usePerformanceMonitor'
import { useStableCallback, useExternalDataChangeDetector } from '../../../../hooks/useStableReference'
import { useSubscriptionFilter } from '../../../../hooks/useGraphQLSubscriptionFilter'

interface UsePlanDagCQRSOptions {
  projectId: number
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
  const { projectId, readonly = false, onNodeEdit, onNodeDelete } = options

  // Apollo client for CQRS service
  const apollo = useApolloClient()

  // Get client ID at top level to follow React hook rules
  const { clientId } = useSubscriptionFilter()

  // Initialize CQRS service
  const cqrsService = useMemo(() => {
    return new PlanDagCQRSService(apollo, clientId)
  }, [apollo, clientId])

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

  // Refs for stable comparisons
  const previousPlanDagRef = useRef<PlanDag | null>(null)
  const stablePlanDagRef = useRef<PlanDag | null>(null)
  const subscriptionRef = useRef<any>(null)
  const initializedRef = useRef(false)
  const previousChangeIdRef = useRef<number>(0)
  const isSyncingFromExternalRef = useRef<boolean>(false)
  const isDraggingRef = useRef<boolean>(false)

  // Stable plan DAG with change detection
  const stablePlanDag = useMemo(() => {
    if (!planDag) return null

    const isNewData = !previousPlanDagRef.current || !planDagEqual(previousPlanDagRef.current, planDag)
    if (isNewData) {
      console.log('[usePlanDagCQRS] Plan DAG data changed, updating stable reference')
      previousPlanDagRef.current = planDag
      stablePlanDagRef.current = planDag
    }

    return stablePlanDagRef.current
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
    const nodesWithCallbacks = converted.nodes.map(node => ({
      ...node,
      data: {
        ...node.data,
        projectId, // Add projectId for preview queries
        onEdit: () => stableOnNodeEdit(node.id),
        onDelete: () => stableOnNodeDelete(node.id),
        readonly,
        edges: converted.edges, // Add edges for node validation/display
        hasValidConfig: node.data.originalNode?.config &&
          Object.keys(node.data.originalNode.config).length > 0
      }
    }))

    return { nodes: nodesWithCallbacks, edges: converted.edges }
  }, [stablePlanDag, stableOnNodeEdit, stableOnNodeDelete, readonly, projectId])

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
    if (isSyncingFromExternalRef.current) {
      return
    }

    // Skip syncing during drag operations to prevent subscription echo interference
    // Position changes during drag are cosmetic only - actual save happens in handleNodeDragStop
    if (isDraggingRef.current) {
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

      // Check if external data has MORE items (additions from other users)
      // Don't sync when external data has FEWER items (we may have optimistic local changes)
      const hasMoreNodes = reactFlowData.nodes.length > nodes.length
      const hasMoreEdges = reactFlowData.edges.length > edges.length
      const hasSameOrMoreItems = reactFlowData.nodes.length >= nodes.length &&
                                 reactFlowData.edges.length >= edges.length

      // Only sync when:
      // 1. First load (empty)
      // 2. External data has more items (additions from other users)
      // 3. Same count but content changed (updates from other users)
      const shouldSync = hasNewData && (
        isCurrentEmpty ||
        hasMoreNodes ||
        hasMoreEdges ||
        (hasSameOrMoreItems && hasNodeChanges)
      )

      if (shouldSync) {
        const reason = isCurrentEmpty ? 'empty' :
                      hasMoreNodes ? 'nodes-added' :
                      hasMoreEdges ? 'edges-added' :
                      hasNodeChanges ? 'node-changed' : 'unknown'

        console.log('[usePlanDagCQRS] Syncing ReactFlow state from external data change:', {
          changeId: reactFlowDataChange.changeId,
          newNodes: reactFlowData.nodes.length,
          newEdges: reactFlowData.edges.length,
          currentNodes: nodes.length,
          currentEdges: edges.length,
          reason
        })
        isSyncingFromExternalRef.current = true
        setNodes(reactFlowData.nodes)
        setEdges(reactFlowData.edges)
        previousChangeIdRef.current = reactFlowDataChange.changeId
        // Use setTimeout to clear the flag after state updates have propagated
        setTimeout(() => {
          isSyncingFromExternalRef.current = false
        }, 0)
      }
    }
    // Depend on reactFlowDataChange to detect all changes (positions, length, etc.)
  }, [reactFlowDataChange, reactFlowData, nodes, edges])

  // Load initial data and setup subscription - FIXED: prevent infinite loop
  useEffect(() => {
    // Only run setup once per project to prevent infinite loops
    if (initializedRef.current) {
      return
    }

    console.log('[usePlanDagCQRS] Setting up data loading and subscription for project:', projectId)
    initializedRef.current = true

    const loadInitialData = async () => {
      try {
        setLoading(true)
        setError(null)

        // Load initial data using CQRS query service with fresh fetch to avoid stale cache
        const initialData = await cqrsService.queries.getPlanDag({ projectId, fresh: true })

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
    const subscription = cqrsService.subscribeToDeltaUpdates(
      projectId,
      () => stablePlanDagRef.current, // Get current Plan DAG for patch application
      (updatedPlanDag) => {
        console.log('[usePlanDagCQRS] Received delta update via JSON Patch subscription')
        performanceMonitor.trackEvent('websocketMessages')
        setPlanDag(updatedPlanDag)
      },
      (error) => {
        console.error('[usePlanDagCQRS] Delta subscription error, triggering full refresh:', error)
        // On delta error, refresh full state
        cqrsService.queries.getPlanDag({ projectId, fresh: true })
          .then(refreshedData => {
            if (refreshedData) {
              setPlanDag(refreshedData)
            }
          })
          .catch(err => setError(err))
      }
    )

    subscriptionRef.current = subscription
    loadInitialData()

    return () => {
      if (subscriptionRef.current) {
        subscriptionRef.current.unsubscribe()
        subscriptionRef.current = null
      }
      initializedRef.current = false
    }
  }, [projectId])  // Only depend on projectId to prevent infinite loops

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
      const refreshedData = await cqrsService.queries.getPlanDag({ projectId })
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
    isDraggingRef.current = dragging
  }, [])

  // Optimistic update for planDag (used after mutations to prevent stale data syncs)
  const updatePlanDagOptimistically = useCallback((updater: (current: PlanDag | null) => PlanDag | null) => {
    console.log('[usePlanDagCQRS] Applying optimistic update to Plan DAG')
    setPlanDag(current => {
      const updated = updater(current)
      if (updated) {
        // Update stable ref immediately to prevent sync from using stale data
        stablePlanDagRef.current = updated
        previousPlanDagRef.current = updated
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