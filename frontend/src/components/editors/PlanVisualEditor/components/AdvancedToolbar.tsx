import React from 'react';
import {
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconFilter,
  IconGitMerge,
  IconCopy,
  IconFileExport,
  IconArrowRight,
  IconArrowDown,
  IconZoomScan,
  IconPlayerPlay,
  IconPlayerStop,
  IconTrash
} from '@tabler/icons-react';
import { PlanDagNodeType } from '../../../../types/plan-dag';
import { Group } from '../../../layout-primitives';
import { Button } from '../../../ui/button';
import { Separator } from '../../../ui/separator';
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '../../../ui/tooltip';

interface AdvancedToolbarProps {
  readonly?: boolean;
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  onNodePointerDragStart: (event: React.MouseEvent, nodeType: PlanDagNodeType) => void;
  onAutoLayoutHorizontal: () => void;
  onAutoLayoutVertical: () => void;
  onFitView: () => void;
  onPlay: () => void;
  onStop: () => void;
  onClear: () => void;
}

const isTauri = !!(window as any).__TAURI__;

export const AdvancedToolbar: React.FC<AdvancedToolbarProps> = ({
  readonly = false,
  onNodeDragStart,
  onNodePointerDragStart,
  onAutoLayoutHorizontal,
  onAutoLayoutVertical,
  onFitView,
  onPlay,
  onStop,
  onClear,
}) => {
  if (readonly) return null;

  const nodeTypes = [
    {
      type: PlanDagNodeType.DATA_SOURCE,
      label: 'Data Source',
      icon: <IconDatabase size="0.7rem" />,
      color: '#10b981' // Emerald-500
    },
    {
      type: PlanDagNodeType.GRAPH,
      label: 'Graph',
      icon: <IconNetwork size="0.7rem" />,
      color: '#3b82f6' // Blue-500
    },
    {
      type: PlanDagNodeType.TRANSFORM,
      label: 'Transform',
      icon: <IconTransform size="0.7rem" />,
      color: '#8b5cf6' // Violet-500
    },
    {
      type: PlanDagNodeType.FILTER,
      label: 'Filter',
      icon: <IconFilter size="0.7rem" />,
      color: '#8b5cf6' // Violet-500
    },
    {
      type: PlanDagNodeType.MERGE,
      label: 'Merge',
      icon: <IconGitMerge size="0.7rem" />,
      color: '#8b5cf6' // Violet-500
    },
    {
      type: PlanDagNodeType.COPY,
      label: 'Copy',
      icon: <IconCopy size="0.7rem" />,
      color: '#8b5cf6' // Violet-500
    },
    {
      type: PlanDagNodeType.OUTPUT,
      label: 'Output',
      icon: <IconFileExport size="0.7rem" />,
      color: '#f59e0b' // Amber-500
    }
  ];

  const handleNodeDragStart = (event: React.DragEvent, nodeType: PlanDagNodeType) => {
    onNodeDragStart(event, nodeType);
  };

  const handleNodePointerDragStart = (event: React.MouseEvent, nodeType: PlanDagNodeType) => {
    onNodePointerDragStart(event, nodeType);
  };

  return (
    <TooltipProvider>
      <div className="flex items-center justify-between p-4 border-b gap-2">
        {/* Left side - Node Palette, Auto-Layout, Fit View */}
        <Group gap="xs">
          {/* Node Palette */}
          <Group gap="xs">
            <p className="text-xs font-medium text-gray-600">Nodes:</p>
            {nodeTypes.map((nodeType) => (
              <Tooltip key={nodeType.type}>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="secondary"
                    className="h-7 w-7 cursor-grab text-white hover:opacity-80"
                    style={{ backgroundColor: nodeType.color }}
                    draggable={!isTauri}
                    onDragStart={(event) => handleNodeDragStart(event, nodeType.type)}
                    onMouseDown={(event) => handleNodePointerDragStart(event, nodeType.type)}
                  >
                    {nodeType.icon}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  Drag to add {nodeType.label}
                </TooltipContent>
              </Tooltip>
            ))}
          </Group>

          {/* Auto-Layout Operations */}
          <Separator orientation="vertical" className="h-6" />
          <p className="text-xs text-muted-foreground">Auto-layout:</p>
          <Group gap="xs">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" onClick={onAutoLayoutHorizontal}>
                  <IconArrowRight className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Auto-layout Horizontal</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" onClick={onAutoLayoutVertical}>
                  <IconArrowDown className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Auto-layout Vertical</TooltipContent>
            </Tooltip>
          </Group>

          {/* Fit View */}
          <Separator orientation="vertical" className="h-6" />
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="ghost" size="icon" onClick={onFitView}>
                <IconZoomScan className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Fit View (Zoom to see all nodes)</TooltipContent>
          </Tooltip>
        </Group>

        {/* Right side - Execution Controls */}
        <Group gap="xs">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="ghost" size="icon" className="text-red-600" onClick={onStop}>
                <IconPlayerStop className="h-5 w-5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Stop execution</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="ghost" size="icon" className="text-orange-600" onClick={onClear}>
                <IconTrash className="h-5 w-5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Clear execution state (reset all nodes)</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button size="icon" className="bg-green-600 hover:bg-green-700 text-white" onClick={onPlay}>
                <IconPlayerPlay className="h-5 w-5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Execute DAG (optimal execution)</TooltipContent>
          </Tooltip>
        </Group>
      </div>
    </TooltipProvider>
  );
};