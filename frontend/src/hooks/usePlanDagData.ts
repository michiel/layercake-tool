import { useState, useEffect, useRef, useCallback } from 'react'
import { useApolloClient } from '@apollo/client/react'
import { PlanDagDataService } from '../services/PlanDagDataService'
import { PlanDag, PlanDagNode, ReactFlowEdge } from '../types/plan-dag'
import { useSubscriptionFilter } from './useGraphQLSubscriptionFilter'

interface UsePlanDagDataOptions {
  projectId: number
  autoSubscribe?: boolean
}

interface UsePlanDagDataResult {
  // Data state
  planDag: PlanDag | null
  loading: boolean
  error: Error | null

  // Actions - Mutations
  updatePlanDag: (planDag: PlanDag) => Promise<void>
  addNode: (node: Partial<PlanDagNode>) => Promise<PlanDagNode>
  updateNode: (nodeId: string, updates: Partial<PlanDagNode>) => Promise<PlanDagNode>
  deleteNode: (nodeId: string) => Promise<boolean>
  moveNode: (nodeId: string, position: { x: number, y: number }) => Promise<boolean>
  addEdge: (edge: ReactFlowEdge) => Promise<ReactFlowEdge>
  updateEdge: (edgeId: string, updates: Partial<ReactFlowEdge>) => Promise<ReactFlowEdge>
  deleteEdge: (edgeId: string) => Promise<boolean>
  validatePlanDag: (planDag: PlanDag) => Promise<any>

  // Actions - Queries
  refreshData: () => Promise<void>
  invalidateCache: () => void

  // Subscription status
  isSubscribed: boolean
  lastRemoteUpdate: Date | null
}

/**
 * Hook for GraphQL-based Plan DAG data operations
 * Handles only persistent data - completely separate from WebSocket presence
 */
export const usePlanDagData = (options: UsePlanDagDataOptions): UsePlanDagDataResult => {
  const { projectId, autoSubscribe = true } = options

  const apolloClient = useApolloClient()
  const { clientId } = useSubscriptionFilter()

  const [planDag, setPlanDag] = useState<PlanDag | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)
  const [isSubscribed, setIsSubscribed] = useState(false)
  const [lastRemoteUpdate, setLastRemoteUpdate] = useState<Date | null>(null)

  const dataServiceRef = useRef<PlanDagDataService | null>(null)
  const subscriptionRef = useRef<any>(null)

  // Initialize data service
  useEffect(() => {
    if (!dataServiceRef.current) {
      console.log('[usePlanDagData] Initializing PlanDagDataService for project:', projectId)
      dataServiceRef.current = new PlanDagDataService(apolloClient, clientId)
    }
  }, [apolloClient, clientId, projectId])

  // Load initial data
  useEffect(() => {
    const loadInitialData = async () => {
      if (!dataServiceRef.current) return

      try {
        setLoading(true)
        setError(null)

        console.log('[usePlanDagData] Loading initial Plan DAG for project:', projectId)
        const initialPlanDag = await dataServiceRef.current.getPlanDag(projectId)

        setPlanDag(initialPlanDag)
        console.log('[usePlanDagData] Initial Plan DAG loaded:', initialPlanDag?.version)
      } catch (err) {
        const error = err instanceof Error ? err : new Error('Failed to load Plan DAG')
        console.error('[usePlanDagData] Failed to load initial data:', error)
        setError(error)
      } finally {
        setLoading(false)
      }
    }

    loadInitialData()
  }, [projectId])

  // Set up subscription to remote changes
  useEffect(() => {
    if (!dataServiceRef.current || !autoSubscribe) return

    console.log('[usePlanDagData] Setting up subscription for project:', projectId)

    const subscription = dataServiceRef.current.subscribeToPlanDagChanges(
      projectId,
      (updatedPlanDag) => {
        console.log('[usePlanDagData] Received remote Plan DAG update:', updatedPlanDag.version)
        setPlanDag(updatedPlanDag)
        setLastRemoteUpdate(new Date())
      },
      (error) => {
        console.error('[usePlanDagData] Subscription error:', error)
        setError(error)
        setIsSubscribed(false)
      }
    )

    subscriptionRef.current = subscription
    setIsSubscribed(true)

    return () => {
      console.log('[usePlanDagData] Cleaning up subscription')
      subscription?.unsubscribe()
      subscriptionRef.current = null
      setIsSubscribed(false)
    }
  }, [projectId, autoSubscribe])

  // Mutation actions
  const updatePlanDag = useCallback(async (updatedPlanDag: PlanDag): Promise<void> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      await dataServiceRef.current.updatePlanDag(updatedPlanDag)

      // Optimistically update local state
      setPlanDag(updatedPlanDag)
      console.log('[usePlanDagData] Plan DAG updated locally and remotely')
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to update Plan DAG')
      console.error('[usePlanDagData] Update failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const addNode = useCallback(async (node: Partial<PlanDagNode>): Promise<PlanDagNode> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const newNode = await dataServiceRef.current.addNode(node)
      console.log('[usePlanDagData] Node added:', newNode.id)
      return newNode
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to add node')
      console.error('[usePlanDagData] Add node failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const updateNode = useCallback(async (nodeId: string, updates: Partial<PlanDagNode>): Promise<PlanDagNode> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const updatedNode = await dataServiceRef.current.updateNode(nodeId, updates)
      console.log('[usePlanDagData] Node updated:', nodeId)
      return updatedNode
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to update node')
      console.error('[usePlanDagData] Update node failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const deleteNode = useCallback(async (nodeId: string): Promise<boolean> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const result = await dataServiceRef.current.deleteNode(nodeId)
      console.log('[usePlanDagData] Node deleted:', nodeId)
      return result
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to delete node')
      console.error('[usePlanDagData] Delete node failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const moveNode = useCallback(async (nodeId: string, position: { x: number, y: number }): Promise<boolean> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const result = await dataServiceRef.current.moveNode(nodeId, position)
      return result
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to move node')
      console.error('[usePlanDagData] Move node failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const addEdge = useCallback(async (edge: ReactFlowEdge): Promise<ReactFlowEdge> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const newEdge = await dataServiceRef.current.addEdge(edge)
      console.log('[usePlanDagData] Edge added:', newEdge.id)
      return newEdge
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to add edge')
      console.error('[usePlanDagData] Add edge failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const updateEdge = useCallback(async (edgeId: string, updates: Partial<ReactFlowEdge>): Promise<ReactFlowEdge> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const updatedEdge = await dataServiceRef.current.updateEdge(edgeId, updates)
      console.log('[usePlanDagData] Edge updated:', edgeId)
      return updatedEdge
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to update edge')
      console.error('[usePlanDagData] Update edge failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const deleteEdge = useCallback(async (edgeId: string): Promise<boolean> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const result = await dataServiceRef.current.deleteEdge(edgeId)
      console.log('[usePlanDagData] Edge deleted:', edgeId)
      return result
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to delete edge')
      console.error('[usePlanDagData] Delete edge failed:', error)
      setError(error)
      throw error
    }
  }, [])

  const validatePlanDag = useCallback(async (planDagToValidate: PlanDag): Promise<any> => {
    if (!dataServiceRef.current) throw new Error('Data service not initialized')

    try {
      const validationResult = await dataServiceRef.current.validatePlanDag(planDagToValidate)
      console.log('[usePlanDagData] Plan DAG validated')
      return validationResult
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to validate Plan DAG')
      console.error('[usePlanDagData] Validation failed:', error)
      setError(error)
      throw error
    }
  }, [])

  // Query actions
  const refreshData = useCallback(async (): Promise<void> => {
    if (!dataServiceRef.current) return

    try {
      setLoading(true)
      setError(null)

      const freshPlanDag = await dataServiceRef.current.getPlanDag(projectId)
      setPlanDag(freshPlanDag)
      console.log('[usePlanDagData] Data refreshed')
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to refresh data')
      console.error('[usePlanDagData] Refresh failed:', error)
      setError(error)
    } finally {
      setLoading(false)
    }
  }, [projectId])

  const invalidateCache = useCallback((): void => {
    if (!dataServiceRef.current) return

    dataServiceRef.current.invalidateCache(projectId)
    console.log('[usePlanDagData] Cache invalidated')
  }, [projectId])

  return {
    // Data state
    planDag,
    loading,
    error,

    // Actions - Mutations
    updatePlanDag,
    addNode,
    updateNode,
    deleteNode,
    moveNode,
    addEdge,
    updateEdge,
    deleteEdge,
    validatePlanDag,

    // Actions - Queries
    refreshData,
    invalidateCache,

    // Subscription status
    isSubscribed,
    lastRemoteUpdate
  }
}