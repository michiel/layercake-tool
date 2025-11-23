import { DataSetNode } from './nodes/DataSetNode'
import { GraphNode } from './nodes/GraphNode'
import { TransformNode } from './nodes/TransformNode'
import { FilterNode } from './nodes/FilterNode'
import { MergeNode } from './nodes/MergeNode'
import { GraphArtefactNode, TreeArtefactNode } from './nodes/GraphArtefactNode'
import { StoryNode } from './nodes/StoryNode'
import { SequenceArtefactNode } from './nodes/SequenceArtefactNode'

/**
 * Stable nodeTypes mapping for ReactFlow
 * Defined in separate file to prevent recreation during hot module replacement
 */
export const NODE_TYPES = {
  DataSetNode: DataSetNode,
  GraphNode: GraphNode,
  TransformNode: TransformNode,
  FilterNode: FilterNode,
  MergeNode: MergeNode,
  GraphArtefactNode: GraphArtefactNode,
  TreeArtefactNode: TreeArtefactNode,
  StoryNode: StoryNode,
  SequenceArtefactNode: SequenceArtefactNode,
} as const
