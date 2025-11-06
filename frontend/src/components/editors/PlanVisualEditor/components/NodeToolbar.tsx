import React from 'react';
import {
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconGitMerge,
  IconCopy,
  IconFileExport
} from '@tabler/icons-react';
import { PlanDagNodeType } from '../../../../types/plan-dag';
import { Group } from '../../../layout-primitives';
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '../../../ui/tooltip';

const isTauri = !!(window as any).__TAURI__;

interface DraggableNodeProps {
  type: PlanDagNodeType;
  label: string;
  icon: React.ReactNode;
  color: string;
  onDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  onPointerDragStart: (event: React.MouseEvent, nodeType: PlanDagNodeType) => void;
}

const DraggableNode: React.FC<DraggableNodeProps> = ({ type, label, icon, color, onDragStart, onPointerDragStart }) => {
  const handleDragStart = (event: React.DragEvent) => {
    onDragStart(event, type);
  };

  const handleMouseDown = (event: React.MouseEvent) => {
    onPointerDragStart(event, type);
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          className="p-3 rounded-md cursor-grab hover:opacity-80 transition-opacity flex flex-col items-center gap-1 min-w-[60px]"
          style={{ backgroundColor: color }}
          draggable={!isTauri}
          onDragStart={handleDragStart}
          onMouseDown={handleMouseDown}
        >
          <div className="text-white pointer-events-none" style={{ fontSize: '1.2rem' }}>
            {icon}
          </div>
          <p className="text-xs text-white font-medium text-center pointer-events-none leading-none">
            {label}
          </p>
        </div>
      </TooltipTrigger>
      <TooltipContent>
        Drag to add {label}
      </TooltipContent>
    </Tooltip>
  );
};

interface NodeToolbarProps {
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  onNodePointerDragStart: (event: React.MouseEvent, nodeType: PlanDagNodeType) => void;
  readonly?: boolean;
}

export const NodeToolbar: React.FC<NodeToolbarProps> = ({ onNodeDragStart, onNodePointerDragStart, readonly = false }) => {
  if (readonly) {
    return null;
  }

  const nodeTypes = [
    {
      type: PlanDagNodeType.DATA_SOURCE,
      label: 'Data Source',
      icon: <IconDatabase size="1.2rem" />,
      color: '#51cf66'
    },
    {
      type: PlanDagNodeType.GRAPH,
      label: 'Graph',
      icon: <IconNetwork size="1.2rem" />,
      color: '#339af0'
    },
    {
      type: PlanDagNodeType.TRANSFORM,
      label: 'Transform',
      icon: <IconTransform size="1.2rem" />,
      color: '#ff8cc8'
    },
    {
      type: PlanDagNodeType.MERGE,
      label: 'Merge',
      icon: <IconGitMerge size="1.2rem" />,
      color: '#ffd43b'
    },
    {
      type: PlanDagNodeType.COPY,
      label: 'Copy',
      icon: <IconCopy size="1.2rem" />,
      color: '#74c0fc'
    },
    {
      type: PlanDagNodeType.OUTPUT,
      label: 'Output',
      icon: <IconFileExport size="1.2rem" />,
      color: '#ff6b6b'
    }
  ];

  return (
    <TooltipProvider>
      <div className="p-4 shadow-sm bg-gray-50 border-b">
        <Group gap="xs" justify="start">
          <p className="text-sm font-medium text-gray-700 mr-4">
            Drag nodes to canvas:
          </p>
          {nodeTypes.map((nodeType) => (
            <DraggableNode
              key={nodeType.type}
              type={nodeType.type}
              label={nodeType.label}
              icon={nodeType.icon}
              color={nodeType.color}
              onDragStart={onNodeDragStart}
              onPointerDragStart={onNodePointerDragStart}
            />
          ))}
        </Group>
      </div>
    </TooltipProvider>
  );
};