import { useState, useCallback, useRef, useMemo, useEffect } from 'react'
import { useNodesState, useEdgesState, Node, Edge } from 'reactflow'
import { PlanDag, PlanDagNodeType } from '../../../../types/plan-dag'
import { usePlanDag, usePlanDagMutations, usePlanDagSubscription } from '../../../../hooks/usePlanDag'
import { useUnifiedUpdateManager } from './useUnifiedUpdateManager'
import { useSmartValidation } from './useSmartValidation'
import { usePerformanceMonitor } from './usePerformanceMonitor'

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

// Optimised conversion functions
const convertPlanDagToReactFlow = (
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
}

export const usePlanDagState = (options: UsePlanDagStateOptions): PlanDagStateResult => {
  const { projectId, readonly = false, onNodeEdit, onNodeDelete } = options

  // GraphQL data
  const { planDag: rawPlanDag, loading, error, refetch } = usePlanDag(projectId)
  const { lastChange } = usePlanDagSubscription(projectId)
  const mutations = usePlanDagMutations(projectId)

  // Local state
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

  // Unified update manager
  const updateManager = useUnifiedUpdateManager({
    onValidationNeeded: useCallback((planDag: PlanDag) => {
      // Use smart validation instead of direct validation
      smartValidation.scheduleValidation(planDag, 'structural')
    }, [smartValidation]),

    onPersistenceNeeded: useCallback(async (planDag: PlanDag) => {
      try {
        await mutations.updatePlanDag(planDag)
        setIsDirty(false)
      } catch (error) {
        console.error('Failed to save Plan DAG:', error)
        throw error
      }
    }, [mutations]),

    debounceMs: 500,
    throttleMs: 1000,
  })

  // ReactFlow conversion
  const reactFlowData = useMemo(() => {
    if (!stablePlanDag) {
      return { nodes: [], edges: [] }
    }
    return convertPlanDagToReactFlow(stablePlanDag, onNodeEdit, onNodeDelete, readonly)
  }, [stablePlanDag, onNodeEdit, onNodeDelete, readonly])

  // ReactFlow state
  const [nodes, setNodes, onNodesChange] = useNodesState(reactFlowData.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(reactFlowData.edges)

  // Sync ReactFlow state when data changes
  useEffect(() => {
    const shouldSync = (reactFlowData.nodes.length > 0 || reactFlowData.edges.length > 0) &&
                      (nodes.length === 0 && edges.length === 0) ||
                      reactFlowData.nodes.length !== nodes.length ||
                      reactFlowData.edges.length !== edges.length

    if (shouldSync) {
      setNodes(reactFlowData.nodes)
      setEdges(reactFlowData.edges)
    }
  }, [reactFlowData, nodes.length, edges.length, setNodes, setEdges])

  // Handle real-time changes
  useEffect(() => {
    if (lastChange && stablePlanDag) {
      performanceMonitor.trackEvent('websocketMessages')
      updateManager.scheduleStructuralUpdate(stablePlanDag, 'real-time-change')
    }
  }, [lastChange, stablePlanDag, updateManager, performanceMonitor])

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

    // Actions
    savePlanDag,
    validatePlanDag,
    refreshData,
  }
}