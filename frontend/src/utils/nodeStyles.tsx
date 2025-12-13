import { PlanDagNodeType } from '../types/plan-dag'
import {
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconFilter,
  IconGitMerge,
  IconFileExport,
  IconBook,
  IconTimeline,
  IconPresentation,
} from '@tabler/icons-react'

/**
 * Operation categories for nodes with consistent color schemes
 */
export enum OperationCategory {
  INPUT = 'INPUT',       // Data Source
  GRAPH = 'GRAPH',       // Graph operations
  OPERATION = 'OPERATION', // Transform, Filter, Merge
  OUTPUT = 'OUTPUT',     // Artefact nodes
  STORY = 'STORY',       // Story nodes
  PROJECTION = 'PROJECTION', // Projection nodes
}

/**
 * Clean, professional color scheme for node operation types
 */
export const OPERATION_COLORS = {
  [OperationCategory.INPUT]: '#10b981',    // Emerald-500 - Fresh green for data input
  [OperationCategory.GRAPH]: '#3b82f6',    // Blue-500 - Classic blue for graph operations
  [OperationCategory.OPERATION]: '#8b5cf6', // Violet-500 - Purple for data operations
  [OperationCategory.OUTPUT]: '#f59e0b',   // Amber-500 - Warm amber for output
  [OperationCategory.STORY]: '#3b82f6',    // Blue-500 - Blue for story nodes
  [OperationCategory.PROJECTION]: '#f97316', // Orange-500 - Projection nodes
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
      return OperationCategory.OPERATION
    case PlanDagNodeType.GRAPH_ARTEFACT:
    case PlanDagNodeType.TREE_ARTEFACT:
    case PlanDagNodeType.SEQUENCE_ARTEFACT:
      return OperationCategory.OUTPUT
    case PlanDagNodeType.STORY:
      return OperationCategory.STORY
    case PlanDagNodeType.PROJECTION:
      return OperationCategory.PROJECTION
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
    case PlanDagNodeType.GRAPH_ARTEFACT:
    case PlanDagNodeType.TREE_ARTEFACT:
      return <IconFileExport {...iconProps} />
    case PlanDagNodeType.STORY:
      return <IconBook {...iconProps} />
    case PlanDagNodeType.SEQUENCE_ARTEFACT:
      return <IconTimeline {...iconProps} />
    case PlanDagNodeType.PROJECTION:
      return <IconPresentation {...iconProps} />
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
    case PlanDagNodeType.GRAPH_ARTEFACT:
      return 'Graph Artefact'
    case PlanDagNodeType.TREE_ARTEFACT:
      return 'Tree Artefact'
    case PlanDagNodeType.PROJECTION:
      return 'Projection'
    case PlanDagNodeType.STORY:
      return 'Story'
    case PlanDagNodeType.SEQUENCE_ARTEFACT:
      return 'Sequence Artefact'
    default:
      return 'Unknown'
  }
}
