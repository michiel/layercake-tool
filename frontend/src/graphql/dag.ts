import { gql } from '@apollo/client';

// Fragments
export const PLAN_NODE_FRAGMENT = gql`
  fragment PlanNodeFragment on PlanNode {
    id
    planId
    nodeType
    name
    description
    configuration
    graphId
    positionX
    positionY
    createdAt
    updatedAt
  }
`;

export const DAG_PLAN_FRAGMENT = gql`
  fragment DagPlanFragment on DagPlan {
    nodes {
      ...PlanNodeFragment
    }
    edges {
      source
      target
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const GRAPH_ARTIFACT_FRAGMENT = gql`
  fragment GraphArtifactFragment on GraphArtifact {
    id
    planId
    planNodeId
    name
    description
    graphData
    metadata
    createdAt
    updatedAt
  }
`;

// Graph data fragments
export const NODE_FRAGMENT = gql`
  fragment NodeFragment on Node {
    id
    projectId
    nodeId
    label
    layerId
    properties
  }
`;

export const EDGE_FRAGMENT = gql`
  fragment EdgeFragment on Edge {
    id
    projectId
    sourceNodeId
    targetNodeId
    properties
  }
`;

export const LAYER_FRAGMENT = gql`
  fragment LayerFragment on Layer {
    id
    projectId
    layerId
    name
    color
    properties
  }
`;

// Queries
export const GET_PROJECT = gql`
  query GetProject($id: Int!) {
    project(id: $id) {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`;

export const GET_PLAN = gql`
  query GetPlan($id: Int!) {
    plan(id: $id) {
      id
      name
      projectId
      planContent
      planFormat
      planSchemaVersion
      status
      createdAt
      updatedAt
    }
  }
`;

export const GET_PLANS_FOR_PROJECT = gql`
  query GetPlansForProject($projectId: Int!) {
    plans(projectId: $projectId) {
      id
      name
      status
      createdAt
      updatedAt
    }
  }
`;

export const GET_PLAN_DAG = gql`
  query GetPlanDag($planId: Int!) {
    planDag(planId: $planId) {
      ...DagPlanFragment
    }
  }
  ${DAG_PLAN_FRAGMENT}
`;

export const GET_PLAN_NODES = gql`
  query GetPlanNodes($planId: Int!) {
    planNodes(planId: $planId) {
      ...PlanNodeFragment
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const GET_PLAN_NODE = gql`
  query GetPlanNode($id: String!) {
    planNode(id: $id) {
      ...PlanNodeFragment
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const GET_GRAPH_ARTIFACT = gql`
  query GetGraphArtifact($planNodeId: String!) {
    graphArtifact(planNodeId: $planNodeId) {
      ...GraphArtifactFragment
    }
  }
  ${GRAPH_ARTIFACT_FRAGMENT}
`;

export const GET_GRAPH_ARTIFACTS = gql`
  query GetGraphArtifacts($planId: Int!) {
    graphArtifacts(planId: $planId) {
      ...GraphArtifactFragment
    }
  }
  ${GRAPH_ARTIFACT_FRAGMENT}
`;

// Graph data queries
export const GET_NODES = gql`
  query GetNodes($projectId: Int!) {
    nodes(projectId: $projectId) {
      ...NodeFragment
    }
  }
  ${NODE_FRAGMENT}
`;

export const GET_EDGES = gql`
  query GetEdges($projectId: Int!) {
    edges(projectId: $projectId) {
      ...EdgeFragment
    }
  }
  ${EDGE_FRAGMENT}
`;

export const GET_LAYERS = gql`
  query GetLayers($projectId: Int!) {
    layers(projectId: $projectId) {
      ...LayerFragment
    }
  }
  ${LAYER_FRAGMENT}
`;

export const GET_GRAPH_DATA = gql`
  query GetGraphData($projectId: Int!) {
    graphData(projectId: $projectId) {
      nodes {
        ...NodeFragment
      }
      edges {
        ...EdgeFragment
      }
      layers {
        ...LayerFragment
      }
    }
  }
  ${NODE_FRAGMENT}
  ${EDGE_FRAGMENT}
  ${LAYER_FRAGMENT}
`;

// Mutations
export const CREATE_PLAN_NODE = gql`
  mutation CreatePlanNode($input: CreatePlanNodeInput!) {
    create_plan_node(input: $input) {
      ...PlanNodeFragment
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const UPDATE_PLAN_NODE = gql`
  mutation UpdatePlanNode($id: String!, $input: UpdatePlanNodeInput!) {
    update_plan_node(id: $id, input: $input) {
      ...PlanNodeFragment
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const DELETE_PLAN_NODE = gql`
  mutation DeletePlanNode($id: String!) {
    delete_planNode(id: $id)
  }
`;

// Graph data mutations
export const CREATE_NODE = gql`
  mutation CreateNode($projectId: Int!, $input: CreateNodeInput!) {
    create_node(project_id: $projectId, input: $input) {
      ...NodeFragment
    }
  }
  ${NODE_FRAGMENT}
`;

export const UPDATE_NODE = gql`
  mutation UpdateNode($nodeId: String!, $input: UpdateNodeInput!) {
    update_node(node_id: $nodeId, input: $input) {
      ...NodeFragment
    }
  }
  ${NODE_FRAGMENT}
`;

export const DELETE_NODE = gql`
  mutation DeleteNode($nodeId: String!) {
    delete_node(node_id: $nodeId)
  }
`;

export const CREATE_EDGE = gql`
  mutation CreateEdge($projectId: Int!, $input: CreateEdgeInput!) {
    create_edge(project_id: $projectId, input: $input) {
      ...EdgeFragment
    }
  }
  ${EDGE_FRAGMENT}
`;

export const UPDATE_EDGE = gql`
  mutation UpdateEdge($edgeId: Int!, $input: UpdateEdgeInput!) {
    update_edge(edge_id: $edgeId, input: $input) {
      ...EdgeFragment
    }
  }
  ${EDGE_FRAGMENT}
`;

export const DELETE_EDGE = gql`
  mutation DeleteEdge($edgeId: Int!) {
    delete_edge(edge_id: $edgeId)
  }
`;

export const CREATE_LAYER = gql`
  mutation CreateLayer($projectId: Int!, $input: CreateLayerInput!) {
    create_layer(project_id: $projectId, input: $input) {
      ...LayerFragment
    }
  }
  ${LAYER_FRAGMENT}
`;

export const UPDATE_LAYER = gql`
  mutation UpdateLayer($layerId: String!, $input: UpdateLayerInput!) {
    update_layer(layer_id: $layerId, input: $input) {
      ...LayerFragment
    }
  }
  ${LAYER_FRAGMENT}
`;

export const DELETE_LAYER = gql`
  mutation DeleteLayer($layerId: String!) {
    delete_layer(layer_id: $layerId)
  }
`;

// Input types for TypeScript
export interface CreatePlanNodeInput {
  plan_id: number;
  node_type: string;
  name: string;
  description?: string | null;
  configuration: string;
  position_x?: number | null;
  position_y?: number | null;
}

export interface UpdatePlanNodeInput {
  name?: string;
  description?: string | null;
  configuration?: string;
  position_x?: number | null;
  position_y?: number | null;
}

// Graph data input types
export interface CreateNodeInput {
  node_id: string;
  label: string;
  layer_id?: string | null;
  properties?: any;
}

export interface UpdateNodeInput {
  label?: string;
  layer_id?: string | null;
  properties?: any;
}

export interface CreateEdgeInput {
  source_node_id: string;
  target_node_id: string;
  properties?: any;
}

export interface UpdateEdgeInput {
  source_node_id?: string;
  target_node_id?: string;
  properties?: any;
}

export interface CreateLayerInput {
  layer_id: string;
  name: string;
  color?: string | null;
  properties?: any;
}

export interface UpdateLayerInput {
  name?: string;
  color?: string | null;
  properties?: any;
}

// Query result types
export interface GetPlanDagResponse {
  plan_dag: {
    nodes: Array<{
      id: string;
      plan_id: number;
      node_type: string;
      name: string;
      description?: string | null;
      configuration: string;
      graph_id?: string | null;
      position_x?: number | null;
      position_y?: number | null;
      created_at: string;
      updated_at: string;
    }>;
    edges: Array<{
      source: string;
      target: string;
    }>;
  } | null;
}

export interface GetPlanNodesResponse {
  plan_nodes: Array<{
    id: string;
    plan_id: number;
    node_type: string;
    name: string;
    description?: string | null;
    configuration: string;
    graph_id?: string | null;
    position_x?: number | null;
    position_y?: number | null;
    created_at: string;
    updated_at: string;
  }>;
}

export interface GetPlanNodeResponse {
  plan_node: {
    id: string;
    plan_id: number;
    node_type: string;
    name: string;
    description?: string | null;
    configuration: string;
    graph_id?: string | null;
    position_x?: number | null;
    position_y?: number | null;
    created_at: string;
    updated_at: string;
  } | null;
}

export interface GetGraphArtifactResponse {
  graph_artifact: {
    id: string;
    plan_id: number;
    plan_node_id: string;
    name: string;
    description?: string | null;
    graph_data: string;
    metadata?: string | null;
    created_at: string;
    updated_at: string;
  } | null;
}

export interface GetGraphArtifactsResponse {
  graph_artifacts: Array<{
    id: string;
    plan_id: number;
    plan_node_id: string;
    name: string;
    description?: string | null;
    graph_data: string;
    metadata?: string | null;
    created_at: string;
    updated_at: string;
  }>;
}