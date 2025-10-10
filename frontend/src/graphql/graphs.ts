import { gql } from '@apollo/client'

export interface Layer {
  id: number;
  layerId: string;
  name: string;
  color?: string;
  properties?: {
    background_color?: string;
    border_color?: string;
    text_color?: string;
    [key: string]: any;
  };
}

export interface GraphNode {
  id: string;
  label?: string;
  layer?: string;
  weight?: number;
  isPartition: boolean;
  belongsTo?: string;
  attrs?: any;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  layer?: string;
  weight?: number;
  attrs?: any;
}

export interface Graph {
  id: number
  name: string
  nodeId: string
  executionState: string
  nodeCount: number
  edgeCount: number
  createdAt: string
  updatedAt: string
  layers: Layer[];
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
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
        layerId
        name
        color
        properties
      }
    }
  }
`

export const GET_GRAPH_DETAILS = gql`
  query GetGraphDetails($id: Int!) {
    graph(id: $id) {
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
        layerId
        name
        color
        properties
      }
      graphNodes {
        id
        label
        layer
        weight
        isPartition
        belongsTo
        attrs
      }
      graphEdges {
        id
        source
        target
        label
        layer
        weight
        attrs
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

export const EXECUTE_NODE = gql`
  mutation ExecuteNode($projectId: Int!, $nodeId: String!) {
    executeNode(projectId: $projectId, nodeId: $nodeId) {
      success
      message
      nodeId
    }
  }
`

export const UPDATE_GRAPH_NODE = gql`
  mutation UpdateGraphNode(
    $graphId: Int!
    $nodeId: String!
    $label: String
    $layer: String
    $attrs: JSON
  ) {
    updateGraphNode(
      graphId: $graphId
      nodeId: $nodeId
      label: $label
      layer: $layer
      attrs: $attrs
    ) {
      id
      label
      layer
      attrs
    }
  }
`

export const UPDATE_LAYER_PROPERTIES = gql`
  mutation UpdateLayerProperties(
    $id: Int!
    $name: String
    $properties: JSON
  ) {
    updateLayerProperties(
      id: $id
      name: $name
      properties: $properties
    ) {
      id
      layerId
      name
      properties
    }
  }
`