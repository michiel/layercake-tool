import { gql } from '@apollo/client'

export interface Layer {
  id: number;
  layerId: string;
  name: string;
  backgroundColor?: string;
  textColor?: string;
  borderColor?: string;
  alias?: string | null;
  comment?: string;
  properties?: any;
  datasetId?: number;
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
  annotations?: string | null
  hasPendingEdits?: boolean
  lastEditSequence?: number
  lastReplayAt?: string
  layers: Layer[];
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
}

export interface GraphEdit {
  id: number
  graphId: number
  targetType: string
  targetId: string
  operation: string
  fieldName?: string
  oldValue?: any
  newValue?: any
  sequenceNumber: number
  applied: boolean
  createdAt: string
  createdBy?: number
}

export interface EditResult {
  sequenceNumber: number
  targetType: string
  targetId: string
  operation: string
  result: string
  message: string
}

export interface ReplaySummary {
  total: number
  applied: number
  skipped: number
  failed: number
  details: EditResult[]
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
      annotations
      layers {
        id
        layerId
        name
        backgroundColor
        textColor
        borderColor
        alias
        comment
        properties
        datasetId
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
      annotations
      layers {
        id
        layerId
        name
        backgroundColor
        textColor
        borderColor
        alias
        comment
        properties
        datasetId
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

export const CREATE_LAYER = gql`
  mutation CreateLayer($input: CreateLayerInput!) {
    createLayer(input: $input) {
      id
      layerId
      name
      backgroundColor
      textColor
      borderColor
      alias
      comment
      properties
      datasetId
    }
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
    $belongsTo: String
  ) {
    updateGraphNode(
      graphId: $graphId
      nodeId: $nodeId
      label: $label
      layer: $layer
      attrs: $attrs
      belongsTo: $belongsTo
    ) {
      id
      label
      layer
      attrs
      belongsTo
    }
  }
`

export const ADD_GRAPH_NODE = gql`
  mutation AddGraphNode(
    $graphId: Int!
    $id: String!
    $label: String
    $layer: String
    $isPartition: Boolean!
    $belongsTo: String
    $weight: Float
    $attrs: JSON
  ) {
    addGraphNode(
      graphId: $graphId
      id: $id
      label: $label
      layer: $layer
      isPartition: $isPartition
      belongsTo: $belongsTo
      weight: $weight
      attrs: $attrs
    ) {
      id
      label
      layer
      isPartition
      belongsTo
      weight
      attrs
    }
  }
`

export const ADD_GRAPH_EDGE = gql`
  mutation AddGraphEdge(
    $graphId: Int!
    $id: String!
    $source: String!
    $target: String!
    $label: String
    $layer: String
    $weight: Float
    $attrs: JSON
  ) {
    addGraphEdge(
      graphId: $graphId
      id: $id
      source: $source
      target: $target
      label: $label
      layer: $layer
      weight: $weight
      attrs: $attrs
    ) {
      id
      source
      target
      label
      layer
      weight
      attrs
    }
  }
`

export const UPDATE_GRAPH_EDGE = gql`
  mutation UpdateGraphEdge(
    $graphId: Int!
    $edgeId: String!
    $label: String
    $layer: String
    $attrs: JSON
  ) {
    updateGraphEdge(
      graphId: $graphId
      edgeId: $edgeId
      label: $label
      layer: $layer
      attrs: $attrs
    ) {
      id
      source
      target
      label
      layer
      attrs
    }
  }
`

export const DELETE_GRAPH_EDGE = gql`
  mutation DeleteGraphEdge($graphId: Int!, $edgeId: String!) {
    deleteGraphEdge(graphId: $graphId, edgeId: $edgeId)
  }
`

export const DELETE_GRAPH_NODE = gql`
  mutation DeleteGraphNode($graphId: Int!, $nodeId: String!) {
    deleteGraphNode(graphId: $graphId, nodeId: $nodeId)
  }
`

export const UPDATE_LAYER_PROPERTIES = gql`
  mutation UpdateLayerProperties(
    $id: Int!
    $name: String
    $alias: String
    $properties: JSON
  ) {
    updateLayerProperties(
      id: $id
      name: $name
      alias: $alias
      properties: $properties
    ) {
      id
      layerId
      name
      alias
      properties
    }
  }
`

export const BULK_UPDATE_GRAPH_DATA = gql`
  mutation BulkUpdateGraphData(
    $graphId: Int!
    $nodes: [GraphNodeUpdateInput!]
    $layers: [LayerUpdateInput!]
  ) {
    bulkUpdateGraphData(
      graphId: $graphId
      nodes: $nodes
      layers: $layers
    )
  }
`

export const GET_GRAPH_EDITS = gql`
  query GetGraphEdits($graphId: Int!, $unappliedOnly: Boolean) {
    graphEdits(graphId: $graphId, unappliedOnly: $unappliedOnly) {
      id
      graphId
      targetType
      targetId
      operation
      fieldName
      oldValue
      newValue
      sequenceNumber
      applied
      createdAt
      createdBy
    }
  }
`

export const GET_GRAPH_EDIT_COUNT = gql`
  query GetGraphEditCount($graphId: Int!, $unappliedOnly: Boolean) {
    graphEditCount(graphId: $graphId, unappliedOnly: $unappliedOnly)
  }
`

export const REPLAY_GRAPH_EDITS = gql`
  mutation ReplayGraphEdits($graphId: Int!) {
    replayGraphEdits(graphId: $graphId) {
      total
      applied
      skipped
      failed
      details {
        sequenceNumber
        targetType
        targetId
        operation
        result
        message
      }
    }
  }
`

export const CLEAR_GRAPH_EDITS = gql`
  mutation ClearGraphEdits($graphId: Int!) {
    clearGraphEdits(graphId: $graphId)
  }
`
