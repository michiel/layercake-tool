import type { EdgeMetadata } from '../types/plan-dag'

export type EdgeDataType = EdgeMetadata['dataType'] | undefined

const EDGE_COLORS: Record<EdgeMetadata['dataType'], string> = {
  GRAPH_DATA: '#868e96',
  GRAPH_REFERENCE: '#228be6',
  SEQUENCE_DATA: '#f97316',
}

const EDGE_LABELS: Record<EdgeMetadata['dataType'], string> = {
  GRAPH_DATA: 'Data',
  GRAPH_REFERENCE: 'Graph Ref',
  SEQUENCE_DATA: 'Sequence',
}

export const getEdgeColor = (dataType: EdgeDataType): string => {
  if (!dataType) {
    return EDGE_COLORS.GRAPH_DATA
  }
  return EDGE_COLORS[dataType] ?? EDGE_COLORS.GRAPH_DATA
}

export const getEdgeLabel = (dataType: EdgeDataType): string => {
  if (!dataType) {
    return EDGE_LABELS.GRAPH_DATA
  }
  return EDGE_LABELS[dataType] ?? EDGE_LABELS.GRAPH_DATA
}
