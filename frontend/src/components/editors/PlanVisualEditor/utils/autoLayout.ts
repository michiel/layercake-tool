import { Node, Edge } from 'reactflow';
import ELK from 'elkjs/lib/elk.bundled.js';

const elk = new ELK();

export interface LayoutOptions {
  direction: 'horizontal' | 'vertical';
  nodeSpacing: number;
  rankSpacing: number;
  nodeWidth?: number;
  nodeHeight?: number;
}

/**
 * Auto-layout nodes using ELK (Eclipse Layout Kernel)
 * Based on ReactFlow's elkjs example
 */
export async function autoLayout(
  nodes: Node[],
  edges: Edge[],
  options: LayoutOptions
): Promise<Node[]> {
  if (nodes.length === 0) {
    return nodes;
  }

  const isHorizontal = options.direction === 'horizontal';
  const nodeWidth = options.nodeWidth || 200;
  const nodeHeight = options.nodeHeight || 100;

  // Configure ELK options
  const elkOptions = {
    'elk.algorithm': 'layered',
    'elk.direction': isHorizontal ? 'RIGHT' : 'DOWN',
    'elk.spacing.nodeNode': String(options.nodeSpacing),
    'elk.layered.spacing.nodeNodeBetweenLayers': String(options.rankSpacing),
    'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
    'elk.layered.nodePlacement.strategy': 'SIMPLE',
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

    // Map layouted positions back to nodes
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

    return layoutedNodes;
  } catch (error) {
    console.error('ELK layout failed:', error);
    // Return original nodes on error
    return nodes;
  }
}
