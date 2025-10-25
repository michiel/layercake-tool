import React from 'react';
import { Group, ActionIcon, Tooltip, Text, Divider } from '@mantine/core';
import {
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconGitMerge,
  IconCopy,
  IconFileExport,
  IconArrowRight,
  IconArrowDown,
  IconZoomScan
} from '@tabler/icons-react';
import { PlanDagNodeType } from '../../../../types/plan-dag';

interface AdvancedToolbarProps {
  readonly?: boolean;
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  onNodePointerDragStart: (event: React.MouseEvent, nodeType: PlanDagNodeType) => void;
  onAutoLayoutHorizontal: () => void;
  onAutoLayoutVertical: () => void;
  onFitView: () => void;
}

const isTauri = !!(window as any).__TAURI__;

export const AdvancedToolbar: React.FC<AdvancedToolbarProps> = ({
  readonly = false,
  onNodeDragStart,
  onNodePointerDragStart,
  onAutoLayoutHorizontal,
  onAutoLayoutVertical,
  onFitView,
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
    <Group gap="xs" p="md" style={{ borderBottom: '1px solid #e9ecef' }}>
      {/* Node Palette */}
      <Group gap={4}>
        <Text size="xs" fw={500} c="gray.6">Nodes:</Text>
        {nodeTypes.map((nodeType) => (
          <Tooltip key={nodeType.type} label={`Drag to add ${nodeType.label}`}>
            <ActionIcon
              size="sm"
              variant="light"
              style={{ backgroundColor: nodeType.color, color: 'white', cursor: 'grab' }}
              draggable={!isTauri}
              onDragStart={(event) => handleNodeDragStart(event, nodeType.type)}
              onMouseDown={(event) => handleNodePointerDragStart(event, nodeType.type)}
            >
              {nodeType.icon}
            </ActionIcon>
          </Tooltip>
        ))}
      </Group>

      {/* Auto-Layout Operations */}
      <Divider orientation="vertical" />
      <Text size="xs" c="dimmed">Auto-layout:</Text>
      <Group gap="xs">
        <Tooltip label="Auto-layout Horizontal" position="bottom">
          <ActionIcon variant="subtle" color="blue" onClick={onAutoLayoutHorizontal}>
            <IconArrowRight size="1rem" />
          </ActionIcon>
        </Tooltip>

        <Tooltip label="Auto-layout Vertical" position="bottom">
          <ActionIcon variant="subtle" color="blue" onClick={onAutoLayoutVertical}>
            <IconArrowDown size="1rem" />
          </ActionIcon>
        </Tooltip>
      </Group>

      {/* Fit View */}
      <Divider orientation="vertical" />
      <Tooltip label="Fit View (Zoom to see all nodes)" position="bottom">
        <ActionIcon variant="subtle" color="gray" onClick={onFitView}>
          <IconZoomScan size="1rem" />
        </ActionIcon>
      </Tooltip>
    </Group>
  );
};