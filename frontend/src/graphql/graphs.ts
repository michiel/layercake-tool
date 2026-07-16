import { gql, type TypedDocumentNode } from '@apollo/client'
import { AttributesMap } from '@/utils/attributes'

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
  attributes?: AttributesMap;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  layer?: string;
  weight?: number;
  attrs?: any;
  attributes?: AttributesMap;
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

export interface GraphValidationResult {
  graphId: number
  projectId: number
  isValid: boolean
  errors: string[]
  warnings: string[]
  nodeCount: number
  edgeCount: number
  layerCount: number
  checkedAt: string
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

export const GET_GRAPHS: TypedDocumentNode<
  { graphs: Graph[] },
  { projectId: number }
> = gql`
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

export const GET_GRAPH_DETAILS: TypedDocumentNode<
  { graph: Graph | null },
  { id: number }
> = gql`
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
        attributes
      }
      graphEdges {
        id
        source
        target
        label
        layer
        weight
        attrs
        attributes
      }
    }
  }
`

export const CREATE_GRAPH: TypedDocumentNode<
  { createGraph: { id: number } },
  { input: Record<string, unknown> }
> = gql`
  mutation CreateGraph($input: CreateGraphInput!) {
    createGraph(input: $input) {
      id
    }
  }
`

export const UPDATE_GRAPH: TypedDocumentNode<
  { updateGraph: { id: number } },
  { id: number; input: Record<string, unknown> }
> = gql`
  mutation UpdateGraph($id: Int!, $input: UpdateGraphInput!) {
    updateGraph(id: $id, input: $input) {
      id
    }
  }
`

export const DELETE_GRAPH: TypedDocumentNode<
  { deleteGraph: boolean },
  { id: number }
> = gql`
  mutation DeleteGraph($id: Int!) {
    deleteGraph(id: $id)
  }
`

export const EXECUTE_NODE: TypedDocumentNode<
  { executeNode: { success: boolean; message: string; nodeId: string } },
  { projectId: number; nodeId: string }
> = gql`
  mutation ExecuteNode($projectId: Int!, $nodeId: String!) {
    executeNode(projectId: $projectId, nodeId: $nodeId) {
      success
      message
      nodeId
    }
  }
`

export const VALIDATE_GRAPH: TypedDocumentNode<
  { validateGraph: GraphValidationResult },
  { id: number }
> = gql`
  mutation ValidateGraph($id: Int!) {
    validateGraph(id: $id) {
      graphId
      projectId
      isValid
      errors
      warnings
      nodeCount
      edgeCount
      layerCount
      checkedAt
    }
  }
`

export const UPDATE_GRAPH_NODE: TypedDocumentNode<
  {
    updateGraphNode: {
      id: string
      label?: string
      layer?: string
      attributes?: AttributesMap
      belongsTo?: string
    }
  },
  {
    graphId: number
    nodeId: string
    label?: string
    layer?: string
    attributes?: unknown
    belongsTo?: string
  }
> = gql`
  mutation UpdateGraphNode(
    $graphId: Int!
    $nodeId: String!
    $label: String
    $layer: String
    $attributes: JSON
    $belongsTo: String
  ) {
    updateGraphNode(
      graphId: $graphId
      nodeId: $nodeId
      label: $label
      layer: $layer
      attributes: $attributes
      belongsTo: $belongsTo
    ) {
      id
      label
      layer
      attributes
      belongsTo
    }
  }
`

export const ADD_GRAPH_NODE: TypedDocumentNode<
  {
    addGraphNode: {
      id: string
      label?: string
      layer?: string
      isPartition: boolean
      belongsTo?: string
      weight?: number
      attributes?: AttributesMap
    }
  },
  {
    graphId: number
    id: string
    label?: string
    layer?: string
    isPartition: boolean
    belongsTo?: string
    weight?: number
    attributes?: unknown
  }
> = gql`
  mutation AddGraphNode(
    $graphId: Int!
    $id: String!
    $label: String
    $layer: String
    $isPartition: Boolean!
    $belongsTo: String
    $weight: Float
    $attributes: JSON
  ) {
    addGraphNode(
      graphId: $graphId
      id: $id
      label: $label
      layer: $layer
      isPartition: $isPartition
      belongsTo: $belongsTo
      weight: $weight
      attributes: $attributes
    ) {
      id
      label
      layer
      isPartition
      belongsTo
      weight
      attributes
    }
  }
`

export const ADD_GRAPH_EDGE: TypedDocumentNode<
  {
    addGraphEdge: {
      id: string
      source: string
      target: string
      label?: string
      layer?: string
      weight?: number
      attributes?: AttributesMap
    }
  },
  {
    graphId: number
    id: string
    source: string
    target: string
    label?: string
    layer?: string
    weight?: number
    attributes?: unknown
  }
> = gql`
  mutation AddGraphEdge(
    $graphId: Int!
    $id: String!
    $source: String!
    $target: String!
    $label: String
    $layer: String
    $weight: Float
    $attributes: JSON
  ) {
    addGraphEdge(
      graphId: $graphId
      id: $id
      source: $source
      target: $target
      label: $label
      layer: $layer
      weight: $weight
      attributes: $attributes
    ) {
      id
      source
      target
      label
      layer
      weight
      attributes
    }
  }
`

export const UPDATE_GRAPH_EDGE: TypedDocumentNode<
  {
    updateGraphEdge: {
      id: string
      source: string
      target: string
      label?: string
      layer?: string
      attributes?: AttributesMap
    }
  },
  {
    graphId: number
    edgeId: string
    label?: string
    layer?: string
    attributes?: unknown
  }
> = gql`
  mutation UpdateGraphEdge(
    $graphId: Int!
    $edgeId: String!
    $label: String
    $layer: String
    $attributes: JSON
  ) {
    updateGraphEdge(
      graphId: $graphId
      edgeId: $edgeId
      label: $label
      layer: $layer
      attributes: $attributes
    ) {
      id
      source
      target
      label
      layer
      attributes
    }
  }
`

export const DELETE_GRAPH_EDGE: TypedDocumentNode<
  { deleteGraphEdge: boolean },
  { graphId: number; edgeId: string }
> = gql`
  mutation DeleteGraphEdge($graphId: Int!, $edgeId: String!) {
    deleteGraphEdge(graphId: $graphId, edgeId: $edgeId)
  }
`

export const DELETE_GRAPH_NODE: TypedDocumentNode<
  { deleteGraphNode: boolean },
  { graphId: number; nodeId: string }
> = gql`
  mutation DeleteGraphNode($graphId: Int!, $nodeId: String!) {
    deleteGraphNode(graphId: $graphId, nodeId: $nodeId)
  }
`

export const BULK_UPDATE_GRAPH_DATA: TypedDocumentNode<
  { bulkUpdateGraphData: boolean },
  { graphId: number; nodes?: Record<string, unknown>[] }
> = gql`
  mutation BulkUpdateGraphData(
    $graphId: Int!
    $nodes: [GraphNodeUpdateInput!]
  ) {
    bulkUpdateGraphData(
      graphId: $graphId
      nodes: $nodes
    )
  }
`

export const GET_GRAPH_EDITS: TypedDocumentNode<
  { graphEdits: GraphEdit[] },
  { graphId: number; unappliedOnly?: boolean }
> = gql`
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

export const GET_GRAPH_EDIT_COUNT: TypedDocumentNode<
  { graphEditCount: number },
  { graphId: number; unappliedOnly?: boolean }
> = gql`
  query GetGraphEditCount($graphId: Int!, $unappliedOnly: Boolean) {
    graphEditCount(graphId: $graphId, unappliedOnly: $unappliedOnly)
  }
`

export const REPLAY_GRAPH_EDITS: TypedDocumentNode<
  { replayGraphEdits: ReplaySummary },
  { graphId: number }
> = gql`
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

export const CLEAR_GRAPH_EDITS: TypedDocumentNode<
  { clearGraphEdits: boolean },
  { graphId: number }
> = gql`
  mutation ClearGraphEdits($graphId: Int!) {
    clearGraphEdits(graphId: $graphId)
  }
`
