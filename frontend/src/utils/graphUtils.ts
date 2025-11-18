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

  // Layout each subgraph separately
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

    // Add nodes to dagre graph
    nodes.forEach(node => {
      if (node.isPartition) {
        // Group node - use compact minimum size
        const minWidth = 200;
        const minHeight = 120;

        g.setNode(node.id, {
          width: minWidth,
          height: minHeight,
        });
      } else {
        g.setNode(node.id, {
          width: nodeWidth,
          height: nodeHeight,
        });
      }
    });

    // Add edges that connect nodes in this subgraph
    lcGraph.graphEdges.forEach(edge => {
      const sourceNode = nodeMap.get(edge.source);
      const targetNode = nodeMap.get(edge.target);

      // Only add edge if both nodes are in this subgraph
      if (sourceNode && targetNode && nodes.some(n => n.id === edge.source) && nodes.some(n => n.id === edge.target)) {
        g.setEdge(edge.source, edge.target);
      }
    });

    // Run layout
    dagre.layout(g);

    // Extract positioned nodes
    return nodes.map(node => {
      const positioned = g.node(node.id);
      if (!positioned) return null;

      const isGroup = node.isPartition;
      const width = isGroup ? Math.max(positioned.width || 200, 200) : nodeWidth;
      const height = isGroup ? Math.max(positioned.height || 120, 120) : nodeHeight;

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
          const positionedChildren = layoutSubgraph(children);

          // Calculate bounding box of children to size the group
          const GROUP_PADDING = 32;
          let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

          positionedChildren.forEach(child => {
            minX = Math.min(minX, child.position.x);
            minY = Math.min(minY, child.position.y);
            maxX = Math.max(maxX, child.position.x + child.width);
            maxY = Math.max(maxY, child.position.y + child.height);
          });

          // Calculate required group size based on children
          const childrenWidth = maxX - minX;
          const childrenHeight = maxY - minY;
          const requiredWidth = Math.max(200, childrenWidth + GROUP_PADDING * 2);
          const requiredHeight = Math.max(120, childrenHeight + GROUP_PADDING * 2);

          // Update group node size
          const groupNodeIndex = reactFlowNodes.findIndex(n => n.id === node.id);
          if (groupNodeIndex >= 0) {
            reactFlowNodes[groupNodeIndex] = {
              ...reactFlowNodes[groupNodeIndex],
              width: requiredWidth,
              height: requiredHeight,
              style: {
                ...reactFlowNodes[groupNodeIndex].style,
                width: requiredWidth,
                height: requiredHeight,
              },
            };
          }

          // Position children with padding offset
          const offsetX = GROUP_PADDING - minX;
          const offsetY = GROUP_PADDING - minY;

          positionedChildren.forEach(childPositioned => {
            const childNode = childPositioned.graphNode;
            const layerStyle = getLayerStyle(childNode.layer, layerMap);

            reactFlowNodes.push({
              id: childNode.id,
              type: 'editable',
              position: {
                x: childPositioned.position.x + offsetX,
                y: childPositioned.position.y + offsetY,
              },
              data: {
                label: childNode.label || childNode.id,
                style: {
                  backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
                  border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
                  color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
                },
              },
              width: childPositioned.width,
              height: childPositioned.height,
              style: {
                zIndex: 50,
                backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
                border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
                color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
              },
              parentNode: node.id,
            });
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
