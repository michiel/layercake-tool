import React from 'react';
import { Handle, Position, Node } from '@xyflow/react';
import { PlanNode } from '../../types/dag';

interface CustomNodeProps {
  data: {
    label: string;
    description?: string;
    configuration: string;
    planNode: PlanNode;
  };
  selected?: boolean;
}

const NodeWrapper: React.FC<{
  children: React.ReactNode;
  className?: string;
  selected?: boolean;
}> = ({ children, className = '', selected }) => (
  <div
    className={`
      px-4 py-2 shadow-md rounded-md border-2 min-w-[150px] max-w-[200px]
      ${selected ? 'border-blue-500 shadow-lg' : 'border-gray-200 hover:border-gray-300'}
      ${className}
    `}
  >
    {children}
  </div>
);

const InputNode: React.FC<CustomNodeProps> = ({ data, selected }) => (
  <NodeWrapper className="bg-green-100 border-green-300" selected={selected}>
    <Handle type="source" position={Position.Right} />
    <div>
      <div className="font-bold text-sm text-green-800">{data.label}</div>
      {data.description && (
        <div className="text-xs text-green-600 mt-1">{data.description}</div>
      )}
      <div className="text-xs text-gray-500 mt-1">Input</div>
    </div>
  </NodeWrapper>
);

const TransformNode: React.FC<CustomNodeProps> = ({ data, selected }) => (
  <NodeWrapper className="bg-blue-100 border-blue-300" selected={selected}>
    <Handle type="target" position={Position.Left} />
    <Handle type="source" position={Position.Right} />
    <div>
      <div className="font-bold text-sm text-blue-800">{data.label}</div>
      {data.description && (
        <div className="text-xs text-blue-600 mt-1">{data.description}</div>
      )}
      <div className="text-xs text-gray-500 mt-1">Transform</div>
    </div>
  </NodeWrapper>
);

const OutputNode: React.FC<CustomNodeProps> = ({ data, selected }) => (
  <NodeWrapper className="bg-red-100 border-red-300" selected={selected}>
    <Handle type="target" position={Position.Left} />
    <div>
      <div className="font-bold text-sm text-red-800">{data.label}</div>
      {data.description && (
        <div className="text-xs text-red-600 mt-1">{data.description}</div>
      )}
      <div className="text-xs text-gray-500 mt-1">Output</div>
    </div>
  </NodeWrapper>
);

const MergeNode: React.FC<CustomNodeProps> = ({ data, selected }) => (
  <NodeWrapper className="bg-yellow-100 border-yellow-300" selected={selected}>
    <Handle type="target" position={Position.Left} style={{ top: '25%' }} />
    <Handle type="target" position={Position.Left} style={{ top: '75%' }} />
    <Handle type="source" position={Position.Right} />
    <div>
      <div className="font-bold text-sm text-yellow-800">{data.label}</div>
      {data.description && (
        <div className="text-xs text-yellow-600 mt-1">{data.description}</div>
      )}
      <div className="text-xs text-gray-500 mt-1">Merge</div>
    </div>
  </NodeWrapper>
);

const SplitNode: React.FC<CustomNodeProps> = ({ data, selected }) => (
  <NodeWrapper className="bg-purple-100 border-purple-300" selected={selected}>
    <Handle type="target" position={Position.Left} />
    <Handle type="source" position={Position.Right} style={{ top: '25%' }} />
    <Handle type="source" position={Position.Right} style={{ top: '75%' }} />
    <div>
      <div className="font-bold text-sm text-purple-800">{data.label}</div>
      {data.description && (
        <div className="text-xs text-purple-600 mt-1">{data.description}</div>
      )}
      <div className="text-xs text-gray-500 mt-1">Split</div>
    </div>
  </NodeWrapper>
);

export const CustomNodeTypes = {
  InputNode,
  TransformNode,
  OutputNode,
  MergeNode,
  SplitNode,
};