import { PlanDagNodeType, ConnectionType } from '../types/plan-dag'

/**
 * Validates if a connection between two node types is allowed
 */
export const validateConnection = (
  sourceType: PlanDagNodeType,
  targetType: PlanDagNodeType
): ConnectionType => {
  // Define valid connections based on Plan DAG flow logic
  const validConnections: Record<PlanDagNodeType, PlanDagNodeType[]> = {
    [PlanDagNodeType.DATA_SOURCE]: [
      PlanDagNodeType.GRAPH,     // DataSources primarily connect to Graph nodes
      PlanDagNodeType.MERGE,     // Can also merge multiple data sources
      PlanDagNodeType.TRANSFORM, // Or transform data directly
      PlanDagNodeType.OUTPUT,    // Or output directly
    ],
    [PlanDagNodeType.GRAPH]: [
      PlanDagNodeType.TRANSFORM,
      PlanDagNodeType.COPY,
      PlanDagNodeType.OUTPUT,
    ],
    [PlanDagNodeType.TRANSFORM]: [
      PlanDagNodeType.MERGE,
      PlanDagNodeType.COPY,
      PlanDagNodeType.OUTPUT,
      PlanDagNodeType.TRANSFORM, // Allow chaining transforms
    ],
    [PlanDagNodeType.MERGE]: [
      PlanDagNodeType.TRANSFORM,
      PlanDagNodeType.COPY,
      PlanDagNodeType.OUTPUT,
    ],
    [PlanDagNodeType.COPY]: [
      PlanDagNodeType.TRANSFORM,
      PlanDagNodeType.OUTPUT,
    ],
    [PlanDagNodeType.OUTPUT]: [], // Output nodes have no outgoing connections
  }

  const allowedTargets = validConnections[sourceType] || []
  const isValid = allowedTargets.includes(targetType)

  // Determine data type based on source node
  const getDataType = (source: PlanDagNodeType): 'GraphData' | 'GraphReference' => {
    switch (source) {
      case PlanDagNodeType.GRAPH:
        return 'GraphReference'
      case PlanDagNodeType.DATA_SOURCE:
      case PlanDagNodeType.TRANSFORM:
      case PlanDagNodeType.MERGE:
      case PlanDagNodeType.COPY:
        return 'GraphData'
      default:
        return 'GraphData'
    }
  }

  if (!isValid) {
    // Provide more specific error messages for common invalid connections
    let errorMessage = `Cannot connect ${sourceType} to ${targetType}`

    if (sourceType === PlanDagNodeType.DATA_SOURCE) {
      errorMessage = `DataSource nodes can only connect to Graph, Merge, Transform, or Output nodes`
    } else if (targetType === PlanDagNodeType.DATA_SOURCE) {
      errorMessage = `DataSource nodes cannot receive input connections (they are source nodes)`
    } else if (targetType === PlanDagNodeType.GRAPH) {
      errorMessage = `Only DataSource nodes can connect to Graph nodes`
    }

    return {
      sourceType,
      targetType,
      dataType: getDataType(sourceType),
      isValid: false,
      errorMessage,
    }
  }

  return {
    sourceType,
    targetType,
    dataType: getDataType(sourceType),
    isValid: true,
  }
}

/**
 * Validates if a node can accept multiple inputs
 */
export const canAcceptMultipleInputs = (nodeType: PlanDagNodeType): boolean => {
  switch (nodeType) {
    case PlanDagNodeType.MERGE:
      return true
    case PlanDagNodeType.DATA_SOURCE:
    case PlanDagNodeType.GRAPH:
    case PlanDagNodeType.TRANSFORM:
    case PlanDagNodeType.COPY:
    case PlanDagNodeType.OUTPUT:
      return false
    default:
      return false
  }
}

/**
 * Validates if a node can have multiple outputs
 */
export const canHaveMultipleOutputs = (nodeType: PlanDagNodeType): boolean => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
    case PlanDagNodeType.GRAPH:
    case PlanDagNodeType.TRANSFORM:
    case PlanDagNodeType.MERGE:
    case PlanDagNodeType.COPY:
      return true
    case PlanDagNodeType.OUTPUT:
      return false
    default:
      return false
  }
}

/**
 * Gets the required input count for a node type
 */
export const getRequiredInputCount = (nodeType: PlanDagNodeType): number => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
    case PlanDagNodeType.GRAPH:
      return 0 // These are source nodes
    case PlanDagNodeType.TRANSFORM:
    case PlanDagNodeType.COPY:
    case PlanDagNodeType.OUTPUT:
      return 1 // These require exactly one input
    case PlanDagNodeType.MERGE:
      return 2 // Merge requires at least two inputs
    default:
      return 0
  }
}

/**
 * Gets the display color for a node type
 */
export const getNodeTypeColor = (nodeType: PlanDagNodeType): string => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return '#51cf66' // Green
    case PlanDagNodeType.GRAPH:
      return '#339af0' // Blue
    case PlanDagNodeType.TRANSFORM:
      return '#ff8cc8' // Pink
    case PlanDagNodeType.MERGE:
      return '#ffd43b' // Yellow
    case PlanDagNodeType.COPY:
      return '#74c0fc' // Light blue
    case PlanDagNodeType.OUTPUT:
      return '#ff6b6b' // Red
    default:
      return '#868e96' // Gray
  }
}

/**
 * Gets the display icon for a node type
 */
export const getNodeTypeIcon = (nodeType: PlanDagNodeType): string => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return 'import'
    case PlanDagNodeType.GRAPH:
      return 'sitemap'
    case PlanDagNodeType.TRANSFORM:
      return 'transform'
    case PlanDagNodeType.MERGE:
      return 'merge'
    case PlanDagNodeType.COPY:
      return 'copy'
    case PlanDagNodeType.OUTPUT:
      return 'export'
    default:
      return 'box'
  }
}