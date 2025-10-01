import { ApolloClient } from '@apollo/client'
import { PlanDagCommandService } from './PlanDagCommandService'
import { PlanDagQueryService } from './PlanDagQueryService'
import { ReactFlowAdapter } from '../adapters/ReactFlowAdapter'

/**
 * Unified CQRS Service for Plan DAG operations
 * Combines separated command and query services
 * Provides a clean interface for components while maintaining architectural separation
 */
export class PlanDagCQRSService {
  private commandService: PlanDagCommandService
  private queryService: PlanDagQueryService
  private clientId: string

  constructor(apollo: ApolloClient, clientId: string) {
    this.clientId = clientId

    // Initialize separated services
    this.commandService = new PlanDagCommandService(apollo, this.clientId)
    this.queryService = new PlanDagQueryService(apollo, this.clientId)

    console.log('[PlanDagCQRSService] Initialized with client ID:', this.clientId)
  }

  // Command Operations (Write)
  get commands() {
    return {
      createNode: this.commandService.createNode.bind(this.commandService),
      updateNode: this.commandService.updateNode.bind(this.commandService),
      deleteNode: this.commandService.deleteNode.bind(this.commandService),
      moveNode: this.commandService.moveNode.bind(this.commandService),
      createEdge: this.commandService.createEdge.bind(this.commandService),
      deleteEdge: this.commandService.deleteEdge.bind(this.commandService),
      updatePlanDag: this.commandService.updatePlanDag.bind(this.commandService),
      validatePlanDag: this.commandService.validatePlanDag.bind(this.commandService)
    }
  }

  // Query Operations (Read)
  get queries() {
    return {
      getPlanDag: this.queryService.getPlanDag.bind(this.queryService),
      subscribeToPlanDagChanges: this.queryService.subscribeToPlanDagChanges.bind(this.queryService),
      subscribeToPlanDagDeltas: this.queryService.subscribeToPlanDagDeltas.bind(this.queryService),
      watchPlanDag: this.queryService.watchPlanDag.bind(this.queryService),
      invalidateCache: this.queryService.invalidateCache.bind(this.queryService)
    }
  }

  // ReactFlow Integration
  get adapter() {
    return ReactFlowAdapter
  }

  // Convenience methods for common patterns
  async createNodeWithReactFlowData(
    projectId: number,
    nodeType: string,
    position: { x: number; y: number },
    metadata?: any
  ) {
    console.log('[PlanDagCQRSService] Creating node with ReactFlow data')

    return this.commands.createNode({
      projectId,
      nodeType,
      node: {
        id: `node-${Date.now()}`,
        nodeType: nodeType as any,
        position,
        metadata: {
          label: metadata?.label || `${nodeType} Node`,
          description: metadata?.description || ''
        },
        config: metadata?.config || {}
      }
    })
  }

  async updateNodePosition(
    projectId: number,
    nodeId: string,
    position: { x: number; y: number }
  ) {
    console.log('[PlanDagCQRSService] Updating node position')

    return this.commands.moveNode({
      projectId,
      nodeId,
      position
    })
  }

  async createEdgeFromReactFlow(
    projectId: number,
    sourceId: string,
    targetId: string,
    sourceHandle?: string,
    targetHandle?: string
  ) {
    console.log('[PlanDagCQRSService] Creating edge from ReactFlow')

    return this.commands.createEdge({
      projectId,
      edge: {
        id: `edge-${sourceId}-${targetId}-${Date.now()}`,
        source: sourceId,
        target: targetId,
        sourceHandle,
        targetHandle,
        metadata: {
          label: '',
          dataType: 'GRAPH_DATA' as any
        }
      }
    })
  }

  // Bulk operations
  async syncReactFlowChanges(
    projectId: number,
    reactFlowNodes: any[],
    reactFlowEdges: any[]
  ) {
    console.log('[PlanDagCQRSService] Syncing ReactFlow changes to Plan DAG')

    try {
      // Convert ReactFlow data to Plan DAG format
      const planDag = this.adapter.reactFlowToPlanDag(reactFlowNodes, reactFlowEdges)

      // Update the entire Plan DAG
      await this.commands.updatePlanDag({
        projectId,
        planDag
      })

      console.log('[PlanDagCQRSService] ReactFlow changes synced successfully')
    } catch (error) {
      console.error('[PlanDagCQRSService] Failed to sync ReactFlow changes:', error)
      throw error
    }
  }

  // Real-time subscription with ReactFlow adapter
  subscribeToReactFlowUpdates(
    projectId: number,
    onUpdate: (nodes: any[], edges: any[]) => void,
    onError?: (error: Error) => void
  ) {
    console.log('[PlanDagCQRSService] Setting up ReactFlow subscription')

    return this.queries.subscribeToPlanDagChanges(
      { projectId },
      (planDag) => {
        // Convert Plan DAG to ReactFlow format
        const { nodes, edges } = this.adapter.planDagToReactFlow(planDag)
        onUpdate(nodes, edges)
      },
      onError
    )
  }

  // Delta-based subscription with ReactFlow adapter (efficient updates)
  subscribeToDeltaUpdates(
    projectId: number,
    getCurrentPlanDag: () => any | null,
    onUpdate: (planDag: any) => void,
    onError?: (error: Error) => void
  ) {
    console.log('[PlanDagCQRSService] Setting up delta subscription')

    return this.queries.subscribeToPlanDagDeltas(
      { projectId },
      getCurrentPlanDag,
      onUpdate,
      onError
    )
  }

  // Health check and diagnostics
  getServiceInfo() {
    return {
      clientId: this.clientId,
      services: {
        command: 'PlanDagCommandService',
        query: 'PlanDagQueryService',
        adapter: 'ReactFlowAdapter'
      },
      patterns: ['CQRS', 'Adapter Pattern', 'Event Sourcing Ready']
    }
  }
}