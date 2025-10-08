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
  layers: Layer[],
  nodeWidth: number = 170,
  nodeHeight: number = 50
) => {
  const graph: ElkNode = {
    id: 'root',
    layoutOptions: elkOptions,
    children: [],
    edges: [],
  };

  // Create a map for quick lookup of layers by ID
  const layerMap = new Map<string, Layer>();
  layers.forEach(layer => layerMap.set(layer.id.toString(), layer));

  // Group nodes by layer for sub-flows
  const nodesByLayer = new Map<string, GraphNode[]>();
  lcGraph.graphNodes.forEach(node => {
    const layerId = node.layer || 'default';
    if (!nodesByLayer.has(layerId)) {
      nodesByLayer.set(layerId, []);
    }
    nodesByLayer.get(layerId)?.push(node);
  });

  // Add layers as parent nodes (sub-flows)
  nodesByLayer.forEach((nodesInLayer, layerId) => {
    const layer = layerMap.get(layerId);
    const layerNode: ElkNode = {
      id: `layer-${layerId}`,
      // width: nodeWidth, // Sub-flows don't need fixed width/height
      // height: nodeHeight,
      labels: [{ text: layer?.name || `Layer ${layerId}` }],
      layoutOptions: {
        'elk.padding': '[top=40,left=20,bottom=20,right=20]',
        'elk.direction': 'DOWN',
        'elk.spacing.nodeNode': '50',
        'elk.spacing.nodeNodeBetweenLayers': '50',
      },
      children: [],
      edges: [],
    };

    nodesInLayer.forEach(node => {
      layerNode.children?.push({
        id: node.id,
        width: nodeWidth,
        height: nodeHeight,
        labels: [{ text: node.label || node.id }],
      });
    });
    graph.children?.push(layerNode);
  });

  // Add nodes that don't belong to any specific layer
  const unlayeredNodes = lcGraph.graphNodes.filter(node => !node.layer || !nodesByLayer.has(node.layer));
  unlayeredNodes.forEach(node => {
    graph.children?.push({
      id: node.id,
      width: nodeWidth,
      height: nodeHeight,
      labels: [{ text: node.label || node.id }],
    });
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

  // Process layouted nodes and edges
  elkGraph.children?.forEach((elkNode: ElkNode) => {
    if (elkNode.children) { // This is a layer (sub-flow)
      reactFlowNodes.push({
        id: elkNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: { label: elkNode.labels?.[0]?.text || elkNode.id },
        type: 'group',
        style: {
          width: elkNode.width || undefined,
          height: elkNode.height || undefined,
        },
      });

      elkNode.children.forEach((childNode: ElkNode) => {
        reactFlowNodes.push({
          id: childNode.id,
          position: { x: childNode.x || 0, y: childNode.y || 0 },
          data: { label: childNode.labels?.[0]?.text || childNode.id },
          parentNode: elkNode.id,
          extent: 'parent',
        });
      });
    } else { // Regular node
      reactFlowNodes.push({
        id: elkNode.id,
        position: { x: elkNode.x || 0, y: elkNode.y || 0 },
        data: { label: elkNode.labels?.[0]?.text || elkNode.id },
      });
    }
  });

  elkGraph.edges?.forEach((edge: ElkExtendedEdge) => {
    reactFlowEdges.push({
      id: edge.id,
      source: edge.sources && edge.sources.length > 0 ? edge.sources[0] : '',
      target: edge.targets && edge.targets.length > 0 ? edge.targets[0] : '',
      label: edge.labels?.[0]?.text || '',
      type: 'default',
      markerEnd: { type: MarkerType.ArrowClosed },
    });
  });

  return { nodes: reactFlowNodes, edges: reactFlowEdges };
};
