import React, { useEffect, useCallback, useMemo } from 'react';
import ReactFlow, { Controls, Background, MiniMap, useNodesState, useEdgesState, useReactFlow, BackgroundVariant, Node } from 'reactflow';

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
}

export const LayercakeGraphEditor: React.FC<LayercakeGraphEditorProps> = ({ graph, onNodeSelect, layerVisibility }) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const { fitView } = useReactFlow();

  const nodeTypes = useMemo(() => ({ group: GroupNode }), []);
  const edgeTypes = useMemo(() => ({ floating: FloatingEdge }), []);

  // Filter graph based on layer visibility
  const filteredGraph = useMemo(() => {
    if (!layerVisibility || layerVisibility.size === 0) {
      return graph;
    }

    const visibleNodes = graph.graphNodes.filter(node => {
      if (!node.layer) return true; // No layer = always visible
      return layerVisibility.get(node.layer) !== false;
    });

    const visibleNodeIds = new Set(visibleNodes.map(n => n.id));

    const visibleEdges = graph.graphEdges.filter(edge => {
      // Edge is visible if both source and target nodes are visible
      const sourceVisible = visibleNodeIds.has(edge.source);
      const targetVisible = visibleNodeIds.has(edge.target);

      // Also check edge's own layer
      if (edge.layer) {
        const edgeLayerVisible = layerVisibility.get(edge.layer) !== false;
        return sourceVisible && targetVisible && edgeLayerVisible;
      }

      return sourceVisible && targetVisible;
    });

    return {
      ...graph,
      graphNodes: visibleNodes,
      graphEdges: visibleEdges,
    };
  }, [graph, layerVisibility]);

  const onLayout = useCallback(async () => {
    if (!filteredGraph || !filteredGraph.graphNodes || !filteredGraph.graphEdges) return;

    const layouted = await getLayoutedElements(filteredGraph, filteredGraph.layers);

    setNodes(layouted.nodes);
    setEdges(layouted.edges);

    window.requestAnimationFrame(() => {
      fitView();
    });
  }, [filteredGraph, setNodes, setEdges, fitView]);

  useEffect(() => {
    onLayout();
  }, [filteredGraph, onLayout]);

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
