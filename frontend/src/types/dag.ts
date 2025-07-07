// DAG-related types for the frontend

export interface PlanNode {
  id: string;
  plan_id: number;
  node_type: string;
  name: string;
  description?: string | null;
  configuration: string; // JSON configuration
  graph_id?: string | null;
  position_x?: number | null;
  position_y?: number | null;
  created_at: string;
  updated_at: string;
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
  plan_id: number;
  plan_node_id: string;
  name: string;
  description?: string | null;
  graph_data: string; // JSON graph data
  metadata?: string | null; // JSON metadata
  created_at: string;
  updated_at: string;
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
  plan_id: number;
  node_type: string;
  name: string;
  description?: string | null;
  configuration: string;
  position_x?: number | null;
  position_y?: number | null;
}

export interface UpdatePlanNodeInput {
  name?: string;
  description?: string | null;
  configuration?: string;
  position_x?: number | null;
  position_y?: number | null;
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