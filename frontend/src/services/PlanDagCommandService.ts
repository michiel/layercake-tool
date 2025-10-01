import { ApolloClient } from '@apollo/client'
import * as PlanDagGraphQL from '../graphql/plan-dag'
import { PlanDag, PlanDagNode, ReactFlowEdge } from '../types/plan-dag'
import { createMutationContext } from '../hooks/useGraphQLSubscriptionFilter'

/**
 * CQRS Command Service - Handles all mutations (writes)
 * Separated from query operations to eliminate circular dependencies
 * Does not listen to subscriptions - only executes commands
 */
export class PlanDagCommandService {
  constructor(
    private apollo: ApolloClient,
    private clientId: string
  ) {}

  // Core Plan DAG Commands
  async createNode(command: CreateNodeCommand): Promise<PlanDagNode> {
    try {
      console.log('[PlanDagCommandService] Creating node:', command.nodeType)

      const result = await this.apollo.mutate({
        mutation: PlanDagGraphQL.ADD_PLAN_DAG_NODE,
        variables: {
          projectId: command.projectId,
          node: command.node
        },
        context: createMutationContext(this.clientId)
      })

      const response = (result.data as any)?.addPlanDagNode
      if (!response?.success) {
        throw new Error(`Failed to create node: ${response?.errors?.join(', ') || 'Unknown error'}`)
      }
      const createdNode = response.node
      console.log('[PlanDagCommandService] Node created successfully:', createdNode?.id)
      return createdNode
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to create node:', error)
      throw error
    }
  }

  async updateNode(command: UpdateNodeCommand): Promise<PlanDagNode> {
    try {
      console.log('[PlanDagCommandService] Updating node:', command.nodeId)

      const result = await this.apollo.mutate({
        mutation: PlanDagGraphQL.UPDATE_PLAN_DAG_NODE,
        variables: {
          projectId: command.projectId,
          nodeId: command.nodeId,
          updates: command.updates
        },
        context: createMutationContext(this.clientId)
      })

      const response = (result.data as any)?.updatePlanDagNode
      if (!response?.success) {
        throw new Error(`Failed to update node: ${response?.errors?.join(', ') || 'Unknown error'}`)
      }
      const updatedNode = response.node
      console.log('[PlanDagCommandService] Node updated successfully')
      return updatedNode
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to update node:', error)
      throw error
    }
  }

  async deleteNode(command: DeleteNodeCommand): Promise<boolean> {
    try {
      console.log('[PlanDagCommandService] Deleting node:', command.nodeId)

      const result = await this.apollo.mutate({
        mutation: PlanDagGraphQL.DELETE_PLAN_DAG_NODE,
        variables: {
          projectId: command.projectId,
          nodeId: command.nodeId
        },
        context: createMutationContext(this.clientId)
      })

      const success = (result.data as any)?.deletePlanDagNode || false
      console.log('[PlanDagCommandService] Node deleted successfully')
      return success
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to delete node:', error)
      throw error
    }
  }

  async moveNode(command: MoveNodeCommand): Promise<boolean> {
    try {
      console.log('[PlanDagCommandService] Moving node:', command.nodeId, command.position)

      const result = await this.apollo.mutate({
        mutation: PlanDagGraphQL.MOVE_PLAN_DAG_NODE,
        variables: {
          projectId: command.projectId,
          nodeId: command.nodeId,
          position: command.position
        },
        context: createMutationContext(this.clientId),
        // Position updates are frequent - use optimistic response
        optimisticResponse: {
          movePlanDagNode: true
        }
      })

      return (result.data as any)?.movePlanDagNode || false
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to move node:', error)
      throw error
    }
  }

  // Edge Commands
  async createEdge(command: CreateEdgeCommand): Promise<ReactFlowEdge> {
    try {
      console.log('[PlanDagCommandService] Creating edge:', command.edge.source, '->', command.edge.target)

      const result = await this.apollo.mutate({
        mutation: PlanDagGraphQL.ADD_PLAN_DAG_EDGE,
        variables: {
          projectId: command.projectId,
          edge: command.edge
        },
        context: createMutationContext(this.clientId)
      })

      const response = (result.data as any)?.addPlanDagEdge
      if (!response?.success) {
        throw new Error(`Failed to create edge: ${response?.errors?.join(', ') || 'Unknown error'}`)
      }
      const createdEdge = response.edge
      console.log('[PlanDagCommandService] Edge created successfully:', createdEdge?.id)
      return createdEdge
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to create edge:', error)
      throw error
    }
  }

  async deleteEdge(command: DeleteEdgeCommand): Promise<boolean> {
    try {
      console.log('[PlanDagCommandService] Deleting edge:', command.edgeId)

      const result = await this.apollo.mutate({
        mutation: PlanDagGraphQL.DELETE_PLAN_DAG_EDGE,
        variables: {
          projectId: command.projectId,
          edgeId: command.edgeId
        },
        context: createMutationContext(this.clientId)
      })

      const success = (result.data as any)?.deletePlanDagEdge || false
      console.log('[PlanDagCommandService] Edge deleted successfully')
      return success
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to delete edge:', error)
      throw error
    }
  }

  // Bulk Plan DAG Commands
  async updatePlanDag(command: UpdatePlanDagCommand): Promise<void> {
    try {
      console.log('[PlanDagCommandService] Updating entire Plan DAG:', command.planDag.version)

      await this.apollo.mutate({
        mutation: PlanDagGraphQL.UPDATE_PLAN_DAG,
        variables: {
          projectId: command.projectId,
          planDag: command.planDag
        },
        context: createMutationContext(this.clientId)
      })

      console.log('[PlanDagCommandService] Plan DAG updated successfully')
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to update Plan DAG:', error)
      throw error
    }
  }

  // Validation Commands (read-only but treated as command for consistency)
  async validatePlanDag(command: ValidatePlanDagCommand): Promise<any> {
    try {
      console.log('[PlanDagCommandService] Validating Plan DAG')

      const result = await this.apollo.query({
        query: PlanDagGraphQL.VALIDATE_PLAN_DAG,
        variables: { planDag: command.planDag },
        fetchPolicy: 'no-cache' // Always get fresh validation
      })

      console.log('[PlanDagCommandService] Plan DAG validated')
      return (result.data as any)?.validatePlanDag || null
    } catch (error) {
      console.error('[PlanDagCommandService] Failed to validate Plan DAG:', error)
      throw error
    }
  }
}

// Command Types
export interface CreateNodeCommand {
  projectId: number
  node: Partial<PlanDagNode>
  nodeType: string
}

export interface UpdateNodeCommand {
  projectId: number
  nodeId: string
  updates: Partial<PlanDagNode>
}

export interface DeleteNodeCommand {
  projectId: number
  nodeId: string
}

export interface MoveNodeCommand {
  projectId: number
  nodeId: string
  position: { x: number; y: number }
}

export interface CreateEdgeCommand {
  projectId: number
  edge: ReactFlowEdge
}

export interface DeleteEdgeCommand {
  projectId: number
  edgeId: string
}

export interface UpdatePlanDagCommand {
  projectId: number
  planDag: PlanDag
}

export interface ValidatePlanDagCommand {
  planDag: PlanDag
}

export type PlanDagCommand =
  | CreateNodeCommand
  | UpdateNodeCommand
  | DeleteNodeCommand
  | MoveNodeCommand
  | CreateEdgeCommand
  | DeleteEdgeCommand
  | UpdatePlanDagCommand
  | ValidatePlanDagCommand