import { gql } from '@apollo/client'

export interface Layer {
  id: string;
  name: string;
  color: string;
}

export interface Graph {
  id: string
  name: string
  nodeId: string
  executionState: string
  nodeCount: number
  edgeCount: number
  createdAt: string
  updatedAt: string
  layers: Layer[];
}

export const GET_GRAPHS = gql`
  query GetGraphs($projectId: Int!) {
    graphs(projectId: $projectId) {
      id
      name
      nodeId
      executionState
      nodeCount
      edgeCount
      createdAt
      updatedAt
      layers {
        id
        name
        color
      }
    }
  }
`

export const CREATE_GRAPH = gql`
  mutation CreateGraph($input: CreateGraphInput!) {
    createGraph(input: $input) {
      id
    }
  }
`

export const UPDATE_GRAPH = gql`
  mutation UpdateGraph($id: Int!, $input: UpdateGraphInput!) {
    updateGraph(id: $id, input: $input) {
      id
    }
  }
`

export const DELETE_GRAPH = gql`
  mutation DeleteGraph($id: Int!) {
    deleteGraph(id: $id)
  }
`