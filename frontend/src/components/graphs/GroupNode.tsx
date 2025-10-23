import React from 'react';
import { NodeProps, Handle, Position } from 'reactflow';

export const GroupNode: React.FC<NodeProps> = () => {
  // Empty component - labels are rendered as separate nodes
  return (
    <>
      <Handle type="target" position={Position.Top} />
      <Handle type="target" position={Position.Left} />
      <div style={{ width: '100%', height: '100%' }} />
      <Handle type="source" position={Position.Right} />
      <Handle type="source" position={Position.Bottom} />
    </>
  );
};
