import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs';
import { Node, Edge, MarkerType } from 'reactflow';
import { Graph, GraphNode, Layer } from '../graphql/graphs';

const elk = new ELK();

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

const elkOptions = {
  'elk.algorithm': 'layered',
  'elk.spacing.nodeNode': '75',
  'elk.spacing.nodeNodeBetweenLayers': '75',
  'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
  'elk.layered.mergeEdges': 'true',
  'elk.layered.feedbackEdges': 'true',
  'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
  'elk.layered.cycleBreaking.strategy': 'DEPTH_FIRST',
};

interface LayoutOptions {
  disableSubflows?: boolean;
  orientation?: 'vertical' | 'horizontal';
  nodeSpacing?: number;
  rankSpacing?: number;
  minEdgeLength?: number;
}

// Function to convert LcGraph to React Flow elements
export const getLayoutedElements = async (
  lcGraph: Graph,
  layers: Layer[],
  nodeWidth: number = 170,
  nodeHeight: number = 50,
  options: LayoutOptions = {}
) => {
  const disableSubflows = options.disableSubflows === true;
  const orientation = options.orientation ?? 'vertical';
  const layoutDirection = orientation === 'horizontal' ? 'RIGHT' : 'DOWN';
  const nodeSpacing = options.nodeSpacing ?? 75;
  const rankSpacing = options.rankSpacing ?? 75;
  const minEdgeLength = options.minEdgeLength ?? 50;
  // Create node lookup map
  const nodeMap = new Map<string, GraphNode>();
  lcGraph.graphNodes.forEach(node => nodeMap.set(node.id, node));

  // Create layer lookup map by layerId
  const layerMap = new Map<string, Layer>();
  layers.forEach(layer => layerMap.set(layer.layerId, layer));

  // Build ELK graph structure recursively
  const buildElkNode = (nodeId: string): ElkNode | null => {
    const node = nodeMap.get(nodeId);
    if (!node) return null;

    if (node.isPartition) {
      // This is a subflow - find children
      const children = lcGraph.graphNodes.filter(n => n.belongsTo === nodeId);

      const elkNode: ElkNode = {
        id: node.id,
        labels: [{ text: node.label || node.id }],
        layoutOptions: {
          'elk.padding': '[top=40,left=20,bottom=20,right=20]',
          'elk.direction': layoutDirection,
          'elk.spacing.nodeNode': String(nodeSpacing),
          'elk.spacing.nodeNodeBetweenLayers': String(rankSpacing),
        },
        children: [],
        edges: [],
      };

      // Add children recursively
      children.forEach(child => {
        const childElk = buildElkNode(child.id);
        if (childElk) {
          elkNode.children?.push(childElk);
        }
      });

      return elkNode;
    } else {
      // Regular node
      return {
        id: node.id,
        width: nodeWidth,
        height: nodeHeight,
        labels: [{ text: node.label || node.id }],
      };
    }
  };

  // Find root nodes (no belongsTo or belongsTo references non-existent node)
  const rootNodes = lcGraph.graphNodes.filter(n =>
    !n.belongsTo || !nodeMap.has(n.belongsTo)
  );

  const graph: ElkNode = {
    id: 'root',
    layoutOptions: {
      ...elkOptions,
      'elk.direction': layoutDirection,
      'elk.spacing.nodeNode': String(nodeSpacing),
      'elk.spacing.nodeNodeBetweenLayers': String(rankSpacing),
      'elk.layered.spacing.edgeNodeBetweenLayers': String(minEdgeLength),
      ...(disableSubflows ? { 'elk.hierarchyHandling': 'INCLUDE_CHILDREN' } : {}),
    },
    children: [],
    edges: [],
  };

  if (disableSubflows) {
    graph.children = lcGraph.graphNodes.map(node => ({
      id: node.id,
      width: nodeWidth,
      height: nodeHeight,
      labels: [{ text: node.label || node.id }],
    }));
  } else {
    rootNodes.forEach(rootNode => {
      const elkNode = buildElkNode(rootNode.id);
      if (elkNode) {
        graph.children?.push(elkNode);
      }
    });
  }

  // Add edges
  lcGraph.graphEdges.forEach(edge => {
    graph.edges?.push({
      id: edge.id,
      sources: [edge.source],
      targets: [edge.target],
      labels: [{ text: edge.label || '' }],
    });
  });

  const elkGraph = await elk.layout(graph);

  const reactFlowNodes: Node[] = [];
  const reactFlowEdges: Edge[] = [];

  // Calculate depth for z-index (deeper = higher z-index)
  const depthMap = new Map<string, number>();
  const calculateDepth = (nodeId: string, depth: number = 0) => {
    depthMap.set(nodeId, depth);

    if (disableSubflows) {
      lcGraph.graphNodes
        .filter(n => n.belongsTo === nodeId)
        .forEach(child => calculateDepth(child.id, depth + 1));
      return;
    }

    const node = nodeMap.get(nodeId);
    if (!disableSubflows && node?.isPartition) {
      lcGraph.graphNodes
        .filter(n => n.belongsTo === nodeId)
        .forEach(child => calculateDepth(child.id, depth + 1));
    }
  };
  rootNodes.forEach(n => calculateDepth(n.id));

  // Process layouted nodes recursively
  const processElkNode = (elkNode: ElkNode, parentId?: string) => {
    const node = nodeMap.get(elkNode.id);
    const depth = depthMap.get(elkNode.id) || 0;

    if (node?.isPartition) {
      // This is a subflow (group node)
      const groupLabel = elkNode.labels?.[0]?.text || elkNode.id;
      const layerStyle = getLayerStyle(node.layer, layerMap);

      // Build inline style string with !important to override React Flow defaults
      const borderColor = layerStyle?.borderColor || DEFAULT_GROUP_BORDER;
      const bgColor = layerStyle?.backgroundColor || DEFAULT_GROUP_BG;

      // Ensure minimum size for empty containers
      const minContainerWidth = 250;
      const minContainerHeight = 150;
      const containerWidth = Math.max(elkNode.width || minContainerWidth, minContainerWidth);
      const containerHeight = Math.max(elkNode.height || minContainerHeight, minContainerHeight);

      reactFlowNodes.push({
        id: elkNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: {
          label: groupLabel,
        },
        type: 'group',
        width: containerWidth,
        height: containerHeight,
        style: {
          width: containerWidth,
          height: containerHeight,
          backgroundColor: bgColor,
          border: `2px solid ${borderColor}`,
          borderRadius: '8px',
          zIndex: -100 + depth, // Nested groups above parents, but all below edges (10) and nodes (50)
          // Force these styles to override React Flow defaults, especially for nested nodes
          borderColor: borderColor,
          borderWidth: '2px',
          borderStyle: 'solid',
        },
        className: 'layercake-group-node',
        ...(parentId ? { parentNode: parentId } : {}),
      });

      // Add label as a separate node with high z-index
      reactFlowNodes.push({
        id: `${elkNode.id}-label`,
        type: 'labelNode',
        position: { x: 10, y: 6 },
        data: {
          label: groupLabel,
          style: {
            color: layerStyle?.textColor || '#666',
          },
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
          minWidth: 'auto',
          width: 'auto',
          height: 'auto',
        },
        parentNode: elkNode.id,
      });

      // Process children if any
      if (elkNode.children) {
        elkNode.children.forEach(childElk => {
          processElkNode(childElk, elkNode.id);
        });
      }
    } else {
      // Regular node
      const layerStyle = getLayerStyle(node?.layer, layerMap);

      reactFlowNodes.push({
        id: elkNode.id,
        type: 'editable',
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: {
          label: elkNode.labels?.[0]?.text || elkNode.id,
          style: {
            backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
            border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
            color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
          },
        },
        width: elkNode.width,
        height: elkNode.height,
        style: {
          zIndex: 50, // Regular nodes above edges (10) and groups (negative)
          backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
          border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
          color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
        },
        ...(parentId ? { parentNode: parentId } : {}),
      });
    }
  };

  elkGraph.children?.forEach(elkNode => processElkNode(elkNode));

  elkGraph.edges?.forEach((edge: ElkExtendedEdge) => {
    reactFlowEdges.push({
      id: edge.id,
      source: edge.sources && edge.sources.length > 0 ? edge.sources[0] : '',
      target: edge.targets && edge.targets.length > 0 ? edge.targets[0] : '',
      label: edge.labels?.[0]?.text || '',
      type: 'floating',
      markerEnd: { type: MarkerType.ArrowClosed },
      style: {
        zIndex: 10, // Edges above group backgrounds but below nodes
        strokeWidth: 2,
        stroke: '#b1b1b7',
      },
    });
  });

  return { nodes: reactFlowNodes, edges: reactFlowEdges };
};
