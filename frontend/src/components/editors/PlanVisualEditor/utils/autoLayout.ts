import { Node, Edge } from 'reactflow';
import ELK from 'elkjs/lib/elk.bundled.js';

const elk = new ELK();

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
 * Auto-layout nodes and edges using ELK (Eclipse Layout Kernel)
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

  // Configure ELK options for clean, readable layouts
  const elkOptions = {
    'elk.algorithm': 'layered',
    'elk.direction': isHorizontal ? 'RIGHT' : 'DOWN',

    // Spacing configuration
    'elk.spacing.nodeNode': String(nodeSpacing),
    'elk.layered.spacing.nodeNodeBetweenLayers': String(rankSpacing),
    'elk.spacing.edgeNode': String(Math.min(nodeSpacing, rankSpacing) * 0.5),
    'elk.spacing.edgeEdge': '30',

    // Layout strategy for better visual organization
    'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
    'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
    'elk.layered.cycleBreaking.strategy': 'GREEDY',

    // Edge routing for cleaner connections
    'elk.edgeRouting': 'ORTHOGONAL',
    'elk.layered.unnecessaryBendpoints': 'true',

    // Hierarchical layout for better organization
    'elk.hierarchyHandling': 'INCLUDE_CHILDREN',

    // Padding around the layout
    'elk.padding': '[top=20,left=20,bottom=20,right=20]',
  };

  // Build ELK graph structure
  const graph = {
    id: 'root',
    layoutOptions: elkOptions,
    children: nodes.map((node) => ({
      id: node.id,
      width: nodeWidth,
      height: nodeHeight,
    })),
    edges: edges.map((edge) => ({
      id: edge.id,
      sources: [edge.source],
      targets: [edge.target],
    })),
  };

  try {
    // Perform layout
    const layoutedGraph = await elk.layout(graph);

    // Map layouted positions back to nodes with proper handle positions
    const layoutedNodes = layoutedGraph.children?.map((layoutedNode) => {
      const originalNode = nodes.find((n) => n.id === layoutedNode.id);
      if (!originalNode) return null;

      return {
        ...originalNode,
        position: {
          x: layoutedNode.x ?? 0,
          y: layoutedNode.y ?? 0,
        },
        sourcePosition: isHorizontal ? 'right' : 'bottom',
        targetPosition: isHorizontal ? 'left' : 'top',
      } as Node;
    }).filter((node): node is Node => node !== null) ?? [];

    // Update edges to use correct handles based on layout direction
    const layoutedEdges = edges.map((edge) => ({
      ...edge,
      sourceHandle: isHorizontal ? 'output-right' : 'output-bottom',
      targetHandle: isHorizontal ? 'input-left' : 'input-top',
    }));

    return { nodes: layoutedNodes, edges: layoutedEdges };
  } catch (error) {
    console.error('ELK layout failed:', error);
    // Return original nodes and edges on error
    return { nodes, edges };
  }
}
