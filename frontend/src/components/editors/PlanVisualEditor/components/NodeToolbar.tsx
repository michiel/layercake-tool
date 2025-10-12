import React from 'react';
import { Group, Paper, Text, ActionIcon, Tooltip } from '@mantine/core';
import {
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconGitMerge,
  IconCopy,
  IconFileExport
} from '@tabler/icons-react';
import { PlanDagNodeType } from '../../../../types/plan-dag';

interface DraggableNodeProps {
  type: PlanDagNodeType;
  label: string;
  icon: React.ReactNode;
  color: string;
  onDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
}

const DraggableNode: React.FC<DraggableNodeProps> = ({ type, label, icon, color, onDragStart }) => {
  const handleDragStart = (event: React.DragEvent) => {
    onDragStart(event, type);
  };

  return (
    <Tooltip label={`Drag to add ${label}`} position="bottom">
      <Paper
        p="sm"
        radius="md"
        bg={color}
        style={{
          cursor: 'grab',
          minWidth: '60px',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          gap: '4px',
          ':hover': {
            opacity: 0.8
          }
        }}
        draggable
        onDragStart={handleDragStart}
      >
        <ActionIcon
          variant="transparent"
          size="lg"
          color="white"
          style={{ pointerEvents: 'none' }}
        >
          {icon}
        </ActionIcon>
        <Text
          size="xs"
          c="white"
          fw={500}
          ta="center"
          style={{ pointerEvents: 'none', lineHeight: 1 }}
        >
          {label}
        </Text>
      </Paper>
    </Tooltip>
  );
};

interface NodeToolbarProps {
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  readonly?: boolean;
}

export const NodeToolbar: React.FC<NodeToolbarProps> = ({ onNodeDragStart, readonly = false }) => {
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
    <Paper p="md" shadow="sm" bg="gray.0" style={{ borderBottom: '1px solid #e9ecef' }}>
      <Group gap="xs" justify="flex-start">
        <Text size="sm" fw={500} c="gray.7" mr="md">
          Drag nodes to canvas:
        </Text>
        {nodeTypes.map((nodeType) => (
          <DraggableNode
            key={nodeType.type}
            type={nodeType.type}
            label={nodeType.label}
            icon={nodeType.icon}
            color={nodeType.color}
            onDragStart={onNodeDragStart}
          />
        ))}
      </Group>
    </Paper>
  );
};