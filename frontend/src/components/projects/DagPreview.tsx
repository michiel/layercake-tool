import React, { useMemo } from 'react';
import { useQuery } from '@apollo/client';
import { ReactFlow, Node, Edge, Background, BackgroundVariant } from '@xyflow/react';
import { GET_PLAN_DAG } from '../../graphql/dag';
import { DagPlan } from '../../types/dag';
import { Loading } from '../ui/Loading';

interface DagPreviewProps {
  planId: number;
}

export const DagPreview: React.FC<DagPreviewProps> = ({ planId }) => {
  const { data, loading, error } = useQuery(GET_PLAN_DAG, {
    variables: { planId },
  });

  const { nodes, edges } = useMemo(() => {
    if (!data?.plan_dag) {
      return { nodes: [], edges: [] };
    }

    const dagPlan: DagPlan = data.plan_dag;

    // Convert plan nodes to ReactFlow nodes with simplified styling for preview
    const reactFlowNodes: Node[] = dagPlan.nodes.map((planNode) => ({
      id: planNode.id,
      type: 'default',
      position: {
        x: (planNode.position_x || 0) * 0.3, // Scale down for preview
        y: (planNode.position_y || 0) * 0.3,
      },
      data: {
        label: planNode.name.length > 12 ? planNode.name.substring(0, 12) + '...' : planNode.name,
      },
      style: {
        width: 80,
        height: 40,
        fontSize: '10px',
        border: '1px solid #d1d5db',
        borderRadius: '6px',
        backgroundColor: getNodeColor(planNode.node_type),
        color: '#374151',
      },
      draggable: false,
      selectable: false,
    }));

    // Convert DAG edges to ReactFlow edges
    const reactFlowEdges: Edge[] = dagPlan.edges.map((dagEdge, index) => ({
      id: `edge-${index}`,
      source: dagEdge.source,
      target: dagEdge.target,
      type: 'smoothstep',
      style: {
        stroke: '#9ca3af',
        strokeWidth: 1,
      },
      animated: false,
    }));

    return { nodes: reactFlowNodes, edges: reactFlowEdges };
  }, [data]);

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loading size="sm" />
      </div>
    );
  }

  if (error || !data?.plan_dag) {
    return (
      <div className="h-full flex items-center justify-center text-gray-400">
        <div className="text-center">
          <div className="text-2xl mb-2">📊</div>
          <div className="text-xs">No DAG data</div>
        </div>
      </div>
    );
  }

  if (nodes.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-gray-400">
        <div className="text-center">
          <div className="text-2xl mb-2">📝</div>
          <div className="text-xs">Empty plan</div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full w-full">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        fitView
        attributionPosition="bottom-left"
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        panOnDrag={false}
        zoomOnScroll={false}
        zoomOnPinch={false}
        zoomOnDoubleClick={false}
        preventScrolling={true}
      >
        <Background variant={BackgroundVariant.Dots} gap={8} size={0.5} color="#e5e7eb" />
      </ReactFlow>
    </div>
  );
};

const getNodeColor = (nodeType: string): string => {
  switch (nodeType) {
    case 'input':
      return '#dcfce7'; // green-100
    case 'transform':
      return '#dbeafe'; // blue-100
    case 'output':
      return '#fecaca'; // red-100
    case 'merge':
      return '#fef3c7'; // yellow-100
    case 'split':
      return '#e9d5ff'; // purple-100
    default:
      return '#f3f4f6'; // gray-100
  }
};