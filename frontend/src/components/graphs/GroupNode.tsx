import React from 'react';
import { NodeProps } from 'reactflow';

export const GroupNode: React.FC<NodeProps> = () => {
  // Empty component - labels are rendered as separate nodes
  return <div style={{ width: '100%', height: '100%' }} />;
};
