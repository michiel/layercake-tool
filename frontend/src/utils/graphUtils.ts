import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs';
import { Node, Edge, MarkerType } from 'reactflow';
import { Graph, GraphNode, Layer } from '../graphql/graphs';

const elk = new ELK();

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
  _layers: Layer[],
  nodeWidth: number = 170,
  nodeHeight: number = 50
) => {
  // Create node lookup map
  const nodeMap = new Map<string, GraphNode>();
  lcGraph.graphNodes.forEach(node => nodeMap.set(node.id, node));

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
      reactFlowNodes.push({
        id: elkNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: { label: elkNode.labels?.[0]?.text || elkNode.id },
        type: 'group',
        style: {
          width: elkNode.width || undefined,
          height: elkNode.height || undefined,
          zIndex: -depth, // Containing subflows have lower z-index
        },
        ...(parentId ? { parentNode: parentId, extent: 'parent' as const } : {}),
      });

      // Process children if any
      if (elkNode.children) {
        elkNode.children.forEach(childElk => {
          processElkNode(childElk, elkNode.id);
        });
      }
    } else {
      // Regular node
      reactFlowNodes.push({
        id: elkNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: { label: elkNode.labels?.[0]?.text || elkNode.id },
        style: {
          zIndex: 1, // Regular nodes always on top
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
      type: 'default',
      markerEnd: { type: MarkerType.ArrowClosed },
      style: {
        zIndex: 0, // Edges between nodes and subflows
      },
    });
  });

  return { nodes: reactFlowNodes, edges: reactFlowEdges };
};
