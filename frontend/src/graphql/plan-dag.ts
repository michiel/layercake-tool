import { gql } from '@apollo/client'

// Plan DAG Queries
export const GET_PLAN_DAG = gql`
  query GetPlanDag($projectId: Int!) {
    getPlanDag(projectId: $projectId) {
      version
      nodes {
        id
        nodeType
        position {
          x
          y
        }
        metadata {
          label
          description
        }
        config
      }
      edges {
        id
        source
        target
        metadata {
          label
          dataType
        }
      }
      metadata {
        version
        name
        description
        created
        lastModified
        author
      }
    }
  }
`

export const VALIDATE_PLAN_DAG = gql`
  query ValidatePlanDag($planDag: PlanDagInput!) {
    validatePlanDag(planDag: $planDag) {
      isValid
      errors {
        nodeId
        edgeId
        nodeType
        message
      }
      warnings {
        nodeId
        edgeId
        nodeType
        message
      }
    }
  }
`

// Plan DAG Mutations
export const UPDATE_PLAN_DAG = gql`
  mutation UpdatePlanDag($projectId: Int!, $planDag: PlanDagInput!) {
    updatePlanDag(projectId: $projectId, planDag: $planDag) {
      success
      errors
      plan_dag {
        version
        nodes {
          id
          nodeType
          position {
            x
            y
          }
          metadata {
            label
            description
          }
          config
        }
        edges {
          id
          source
          target
          metadata {
            label
            dataType
          }
        }
        metadata {
          version
          name
          description
          created
          lastModified
          author
        }
      }
    }
  }
`

export const ADD_PLAN_DAG_NODE = gql`
  mutation AddPlanDagNode($projectId: Int!, $node: PlanDagNodeInput!) {
    addPlanDagNode(projectId: $projectId, node: $node) {
      success
      errors
      node {
        id
        nodeType
        position {
          x
          y
        }
        metadata {
          label
          description
        }
        config
      }
    }
  }
`

export const UPDATE_PLAN_DAG_NODE = gql`
  mutation UpdatePlanDagNode($projectId: Int!, $nodeId: String!, $updates: PlanDagNodeUpdateInput!) {
    updatePlanDagNode(projectId: $projectId, nodeId: $nodeId, updates: $updates) {
      success
      errors
      node {
        id
        nodeType
        position {
          x
          y
        }
        metadata {
          label
          description
        }
        config
      }
    }
  }
`

export const DELETE_PLAN_DAG_NODE = gql`
  mutation DeletePlanDagNode($projectId: Int!, $nodeId: String!) {
    deletePlanDagNode(projectId: $projectId, nodeId: $nodeId) {
      success
      errors
      deletedNodeId
    }
  }
`

export const ADD_PLAN_DAG_EDGE = gql`
  mutation AddPlanDagEdge($projectId: Int!, $edge: PlanDagEdgeInput!) {
    addPlanDagEdge(projectId: $projectId, edge: $edge) {
      success
      errors
      edge {
        id
        source
        target
        metadata {
          label
          dataType
        }
      }
    }
  }
`

export const DELETE_PLAN_DAG_EDGE = gql`
  mutation DeletePlanDagEdge($projectId: Int!, $edgeId: String!) {
    deletePlanDagEdge(projectId: $projectId, edgeId: $edgeId) {
      success
      errors
      deletedEdgeId
    }
  }
`

export const MOVE_PLAN_DAG_NODE = gql`
  mutation MovePlanDagNode($projectId: Int!, $nodeId: String!, $position: PositionInput!) {
    movePlanDagNode(projectId: $projectId, nodeId: $nodeId, position: $position) {
      success
      errors
      node {
        id
        position {
          x
          y
        }
      }
    }
  }
`

// Plan DAG Subscriptions for real-time collaboration
export const PLAN_DAG_CHANGED_SUBSCRIPTION = gql`
  subscription PlanDagChanged($projectId: Int!) {
    planDagChanged(projectId: $projectId) {
      type
      projectId
      changeId
      timestamp
      userId
      change {
        ... on PlanDagNodeChange {
          node {
            id
            nodeType
            position {
              x
              y
            }
            metadata {
              label
              description
            }
            config
          }
          operation
        }
        ... on PlanDagEdgeChange {
          edge {
            id
            source
            target
            metadata {
              label
              dataType
            }
          }
          operation
        }
        ... on PlanDagMetadataChange {
          metadata {
            version
            name
            description
            created
            lastModified
            author
          }
        }
      }
    }
  }
`

export const USER_PRESENCE_SUBSCRIPTION = gql`
  subscription UserPresenceChanged($planId: ID!) {
    userPresenceChanged(planId: $planId) {
      userId
      userName
      avatarColor
      cursorPosition {
        x
        y
      }
      selectedNodeId
      isActive
      lastSeen
    }
  }
`

// Collaboration Mutations

export const JOIN_PROJECT_COLLABORATION = gql`
  mutation JoinProjectCollaboration($projectId: Int!) {
    joinProjectCollaboration(projectId: $projectId)
  }
`

export const LEAVE_PROJECT_COLLABORATION = gql`
  mutation LeaveProjectCollaboration($projectId: Int!) {
    leaveProjectCollaboration(projectId: $projectId)
  }
`

// TypeScript types for the GraphQL operations
export interface PlanDagQueryVariables {
  projectId: number
}

export interface PlanDagMutationVariables {
  projectId: number
  planDag?: any
  nodeId?: string
  edgeId?: string
  node?: any
  edge?: any
  updates?: any
  position?: { x: number; y: number }
}

export interface PlanDagSubscriptionVariables {
  projectId: number
}