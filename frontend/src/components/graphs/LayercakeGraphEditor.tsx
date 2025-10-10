import React, { useEffect, useCallback, useMemo } from 'react';
import ReactFlow, { Controls, Background, MiniMap, useNodesState, useEdgesState, useReactFlow, BackgroundVariant, Node, Edge } from 'reactflow';

import 'reactflow/dist/style.css';
import '../../styles/reactFlow.css'; // Custom styles

import { Graph } from '../../graphql/graphs';
import { getLayoutedElements } from '../../utils/graphUtils';
import { GroupNode } from './GroupNode';
import { FloatingEdge } from './FloatingEdge';

interface LayercakeGraphEditorProps {
  graph: Graph;
  onNodeSelect?: (nodeId: string | null) => void;
  layerVisibility?: Map<string, boolean>;
  onNodesInitialized?: (setNodes: React.Dispatch<React.SetStateAction<Node[]>>, setEdges: React.Dispatch<React.SetStateAction<Edge[]>>) => void;
}

export const LayercakeGraphEditor: React.FC<LayercakeGraphEditorProps> = ({
  graph,
  onNodeSelect,
  layerVisibility,
  onNodesInitialized
}) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const { fitView } = useReactFlow();

  const nodeTypes = useMemo(() => ({ group: GroupNode }), []);
  const edgeTypes = useMemo(() => ({ floating: FloatingEdge }), []);

  const onLayout = useCallback(async () => {
    if (!graph || !graph.graphNodes || !graph.graphEdges) return;

    const layouted = await getLayoutedElements(graph, graph.layers);

    setNodes(layouted.nodes);
    setEdges(layouted.edges);

    window.requestAnimationFrame(() => {
      fitView();
    });
  }, [graph, setNodes, setEdges, fitView]);

  useEffect(() => {
    onLayout();
  }, [graph, onLayout]);

  // Notify parent that nodes/edges are ready for manipulation
  useEffect(() => {
    if (onNodesInitialized && nodes.length > 0) {
      onNodesInitialized(setNodes, setEdges);
    }
  }, [nodes.length, onNodesInitialized]);

  // Update hidden property when visibility changes (no re-layout!)
  useEffect(() => {
    if (!layerVisibility || layerVisibility.size === 0) return;

    setNodes(currentNodes => {
      return currentNodes.map(node => {
        // Skip label nodes
        if (node.id.endsWith('-label')) {
          return node;
        }

        // Find the corresponding graph node to get layer info
        const graphNode = graph.graphNodes.find(gn => gn.id === node.id);
        if (!graphNode) return node;

        const shouldHide = graphNode.layer
          ? layerVisibility.get(graphNode.layer) === false
          : false;

        return { ...node, hidden: shouldHide };
      });
    });

    setEdges(currentEdges => {
      return currentEdges.map(edge => {
        const graphEdge = graph.graphEdges.find(ge => ge.id === edge.id);
        if (!graphEdge) return edge;

        // Hide edge if its layer is hidden
        const edgeLayerHidden = graphEdge.layer
          ? layerVisibility.get(graphEdge.layer) === false
          : false;

        // Hide edge if source or target is hidden
        const sourceNode = graph.graphNodes.find(n => n.id === graphEdge.source);
        const targetNode = graph.graphNodes.find(n => n.id === graphEdge.target);

        const sourceHidden = sourceNode?.layer
          ? layerVisibility.get(sourceNode.layer) === false
          : false;
        const targetHidden = targetNode?.layer
          ? layerVisibility.get(targetNode.layer) === false
          : false;

        const shouldHide = edgeLayerHidden || sourceHidden || targetHidden;

        return { ...edge, hidden: shouldHide };
      });
    });
  }, [layerVisibility, graph, setNodes, setEdges]);

  // Handle node selection
  const handleSelectionChange = useCallback(({ nodes }: { nodes: Node[] }) => {
    // Filter out label nodes (they end with '-label')
    const selectedNodes = nodes.filter(node => !node.id.endsWith('-label'));

    if (onNodeSelect) {
      if (selectedNodes.length > 0) {
        onNodeSelect(selectedNodes[0].id);
      } else {
        onNodeSelect(null);
      }
    }
  }, [onNodeSelect]);

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onSelectionChange={handleSelectionChange}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        attributionPosition="bottom-left"
      >
        <Controls />
        <MiniMap />
        <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
      </ReactFlow>
    </div>
  );
};
