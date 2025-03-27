import {
  ReactFlow,
  Node,
  Edge,
  Controls,
  Background,
  Panel,
  useNodesState,
  useEdgesState,
  ConnectionLineType,
  Position,
} from '@xyflow/react';
// CSS will be imported in main.tsx
import { Card } from 'antd';
import { Plan } from '../types';

interface PlanVisualizerProps {
  plan: Plan;
}

const PlanVisualizer = ({ plan }: PlanVisualizerProps) => {
  // Create nodes for the plan visualization
  const createInitialNodes = (): Node[] => {
    const nodes: Node[] = [];
    
    // Add import profile nodes
    plan.import.profiles.forEach((profile, index) => {
      nodes.push({
        id: `import-profile-${index}`,
        type: 'input',
        data: { 
          label: `${profile.filename} (${profile.filetype})`,
        },
        position: { x: 100, y: 100 + (index * 100) },
        style: {
          background: '#e6f4ff',
          border: '1px solid #91caff',
          borderRadius: '4px',
          padding: '10px',
          width: 180,
        },
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      });
    });

    // Graph processing node (middle)
    nodes.push({
      id: 'graph',
      data: { 
        label: 'Graph Processing',
      },
      position: { x: 400, y: 200 },
      style: {
        background: '#f6ffed',
        border: '1px solid #b7eb8f',
        borderRadius: '4px',
        padding: '10px',
        width: 180,
      },
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
    });

    // Add export profile nodes
    plan.export.profiles.forEach((profile, index) => {
      nodes.push({
        id: `export-profile-${index}`,
        type: 'output',
        data: { 
          label: `${profile.filename} (${profile.exporter})`,
        },
        position: { x: 700, y: 100 + (index * 100) },
        style: {
          background: '#fff1f0',
          border: '1px solid #ffa39e',
          borderRadius: '4px',
          padding: '10px',
          width: 180,
        },
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      });
    });

    return nodes;
  };

  // Create edges for the plan visualization
  const createInitialEdges = (): Edge[] => {
    const edges: Edge[] = [];
    
    // Connect import profile nodes to graph processing
    plan.import.profiles.forEach((_, index) => {
      edges.push({
        id: `import-profile-to-graph-${index}`,
        source: `import-profile-${index}`,
        target: 'graph',
        animated: true,
        style: { stroke: '#b7eb8f' },
        type: 'smoothstep',
      });
    });

    // Connect graph processing to export profile nodes
    plan.export.profiles.forEach((_, index) => {
      edges.push({
        id: `graph-to-export-profile-${index}`,
        source: 'graph',
        target: `export-profile-${index}`,
        animated: true,
        style: { stroke: '#ffa39e' },
        type: 'smoothstep',
      });
    });

    return edges;
  };

  const [nodes, , onNodesChange] = useNodesState(createInitialNodes());
  const [edges, , onEdgesChange] = useEdgesState(createInitialEdges());

  return (
    <Card title="Plan Flow Visualization">
      <div style={{ height: '60vh', border: '1px solid #f0f0f0', borderRadius: '4px' }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          connectionLineType={ConnectionLineType.SmoothStep}
          fitView
          fitViewOptions={{ padding: 0.3 }}
          defaultEdgeOptions={{ type: 'smoothstep' }}
          proOptions={{ hideAttribution: true }}
          nodeOrigin={[0.5, 0.5]}
        >
          <Background color="#f0f0f0" gap={16} />
          <Controls />
          <Panel position="top-right">
            <div style={{ padding: '10px', background: 'white', borderRadius: '4px', border: '1px solid #f0f0f0' }}>
              <p style={{ margin: 0 }}>Plan Flow: Import → Graph → Export</p>
            </div>
          </Panel>
        </ReactFlow>
      </div>
    </Card>
  );
};

export default PlanVisualizer;