import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs/lib/elk.bundled.js';
import { Node, Edge, MarkerType } from 'reactflow';
import { Graph, GraphNode, Layer } from '../graphql/graphs';

// Default colors for nodes and groups
const DEFAULT_NODE_BG = '#ffffff';
const DEFAULT_NODE_BORDER = '#eee';
const DEFAULT_NODE_TEXT = '#000000';
const DEFAULT_GROUP_BG = 'rgba(240, 240, 240, 0.5)';
const DEFAULT_GROUP_BORDER = '#999';

// Helper to get layer styling
const getLayerStyle = (layerId: string | undefined, layerMap: Map<string, Layer>) => {
  if (!layerId) return null;
  const layer = layerMap.get(layerId);
  if (!layer) return null;

  return {
    backgroundColor: layer.backgroundColor ? `#${layer.backgroundColor}` : null,
    borderColor: layer.borderColor ? `#${layer.borderColor}` : null,
    textColor: layer.textColor ? `#${layer.textColor}` : null,
  };
};

export interface ElkLayoutOptions {
  orientation?: 'vertical' | 'horizontal';
  nodeSpacing?: number;
  rankSpacing?: number;
}

const elk = new ELK();

/**
 * Layout graph using ELK.js for superior hierarchical graph support
 */
export const getElkLayoutedElements = async (
  lcGraph: Graph,
  layers: Layer[],
  nodeWidth: number = 170,
  nodeHeight: number = 50,
  options: ElkLayoutOptions = {}
): Promise<{ nodes: Node[]; edges: Edge[] }> => {
  const orientation = options.orientation ?? 'vertical';
  const nodeSpacing = options.nodeSpacing ?? 40;
  const rankSpacing = options.rankSpacing ?? 50;

  // Create lookup maps
  const nodeMap = new Map<string, GraphNode>();
  lcGraph.graphNodes.forEach(node => nodeMap.set(node.id, node));

  const layerMap = new Map<string, Layer>();
  layers.forEach(layer => layerMap.set(layer.layerId, layer));

  // Build ELK graph structure with hierarchy
  const buildElkNode = (graphNode: GraphNode, depth: number): ElkNode => {
    const children = lcGraph.graphNodes.filter(n => n.belongsTo === graphNode.id);
    const isGroup = graphNode.isPartition;

    const elkNode: ElkNode = {
      id: graphNode.id,
      width: isGroup ? undefined : nodeWidth,
      height: isGroup ? undefined : nodeHeight,
      labels: [{ text: graphNode.label || graphNode.id }],
    };

    if (isGroup && children.length > 0) {
      elkNode.children = children.map(child => buildElkNode(child, depth + 1));

      // Add edges between children
      const childEdges: ElkExtendedEdge[] = [];
      lcGraph.graphEdges.forEach(edge => {
        if (children.some(c => c.id === edge.source) && children.some(c => c.id === edge.target)) {
          childEdges.push({
            id: edge.id,
            sources: [edge.source],
            targets: [edge.target],
          });
        }
      });
      elkNode.edges = childEdges;

      // Layout options for compound nodes
      elkNode.layoutOptions = {
        'elk.padding': '[top=32,left=32,bottom=32,right=32]',
      };
    }

    return elkNode;
  };

  // Find root nodes
  const rootNodes = lcGraph.graphNodes.filter(n =>
    !n.belongsTo || !nodeMap.has(n.belongsTo)
  );

  // Build root ELK graph
  const elkGraph: ElkNode = {
    id: 'root',
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': orientation === 'horizontal' ? 'RIGHT' : 'DOWN',
      'elk.spacing.nodeNode': String(nodeSpacing),
      'elk.layered.spacing.nodeNodeBetweenLayers': String(rankSpacing),
      'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
      'elk.layered.nodePlacement.strategy': 'BRANDES_KOEPF',
      'elk.layered.spacing.edgeNodeBetweenLayers': '20',
      'elk.layered.spacing.edgeEdgeBetweenLayers': '15',
    },
    children: rootNodes.map(node => buildElkNode(node, 0)),
    edges: [],
  };

  // Add root-level edges
  lcGraph.graphEdges.forEach(edge => {
    if (rootNodes.some(n => n.id === edge.source) && rootNodes.some(n => n.id === edge.target)) {
      elkGraph.edges!.push({
        id: edge.id,
        sources: [edge.source],
        targets: [edge.target],
      });
    }
  });

  // Run ELK layout
  const layoutedGraph = await elk.layout(elkGraph);

  // Convert ELK result to React Flow nodes
  const reactFlowNodes: Node[] = [];
  const reactFlowEdges: Edge[] = [];

  // Calculate depth for z-index
  const depthMap = new Map<string, number>();
  const calculateDepth = (nodeId: string, depth: number = 0) => {
    depthMap.set(nodeId, depth);
    const node = nodeMap.get(nodeId);
    if (node?.isPartition) {
      lcGraph.graphNodes
        .filter(n => n.belongsTo === nodeId)
        .forEach(child => calculateDepth(child.id, depth + 1));
    }
  };
  rootNodes.forEach(n => calculateDepth(n.id));

  // Convert ELK nodes to React Flow nodes
  const convertElkNode = (elkNode: ElkNode, parentId?: string) => {
    const graphNode = nodeMap.get(elkNode.id);
    if (!graphNode) return;

    const depth = depthMap.get(elkNode.id) || 0;
    const layerStyle = getLayerStyle(graphNode.layer, layerMap);
    const isGroup = graphNode.isPartition;

    if (isGroup) {
      const borderColor = layerStyle?.borderColor || DEFAULT_GROUP_BORDER;
      const bgColor = layerStyle?.backgroundColor || DEFAULT_GROUP_BG;

      reactFlowNodes.push({
        id: graphNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: { label: graphNode.label || graphNode.id },
        type: 'group',
        width: elkNode.width,
        height: elkNode.height,
        style: {
          width: elkNode.width,
          height: elkNode.height,
          backgroundColor: bgColor,
          border: `2px solid ${borderColor}`,
          borderRadius: '8px',
          zIndex: -100 + depth,
        },
        className: 'layercake-group-node',
        ...(parentId ? { parentNode: parentId } : {}),
      });

      // Add label node
      reactFlowNodes.push({
        id: `${graphNode.id}-label`,
        type: 'labelNode',
        position: { x: 10, y: 6 },
        data: {
          label: graphNode.label || graphNode.id,
          style: { color: layerStyle?.textColor || '#666' },
        },
        draggable: false,
        selectable: false,
        connectable: false,
        style: {
          background: 'transparent',
          border: 'none',
          fontSize: '11px',
          fontWeight: '500',
          color: layerStyle?.textColor || '#666',
          padding: 0,
          zIndex: 100,
        },
        parentNode: graphNode.id,
      });

      // Process children
      if (elkNode.children) {
        elkNode.children.forEach(child => convertElkNode(child, graphNode.id));
      }
    } else {
      reactFlowNodes.push({
        id: graphNode.id,
        type: 'editable',
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: {
          label: graphNode.label || graphNode.id,
          style: {
            backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
            border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
            color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
          },
        },
        width: elkNode.width,
        height: elkNode.height,
        style: {
          zIndex: 50,
          backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
          border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
          color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
        },
        ...(parentId ? { parentNode: parentId } : {}),
      });
    }
  };

  // Convert all nodes
  if (layoutedGraph.children) {
    layoutedGraph.children.forEach(child => convertElkNode(child));
  }

  // Add all edges
  lcGraph.graphEdges.forEach(edge => {
    reactFlowEdges.push({
      id: edge.id,
      source: edge.source,
      target: edge.target,
      label: edge.label || '',
      type: 'floating',
      markerEnd: { type: MarkerType.ArrowClosed },
      style: {
        zIndex: 10,
        strokeWidth: 2,
        stroke: '#b1b1b7',
      },
    });
  });

  return { nodes: reactFlowNodes, edges: reactFlowEdges };
};
