import { DataSourceNode } from './nodes/DataSourceNode'
import { GraphNode } from './nodes/GraphNode'
import { TransformNode } from './nodes/TransformNode'
import { FilterNode } from './nodes/FilterNode'
import { MergeNode } from './nodes/MergeNode'
import { OutputNode } from './nodes/OutputNode'

/**
 * Stable nodeTypes mapping for ReactFlow
 * Defined in separate file to prevent recreation during hot module replacement
 */
export const NODE_TYPES = {
  DataSourceNode: DataSourceNode,
  GraphNode: GraphNode,
  TransformNode: TransformNode,
  FilterNode: FilterNode,
  MergeNode: MergeNode,
  OutputNode: OutputNode,
} as const
