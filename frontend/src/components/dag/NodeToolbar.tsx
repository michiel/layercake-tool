import React from 'react';
import { Button } from '../ui/Button';

interface NodeToolbarProps {
  onAddNode: (nodeType: string) => void;
}

export const NodeToolbar: React.FC<NodeToolbarProps> = ({ onAddNode }) => {
  const nodeTypes = [
    { type: 'input', label: 'Input', icon: '📥', color: 'bg-green-500' },
    { type: 'transform', label: 'Transform', icon: '🔄', color: 'bg-blue-500' },
    { type: 'output', label: 'Output', icon: '📤', color: 'bg-red-500' },
    { type: 'merge', label: 'Merge', icon: '🔗', color: 'bg-yellow-500' },
    { type: 'split', label: 'Split', icon: '🔀', color: 'bg-purple-500' },
  ];

  return (
    <div className="bg-white rounded-lg shadow-md p-4 min-w-[200px]">
      <h3 className="text-sm font-semibold text-gray-700 mb-3">Add Node</h3>
      <div className="space-y-2">
        {nodeTypes.map((nodeType) => (
          <Button
            key={nodeType.type}
            variant="outline"
            size="sm"
            onClick={() => onAddNode(nodeType.type)}
            className="w-full justify-start"
          >
            <span className="mr-2 text-lg">{nodeType.icon}</span>
            <span className="text-sm">{nodeType.label}</span>
          </Button>
        ))}
      </div>
    </div>
  );
};