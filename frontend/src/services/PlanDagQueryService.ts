import { ApolloClient } from '@apollo/client'
import * as PlanDagGraphQL from '../graphql/plan-dag'
import { PlanDag } from '../types/plan-dag'

/**
 * CQRS Query Service - Handles all queries and subscriptions (reads)
 * Separated from command operations to eliminate circular dependencies
 * Only listens to subscriptions, never triggers mutations
 */
export class PlanDagQueryService {
  constructor(
    private apollo: ApolloClient,
    private clientId: string
  ) {}

  // Query Operations
  async getPlanDag(query: GetPlanDagQuery): Promise<PlanDag | null> {
    try {
      console.log('[PlanDagQueryService] Loading Plan DAG for project:', query.projectId)

      const result = await this.apollo.query({
        query: PlanDagGraphQL.GET_PLAN_DAG,
        variables: { projectId: query.projectId },
        fetchPolicy: query.fresh ? 'network-only' : 'cache-first'
      })

      const planDag = (result.data as any)?.getPlanDag || null
      console.log('[PlanDagQueryService] Plan DAG loaded:', planDag?.version)
      return planDag
    } catch (error) {
      console.error('[PlanDagQueryService] Failed to get Plan DAG:', error)
      throw error
    }
  }

  // Subscription Operations
  subscribeToPlanDagChanges(
    query: SubscribeToPlanDagQuery,
    onUpdate: (planDag: PlanDag) => void,
    onError?: (error: Error) => void
  ) {
    console.log('[PlanDagQueryService] Setting up Plan DAG subscription for project:', query.projectId)

    const subscription = this.apollo.subscribe({
      query: PlanDagGraphQL.PLAN_DAG_CHANGED_SUBSCRIPTION,
      variables: { projectId: query.projectId }
    })

    return subscription.subscribe({
      next: (result: any) => {
        const subscriptionData = result.data?.planDagChanged

        if (subscriptionData) {
          // Filter out updates from this client using built-in filtering
          const updateClientId = subscriptionData.clientId || subscriptionData.mutation?.clientId

          if (updateClientId === this.clientId) {
            console.log('[PlanDagQueryService] Filtered out own subscription update')
            return
          }

          console.log('[PlanDagQueryService] Processing remote Plan DAG update from client:', updateClientId)

          // Extract the plan DAG data from the change notification
          const updatedPlanDag = this.extractPlanDagFromChange(subscriptionData)
          if (updatedPlanDag) {
            onUpdate(updatedPlanDag)
          }
        }
      },
      error: (error: any) => {
        console.error('[PlanDagQueryService] Subscription error:', error)
        onError?.(error)
      }
    })
  }

  // Cache Management
  invalidateCache(query: InvalidateCacheQuery): void {
    console.log('[PlanDagQueryService] Invalidating cache for project:', query.projectId)

    this.apollo.cache.evict({
      fieldName: 'getPlanDag',
      args: { projectId: query.projectId }
    })

    this.apollo.cache.gc()
  }

  // Helper method to extract Plan DAG from subscription change
  private extractPlanDagFromChange(changeData: any): PlanDag | null {
    try {
      // The subscription might contain different change types
      if (changeData.planDag) {
        return changeData.planDag
      }

      // If it's a node/edge change, we might need to reconstruct the full DAG
      // For now, we'll trigger a refresh by returning null and letting the caller handle it
      if (changeData.change) {
        console.log('[PlanDagQueryService] Received partial change, triggering full refresh')
        return null
      }

      return null
    } catch (error) {
      console.error('[PlanDagQueryService] Failed to extract Plan DAG from change:', error)
      return null
    }
  }

  // Reactive Queries (for real-time updates without subscriptions)
  watchPlanDag(
    query: WatchPlanDagQuery,
    onUpdate: (planDag: PlanDag | null) => void,
    onError?: (error: Error) => void
  ) {
    console.log('[PlanDagQueryService] Setting up Plan DAG watch for project:', query.projectId)

    const watchQuery = this.apollo.watchQuery({
      query: PlanDagGraphQL.GET_PLAN_DAG,
      variables: { projectId: query.projectId },
      fetchPolicy: 'cache-and-network',
      errorPolicy: 'all'
    })

    return watchQuery.subscribe({
      next: (result) => {
        const planDag = (result.data as any)?.getPlanDag || null
        console.log('[PlanDagQueryService] Watch update:', planDag?.version)
        onUpdate(planDag)
      },
      error: (error) => {
        console.error('[PlanDagQueryService] Watch error:', error)
        onError?.(error)
      }
    })
  }
}

// Query Types
export interface GetPlanDagQuery {
  projectId: number
  fresh?: boolean // Force network request
}

export interface SubscribeToPlanDagQuery {
  projectId: number
  includePartialUpdates?: boolean
}

export interface InvalidateCacheQuery {
  projectId: number
}

export interface WatchPlanDagQuery {
  projectId: number
}

export type PlanDagQuery =
  | GetPlanDagQuery
  | SubscribeToPlanDagQuery
  | InvalidateCacheQuery
  | WatchPlanDagQuery