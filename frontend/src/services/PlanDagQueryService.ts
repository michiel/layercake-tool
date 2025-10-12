import { ApolloClient } from '@apollo/client'
import * as PlanDagGraphQL from '../graphql/plan-dag'
import { PlanDag } from '../types/plan-dag'
import { applyPatch, Operation } from 'fast-json-patch'

/**
 * CQRS Query Service - Handles all queries and subscriptions (reads)
 * Separated from command operations to eliminate circular dependencies
 * Only listens to subscriptions, never triggers mutations
 */
export class PlanDagQueryService {
  private lastMutationTimestamp = 0
  private readonly MUTATION_ECHO_WINDOW_MS = 500 // Ignore subscription echos for 500ms after mutation

  constructor(
    private apollo: ApolloClient,
    private clientId: string
  ) {}

  // Call this method after any mutation to suppress echo
  markMutationOccurred(): void {
    this.lastMutationTimestamp = Date.now()
  }

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

  // Delta-based subscription using JSON Patch for efficient updates
  subscribeToPlanDagDeltas(
    query: SubscribeToPlanDagQuery,
    getCurrentPlanDag: () => PlanDag | null, // Function to get current state
    onUpdate: (planDag: PlanDag) => void,
    onError?: (error: Error) => void
  ) {
    console.log('[PlanDagQueryService] Setting up delta subscription for project:', query.projectId, 'clientId:', this.clientId)

    const subscription = this.apollo.subscribe({
      query: PlanDagGraphQL.PLAN_DAG_DELTA_SUBSCRIPTION,
      variables: { projectId: query.projectId }
    })

    console.log('[PlanDagQueryService] Delta subscription created, waiting for updates...')

    return subscription.subscribe({
      next: (result: any) => {
        console.log('[PlanDagQueryService] Raw subscription result:', result)
        const deltaData = result.data?.planDagDeltaChanged

        if (deltaData) {
          console.log('[PlanDagQueryService] Received delta update:', {
            version: deltaData.version,
            operations: deltaData.operations.length,
            userId: deltaData.userId,
            clientId: this.clientId
          })

          // Skip subscription updates shortly after own mutations to prevent echo
          // Use command service timestamp if available (coordinated via CQRS service)
          const commandTimestamp = (this as any).getCommandTimestamp?.() || this.lastMutationTimestamp
          const timeSinceLastMutation = Date.now() - commandTimestamp
          if (timeSinceLastMutation < this.MUTATION_ECHO_WINDOW_MS) {
            console.log('[PlanDagQueryService] Skipping subscription update (recent mutation echo):', {
              timeSinceLastMutation: `${timeSinceLastMutation}ms`,
              window: `${this.MUTATION_ECHO_WINDOW_MS}ms`,
              usingCommandTimestamp: !!(this as any).getCommandTimestamp
            })
            return
          }

          // Get current Plan DAG from the callback
          const localPlanDag = getCurrentPlanDag()

          if (!localPlanDag) {
            console.warn('[PlanDagQueryService] No local Plan DAG to apply patch to, skipping')
            return
          }

          try {
            // Convert GraphQL operations to fast-json-patch format
            const operations: Operation[] = deltaData.operations.map((op: any) => ({
              op: op.op.toLowerCase(),
              path: op.path,
              ...(op.value !== null && op.value !== undefined ? { value: op.value } : {}),
              ...(op.from ? { from: op.from } : {})
            }))

            // Apply JSON Patch to local state
            const patchResult = applyPatch(
              JSON.parse(JSON.stringify(localPlanDag)),
              operations,
              true, // validate
              false // mutate (we want a new object)
            )

            if (patchResult.newDocument) {
              const updatedPlanDag = patchResult.newDocument as PlanDag
              updatedPlanDag.version = deltaData.version

              console.log('[PlanDagQueryService] Successfully applied delta patch:', {
                oldVersion: localPlanDag.version,
                newVersion: updatedPlanDag.version,
                operations: operations.length
              })

              onUpdate(updatedPlanDag)
            } else {
              console.error('[PlanDagQueryService] Patch application failed')
            }
          } catch (error) {
            console.error('[PlanDagQueryService] Error applying JSON Patch:', error)
            // On patch error, trigger a full refresh
            onError?.(error as Error)
          }
        }
      },
      error: (error: any) => {
        console.error('[PlanDagQueryService] Delta subscription error:', error)
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