import { useQuery, useMutation, useSubscription } from '@apollo/client/react'
import { useCallback, useMemo } from 'react'
import {
  GET_PLAN_DAG,
  VALIDATE_PLAN_DAG,
  UPDATE_PLAN_DAG,
  ADD_PLAN_DAG_NODE,
  UPDATE_PLAN_DAG_NODE,
  DELETE_PLAN_DAG_NODE,
  ADD_PLAN_DAG_EDGE,
  DELETE_PLAN_DAG_EDGE,
  MOVE_PLAN_DAG_NODE,
  PLAN_DAG_CHANGED_SUBSCRIPTION,
  USER_PRESENCE_SUBSCRIPTION,
  UPDATE_CURSOR_POSITION,
  JOIN_PROJECT_COLLABORATION,
  LEAVE_PROJECT_COLLABORATION,
  type PlanDagQueryVariables,
  type PlanDagMutationVariables,
  type PlanDagSubscriptionVariables
} from '../graphql/plan-dag'
import { PlanDag, PlanDagNode, PlanDagEdge, Position } from '../types/plan-dag'

// Hook for fetching Plan DAG data
export const usePlanDag = (projectId: number) => {
  const { data, loading, error, refetch } = useQuery(GET_PLAN_DAG, {
    variables: { projectId } as PlanDagQueryVariables,
    errorPolicy: 'all',
    notifyOnNetworkStatusChange: true,
  })

  const planDag = useMemo(() => data?.getPlanDag, [data])

  return {
    planDag,
    loading,
    error,
    refetch,
  }
}

// Hook for Plan DAG validation
export const usePlanDagValidation = () => {
  const [validatePlanDag, { data, loading, error }] = useMutation(VALIDATE_PLAN_DAG)

  const validate = useCallback((planDag: PlanDag) => {
    return validatePlanDag({
      variables: { planDag },
    })
  }, [validatePlanDag])

  return {
    validate,
    validationResult: data?.validatePlanDag,
    loading,
    error,
  }
}

// Hook for Plan DAG mutations
export const usePlanDagMutations = (projectId: number) => {
  const [updatePlanDag] = useMutation(UPDATE_PLAN_DAG, {
    optimisticResponse: (variables) => ({
      updatePlanDag: {
        __typename: 'PlanDagResponse',
        success: true,
        errors: [],
        planDag: {
          __typename: 'PlanDag',
          version: variables.planDag.version,
          nodes: variables.planDag.nodes.map((node: any) => ({
            __typename: 'PlanDagNode',
            id: node.id,
            nodeType: node.nodeType,
            position: {
              __typename: 'Position',
              x: node.position.x,
              y: node.position.y,
            },
            metadata: {
              __typename: 'NodeMetadata',
              label: node.metadata.label,
              description: node.metadata.description,
            },
            config: node.config || '{}',
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          })),
          edges: variables.planDag.edges.map((edge: any) => ({
            __typename: 'PlanDagEdge',
            id: edge.id,
            source: edge.source,
            target: edge.target,
            metadata: {
              __typename: 'EdgeMetadata',
              label: edge.metadata.label,
              dataType: edge.metadata.dataType,
            },
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          })),
          metadata: {
            __typename: 'PlanDagMetadata',
            version: variables.planDag.metadata.version,
            name: variables.planDag.metadata.name,
            description: variables.planDag.metadata.description,
            created: variables.planDag.metadata.created || new Date().toISOString(),
            lastModified: new Date().toISOString(),
            author: variables.planDag.metadata.author || 'Unknown',
          },
        },
      },
    }),
    update: (cache, { data }) => {
      if (data?.updatePlanDag?.success) {
        cache.writeQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
          data: {
            getPlanDag: data.updatePlanDag.planDag,
          },
        })
      }
    },
  })

  const [addNode] = useMutation(ADD_PLAN_DAG_NODE, {
    optimisticResponse: (variables) => ({
      addPlanDagNode: {
        __typename: 'NodeResponse',
        success: true,
        errors: [],
        node: {
          __typename: 'PlanDagNode',
          id: variables.node.id,
          nodeType: variables.node.nodeType,
          position: variables.node.position,
          metadata: variables.node.metadata,
          config: variables.node.config || '{}',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.addPlanDagNode?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.getPlanDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              getPlanDag: {
                ...existing.getPlanDag,
                nodes: [...existing.getPlanDag.nodes, data.addPlanDagNode.node],
              },
            },
          })
        }
      }
    },
  })

  const [updateNode] = useMutation(UPDATE_PLAN_DAG_NODE, {
    optimisticResponse: (variables) => ({
      updatePlanDagNode: {
        __typename: 'NodeResponse',
        success: true,
        errors: [],
        node: {
          __typename: 'PlanDagNode',
          id: variables.nodeId,
          nodeType: variables.updates.nodeType,
          position: variables.updates.position,
          metadata: variables.updates.metadata,
          config: variables.updates.config || '{}',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      },
    }),
  })

  const [deleteNode] = useMutation(DELETE_PLAN_DAG_NODE, {
    optimisticResponse: (variables) => ({
      deletePlanDagNode: {
        __typename: 'NodeResponse',
        success: true,
        errors: [],
        node: null,
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.deletePlanDagNode?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.getPlanDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              getPlanDag: {
                ...existing.getPlanDag,
                nodes: existing.getPlanDag.nodes.filter(
                  (node: any) => node.id !== variables.nodeId
                ),
                edges: existing.getPlanDag.edges.filter(
                  (edge: any) => edge.source !== variables.nodeId && edge.target !== variables.nodeId
                ),
              },
            },
          })
        }
      }
    },
  })

  const [addEdge] = useMutation(ADD_PLAN_DAG_EDGE, {
    optimisticResponse: (variables) => ({
      addPlanDagEdge: {
        __typename: 'EdgeResponse',
        success: true,
        errors: [],
        edge: {
          __typename: 'PlanDagEdge',
          id: variables.edge.id,
          source: variables.edge.source,
          target: variables.edge.target,
          metadata: variables.edge.metadata,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.addPlanDagEdge?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.getPlanDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              getPlanDag: {
                ...existing.getPlanDag,
                edges: [...existing.getPlanDag.edges, data.addPlanDagEdge.edge],
              },
            },
          })
        }
      }
    },
  })

  const [deleteEdge] = useMutation(DELETE_PLAN_DAG_EDGE, {
    optimisticResponse: (variables) => ({
      deletePlanDagEdge: {
        __typename: 'EdgeResponse',
        success: true,
        errors: [],
        edge: null,
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.deletePlanDagEdge?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.getPlanDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              getPlanDag: {
                ...existing.getPlanDag,
                edges: existing.getPlanDag.edges.filter(
                  (edge: any) => edge.id !== variables.edgeId
                ),
              },
            },
          })
        }
      }
    },
  })

  const [moveNode] = useMutation(MOVE_PLAN_DAG_NODE, {
    optimisticResponse: (variables) => ({
      movePlanDagNode: {
        __typename: 'NodeResponse',
        success: true,
        errors: [],
        node: {
          __typename: 'PlanDagNode',
          id: variables.nodeId,
          position: variables.position,
          nodeType: 'Input', // Placeholder - will be updated from server response
          metadata: { label: 'Moving Node', description: null },
          config: '{}',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      },
    }),
  })

  // Wrapper functions with proper typing
  const mutations = useMemo(() => ({
    updatePlanDag: (planDag: PlanDag) =>
      updatePlanDag({ variables: { projectId, planDag } as PlanDagMutationVariables }),

    addNode: (node: Partial<PlanDagNode>) =>
      addNode({ variables: { projectId, node } as PlanDagMutationVariables }),

    updateNode: (nodeId: string, updates: Partial<PlanDagNode>) =>
      updateNode({ variables: { projectId, nodeId, updates } as PlanDagMutationVariables }),

    deleteNode: (nodeId: string) =>
      deleteNode({ variables: { projectId, nodeId } as PlanDagMutationVariables }),

    addEdge: (edge: Partial<PlanDagEdge>) =>
      addEdge({ variables: { projectId, edge } as PlanDagMutationVariables }),

    deleteEdge: (edgeId: string) =>
      deleteEdge({ variables: { projectId, edgeId } as PlanDagMutationVariables }),

    moveNode: (nodeId: string, position: Position) =>
      moveNode({ variables: { projectId, nodeId, position } as PlanDagMutationVariables }),
  }), [projectId, updatePlanDag, addNode, updateNode, deleteNode, addEdge, deleteEdge, moveNode])

  return mutations
}

// Hook for real-time Plan DAG changes
export const usePlanDagSubscription = (projectId: number) => {
  const { data, loading, error } = useSubscription(PLAN_DAG_CHANGED_SUBSCRIPTION, {
    variables: { projectId } as PlanDagSubscriptionVariables,
  })

  const lastChange = useMemo(() => data?.planDagChanged, [data])

  return {
    lastChange,
    loading,
    error,
  }
}

// Hook for user presence in collaborative editing
export const useUserPresence = (projectId: number) => {
  const { data, loading, error } = useSubscription(USER_PRESENCE_SUBSCRIPTION, {
    variables: { projectId } as PlanDagSubscriptionVariables,
  })

  const users = useMemo(() => data?.userPresence || [], [data])

  return {
    users,
    loading,
    error,
  }
}

// Hook for collaboration features
export const useCollaboration = (projectId: number) => {
  const [updateCursorPosition] = useMutation(UPDATE_CURSOR_POSITION)
  const [joinCollaboration] = useMutation(JOIN_PROJECT_COLLABORATION)
  const [leaveCollaboration] = useMutation(LEAVE_PROJECT_COLLABORATION)

  const broadcastCursorPosition = useCallback((positionX: number, positionY: number, selectedNodeId?: string) => {
    // Validate that positions are valid numbers before sending
    if (typeof positionX !== 'number' || typeof positionY !== 'number' ||
        isNaN(positionX) || isNaN(positionY) ||
        !isFinite(positionX) || !isFinite(positionY)) {
      console.warn('Invalid cursor position values:', { positionX, positionY })
      return
    }

    updateCursorPosition({
      variables: {
        projectId,
        positionX,
        positionY,
        selectedNodeId
      }
    }).catch(err => {
      console.warn('Failed to broadcast cursor position:', err)
    })
  }, [projectId, updateCursorPosition])

  const joinProject = useCallback(() => {
    return joinCollaboration({
      variables: { projectId }
    })
  }, [projectId, joinCollaboration])

  const leaveProject = useCallback(() => {
    return leaveCollaboration({
      variables: { projectId }
    })
  }, [projectId, leaveCollaboration])

  return {
    broadcastCursorPosition,
    joinProject,
    leaveProject,
  }
}