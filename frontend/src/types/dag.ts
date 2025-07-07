// DAG-related types for the frontend

export interface PlanNode {
  id: string;
  planId: number;
  nodeType: string;
  name: string;
  description?: string | null;
  configuration: string; // JSON configuration
  graphId?: string | null;
  positionX?: number | null;
  positionY?: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface DagEdge {
  source: string;
  target: string;
}

export interface DagPlan {
  nodes: PlanNode[];
  edges: DagEdge[];
}

export interface GraphArtifact {
  id: string;
  planId: number;
  planNodeId: string;
  name: string;
  description?: string | null;
  graphData: string; // JSON graph data
  metadata?: string | null; // JSON metadata
  createdAt: string;
  updatedAt: string;
}

export interface GraphStatistics {
  node_count: number;
  edge_count: number;
  layer_count: number;
  nodes_per_layer: LayerNodeCount[];
  edges_per_layer: LayerEdgeCount[];
  connected_components: number;
  density: number;
}

export interface LayerNodeCount {
  layer: string;
  count: number;
}

export interface LayerEdgeCount {
  layer: string;
  count: number;
}

export interface GraphValidationResult {
  is_valid: boolean;
  errors: string[];
  warnings: string[];
}

export interface GraphDiff {
  added_nodes: string[];
  removed_nodes: string[];
  added_edges: string[];
  removed_edges: string[];
}

// Input types for mutations
export interface CreatePlanNodeInput {
  planId: number;
  nodeType: string;
  name: string;
  description?: string | null;
  configuration: string;
  positionX?: number | null;
  positionY?: number | null;
}

export interface UpdatePlanNodeInput {
  name?: string;
  description?: string | null;
  configuration?: string;
  positionX?: number | null;
  positionY?: number | null;
}

// Node type definitions
export type NodeType = 'input' | 'transform' | 'output' | 'merge' | 'split';

export interface NodeTypeConfig {
  label: string;
  icon: string;
  color: string;
  description: string;
}

export const NODE_TYPE_CONFIGS: Record<NodeType, NodeTypeConfig> = {
  input: {
    label: 'Input Node',
    icon: '📥',
    color: 'green',
    description: 'Loads data from external sources',
  },
  transform: {
    label: 'Transform Node',
    icon: '🔄',
    color: 'blue',
    description: 'Applies transformations to graph data',
  },
  output: {
    label: 'Output Node',
    icon: '📤',
    color: 'red',
    description: 'Exports or saves processed data',
  },
  merge: {
    label: 'Merge Node',
    icon: '🔗',
    color: 'yellow',
    description: 'Combines multiple graph inputs',
  },
  split: {
    label: 'Split Node',
    icon: '🔀',
    color: 'purple',
    description: 'Splits graph into multiple outputs',
  },
};