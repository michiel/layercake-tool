import { useState, useCallback, useRef, useMemo, useEffect } from 'react'
import { useNodesState, useEdgesState, Node, Edge } from 'reactflow'
import { PlanDag, PlanDagNodeType } from '../../../../types/plan-dag'
import { usePlanDag, usePlanDagMutations, usePlanDagSubscription } from '../../../../hooks/usePlanDag'
import { useUnifiedUpdateManager } from './useUnifiedUpdateManager'
import { useSmartValidation } from './useSmartValidation'
import { usePerformanceMonitor } from './usePerformanceMonitor'
import { useStableCallback, useExternalDataChangeDetector } from '../../../../hooks/useStableReference'
import { useSubscriptionFilter } from '../../../../hooks/useGraphQLSubscriptionFilter'

interface UsePlanDagStateOptions {
  projectId: number
  readonly?: boolean
  onNodeEdit?: (nodeId: string) => void
  onNodeDelete?: (nodeId: string) => void
}

interface PlanDagStateResult {
  // Data state
  planDag: PlanDag | null
  loading: boolean
  error: any
  loadingTimeout: boolean

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


export const usePlanDagState = (options: UsePlanDagStateOptions): PlanDagStateResult => {
  const { projectId, readonly = false, onNodeEdit, onNodeDelete } = options

  // Stable callbacks to prevent reference instability
  const stableOnNodeEdit = useStableCallback(onNodeEdit || (() => {}))
  const stableOnNodeDelete = useStableCallback(onNodeDelete || (() => {}))

  // Subscription filtering to prevent client self-updates
  const { filterSubscriptionData } = useSubscriptionFilter()

  // GraphQL data with error handling
  const { planDag: rawPlanDag, loading, error, refetch } = usePlanDag(projectId)
  const { lastChange } = usePlanDagSubscription(projectId)
  const mutations = usePlanDagMutations(projectId)

  // Timeout detection for stuck loading states
  const loadingTimeoutRef = useRef<number | null>(null)
  const [loadingTimeout, setLoadingTimeout] = useState(false)

  useEffect(() => {
    if (loading && !loadingTimeout) {
      // Set a timeout to detect if loading is stuck
      loadingTimeoutRef.current = setTimeout(() => {
        console.error('GraphQL query timeout - loading stuck for 30 seconds')
        setLoadingTimeout(true)
      }, 30000) // 30 second timeout
    } else {
      // Clear timeout when loading completes
      if (loadingTimeoutRef.current) {
        clearTimeout(loadingTimeoutRef.current)
        loadingTimeoutRef.current = null
      }
      if (loadingTimeout && !loading) {
        setLoadingTimeout(false)
      }
    }

    return () => {
      if (loadingTimeoutRef.current) {
        clearTimeout(loadingTimeoutRef.current)
      }
    }
  }, [loading, loadingTimeout])

  // Local state
  const [isDirty, setIsDirty] = useState(false)

  // Performance monitoring (temporarily disabled - see usePerformanceMonitor refactor)
  const performanceMonitor = usePerformanceMonitor({
    enabled: false, // Disabled due to infinite render loop bug
    maxRenderTime: 16, // 60fps budget
    maxRendersPerSecond: 60,
    maxEventFrequency: 10,
    memoryWarningThreshold: 150, // MB
  })

  // Refs for stable comparisons
  const previousPlanDagRef = useRef<PlanDag | null>(null)
  const stablePlanDagRef = useRef<PlanDag | null>(null)

  // Stable conversion function to prevent infinite loops
  const convertPlanDagToReactFlow = useCallback((
    planDag: PlanDag,
    onEdit?: (nodeId: string) => void,
    onDelete?: (nodeId: string) => void,
    readonly?: boolean
  ): { nodes: Node[]; edges: Edge[] } => {
    if (!planDag.edges || !Array.isArray(planDag.edges)) {
      return { nodes: [], edges: [] }
    }

    const edges: Edge[] = planDag.edges.map((edge: any) => ({
      id: String(edge.id),
      source: String(edge.source),
      target: String(edge.target),
      sourceHandle: edge.sourceHandle || null,
      targetHandle: edge.targetHandle || null,
      type: 'smoothstep',
      animated: false,
      label: edge.metadata?.label || 'Data',
      metadata: edge.metadata || { label: 'Data', dataType: 'GraphData' },
      style: {
        stroke: edge.metadata?.dataType === 'GraphReference' ? '#228be6' : '#868e96',
        strokeWidth: 2,
      },
      labelStyle: {
        fontSize: 12,
        fontWeight: 500,
      },
    }))

    const nodes: Node[] = planDag.nodes.map((node: any) => {
      const nodeType = typeof node.nodeType === 'string' &&
        (Object.values(PlanDagNodeType) as string[]).includes(node.nodeType) ?
        node.nodeType as PlanDagNodeType : PlanDagNodeType.DATA_SOURCE

      const hasValidConfig = node.config &&
        (typeof node.config === 'object' ||
         (typeof node.config === 'string' && node.config.trim() !== '{}' && node.config.trim() !== ''))

      return {
        id: node.id,
        position: node.position,
        type: nodeType,
        data: {
          label: node.metadata.label,
          nodeType,
          config: typeof node.config === 'string' ? (() => {
            try {
              return JSON.parse(node.config)
            } catch (e) {
              return {}
            }
          })() : node.config,
          metadata: node.metadata,
          onEdit,
          onDelete,
          readonly,
          edges,
          hasValidConfig,
        },
        draggable: true,
        selectable: true,
      }
    })

    return { nodes, edges }
  }, []) // No dependencies - this function is pure

  // Convert raw GraphQL data to properly typed PlanDag
  const convertRawPlanDag = useCallback((raw: any): PlanDag | null => {
    if (!raw) return null

    return {
      ...raw,
      nodes: raw.nodes.map((node: any) => ({
        ...node,
        nodeType: typeof node.nodeType === 'string' &&
          (Object.values(PlanDagNodeType) as string[]).includes(node.nodeType) ?
          node.nodeType as PlanDagNodeType : PlanDagNodeType.DATA_SOURCE
      }))
    }
  }, [])

  // Stable plan DAG with change detection
  const stablePlanDag = useMemo(() => {
    if (!rawPlanDag) return null

    const convertedPlanDag = convertRawPlanDag(rawPlanDag)
    if (!convertedPlanDag) return null

    const isNewData = !previousPlanDagRef.current || !planDagEqual(previousPlanDagRef.current, convertedPlanDag)
    if (isNewData) {
      previousPlanDagRef.current = convertedPlanDag
      stablePlanDagRef.current = convertedPlanDag
    }

    return stablePlanDagRef.current
  }, [rawPlanDag, convertRawPlanDag])

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
      await mutations.updatePlanDag(planDag)
      setIsDirty(false)
    } catch (error) {
      console.error('Failed to save Plan DAG:', error)
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

  // ReactFlow conversion with stable callbacks
  const reactFlowData = useMemo(() => {
    if (!stablePlanDag) {
      return { nodes: [], edges: [] }
    }
    return convertPlanDagToReactFlow(stablePlanDag, stableOnNodeEdit, stableOnNodeDelete, readonly)
  }, [stablePlanDag, stableOnNodeEdit, stableOnNodeDelete, readonly, convertPlanDagToReactFlow])

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
        console.log('[usePlanDagState] Syncing ReactFlow state from external data change:', {
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
    // Only depend on stable change detection ID, not objects that can cause infinite loops
  }, [reactFlowDataChange.changeId])

  // Handle real-time changes with subscription filtering
  useEffect(() => {
    if (lastChange && stablePlanDag) {
      // Filter out updates from this client to prevent circular updates
      const filteredChange = filterSubscriptionData(lastChange)

      if (filteredChange) {
        console.log('[usePlanDagState] Processing filtered real-time change:', filteredChange)
        performanceMonitor.trackEvent('websocketMessages')
        updateManager.scheduleStructuralUpdate(stablePlanDag, 'real-time-change')
      } else {
        console.log('[usePlanDagState] Filtered out own subscription update')
      }
    }
    // Only depend on stable data, not objects that change on every render
  }, [lastChange, stablePlanDag])

  // Actions
  const savePlanDag = useCallback(async () => {
    if (!stablePlanDag || readonly) return

    updateManager.scheduleStructuralUpdate(stablePlanDag, 'manual-save')
    await updateManager.flushOperations()
  }, [stablePlanDag, readonly, updateManager])

  const validatePlanDag = useCallback(() => {
    if (stablePlanDag) {
      performanceMonitor.trackEvent('validations')
      smartValidation.validateNow(stablePlanDag)
    }
  }, [stablePlanDag, smartValidation, performanceMonitor])

  const refreshData = useCallback(() => {
    refetch()
  }, [refetch])

  return {
    // Data state
    planDag: stablePlanDag,
    loading,
    error: loadingTimeout
      ? new Error('Query timeout: Server took too long to respond. Please refresh the page.')
      : error,
    loadingTimeout,

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

    // Actions
    savePlanDag,
    validatePlanDag,
    refreshData,
  }
}