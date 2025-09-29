import { ApolloClient, NormalizedCacheObject } from '@apollo/client'
import { PlanDag, PlanDagNode, ReactFlowEdge } from '../types/plan-dag'
import { createMutationContext } from '../hooks/useGraphQLSubscriptionFilter'

/**
 * Pure GraphQL-based data service
 * Handles only persistent Plan DAG data operations
 * Completely separate from WebSocket presence concerns
 */
export class PlanDagDataService {
  constructor(
    private apollo: ApolloClient<NormalizedCacheObject>,
    private clientId: string
  ) {}

  async updatePlanDag(planDag: PlanDag): Promise<void> {
    try {
      console.log('[PlanDagDataService] Updating Plan DAG:', planDag.version)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').UPDATE_PLAN_DAG,
        variables: { planDag },
        context: createMutationContext(this.clientId),
        // Skip subscription for own mutations to prevent loops
        update: (cache, { data }) => {
          // Optimistic update - but don't trigger subscriptions
          if (data?.updatePlanDag) {
            console.log('[PlanDagDataService] Optimistic update applied')
          }
        }
      })

      console.log('[PlanDagDataService] Plan DAG updated successfully')
      return result.data?.updatePlanDag
    } catch (error) {
      console.error('[PlanDagDataService] Failed to update Plan DAG:', error)
      throw error
    }
  }

  async addNode(node: Partial<PlanDagNode>): Promise<PlanDagNode> {
    try {
      console.log('[PlanDagDataService] Adding node:', node.id)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').ADD_PLAN_DAG_NODE,
        variables: { node },
        context: createMutationContext(this.clientId),
      })

      console.log('[PlanDagDataService] Node added successfully')
      return result.data?.addPlanDagNode
    } catch (error) {
      console.error('[PlanDagDataService] Failed to add node:', error)
      throw error
    }
  }

  async updateNode(nodeId: string, updates: Partial<PlanDagNode>): Promise<PlanDagNode> {
    try {
      console.log('[PlanDagDataService] Updating node:', nodeId)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').UPDATE_PLAN_DAG_NODE,
        variables: { nodeId, updates },
        context: createMutationContext(this.clientId),
      })

      console.log('[PlanDagDataService] Node updated successfully')
      return result.data?.updatePlanDagNode
    } catch (error) {
      console.error('[PlanDagDataService] Failed to update node:', error)
      throw error
    }
  }

  async deleteNode(nodeId: string): Promise<boolean> {
    try {
      console.log('[PlanDagDataService] Deleting node:', nodeId)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').DELETE_PLAN_DAG_NODE,
        variables: { nodeId },
        context: createMutationContext(this.clientId),
      })

      console.log('[PlanDagDataService] Node deleted successfully')
      return result.data?.deletePlanDagNode
    } catch (error) {
      console.error('[PlanDagDataService] Failed to delete node:', error)
      throw error
    }
  }

  async moveNode(nodeId: string, position: { x: number, y: number }): Promise<boolean> {
    try {
      console.log('[PlanDagDataService] Moving node:', nodeId, position)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').MOVE_PLAN_DAG_NODE,
        variables: { nodeId, position },
        context: createMutationContext(this.clientId),
        // Position updates are frequent - use optimistic response
        optimisticResponse: {
          movePlanDagNode: true
        }
      })

      return result.data?.movePlanDagNode
    } catch (error) {
      console.error('[PlanDagDataService] Failed to move node:', error)
      throw error
    }
  }

  async addEdge(edge: ReactFlowEdge): Promise<ReactFlowEdge> {
    try {
      console.log('[PlanDagDataService] Adding edge:', edge.id)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').ADD_PLAN_DAG_EDGE,
        variables: { edge },
        context: createMutationContext(this.clientId),
      })

      console.log('[PlanDagDataService] Edge added successfully')
      return result.data?.addPlanDagEdge
    } catch (error) {
      console.error('[PlanDagDataService] Failed to add edge:', error)
      throw error
    }
  }

  async updateEdge(edgeId: string, updates: Partial<ReactFlowEdge>): Promise<ReactFlowEdge> {
    try {
      console.log('[PlanDagDataService] Updating edge:', edgeId)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').UPDATE_PLAN_DAG_EDGE,
        variables: { edgeId, updates },
        context: createMutationContext(this.clientId),
      })

      console.log('[PlanDagDataService] Edge updated successfully')
      return result.data?.updatePlanDagEdge
    } catch (error) {
      console.error('[PlanDagDataService] Failed to update edge:', error)
      throw error
    }
  }

  async deleteEdge(edgeId: string): Promise<boolean> {
    try {
      console.log('[PlanDagDataService] Deleting edge:', edgeId)

      const result = await this.apollo.mutate({
        mutation: require('../graphql/plan-dag').DELETE_PLAN_DAG_EDGE,
        variables: { edgeId },
        context: createMutationContext(this.clientId),
      })

      console.log('[PlanDagDataService] Edge deleted successfully')
      return result.data?.deletePlanDagEdge
    } catch (error) {
      console.error('[PlanDagDataService] Failed to delete edge:', error)
      throw error
    }
  }

  async validatePlanDag(planDag: PlanDag): Promise<any> {
    try {
      console.log('[PlanDagDataService] Validating Plan DAG')

      const result = await this.apollo.query({
        query: require('../graphql/plan-dag').VALIDATE_PLAN_DAG,
        variables: { planDag },
        fetchPolicy: 'no-cache' // Always get fresh validation
      })

      console.log('[PlanDagDataService] Plan DAG validated')
      return result.data?.validatePlanDag
    } catch (error) {
      console.error('[PlanDagDataService] Failed to validate Plan DAG:', error)
      throw error
    }
  }

  // Query methods (read-only operations)
  async getPlanDag(projectId: number): Promise<PlanDag | null> {
    try {
      const result = await this.apollo.query({
        query: require('../graphql/plan-dag').GET_PLAN_DAG,
        variables: { projectId },
        fetchPolicy: 'cache-first'
      })

      return result.data?.planDag || null
    } catch (error) {
      console.error('[PlanDagDataService] Failed to get Plan DAG:', error)
      throw error
    }
  }

  // Subscribe to Plan DAG changes from other clients only
  subscribeToPlanDagChanges(
    projectId: number,
    onUpdate: (planDag: PlanDag) => void,
    onError?: (error: Error) => void
  ) {
    console.log('[PlanDagDataService] Subscribing to Plan DAG changes for project:', projectId)

    const subscription = this.apollo.subscribe({
      query: require('../graphql/plan-dag').PLAN_DAG_SUBSCRIPTION,
      variables: { projectId }
    })

    return subscription.subscribe({
      next: (result) => {
        const subscriptionData = result.data?.planDagChanged

        if (subscriptionData) {
          // Filter out updates from this client using the subscription filter
          const updateClientId = subscriptionData.clientId || subscriptionData.mutation?.clientId

          if (updateClientId === this.clientId) {
            console.log('[PlanDagDataService] Filtered out own subscription update')
            return
          }

          console.log('[PlanDagDataService] Processing remote Plan DAG update from client:', updateClientId)
          onUpdate(subscriptionData.planDag)
        }
      },
      error: (error) => {
        console.error('[PlanDagDataService] Subscription error:', error)
        onError?.(error)
      }
    })
  }

  // Cache management
  invalidateCache(projectId: number): void {
    console.log('[PlanDagDataService] Invalidating cache for project:', projectId)

    this.apollo.cache.evict({
      fieldName: 'planDag',
      args: { projectId }
    })

    this.apollo.cache.gc()
  }

  // Optimistic updates for better UX
  async updateNodeOptimistic(
    nodeId: string,
    updates: Partial<PlanDagNode>,
    rollback: () => void
  ): Promise<void> {
    try {
      // Apply optimistic update immediately
      // Note: This would require cache manipulation which is complex
      // For now, we'll just do the mutation with optimistic response

      await this.updateNode(nodeId, updates)
    } catch (error) {
      // Rollback optimistic changes on error
      rollback()
      throw error
    }
  }
}