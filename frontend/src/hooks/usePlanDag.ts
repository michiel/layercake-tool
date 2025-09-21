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

  const planDag = useMemo(() => data?.project?.planDag, [data])

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
    optimisticResponse: true,
    update: (cache, { data }) => {
      if (data?.updatePlanDag?.success) {
        cache.writeQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
          data: {
            project: {
              __typename: 'Project',
              id: projectId,
              planDag: data.updatePlanDag.planDag,
            },
          },
        })
      }
    },
  })

  const [addNode] = useMutation(ADD_PLAN_DAG_NODE, {
    optimisticResponse: (variables) => ({
      addPlanDagNode: {
        __typename: 'PlanDagNodeResult',
        success: true,
        errors: [],
        node: {
          __typename: 'PlanDagNode',
          ...variables.node,
        },
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.addPlanDagNode?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.project?.planDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              project: {
                ...existing.project,
                planDag: {
                  ...existing.project.planDag,
                  nodes: [...existing.project.planDag.nodes, data.addPlanDagNode.node],
                },
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
        __typename: 'PlanDagNodeResult',
        success: true,
        errors: [],
        node: {
          __typename: 'PlanDagNode',
          id: variables.nodeId,
          ...variables.updates,
        },
      },
    }),
  })

  const [deleteNode] = useMutation(DELETE_PLAN_DAG_NODE, {
    optimisticResponse: (variables) => ({
      deletePlanDagNode: {
        __typename: 'PlanDagDeleteResult',
        success: true,
        errors: [],
        deletedNodeId: variables.nodeId,
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.deletePlanDagNode?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.project?.planDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              project: {
                ...existing.project,
                planDag: {
                  ...existing.project.planDag,
                  nodes: existing.project.planDag.nodes.filter(
                    (node: any) => node.id !== variables.nodeId
                  ),
                  edges: existing.project.planDag.edges.filter(
                    (edge: any) => edge.source !== variables.nodeId && edge.target !== variables.nodeId
                  ),
                },
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
        __typename: 'PlanDagEdgeResult',
        success: true,
        errors: [],
        edge: {
          __typename: 'PlanDagEdge',
          ...variables.edge,
        },
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.addPlanDagEdge?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.project?.planDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              project: {
                ...existing.project,
                planDag: {
                  ...existing.project.planDag,
                  edges: [...existing.project.planDag.edges, data.addPlanDagEdge.edge],
                },
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
        __typename: 'PlanDagDeleteResult',
        success: true,
        errors: [],
        deletedEdgeId: variables.edgeId,
      },
    }),
    update: (cache, { data }, { variables }) => {
      if (data?.deletePlanDagEdge?.success) {
        const existing = cache.readQuery({
          query: GET_PLAN_DAG,
          variables: { projectId },
        })

        if (existing?.project?.planDag) {
          cache.writeQuery({
            query: GET_PLAN_DAG,
            variables: { projectId },
            data: {
              project: {
                ...existing.project,
                planDag: {
                  ...existing.project.planDag,
                  edges: existing.project.planDag.edges.filter(
                    (edge: any) => edge.id !== variables.edgeId
                  ),
                },
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
        __typename: 'PlanDagNodeResult',
        success: true,
        errors: [],
        node: {
          __typename: 'PlanDagNode',
          id: variables.nodeId,
          position: variables.position,
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