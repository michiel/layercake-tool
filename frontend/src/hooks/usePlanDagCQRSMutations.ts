import { useCallback, useMemo } from 'react'
import { useApolloClient } from '@apollo/client/react'
import { PlanDagCQRSService } from '../services/PlanDagCQRSService'
import { PlanDagNode, ReactFlowEdge } from '../types/plan-dag'
import { useSubscriptionFilter } from './useGraphQLSubscriptionFilter'

interface UsePlanDagCQRSMutationsOptions {
  projectId: number
  planId: number
}

interface PlanDagCQRSMutations {
  addNode: (node: Partial<PlanDagNode>) => Promise<PlanDagNode>
  updateNode: (nodeId: string, updates: { config?: any; metadata?: any }) => Promise<PlanDagNode>
  deleteNode: (nodeId: string) => Promise<boolean>
  moveNode: (nodeId: string, position: { x: number; y: number }) => Promise<boolean>
  addEdge: (edge: ReactFlowEdge) => Promise<ReactFlowEdge>
  deleteEdge: (edgeId: string) => Promise<void>
  updatePlanDag: (planDag: any) => Promise<void>
  cqrsService: PlanDagCQRSService
}

export const usePlanDagCQRSMutations = (options: UsePlanDagCQRSMutationsOptions): PlanDagCQRSMutations => {
  const { projectId, planId } = options
  const apollo = useApolloClient()

  // Get client ID at top level to follow React hook rules
  const { clientId } = useSubscriptionFilter()

  // Initialize CQRS service
  const cqrsService = useMemo(() => {
    return new PlanDagCQRSService(apollo, clientId)
  }, [apollo, clientId])

  const addNode = useCallback(async (node: Partial<PlanDagNode>): Promise<PlanDagNode> => {
    console.log('[usePlanDagCQRSMutations] Adding node via CQRS:', node.id)

    if (!node.nodeType || !node.position) {
      throw new Error('Node must have nodeType and position')
    }

    return await cqrsService.commands.createNode({
      projectId,
      planId,
      nodeType: node.nodeType as string,
      node: {
        id: node.id!,
        nodeType: node.nodeType,
        position: node.position,
        metadata: node.metadata || { label: '', description: '' },
        config: typeof node.config === 'string' ? node.config : JSON.stringify(node.config || {})
      }
    })
  }, [cqrsService, projectId, planId])

  const updateNode = useCallback(async (nodeId: string, updates: { config?: any; metadata?: any }): Promise<PlanDagNode> => {
    console.log('[usePlanDagCQRSMutations] Updating node via CQRS:', nodeId)

    return await cqrsService.commands.updateNode({
      projectId,
      planId,
      nodeId,
      updates: {
        config: typeof updates.config === 'string' ? updates.config : JSON.stringify(updates.config || {}),
        metadata: updates.metadata
      }
    })
  }, [cqrsService, projectId, planId])

  const deleteNode = useCallback(async (nodeId: string): Promise<boolean> => {
    console.log('[usePlanDagCQRSMutations] Deleting node via CQRS:', nodeId)

    return await cqrsService.commands.deleteNode({
      projectId,
      planId,
      nodeId
    })
  }, [cqrsService, projectId, planId])

  const moveNode = useCallback(async (nodeId: string, position: { x: number; y: number }): Promise<boolean> => {
    console.log('[usePlanDagCQRSMutations] Moving node via CQRS:', nodeId, position)

    return await cqrsService.commands.moveNode({
      projectId,
      planId,
      nodeId,
      position
    })
  }, [cqrsService, projectId, planId])

  const addEdge = useCallback(async (edge: ReactFlowEdge): Promise<ReactFlowEdge> => {
    console.log('[usePlanDagCQRSMutations] Adding edge via CQRS:', edge.id)

    return await cqrsService.commands.createEdge({
      projectId,
      planId,
      edge
    })
  }, [cqrsService, projectId, planId])

  const deleteEdge = useCallback(async (edgeId: string): Promise<void> => {
    console.log('[usePlanDagCQRSMutations] Deleting edge via CQRS:', edgeId)

    await cqrsService.commands.deleteEdge({
      projectId,
      planId,
      edgeId
    })
  }, [cqrsService, projectId, planId])

  const updatePlanDag = useCallback(async (planDag: any): Promise<void> => {
    console.log('[usePlanDagCQRSMutations] Updating Plan DAG via CQRS')

    await cqrsService.commands.updatePlanDag({
      projectId,
      planId,
      planDag
    })
  }, [cqrsService, planId, projectId])

  return {
    addNode,
    updateNode,
    deleteNode,
    moveNode,
    addEdge,
    deleteEdge,
    updatePlanDag,
    cqrsService
  }
}
