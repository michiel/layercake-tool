import React from 'react';

interface NodePaletteProps {
  className?: string;
}

export const NodePalette: React.FC<NodePaletteProps> = ({ className }) => {
  const nodeTypes = [
    { type: 'input', label: 'Input', icon: '📥', color: 'bg-green-500', description: 'Data input node' },
    { type: 'transform', label: 'Transform', icon: '🔄', color: 'bg-blue-500', description: 'Transform data' },
    { type: 'output', label: 'Output', icon: '📤', color: 'bg-red-500', description: 'Data output node' },
    { type: 'merge', label: 'Merge', icon: '🔗', color: 'bg-yellow-500', description: 'Merge multiple inputs' },
    { type: 'split', label: 'Split', icon: '🔀', color: 'bg-purple-500', description: 'Split into multiple outputs' },
  ];

  const onDragStart = (event: React.DragEvent<HTMLDivElement>, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div className={`bg-white border-l border-gray-200 w-64 h-full overflow-y-auto ${className || ''}`}>
      <div className="p-4">
        <h3 className="text-sm font-semibold text-gray-700 mb-4">Node Palette</h3>
        <p className="text-xs text-gray-500 mb-4">Drag nodes onto the canvas to add them</p>
        
        <div className="space-y-2">
          {nodeTypes.map((nodeType) => (
            <div
              key={nodeType.type}
              draggable
              onDragStart={(event) => onDragStart(event, nodeType.type)}
              className="group cursor-move bg-gray-50 hover:bg-gray-100 border border-gray-200 hover:border-gray-300 rounded-lg p-3 transition-all duration-150"
            >
              <div className="flex items-center space-x-3">
                <div className={`w-8 h-8 ${nodeType.color} rounded-md flex items-center justify-center text-white text-sm shrink-0`}>
                  {nodeType.icon}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium text-gray-900">
                    {nodeType.label}
                  </div>
                  <div className="text-xs text-gray-500 mt-1">
                    {nodeType.description}
                  </div>
                </div>
              </div>
              <div className="mt-2 text-xs text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity">
                Drag to canvas to add
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};