import { PlanDagNodeType } from '../types/plan-dag'
import {
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconFilter,
  IconGitMerge,
  IconCopy,
  IconFileExport,
} from '@tabler/icons-react'

/**
 * Operation categories for nodes with consistent color schemes
 */
export enum OperationCategory {
  INPUT = 'INPUT',       // Data Source
  GRAPH = 'GRAPH',       // Graph operations
  OPERATION = 'OPERATION', // Transform, Merge, Copy
  OUTPUT = 'OUTPUT',     // Output
}

/**
 * Clean, professional color scheme for node operation types
 */
export const OPERATION_COLORS = {
  [OperationCategory.INPUT]: '#10b981',    // Emerald-500 - Fresh green for data input
  [OperationCategory.GRAPH]: '#3b82f6',    // Blue-500 - Classic blue for graph operations
  [OperationCategory.OPERATION]: '#8b5cf6', // Violet-500 - Purple for data operations
  [OperationCategory.OUTPUT]: '#f59e0b',   // Amber-500 - Warm amber for output
} as const

/**
 * Get the operation category for a node type
 */
export const getOperationCategory = (nodeType: PlanDagNodeType): OperationCategory => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return OperationCategory.INPUT
    case PlanDagNodeType.GRAPH:
      return OperationCategory.GRAPH
    case PlanDagNodeType.TRANSFORM:
    case PlanDagNodeType.FILTER:
    case PlanDagNodeType.MERGE:
    case PlanDagNodeType.COPY:
      return OperationCategory.OPERATION
    case PlanDagNodeType.OUTPUT:
      return OperationCategory.OUTPUT
    default:
      return OperationCategory.OPERATION
  }
}

/**
 * Get the color for a node type based on its operation category
 */
export const getNodeColor = (nodeType: PlanDagNodeType): string => {
  const category = getOperationCategory(nodeType)
  return OPERATION_COLORS[category]
}

/**
 * Get the icon element for a node type
 */
export const getNodeIcon = (nodeType: PlanDagNodeType, size: string | number = '1.2rem') => {
  const iconProps = { size, stroke: 1.5 }

  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return <IconDatabase {...iconProps} />
    case PlanDagNodeType.GRAPH:
      return <IconNetwork {...iconProps} />
    case PlanDagNodeType.TRANSFORM:
      return <IconTransform {...iconProps} />
    case PlanDagNodeType.FILTER:
      return <IconFilter {...iconProps} />
    case PlanDagNodeType.MERGE:
      return <IconGitMerge {...iconProps} />
    case PlanDagNodeType.COPY:
      return <IconCopy {...iconProps} />
    case PlanDagNodeType.OUTPUT:
      return <IconFileExport {...iconProps} />
    default:
      return <IconNetwork {...iconProps} />
  }
}

/**
 * Get the display label for a node type
 */
export const getNodeTypeLabel = (nodeType: PlanDagNodeType): string => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return 'Data Source'
    case PlanDagNodeType.GRAPH:
      return 'Graph'
    case PlanDagNodeType.TRANSFORM:
      return 'Transform'
    case PlanDagNodeType.FILTER:
      return 'Filter'
    case PlanDagNodeType.MERGE:
      return 'Merge'
    case PlanDagNodeType.COPY:
      return 'Copy'
    case PlanDagNodeType.OUTPUT:
      return 'Output'
    default:
      return 'Unknown'
  }
}
