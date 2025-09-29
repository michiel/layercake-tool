import { useState, useCallback, useRef, useMemo, useEffect } from 'react'
import { useNodesState, useEdgesState, Node, Edge } from 'reactflow'
import { useApolloClient } from '@apollo/client/react'
import { PlanDag } from '../../../../types/plan-dag'
import { PlanDagCQRSService } from '../../../../services/PlanDagCQRSService'
import { ReactFlowAdapter } from '../../../../adapters/ReactFlowAdapter'
import { useUnifiedUpdateManager } from './useUnifiedUpdateManager'
import { useSmartValidation } from './useSmartValidation'
import { usePerformanceMonitor } from './usePerformanceMonitor'
import { useStableCallback, useExternalDataChangeDetector } from '../../../../hooks/useStableReference'

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

  // Initialize CQRS service
  const cqrsService = useMemo(() => {
    return new PlanDagCQRSService(apollo)
  }, [apollo])

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

  // Smart validation system
  const smartValidation = useSmartValidation({
    enabled: !readonly,
    debounceMs: 1500,
    maxValidationRate: 8, // Max 8 validations per minute
  })

  // Stable callbacks for update manager to prevent reference instability
  const stableOnValidationNeeded = useStableCallback((planDag: PlanDag) => {
    smartValidation.scheduleValidation(planDag, 'structural')
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

  // ReactFlow conversion using new adapter
  const reactFlowData = useMemo(() => {
    if (!stablePlanDag) {
      return { nodes: [], edges: [] }
    }

    console.log('[usePlanDagCQRS] Converting Plan DAG to ReactFlow format via adapter')
    const converted = ReactFlowAdapter.planDagToReactFlow(stablePlanDag)

    // Add node callbacks to converted nodes
    const nodesWithCallbacks = converted.nodes.map(node => ({
      ...node,
      data: {
        ...node.data,
        onEdit: onNodeEdit,
        onDelete: onNodeDelete,
        readonly,
        hasValidConfig: node.data.originalNode?.config &&
          Object.keys(node.data.originalNode.config).length > 0
      }
    }))

    return { nodes: nodesWithCallbacks, edges: converted.edges }
  }, [stablePlanDag, onNodeEdit, onNodeDelete, readonly])

  // ReactFlow state
  const [nodes, setNodes, onNodesChange] = useNodesState(reactFlowData.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(reactFlowData.edges)

  // External data change detection to prevent circular dependencies
  const reactFlowDataChange = useExternalDataChangeDetector(reactFlowData)

  // Sync ReactFlow state when external data changes - FIXED: prevent infinite loop
  useEffect(() => {
    // Only sync when external reactFlowData has actually changed
    if (reactFlowDataChange.hasChanged) {
      const hasNewData = reactFlowData.nodes.length > 0 || reactFlowData.edges.length > 0
      const isCurrentEmpty = nodes.length === 0 && edges.length === 0
      const isDifferentLength = reactFlowData.nodes.length !== nodes.length || reactFlowData.edges.length !== edges.length

      const shouldSync = hasNewData && (isCurrentEmpty || isDifferentLength)

      if (shouldSync) {
        console.log('[usePlanDagCQRS] Syncing ReactFlow state from external data change:', {
          changeId: reactFlowDataChange.changeId,
          newNodes: reactFlowData.nodes.length,
          newEdges: reactFlowData.edges.length,
          currentNodes: nodes.length,
          currentEdges: edges.length
        })
        setNodes(reactFlowData.nodes)
        setEdges(reactFlowData.edges)
      }
    }
    // Only depend on external data change detection, not local state
  }, [reactFlowDataChange.changeId, reactFlowData.nodes.length, reactFlowData.edges.length])

  // Load initial data and setup subscription
  useEffect(() => {
    console.log('[usePlanDagCQRS] Setting up data loading and subscription for project:', projectId)

    const loadInitialData = async () => {
      try {
        setLoading(true)
        setError(null)

        // Load initial data using CQRS query service
        const initialData = await cqrsService.queries.getPlanDag({ projectId })

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

    // Setup real-time subscription using CQRS query service
    const subscription = cqrsService.subscribeToReactFlowUpdates(
      projectId,
      (nodes, edges) => {
        console.log('[usePlanDagCQRS] Received real-time update via CQRS subscription')
        performanceMonitor.trackEvent('websocketMessages')

        // Convert ReactFlow data back to Plan DAG using adapter
        const updatedPlanDag = ReactFlowAdapter.reactFlowToPlanDag(nodes, edges)
        setPlanDag(updatedPlanDag)
      },
      (error) => {
        console.error('[usePlanDagCQRS] Subscription error:', error)
        setError(error)
      }
    )

    subscriptionRef.current = subscription
    loadInitialData()

    return () => {
      if (subscriptionRef.current) {
        subscriptionRef.current.unsubscribe()
        subscriptionRef.current = null
      }
    }
  }, [projectId, cqrsService, performanceMonitor])

  // Actions
  const savePlanDag = useCallback(async () => {
    if (!stablePlanDag || readonly) return

    console.log('[usePlanDagCQRS] Manually saving Plan DAG')
    updateManager.scheduleStructuralUpdate(stablePlanDag, 'manual-save')
    await updateManager.flushOperations()
  }, [stablePlanDag, readonly, updateManager])

  const validatePlanDag = useCallback(() => {
    if (stablePlanDag) {
      console.log('[usePlanDagCQRS] Validating Plan DAG')
      performanceMonitor.trackEvent('validations')
      smartValidation.validateNow(stablePlanDag)
    }
  }, [stablePlanDag, smartValidation, performanceMonitor])

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

    // Validation state (from smart validation)
    validationErrors: smartValidation.errors,
    validationLoading: smartValidation.isValidating,
    lastValidation: smartValidation.lastValidation,

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
  }
}