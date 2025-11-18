import dagre from 'dagre';
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
  const layoutDirection = orientation === 'horizontal' ? 'LR' : 'TB';
  const nodeSpacing = options.nodeSpacing ?? 40;
  const rankSpacing = options.rankSpacing ?? 50;

  // Create node lookup map
  const nodeMap = new Map<string, GraphNode>();
  lcGraph.graphNodes.forEach(node => nodeMap.set(node.id, node));

  // Create layer lookup map by layerId
  const layerMap = new Map<string, Layer>();
  layers.forEach(layer => layerMap.set(layer.layerId, layer));

  const reactFlowNodes: Node[] = [];
  const reactFlowEdges: Edge[] = [];

  // Calculate depth for z-index (deeper = higher z-index)
  const depthMap = new Map<string, number>();
  const calculateDepth = (nodeId: string, depth: number = 0) => {
    depthMap.set(nodeId, depth);
    const node = nodeMap.get(nodeId);
    if (!disableSubflows && node?.isPartition) {
      lcGraph.graphNodes
        .filter(n => n.belongsTo === nodeId)
        .forEach(child => calculateDepth(child.id, depth + 1));
    }
  };

  // Find root nodes (no belongsTo or belongsTo references non-existent node)
  const rootNodes = lcGraph.graphNodes.filter(n =>
    !n.belongsTo || !nodeMap.has(n.belongsTo)
  );
  rootNodes.forEach(n => calculateDepth(n.id));

  const GROUP_PADDING = 32;

  // Phase 1: Calculate sizes for all groups recursively (bottom-up)
  const groupSizes = new Map<string, { width: number; height: number }>();

  const calculateGroupSize = (groupId: string): { width: number; height: number } => {
    const children = lcGraph.graphNodes.filter(n => n.belongsTo === groupId);

    if (children.length === 0) {
      return { width: 200, height: 120 };
    }

    // First, calculate sizes for any child groups
    children.forEach(child => {
      if (child.isPartition) {
        const size = calculateGroupSize(child.id);
        groupSizes.set(child.id, size);
      }
    });

    // Layout children to get their positions
    const g = new dagre.graphlib.Graph();
    g.setGraph({
      rankdir: layoutDirection,
      nodesep: nodeSpacing,
      ranksep: rankSpacing,
      edgesep: 30,
      marginx: 0,
      marginy: 0,
    });
    g.setDefaultEdgeLabel(() => ({}));

    children.forEach(child => {
      if (child.isPartition) {
        const size = groupSizes.get(child.id) || { width: 200, height: 120 };
        g.setNode(child.id, size);
      } else {
        g.setNode(child.id, { width: nodeWidth, height: nodeHeight });
      }
    });

    // Add edges between children
    lcGraph.graphEdges.forEach(edge => {
      if (children.some(c => c.id === edge.source) && children.some(c => c.id === edge.target)) {
        g.setEdge(edge.source, edge.target);
      }
    });

    dagre.layout(g);

    // Calculate bounding box
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    children.forEach(child => {
      const pos = g.node(child.id);
      if (pos) {
        const childWidth = child.isPartition
          ? (groupSizes.get(child.id)?.width || 200)
          : nodeWidth;
        const childHeight = child.isPartition
          ? (groupSizes.get(child.id)?.height || 120)
          : nodeHeight;
        minX = Math.min(minX, pos.x - childWidth / 2);
        minY = Math.min(minY, pos.y - childHeight / 2);
        maxX = Math.max(maxX, pos.x + childWidth / 2);
        maxY = Math.max(maxY, pos.y + childHeight / 2);
      }
    });

    const width = Math.max(200, maxX - minX + GROUP_PADDING * 2);
    const height = Math.max(120, maxY - minY + GROUP_PADDING * 2);

    return { width, height };
  };

  // Calculate sizes for all root-level groups
  rootNodes.forEach(node => {
    if (node.isPartition && !disableSubflows) {
      const size = calculateGroupSize(node.id);
      groupSizes.set(node.id, size);
    }
  });

  // Phase 2: Layout with correct sizes
  const layoutSubgraph = (nodes: GraphNode[]) => {
    const g = new dagre.graphlib.Graph({ compound: true });

    g.setGraph({
      rankdir: layoutDirection,
      nodesep: nodeSpacing,
      ranksep: rankSpacing,
      edgesep: 30,
      marginx: 20,
      marginy: 20,
    });

    g.setDefaultEdgeLabel(() => ({}));

    // Add nodes with correct sizes
    nodes.forEach(node => {
      if (node.isPartition) {
        const size = groupSizes.get(node.id) || { width: 200, height: 120 };
        g.setNode(node.id, size);
      } else {
        g.setNode(node.id, { width: nodeWidth, height: nodeHeight });
      }
    });

    // Add edges that connect nodes in this subgraph
    lcGraph.graphEdges.forEach(edge => {
      const sourceNode = nodeMap.get(edge.source);
      const targetNode = nodeMap.get(edge.target);

      if (sourceNode && targetNode && nodes.some(n => n.id === edge.source) && nodes.some(n => n.id === edge.target)) {
        g.setEdge(edge.source, edge.target);
      }
    });

    dagre.layout(g);

    return nodes.map(node => {
      const positioned = g.node(node.id);
      if (!positioned) return null;

      const isGroup = node.isPartition;
      const size = isGroup ? groupSizes.get(node.id) : null;
      const width = size?.width || (isGroup ? 200 : nodeWidth);
      const height = size?.height || (isGroup ? 120 : nodeHeight);

      return {
        id: node.id,
        graphNode: node,
        position: {
          x: positioned.x - width / 2,
          y: positioned.y - height / 2,
        },
        width,
        height,
      };
    }).filter((n): n is NonNullable<typeof n> => n !== null);
  };

  // Layout root level
  const positionedRoots = layoutSubgraph(rootNodes);

  // Build React Flow nodes
  positionedRoots.forEach(positioned => {
    const node = positioned.graphNode;
    const depth = depthMap.get(node.id) || 0;

    if (node.isPartition) {
      // Group node
      const groupLabel = node.label || node.id;
      const layerStyle = getLayerStyle(node.layer, layerMap);
      const borderColor = layerStyle?.borderColor || DEFAULT_GROUP_BORDER;
      const bgColor = layerStyle?.backgroundColor || DEFAULT_GROUP_BG;

      reactFlowNodes.push({
        id: node.id,
        position: positioned.position,
        data: {
          label: groupLabel,
        },
        type: 'group',
        width: positioned.width,
        height: positioned.height,
        style: {
          width: positioned.width,
          height: positioned.height,
          backgroundColor: bgColor,
          border: `2px solid ${borderColor}`,
          borderRadius: '8px',
          zIndex: -100 + depth,
          borderColor: borderColor,
          borderWidth: '2px',
          borderStyle: 'solid',
        },
        className: 'layercake-group-node',
      });

      // Add label node
      reactFlowNodes.push({
        id: `${node.id}-label`,
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
        parentNode: node.id,
      });

      // Layout children if not disabled
      if (!disableSubflows) {
        const children = lcGraph.graphNodes.filter(n => n.belongsTo === node.id);
        if (children.length > 0) {
          // Re-layout children to get their positions within this group
          const g = new dagre.graphlib.Graph();
          g.setGraph({
            rankdir: layoutDirection,
            nodesep: nodeSpacing,
            ranksep: rankSpacing,
            edgesep: 30,
            marginx: 0,
            marginy: 0,
          });
          g.setDefaultEdgeLabel(() => ({}));

          children.forEach(child => {
            if (child.isPartition) {
              const size = groupSizes.get(child.id) || { width: 200, height: 120 };
              g.setNode(child.id, size);
            } else {
              g.setNode(child.id, { width: nodeWidth, height: nodeHeight });
            }
          });

          lcGraph.graphEdges.forEach(edge => {
            if (children.some(c => c.id === edge.source) && children.some(c => c.id === edge.target)) {
              g.setEdge(edge.source, edge.target);
            }
          });

          dagre.layout(g);

          // Calculate bounding box and position children
          let minX = Infinity, minY = Infinity;
          children.forEach(child => {
            const pos = g.node(child.id);
            if (pos) {
              const childWidth = child.isPartition
                ? (groupSizes.get(child.id)?.width || 200)
                : nodeWidth;
              const childHeight = child.isPartition
                ? (groupSizes.get(child.id)?.height || 120)
                : nodeHeight;
              minX = Math.min(minX, pos.x - childWidth / 2);
              minY = Math.min(minY, pos.y - childHeight / 2);
            }
          });

          // Position children with padding offset
          const offsetX = GROUP_PADDING - minX;
          const offsetY = GROUP_PADDING - minY;

          children.forEach(child => {
            const pos = g.node(child.id);
            if (!pos) return;

            const childWidth = child.isPartition
              ? (groupSizes.get(child.id)?.width || 200)
              : nodeWidth;
            const childHeight = child.isPartition
              ? (groupSizes.get(child.id)?.height || 120)
              : nodeHeight;

            const childLayerStyle = getLayerStyle(child.layer, layerMap);

            if (child.isPartition) {
              // Nested group
              const childDepth = depthMap.get(child.id) || 0;
              const borderColor = childLayerStyle?.borderColor || DEFAULT_GROUP_BORDER;
              const bgColor = childLayerStyle?.backgroundColor || DEFAULT_GROUP_BG;

              reactFlowNodes.push({
                id: child.id,
                position: {
                  x: pos.x - childWidth / 2 + offsetX,
                  y: pos.y - childHeight / 2 + offsetY,
                },
                data: { label: child.label || child.id },
                type: 'group',
                width: childWidth,
                height: childHeight,
                style: {
                  width: childWidth,
                  height: childHeight,
                  backgroundColor: bgColor,
                  border: `2px solid ${borderColor}`,
                  borderRadius: '8px',
                  zIndex: -100 + childDepth,
                },
                className: 'layercake-group-node',
                parentNode: node.id,
              });

              // Add label for nested group
              reactFlowNodes.push({
                id: `${child.id}-label`,
                type: 'labelNode',
                position: { x: 10, y: 6 },
                data: {
                  label: child.label || child.id,
                  style: { color: childLayerStyle?.textColor || '#666' },
                },
                draggable: false,
                selectable: false,
                connectable: false,
                style: {
                  background: 'transparent',
                  border: 'none',
                  fontSize: '11px',
                  fontWeight: '500',
                  color: childLayerStyle?.textColor || '#666',
                  padding: 0,
                  zIndex: 100,
                },
                parentNode: child.id,
              });
            } else {
              // Regular child node
              reactFlowNodes.push({
                id: child.id,
                type: 'editable',
                position: {
                  x: pos.x - childWidth / 2 + offsetX,
                  y: pos.y - childHeight / 2 + offsetY,
                },
                data: {
                  label: child.label || child.id,
                  style: {
                    backgroundColor: childLayerStyle?.backgroundColor || DEFAULT_NODE_BG,
                    border: `1px solid ${childLayerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
                    color: childLayerStyle?.textColor || DEFAULT_NODE_TEXT,
                  },
                },
                width: childWidth,
                height: childHeight,
                style: {
                  zIndex: 50,
                  backgroundColor: childLayerStyle?.backgroundColor || DEFAULT_NODE_BG,
                  border: `1px solid ${childLayerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
                  color: childLayerStyle?.textColor || DEFAULT_NODE_TEXT,
                },
                parentNode: node.id,
              });
            }
          });
        }
      }
    } else {
      // Regular node
      const layerStyle = getLayerStyle(node.layer, layerMap);

      reactFlowNodes.push({
        id: node.id,
        type: 'editable',
        position: positioned.position,
        data: {
          label: node.label || node.id,
          style: {
            backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
            border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
            color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
          },
        },
        width: positioned.width,
        height: positioned.height,
        style: {
          zIndex: 50,
          backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
          border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
          color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
        },
      });
    }
  });

  // Add edges
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
