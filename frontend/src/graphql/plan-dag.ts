import { gql } from '@apollo/client'

// Plan DAG Queries
export const GET_PLAN_DAG = gql`
  query GetPlanDag($projectId: Int!, $planId: Int) {
    getPlanDag(projectId: $projectId, planId: $planId) {
      version
      nodes {
        id
        nodeType
        position {
          x
          y
        }
        sourcePosition
        targetPosition
        metadata {
          label
          description
        }
        config
        datasetExecution {
          dataSetId
          filename
          status
          processedAt
          executionState
          errorMessage
        }
        graphExecution {
          graphId
          nodeCount
          edgeCount
          executionState
          computedDate
          errorMessage
          annotations
        }
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
  mutation UpdatePlanDag($projectId: Int!, $planId: Int, $planDag: PlanDagInput!) {
    updatePlanDag(projectId: $projectId, planId: $planId, planDag: $planDag) {
      version
      nodes {
        id
        nodeType
        position {
          x
          y
        }
        sourcePosition
        targetPosition
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

export const ADD_PLAN_DAG_NODE = gql`
  mutation AddPlanDagNode($projectId: Int!, $planId: Int, $node: PlanDagNodeInput!) {
    addPlanDagNode(projectId: $projectId, planId: $planId, node: $node) {
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
`

export const UPDATE_PLAN_DAG_NODE = gql`
  mutation UpdatePlanDagNode($projectId: Int!, $planId: Int, $nodeId: String!, $updates: PlanDagNodeUpdateInput!) {
    updatePlanDagNode(projectId: $projectId, planId: $planId, nodeId: $nodeId, updates: $updates) {
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
`

export const DELETE_PLAN_DAG_NODE = gql`
  mutation DeletePlanDagNode($projectId: Int!, $planId: Int, $nodeId: String!) {
    deletePlanDagNode(projectId: $projectId, planId: $planId, nodeId: $nodeId) {
      id
    }
  }
`

export const ADD_PLAN_DAG_EDGE = gql`
  mutation AddPlanDagEdge($projectId: Int!, $planId: Int, $edge: PlanDagEdgeInput!) {
    addPlanDagEdge(projectId: $projectId, planId: $planId, edge: $edge) {
      id
      source
      target
      metadata {
        label
        dataType
      }
    }
  }
`

export const DELETE_PLAN_DAG_EDGE = gql`
  mutation DeletePlanDagEdge($projectId: Int!, $planId: Int, $edgeId: String!) {
    deletePlanDagEdge(projectId: $projectId, planId: $planId, edgeId: $edgeId) {
      id
    }
  }
`

export const UPDATE_PLAN_DAG_EDGE = gql`
  mutation UpdatePlanDagEdge($projectId: Int!, $planId: Int, $edgeId: String!, $updates: PlanDagEdgeUpdateInput!) {
    updatePlanDagEdge(projectId: $projectId, planId: $planId, edgeId: $edgeId, updates: $updates) {
      id
      source
      target
      metadata {
        label
        dataType
      }
    }
  }
`

export const MOVE_PLAN_DAG_NODE = gql`
  mutation MovePlanDagNode($projectId: Int!, $planId: Int, $nodeId: String!, $position: PositionInput!) {
    movePlanDagNode(projectId: $projectId, planId: $planId, nodeId: $nodeId, position: $position) {
      id
      position {
        x
        y
      }
    }
  }
`

export const BATCH_MOVE_PLAN_DAG_NODES = gql`
  mutation BatchMovePlanDagNodes($projectId: Int!, $planId: Int, $nodePositions: [NodePositionInput!]!) {
    batchMovePlanDagNodes(projectId: $projectId, planId: $planId, nodePositions: $nodePositions) {
      id
      position {
        x
        y
      }
    }
  }
`

export const VALIDATE_AND_MIGRATE_PLAN_DAG = gql`
  mutation ValidateAndMigratePlanDag($projectId: Int!, $planId: Int) {
    validateAndMigratePlanDag(projectId: $projectId, planId: $planId) {
      checkedNodes
      updatedNodes {
        nodeId
        fromType
        toType
        note
      }
      warnings
      errors
    }
  }
`

// Plan DAG Subscriptions for real-time collaboration
export const PLAN_DAG_CHANGED_SUBSCRIPTION = gql`
  subscription PlanDagChanged($projectId: Int!, $planId: Int) {
    planDagChanged(projectId: $projectId, planId: $planId) {
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

// Delta-based subscription for JSON Patch updates
export const PLAN_DAG_DELTA_SUBSCRIPTION = gql`
  subscription PlanDagDeltaChanged($projectId: Int!, $planId: Int) {
    planDagDeltaChanged(projectId: $projectId, planId: $planId) {
      projectId
      version
      userId
      timestamp
      operations {
        op
        path
        value
        from
      }
    }
  }
`

// Execution status subscription for real-time status updates
export const NODE_EXECUTION_STATUS_SUBSCRIPTION = gql`
  subscription NodeExecutionStatusChanged($projectId: Int!) {
    nodeExecutionStatusChanged(projectId: $projectId) {
      projectId
      nodeId
      nodeType
      datasetExecution {
        dataSetId
        filename
        status
        processedAt
        executionState
        errorMessage
      }
      graphExecution {
        graphId
        nodeCount
        edgeCount
        executionState
        computedDate
        errorMessage
        annotations
      }
      timestamp
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
  planId?: number | null
}

export interface PlanDagMutationVariables {
  projectId: number
  planId?: number | null
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
  planId?: number | null
}
