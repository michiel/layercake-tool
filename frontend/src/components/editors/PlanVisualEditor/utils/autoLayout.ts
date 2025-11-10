import { Node, Edge } from 'reactflow';
import dagre from 'dagre';

export interface LayoutOptions {
  direction: 'horizontal' | 'vertical';
  nodeSpacing?: number;
  rankSpacing?: number;
  nodeWidth?: number;
  nodeHeight?: number;
}

// Default layout configuration with improved spacing
const DEFAULT_NODE_WIDTH = 200;
const DEFAULT_NODE_HEIGHT = 100;
const DEFAULT_HORIZONTAL_NODE_SPACING = 150;
const DEFAULT_HORIZONTAL_RANK_SPACING = 350;
const DEFAULT_VERTICAL_NODE_SPACING = 120;
const DEFAULT_VERTICAL_RANK_SPACING = 200;

/**
 * Auto-layout nodes and edges using Dagre (lighter alternative to ELK)
 * Optimized for visual clarity with generous spacing
 * Updates edge connectors based on layout direction
 */
export async function autoLayout(
  nodes: Node[],
  edges: Edge[],
  options: LayoutOptions
): Promise<{ nodes: Node[]; edges: Edge[] }> {
  if (nodes.length === 0) {
    return { nodes, edges };
  }

  const isHorizontal = options.direction === 'horizontal';

  // Use defaults with directional preferences
  const nodeWidth = options.nodeWidth || DEFAULT_NODE_WIDTH;
  const nodeHeight = options.nodeHeight || DEFAULT_NODE_HEIGHT;
  const nodeSpacing = options.nodeSpacing ||
    (isHorizontal ? DEFAULT_HORIZONTAL_NODE_SPACING : DEFAULT_VERTICAL_NODE_SPACING);
  const rankSpacing = options.rankSpacing ||
    (isHorizontal ? DEFAULT_HORIZONTAL_RANK_SPACING : DEFAULT_VERTICAL_RANK_SPACING);

  // Create a new directed graph
  const g = new dagre.graphlib.Graph();

  // Set graph options
  g.setGraph({
    rankdir: isHorizontal ? 'LR' : 'TB',
    nodesep: nodeSpacing,
    ranksep: rankSpacing,
    edgesep: 30,
    marginx: 20,
    marginy: 20,
  });

  // Default edge label config
  g.setDefaultEdgeLabel(() => ({}));

  // Add nodes to the graph
  nodes.forEach((node) => {
    g.setNode(node.id, {
      width: nodeWidth,
      height: nodeHeight,
    });
  });

  // Add edges to the graph
  edges.forEach((edge) => {
    g.setEdge(edge.source, edge.target);
  });

  try {
    // Perform layout
    dagre.layout(g);

    // Map layouted positions back to nodes with proper handle positions
    const layoutedNodes = nodes.map((node) => {
      const nodeWithPosition = g.node(node.id);

      if (!nodeWithPosition) {
        return node;
      }

      return {
        ...node,
        position: {
          // Dagre gives us the center of the node, we need top-left
          x: nodeWithPosition.x - nodeWidth / 2,
          y: nodeWithPosition.y - nodeHeight / 2,
        },
        sourcePosition: isHorizontal ? 'right' : 'bottom',
        targetPosition: isHorizontal ? 'left' : 'top',
      } as Node;
    });

    return { nodes: layoutedNodes, edges };
  } catch (error) {
    console.error('Dagre layout failed:', error);
    // Return original nodes and edges on error
    return { nodes, edges };
  }
}
