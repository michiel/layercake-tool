import React, { useCallback, useEffect, useState } from 'react';
import {
  ReactFlow,
  addEdge,
  useNodesState,
  useEdgesState,
  Controls,
  MiniMap,
  Background,
  BackgroundVariant,
  ReactFlowProvider,
  useReactFlow,
  type Node,
  type Edge,
  type Connection,
  type NodeChange,
  type EdgeChange,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import type { PlanNode, DagPlan } from '../../types/dag';
import { PlanNodeEditor } from './PlanNodeEditor';
import { CustomNodeTypes } from './CustomNodeTypes';

interface DagEditorProps {
  planId: number;
  dagPlan?: DagPlan;
  onDagChange?: (dag: DagPlan) => void;
  readonly?: boolean;
  onNodeAdd?: (nodeType: string, position: { x: number; y: number }) => void;
}

const nodeTypes = {
  input: CustomNodeTypes.InputNode,
  transform: CustomNodeTypes.TransformNode,
  output: CustomNodeTypes.OutputNode,
  merge: CustomNodeTypes.MergeNode,
  split: CustomNodeTypes.SplitNode,
};

export const DagEditor: React.FC<DagEditorProps> = ({
  planId,
  dagPlan,
  onDagChange,
  readonly = false,
  onNodeAdd,
}) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [selectedNode, setSelectedNode] = useState<PlanNode | null>(null);
  const [isNodeEditorOpen, setIsNodeEditorOpen] = useState(false);

  // Convert DAG plan to ReactFlow format
  useEffect(() => {
    if (dagPlan) {
      const reactFlowNodes: Node[] = dagPlan.nodes.map((planNode) => ({
        id: planNode.id,
        type: planNode.nodeType,
        position: {
          x: planNode.positionX || 0,
          y: planNode.positionY || 0,
        },
        data: {
          label: planNode.name,
          description: planNode.description,
          configuration: planNode.configuration,
          planNode,
        },
      }));

      const reactFlowEdges: Edge[] = dagPlan.edges.map((dagEdge, index) => ({
        id: `edge-${index}`,
        source: dagEdge.source,
        target: dagEdge.target,
        type: 'smoothstep',
      }));

      setNodes(reactFlowNodes);
      setEdges(reactFlowEdges);
    }
  }, [dagPlan, setNodes, setEdges]);

  const onConnect = useCallback(
    (params: Edge | Connection) => {
      if (readonly) return;
      setEdges((eds) => addEdge(params, eds));
    },
    [setEdges, readonly]
  );

  const onNodeClick = useCallback(
    (event: React.MouseEvent, node: Node) => {
      if (readonly) return;
      setSelectedNode(node.data.planNode);
      setIsNodeEditorOpen(true);
    },
    [readonly]
  );

  const onNodeDragStop = useCallback(
    (event: React.MouseEvent, node: Node) => {
      if (readonly) return;
      // Update plan node position
      const updatedNodes = nodes.map((n) =>
        n.id === node.id
          ? { ...n, position: node.position }
          : n
      );
      setNodes(updatedNodes);
      
      // Notify parent of changes
      if (onDagChange) {
        const updatedDag: DagPlan = {
          nodes: updatedNodes.map((n) => ({
            ...n.data.planNode,
            position_x: n.position.x,
            position_y: n.position.y,
          })),
          edges: dagPlan?.edges || [],
        };
        onDagChange(updatedDag);
      }
    },
    [nodes, setNodes, onDagChange, dagPlan, readonly]
  );

  const handleNodeSave = useCallback(
    (updatedNode: PlanNode) => {
      const updatedNodes = nodes.map((n) =>
        n.id === updatedNode.id
          ? {
              ...n,
              data: {
                ...n.data,
                label: updatedNode.name,
                description: updatedNode.description,
                configuration: updatedNode.configuration,
                planNode: updatedNode,
              },
            }
          : n
      );
      setNodes(updatedNodes);
      setIsNodeEditorOpen(false);
      setSelectedNode(null);

      // Notify parent of changes
      if (onDagChange) {
        const updatedDag: DagPlan = {
          nodes: updatedNodes.map((n) => n.data.planNode),
          edges: dagPlan?.edges || [],
        };
        onDagChange(updatedDag);
      }
    },
    [nodes, setNodes, onDagChange, dagPlan]
  );

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      
      if (readonly) return;
      
      const nodeType = event.dataTransfer.getData('application/reactflow');
      const rect = (event.target as HTMLElement).getBoundingClientRect();
      const position = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      };
      
      if (onNodeAdd) {
        onNodeAdd(nodeType, position);
      } else {
        // Fallback: create node directly
        const newNode: Node = {
          id: `node-${Date.now()}`,
          type: nodeType,
          position,
          data: {
            label: `New ${nodeType} Node`,
            description: '',
            configuration: '{}',
            planNode: {
              id: `node-${Date.now()}`,
              planId: planId,
              nodeType: nodeType,
              name: `New ${nodeType} Node`,
              description: '',
              configuration: '{}',
              positionX: position.x,
              positionY: position.y,
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString(),
            },
          },
        };

        setNodes((nds) => [...nds, newNode]);
      }
    },
    [planId, setNodes, readonly, onNodeAdd]
  );

  const handleDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDeleteNode = useCallback(
    (nodeId: string) => {
      if (readonly) return;
      
      setNodes((nds) => nds.filter((n) => n.id !== nodeId));
      setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
    },
    [setNodes, setEdges, readonly]
  );

  return (
    <div className="h-full w-full">
      <ReactFlowProvider>
        <div 
          className="h-full w-full"
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        >
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onNodeDragStop={onNodeDragStop}
            nodeTypes={nodeTypes}
            fitView
            attributionPosition="bottom-left"
          >
            <Controls />
            <MiniMap />
            <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
          </ReactFlow>
        </div>

        {selectedNode && (
          <PlanNodeEditor
            planNode={selectedNode}
            isOpen={isNodeEditorOpen}
            onSave={handleNodeSave}
            onCancel={() => {
              setIsNodeEditorOpen(false);
              setSelectedNode(null);
            }}
            onDelete={() => {
              handleDeleteNode(selectedNode.id);
              setIsNodeEditorOpen(false);
              setSelectedNode(null);
            }}
          />
        )}
      </ReactFlowProvider>
    </div>
  );
};