import React from 'react';
import { Group, ActionIcon, Tooltip, Text, Badge, Divider } from '@mantine/core';
import {
  IconCopy,
  IconClipboard,
  IconScissors,
  IconTrash,
  IconCopyPlus,
  IconSelectAll,
  IconClick,
  IconAlignLeft,
  IconAlignRight,
  IconBoxAlignTop,
  IconBoxAlignBottom,
  IconAlignCenter,
  IconLayoutDistributeHorizontal,
  IconLayoutDistributeVertical,
  IconDatabase,
  IconNetwork,
  IconTransform,
  IconGitMerge,
  IconFileExport,
  IconArrowRight,
  IconArrowDown,
  IconZoomScan
} from '@tabler/icons-react';
import { PlanDagNodeType } from '../../../../types/plan-dag';

interface AdvancedToolbarProps {
  selectedNodeCount: number;
  hasClipboardData: boolean;
  clipboardInfo: any;
  canAlign: boolean;
  canDistribute: boolean;
  readonly?: boolean;
  onNodeDragStart: (event: React.DragEvent, nodeType: PlanDagNodeType) => void;
  onNodePointerDragStart: (event: React.MouseEvent, nodeType: PlanDagNodeType) => void;
  onDuplicate: () => void;
  onCopy: () => void;
  onPaste: () => void;
  onCut: () => void;
  onDelete: () => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onAlignLeft: () => void;
  onAlignRight: () => void;
  onAlignTop: () => void;
  onAlignBottom: () => void;
  onAlignCenterHorizontal: () => void;
  onAlignCenterVertical: () => void;
  onDistributeHorizontal: () => void;
  onDistributeVertical: () => void;
  onAutoLayoutHorizontal: () => void;
  onAutoLayoutVertical: () => void;
  onFitView: () => void;
}

const isTauri = !!(window as any).__TAURI__;

export const AdvancedToolbar: React.FC<AdvancedToolbarProps> = ({
  selectedNodeCount,
  hasClipboardData,
  clipboardInfo,
  canAlign,
  canDistribute,
  readonly = false,
  onNodeDragStart,
  onNodePointerDragStart,
  onDuplicate,
  onCopy,
  onPaste,
  onCut,
  onDelete,
  onSelectAll,
  onDeselectAll,
  onAlignLeft,
  onAlignRight,
  onAlignTop,
  onAlignBottom,
  onAlignCenterHorizontal,
  onAlignCenterVertical,
  onDistributeHorizontal,
  onDistributeVertical,
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

      <Divider orientation="vertical" />

      {/* Selection Info */}
      {selectedNodeCount > 0 && (
        <>
          <Badge variant="light" size="sm">
            {selectedNodeCount} selected
          </Badge>
          <Divider orientation="vertical" />
        </>
      )}

      {/* Basic Operations */}
      <Group gap="xs">
        <Tooltip label="Duplicate (Ctrl+D)" position="bottom">
          <ActionIcon
            variant="subtle"
            disabled={selectedNodeCount === 0}
            onClick={onDuplicate}
          >
            <IconCopyPlus size="1rem" />
          </ActionIcon>
        </Tooltip>

        <Tooltip label="Copy (Ctrl+C)" position="bottom">
          <ActionIcon
            variant="subtle"
            disabled={selectedNodeCount === 0}
            onClick={onCopy}
          >
            <IconCopy size="1rem" />
          </ActionIcon>
        </Tooltip>

        <Tooltip
          label={
            hasClipboardData
              ? `Paste (Ctrl+V) - ${clipboardInfo?.nodeCount || 0} nodes`
              : "Paste (Ctrl+V)"
          }
          position="bottom"
        >
          <ActionIcon
            variant="subtle"
            disabled={!hasClipboardData}
            onClick={onPaste}
            color={hasClipboardData ? "blue" : undefined}
          >
            <IconClipboard size="1rem" />
          </ActionIcon>
        </Tooltip>

        <Tooltip label="Cut (Ctrl+X)" position="bottom">
          <ActionIcon
            variant="subtle"
            disabled={selectedNodeCount === 0}
            onClick={onCut}
          >
            <IconScissors size="1rem" />
          </ActionIcon>
        </Tooltip>

        <Tooltip label="Delete (Del)" position="bottom">
          <ActionIcon
            variant="subtle"
            disabled={selectedNodeCount === 0}
            onClick={onDelete}
            color="red"
          >
            <IconTrash size="1rem" />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Divider orientation="vertical" />

      {/* Selection Operations */}
      <Group gap="xs">
        <Tooltip label="Select All (Ctrl+A)" position="bottom">
          <ActionIcon variant="subtle" onClick={onSelectAll}>
            <IconSelectAll size="1rem" />
          </ActionIcon>
        </Tooltip>

        <Tooltip label="Deselect All (Esc)" position="bottom">
          <ActionIcon
            variant="subtle"
            disabled={selectedNodeCount === 0}
            onClick={onDeselectAll}
          >
            <IconClick size="1rem" />
          </ActionIcon>
        </Tooltip>
      </Group>

      {/* Alignment Operations */}
      {canAlign && (
        <>
          <Divider orientation="vertical" />
          <Text size="xs" c="dimmed">Align:</Text>
          <Group gap="xs">
            <Tooltip label="Align Left (Ctrl+Shift+←)" position="bottom">
              <ActionIcon variant="subtle" onClick={onAlignLeft}>
                <IconAlignLeft size="1rem" />
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Align Center Vertically" position="bottom">
              <ActionIcon variant="subtle" onClick={onAlignCenterVertical}>
                <IconAlignCenter size="1rem" style={{ transform: 'rotate(90deg)' }} />
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Align Right (Ctrl+Shift+→)" position="bottom">
              <ActionIcon variant="subtle" onClick={onAlignRight}>
                <IconAlignRight size="1rem" />
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Align Top (Ctrl+Shift+↑)" position="bottom">
              <ActionIcon variant="subtle" onClick={onAlignTop}>
                <IconBoxAlignTop size="1rem" />
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Align Center Horizontally" position="bottom">
              <ActionIcon variant="subtle" onClick={onAlignCenterHorizontal}>
                <IconAlignCenter size="1rem" />
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Align Bottom (Ctrl+Shift+↓)" position="bottom">
              <ActionIcon variant="subtle" onClick={onAlignBottom}>
                <IconBoxAlignBottom size="1rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        </>
      )}

      {/* Distribution Operations */}
      {canDistribute && (
        <>
          <Divider orientation="vertical" />
          <Text size="xs" c="dimmed">Distribute:</Text>
          <Group gap="xs">
            <Tooltip label="Distribute Horizontally" position="bottom">
              <ActionIcon variant="subtle" onClick={onDistributeHorizontal}>
                <IconLayoutDistributeHorizontal size="1rem" />
              </ActionIcon>
            </Tooltip>

            <Tooltip label="Distribute Vertically" position="bottom">
              <ActionIcon variant="subtle" onClick={onDistributeVertical}>
                <IconLayoutDistributeVertical size="1rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        </>
      )}

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