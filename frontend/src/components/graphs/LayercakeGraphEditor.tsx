import React, { useEffect, useCallback, useMemo } from 'react';
import ReactFlow, { Controls, Background, MiniMap, useNodesState, useEdgesState, useReactFlow, BackgroundVariant } from 'reactflow';

import 'reactflow/dist/style.css';
import '../../styles/reactFlow.css'; // Custom styles

import { Graph } from '../../graphql/graphs';
import { getLayoutedElements } from '../../utils/graphUtils';
import { GroupNode } from './GroupNode';
import { FloatingEdge } from './FloatingEdge';

interface LayercakeGraphEditorProps {
  graph: Graph;
}

export const LayercakeGraphEditor: React.FC<LayercakeGraphEditorProps> = ({ graph }) => {
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

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
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
