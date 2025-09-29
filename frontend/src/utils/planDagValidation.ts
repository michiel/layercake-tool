import { PlanDagNodeType, ConnectionType } from '../types/plan-dag'
import { Node, Edge } from 'reactflow'

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
      PlanDagNodeType.GRAPH,       // Graphs can connect to other graphs
      PlanDagNodeType.TRANSFORM,
      PlanDagNodeType.COPY,
      PlanDagNodeType.OUTPUT,
    ],
    [PlanDagNodeType.TRANSFORM]: [
      PlanDagNodeType.GRAPH,       // Transforms can connect to graphs
      PlanDagNodeType.MERGE,
      PlanDagNodeType.COPY,
      PlanDagNodeType.OUTPUT,
      PlanDagNodeType.TRANSFORM, // Allow chaining transforms
    ],
    [PlanDagNodeType.MERGE]: [
      PlanDagNodeType.GRAPH,       // Merges can connect to graphs
      PlanDagNodeType.TRANSFORM,
      PlanDagNodeType.COPY,
      PlanDagNodeType.OUTPUT,
    ],
    [PlanDagNodeType.COPY]: [
      PlanDagNodeType.GRAPH,       // Copies can connect to graphs
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
    case PlanDagNodeType.GRAPH:     // GraphNodes can have multiple inputs (spec requirement)
    case PlanDagNodeType.MERGE:     // MergeNodes merge multiple DataSource/Graph inputs
      return true
    case PlanDagNodeType.DATA_SOURCE: // DataSource nodes cannot have inputs
    case PlanDagNodeType.TRANSFORM:   // TransformNodes can have only one input
    case PlanDagNodeType.COPY:        // CopyNodes can have only one input
    case PlanDagNodeType.OUTPUT:      // OutputNodes can have only one input
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
    case PlanDagNodeType.DATA_SOURCE: // DataSource can have multiple outputs (but not to same target)
    case PlanDagNodeType.GRAPH:       // GraphNodes can have multiple outputs
    case PlanDagNodeType.TRANSFORM:   // TransformNodes can have multiple outputs (but not to same target)
      return true
    case PlanDagNodeType.MERGE:       // MergeNodes output one new GraphNode
    case PlanDagNodeType.COPY:        // CopyNodes output graphs (spec unclear, assuming single)
    case PlanDagNodeType.OUTPUT:      // OutputNodes have no outputs
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
      return 0 // DataSource nodes are pure source nodes
    case PlanDagNodeType.GRAPH:
      return 1 // Graph nodes can accept inputs from other nodes
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

/**
 * Detects if adding a new edge would create a cycle in the DAG
 */
export const wouldCreateCycle = (
  nodes: Node[],
  edges: Edge[],
  newEdge: { source: string; target: string }
): boolean => {
  // Create a temporary edge list including the new edge
  const edgesWithNew = [...edges, {
    id: 'temp',
    source: newEdge.source,
    target: newEdge.target
  }];

  return hasCycle(nodes, edgesWithNew);
};

/**
 * Checks if the graph contains any cycles using DFS
 */
export const hasCycle = (nodes: Node[], edges: Edge[]): boolean => {
  const nodeIds = nodes.map(node => node.id);
  const adjList = createAdjacencyList(nodeIds, edges);

  const visited = new Set<string>();
  const recursionStack = new Set<string>();

  // Check each node as a potential starting point
  for (const nodeId of nodeIds) {
    if (!visited.has(nodeId)) {
      if (hasCycleDFS(nodeId, adjList, visited, recursionStack)) {
        return true;
      }
    }
  }

  return false;
};

/**
 * DFS helper function for cycle detection
 */
const hasCycleDFS = (
  nodeId: string,
  adjList: Map<string, string[]>,
  visited: Set<string>,
  recursionStack: Set<string>
): boolean => {
  visited.add(nodeId);
  recursionStack.add(nodeId);

  const neighbors = adjList.get(nodeId) || [];
  for (const neighbor of neighbors) {
    if (!visited.has(neighbor)) {
      if (hasCycleDFS(neighbor, adjList, visited, recursionStack)) {
        return true;
      }
    } else if (recursionStack.has(neighbor)) {
      // Back edge found - cycle detected
      return true;
    }
  }

  recursionStack.delete(nodeId);
  return false;
};

/**
 * Creates an adjacency list representation of the graph
 */
const createAdjacencyList = (nodeIds: string[], edges: Edge[]): Map<string, string[]> => {
  const adjList = new Map<string, string[]>();

  // Initialize adjacency list for all nodes
  for (const nodeId of nodeIds) {
    adjList.set(nodeId, []);
  }

  // Add edges to adjacency list
  for (const edge of edges) {
    const sourceNeighbors = adjList.get(edge.source) || [];
    sourceNeighbors.push(edge.target);
    adjList.set(edge.source, sourceNeighbors);
  }

  return adjList;
};

/**
 * Validates a connection including cycle detection
 */
export const validateConnectionWithCycleDetection = (
  sourceType: PlanDagNodeType,
  targetType: PlanDagNodeType,
  nodes: Node[],
  edges: Edge[],
  newConnection: { source: string; target: string }
): ConnectionType & { wouldCreateCycle?: boolean } => {
  // First check basic connection validity
  const basicValidation = validateConnection(sourceType, targetType);

  if (!basicValidation.isValid) {
    return basicValidation;
  }

  // Check for cycle creation
  const cycleDetected = wouldCreateCycle(nodes, edges, newConnection);

  if (cycleDetected) {
    return {
      ...basicValidation,
      isValid: false,
      wouldCreateCycle: true,
      errorMessage: 'This connection would create a cycle, which is not allowed in a DAG'
    };
  }

  return {
    ...basicValidation,
    wouldCreateCycle: false
  };
};

/**
 * Node configuration state validation functions (per SPECIFICATION.md)
 */

/**
 * Checks if a node is in CONFIGURED state based on connection requirements
 */
export const isNodeConfigured = (
  nodeType: PlanDagNodeType,
  nodeId: string,
  edges: Edge[],
  hasValidConfig: boolean = true
): boolean => {
  if (!hasValidConfig) {
    return false; // Node-specific configuration must pass validation
  }

  const inputEdges = edges.filter(edge => edge.target === nodeId);
  const outputEdges = edges.filter(edge => edge.source === nodeId);

  switch (nodeType) {
    case PlanDagNodeType.GRAPH:
      // GraphNodes MUST have at least one input to be configured
      return inputEdges.length >= 1;

    case PlanDagNodeType.DATA_SOURCE:
      // DataSource nodes MUST have at least one output connected to be configured
      return outputEdges.length >= 1;

    case PlanDagNodeType.TRANSFORM:
      // TransformNodes MUST have one input and one output to be configured
      return inputEdges.length === 1 && outputEdges.length >= 1;

    case PlanDagNodeType.OUTPUT:
      // OutputNodes MUST have one input to be configured
      return inputEdges.length === 1;

    case PlanDagNodeType.MERGE:
      // MergeNodes need at least 2 inputs to merge (assuming this requirement)
      return inputEdges.length >= 2;

    case PlanDagNodeType.COPY:
      // CopyNodes need 1 input and 1 output (assuming similar to Transform)
      return inputEdges.length === 1 && outputEdges.length >= 1;

    default:
      return false;
  }
};

/**
 * Validates that a node doesn't connect to the same target twice
 * (Required for DataSource and Transform nodes per spec)
 */
export const validateUniqueTargets = (
  nodeId: string,
  edges: Edge[]
): boolean => {
  const outputEdges = edges.filter(edge => edge.source === nodeId);
  const targets = outputEdges.map(edge => edge.target);
  const uniqueTargets = new Set(targets);

  return targets.length === uniqueTargets.size; // No duplicate targets
};

/**
 * Gets minimum required inputs for a node type to be configured
 */
export const getMinimumRequiredInputs = (nodeType: PlanDagNodeType): number => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return 0; // DataSource nodes don't need inputs
    case PlanDagNodeType.GRAPH:
      return 1; // GraphNodes MUST have at least one input
    case PlanDagNodeType.TRANSFORM:
    case PlanDagNodeType.OUTPUT:
      return 1; // These need exactly one input
    case PlanDagNodeType.MERGE:
      return 2; // Merge needs at least two inputs
    case PlanDagNodeType.COPY:
      return 1; // Copy needs one input
    default:
      return 0;
  }
};

/**
 * Gets minimum required outputs for a node type to be configured
 */
export const getMinimumRequiredOutputs = (nodeType: PlanDagNodeType): number => {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
    case PlanDagNodeType.TRANSFORM:
    case PlanDagNodeType.COPY:
      return 1; // These must have at least one output
    case PlanDagNodeType.GRAPH:
    case PlanDagNodeType.MERGE:
      return 0; // These can exist without outputs (terminal nodes)
    case PlanDagNodeType.OUTPUT:
      return 0; // Output nodes have no outputs
    default:
      return 0;
  }
};