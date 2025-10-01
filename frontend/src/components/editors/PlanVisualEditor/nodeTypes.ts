import { DataSourceNode } from './nodes/DataSourceNode'
import { GraphNode } from './nodes/GraphNode'
import { TransformNode } from './nodes/TransformNode'
import { MergeNode } from './nodes/MergeNode'
import { CopyNode } from './nodes/CopyNode'
import { OutputNode } from './nodes/OutputNode'

/**
 * Stable nodeTypes mapping for ReactFlow
 * Defined in separate file to prevent recreation during hot module replacement
 */
export const NODE_TYPES = {
  DataSourceNode: DataSourceNode,
  GraphNode: GraphNode,
  TransformNode: TransformNode,
  MergeNode: MergeNode,
  CopyNode: CopyNode,
  OutputNode: OutputNode,
} as const
