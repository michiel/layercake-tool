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
  if (!layerId) {
    console.log('[getLayerStyle] No layerId provided');
    return null;
  }

  const layer = layerMap.get(layerId);
  console.log('[getLayerStyle] Looking up layer:', { layerId, layer, properties: layer?.properties });

  if (!layer?.properties) {
    console.log('[getLayerStyle] No layer or properties found for layerId:', layerId);
    return null;
  }

  const style = {
    backgroundColor: layer.properties.background_color ? `#${layer.properties.background_color}` : null,
    borderColor: layer.properties.border_color ? `#${layer.properties.border_color}` : null,
    textColor: layer.properties.text_color ? `#${layer.properties.text_color}` : null,
  };

  console.log('[getLayerStyle] Returning style:', style);
  return style;
};

const elkOptions = {
  'elk.algorithm': 'layered',
  'elk.direction': 'DOWN',
  'elk.spacing.nodeNode': '75',
  'elk.spacing.nodeNodeBetweenLayers': '75',
  'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
  'elk.layered.mergeEdges': 'true',
  'elk.layered.feedbackEdges': 'true',
  'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
  'elk.layered.cycleBreaking.strategy': 'DEPTH_FIRST',
};

// Function to convert LcGraph to React Flow elements
export const getLayoutedElements = async (
  lcGraph: Graph,
  layers: Layer[],
  nodeWidth: number = 170,
  nodeHeight: number = 50
) => {
  // Create node lookup map
  const nodeMap = new Map<string, GraphNode>();
  lcGraph.graphNodes.forEach(node => nodeMap.set(node.id, node));

  // Create layer lookup map by layerId
  const layerMap = new Map<string, Layer>();
  layers.forEach(layer => layerMap.set(layer.layerId, layer));

  // Debug: Log nodes to verify belongsTo and isPartition values
  console.log('=== ALL GRAPH NODES (total: ' + lcGraph.graphNodes.length + ') ===');
  lcGraph.graphNodes.forEach(n => {
    console.log(`  ${n.id}: isPartition=${n.isPartition}, belongsTo=${n.belongsTo || 'null'}, label="${n.label}"`);
  });

  console.log('\n=== PARTITION NODES ===');
  lcGraph.graphNodes.filter(n => n.isPartition).forEach(n => {
    const children = lcGraph.graphNodes.filter(c => c.belongsTo === n.id);
    console.log(`  ${n.id} (${n.label}): ${children.length} children - [${children.map(c => c.id).join(', ')}]`);
  });

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
          'elk.direction': 'DOWN',
          'elk.spacing.nodeNode': '50',
          'elk.spacing.nodeNodeBetweenLayers': '50',
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

  console.log('Root nodes:', rootNodes.map(n => ({ id: n.id, label: n.label, isPartition: n.isPartition })));

  const graph: ElkNode = {
    id: 'root',
    layoutOptions: elkOptions,
    children: [],
    edges: [],
  };

  // Build ELK graph from roots
  rootNodes.forEach(rootNode => {
    const elkNode = buildElkNode(rootNode.id);
    if (elkNode) {
      graph.children?.push(elkNode);
    }
  });

  console.log('ELK graph before layout:', JSON.stringify(graph, null, 2));

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
    const node = nodeMap.get(nodeId);
    if (node?.isPartition) {
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
      console.log(`[processElkNode] Processing subflow container:`, {
        id: elkNode.id,
        nodeLayer: node.layer,
        layerMapSize: layerMap.size,
        availableLayers: Array.from(layerMap.keys())
      });
      const layerStyle = getLayerStyle(node.layer, layerMap);

      reactFlowNodes.push({
        id: elkNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: {
          label: groupLabel,
        },
        type: 'group',
        style: {
          width: elkNode.width || undefined,
          height: elkNode.height || undefined,
          backgroundColor: layerStyle?.backgroundColor || DEFAULT_GROUP_BG,
          border: `2px solid ${layerStyle?.borderColor || DEFAULT_GROUP_BORDER}`,
          borderRadius: '8px',
          zIndex: -depth - 1, // Lower z-index for deeper nesting, below edges
        },
        className: 'layercake-group-node',
        ...(parentId ? { parentNode: parentId, extent: 'parent' as const } : {}),
      });

      // Add label as a separate node with high z-index
      reactFlowNodes.push({
        id: `${elkNode.id}-label`,
        position: { x: 10, y: 6 },
        data: { label: groupLabel },
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
          pointerEvents: 'none',
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
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: { label: elkNode.labels?.[0]?.text || elkNode.id },
        style: {
          zIndex: 50, // Regular nodes above edges (10) and groups (negative)
          backgroundColor: layerStyle?.backgroundColor || DEFAULT_NODE_BG,
          border: `1px solid ${layerStyle?.borderColor || DEFAULT_NODE_BORDER}`,
          color: layerStyle?.textColor || DEFAULT_NODE_TEXT,
        },
        ...(parentId ? { parentNode: parentId, extent: 'parent' as const } : {}),
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
