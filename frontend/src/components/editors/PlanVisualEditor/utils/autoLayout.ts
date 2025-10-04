import { Node, Edge } from 'reactflow';

export interface LayoutOptions {
  direction: 'horizontal' | 'vertical';
  nodeSpacing: number;
  rankSpacing: number;
}

interface LayoutNode {
  id: string;
  node: Node;
  rank: number;
  children: string[];
  parents: string[];
}

/**
 * Build a graph structure from nodes and edges
 */
function buildGraph(nodes: Node[], edges: Edge[]): Map<string, LayoutNode> {
  const graph = new Map<string, LayoutNode>();

  // Initialize all nodes
  nodes.forEach(node => {
    graph.set(node.id, {
      id: node.id,
      node,
      rank: -1,
      children: [],
      parents: []
    });
  });

  // Build parent-child relationships
  edges.forEach(edge => {
    const source = graph.get(edge.source);
    const target = graph.get(edge.target);

    if (source && target) {
      source.children.push(edge.target);
      target.parents.push(edge.source);
    }
  });

  return graph;
}

/**
 * Assign ranks to nodes using topological sort (Kahn's algorithm)
 */
function assignRanks(graph: Map<string, LayoutNode>): void {
  // Find all nodes with no parents (root nodes)
  const roots: string[] = [];
  const inDegree = new Map<string, number>();

  graph.forEach((layoutNode, id) => {
    inDegree.set(id, layoutNode.parents.length);
    if (layoutNode.parents.length === 0) {
      roots.push(id);
    }
  });

  // BFS to assign ranks
  const queue = roots.map(id => ({ id, rank: 0 }));

  while (queue.length > 0) {
    const { id, rank } = queue.shift()!;
    const layoutNode = graph.get(id)!;

    layoutNode.rank = rank;

    // Process children
    layoutNode.children.forEach(childId => {
      const currentInDegree = inDegree.get(childId)! - 1;
      inDegree.set(childId, currentInDegree);

      if (currentInDegree === 0) {
        queue.push({ id: childId, rank: rank + 1 });
      }
    });
  }

  // Handle cycles: nodes with rank -1 are part of a cycle
  // Assign them to the next rank after the last processed
  const maxRank = Math.max(...Array.from(graph.values()).map(n => n.rank));
  graph.forEach(layoutNode => {
    if (layoutNode.rank === -1) {
      layoutNode.rank = maxRank + 1;
    }
  });
}

/**
 * Calculate layout positions for horizontal direction (left to right)
 */
function layoutHorizontal(
  graph: Map<string, LayoutNode>,
  options: LayoutOptions
): Node[] {
  // Group nodes by rank
  const ranks = new Map<number, LayoutNode[]>();
  graph.forEach(layoutNode => {
    const rank = layoutNode.rank;
    if (!ranks.has(rank)) {
      ranks.set(rank, []);
    }
    ranks.get(rank)!.push(layoutNode);
  });

  const updatedNodes: Node[] = [];
  const nodeWidth = 200; // Approximate node width
  const nodeHeight = 100; // Approximate node height

  // Position nodes rank by rank
  ranks.forEach((nodesInRank, rank) => {
    const x = rank * (nodeWidth + options.rankSpacing);
    const totalHeight = nodesInRank.length * nodeHeight + (nodesInRank.length - 1) * options.nodeSpacing;
    const startY = -totalHeight / 2;

    nodesInRank.forEach((layoutNode, index) => {
      const y = startY + index * (nodeHeight + options.nodeSpacing);

      updatedNodes.push({
        ...layoutNode.node,
        position: { x, y }
      });
    });
  });

  return updatedNodes;
}

/**
 * Calculate layout positions for vertical direction (top to bottom)
 */
function layoutVertical(
  graph: Map<string, LayoutNode>,
  options: LayoutOptions
): Node[] {
  // Group nodes by rank
  const ranks = new Map<number, LayoutNode[]>();
  graph.forEach(layoutNode => {
    const rank = layoutNode.rank;
    if (!ranks.has(rank)) {
      ranks.set(rank, []);
    }
    ranks.get(rank)!.push(layoutNode);
  });

  const updatedNodes: Node[] = [];
  const nodeWidth = 200; // Approximate node width
  const nodeHeight = 100; // Approximate node height

  // Position nodes rank by rank
  ranks.forEach((nodesInRank, rank) => {
    const y = rank * (nodeHeight + options.rankSpacing);
    const totalWidth = nodesInRank.length * nodeWidth + (nodesInRank.length - 1) * options.nodeSpacing;
    const startX = -totalWidth / 2;

    nodesInRank.forEach((layoutNode, index) => {
      const x = startX + index * (nodeWidth + options.nodeSpacing);

      updatedNodes.push({
        ...layoutNode.node,
        position: { x, y }
      });
    });
  });

  return updatedNodes;
}

/**
 * Auto-layout nodes in a hierarchical structure
 */
export function autoLayout(
  nodes: Node[],
  edges: Edge[],
  options: LayoutOptions
): Node[] {
  if (nodes.length === 0) {
    return nodes;
  }

  // Build graph structure
  const graph = buildGraph(nodes, edges);

  // Assign ranks using topological sort
  assignRanks(graph);

  // Calculate positions based on direction
  if (options.direction === 'horizontal') {
    return layoutHorizontal(graph, options);
  } else {
    return layoutVertical(graph, options);
  }
}
