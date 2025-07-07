import { gql } from '@apollo/client';

// Fragments
export const PLAN_NODE_FRAGMENT = gql`
  fragment PlanNodeFragment on PlanNode {
    id
    plan_id
    node_type
    name
    description
    configuration
    graph_id
    position_x
    position_y
    created_at
    updated_at
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
    plan_id
    plan_node_id
    name
    description
    graph_data
    metadata
    created_at
    updated_at
  }
`;

// Queries
export const GET_PROJECT = gql`
  query GetProject($id: Int!) {
    project(id: $id) {
      id
      name
      description
      created_at
      updated_at
    }
  }
`;

export const GET_PLAN = gql`
  query GetPlan($id: Int!) {
    plan(id: $id) {
      id
      name
      description
      project_id
      plan_content
      plan_format
      plan_schema_version
      status
      created_at
      updated_at
    }
  }
`;

export const GET_PLANS_FOR_PROJECT = gql`
  query GetPlansForProject($projectId: Int!) {
    plans(project_id: $projectId) {
      id
      name
      description
      status
      created_at
      updated_at
    }
  }
`;

export const GET_PLAN_DAG = gql`
  query GetPlanDag($planId: Int!) {
    plan_dag(plan_id: $planId) {
      ...DagPlanFragment
    }
  }
  ${DAG_PLAN_FRAGMENT}
`;

export const GET_PLAN_NODES = gql`
  query GetPlanNodes($planId: Int!) {
    plan_nodes(plan_id: $planId) {
      ...PlanNodeFragment
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const GET_PLAN_NODE = gql`
  query GetPlanNode($id: String!) {
    plan_node(id: $id) {
      ...PlanNodeFragment
    }
  }
  ${PLAN_NODE_FRAGMENT}
`;

export const GET_GRAPH_ARTIFACT = gql`
  query GetGraphArtifact($planNodeId: String!) {
    graph_artifact(plan_node_id: $planNodeId) {
      ...GraphArtifactFragment
    }
  }
  ${GRAPH_ARTIFACT_FRAGMENT}
`;

export const GET_GRAPH_ARTIFACTS = gql`
  query GetGraphArtifacts($planId: Int!) {
    graph_artifacts(plan_id: $planId) {
      ...GraphArtifactFragment
    }
  }
  ${GRAPH_ARTIFACT_FRAGMENT}
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
    delete_plan_node(id: $id)
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