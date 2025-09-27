import React from 'react';
import { Menu, ActionIcon, Group, Text, Divider } from '@mantine/core';
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
} from '@tabler/icons-react';

interface ContextMenuProps {
  opened: boolean;
  onClose: () => void;
  position: { x: number; y: number };
  selectedNodeCount: number;
  hasClipboardData: boolean;
  canAlign: boolean;
  canDistribute: boolean;
  readonly?: boolean;
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
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  opened,
  onClose,
  position,
  selectedNodeCount,
  hasClipboardData,
  canAlign,
  canDistribute,
  readonly = false,
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
}) => {
  if (!opened) return null;

  const handleMenuAction = (action: () => void) => {
    action();
    onClose();
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: position.y,
        left: position.x,
        zIndex: 1000,
        pointerEvents: 'auto',
      }}
    >
      <Menu
        opened={opened}
        onClose={onClose}
        position="bottom-start"
        withArrow={false}
        shadow="md"
        radius="md"
      >
        <Menu.Target>
          <div style={{ width: 1, height: 1 }} />
        </Menu.Target>

        <Menu.Dropdown>
          {/* Basic Operations */}
          {selectedNodeCount > 0 && !readonly && (
            <>
              <Menu.Item
                leftSection={<IconCopyPlus size="1rem" />}
                onClick={() => handleMenuAction(onDuplicate)}
                rightSection={<Text size="xs" c="dimmed">Ctrl+D</Text>}
              >
                Duplicate {selectedNodeCount} node{selectedNodeCount > 1 ? 's' : ''}
              </Menu.Item>

              <Menu.Item
                leftSection={<IconCopy size="1rem" />}
                onClick={() => handleMenuAction(onCopy)}
                rightSection={<Text size="xs" c="dimmed">Ctrl+C</Text>}
              >
                Copy
              </Menu.Item>

              <Menu.Item
                leftSection={<IconScissors size="1rem" />}
                onClick={() => handleMenuAction(onCut)}
                rightSection={<Text size="xs" c="dimmed">Ctrl+X</Text>}
              >
                Cut
              </Menu.Item>

              <Menu.Item
                leftSection={<IconTrash size="1rem" />}
                onClick={() => handleMenuAction(onDelete)}
                rightSection={<Text size="xs" c="dimmed">Del</Text>}
                color="red"
              >
                Delete
              </Menu.Item>

              <Divider />
            </>
          )}

          {/* Paste */}
          {!readonly && (
            <>
              <Menu.Item
                leftSection={<IconClipboard size="1rem" />}
                onClick={() => handleMenuAction(onPaste)}
                disabled={!hasClipboardData}
                rightSection={<Text size="xs" c="dimmed">Ctrl+V</Text>}
              >
                Paste
              </Menu.Item>

              <Divider />
            </>
          )}

          {/* Selection Operations */}
          <Menu.Item
            leftSection={<IconSelectAll size="1rem" />}
            onClick={() => handleMenuAction(onSelectAll)}
            rightSection={<Text size="xs" c="dimmed">Ctrl+A</Text>}
          >
            Select All
          </Menu.Item>

          {selectedNodeCount > 0 && (
            <Menu.Item
              leftSection={<IconClick size="1rem" />}
              onClick={() => handleMenuAction(onDeselectAll)}
              rightSection={<Text size="xs" c="dimmed">Esc</Text>}
            >
              Deselect All
            </Menu.Item>
          )}

          {/* Alignment Operations */}
          {canAlign && !readonly && (
            <>
              <Divider />
              <Menu.Label>Align Nodes</Menu.Label>

              <Group gap="xs" p="xs">
                <ActionIcon
                  size="sm"
                  variant="light"
                  onClick={() => handleMenuAction(onAlignLeft)}
                  title="Align Left (Ctrl+Shift+←)"
                >
                  <IconAlignLeft size="0.8rem" />
                </ActionIcon>

                <ActionIcon
                  size="sm"
                  variant="light"
                  onClick={() => handleMenuAction(onAlignCenterVertical)}
                  title="Align Center Vertically"
                >
                  <IconAlignCenter size="0.8rem" style={{ transform: 'rotate(90deg)' }} />
                </ActionIcon>

                <ActionIcon
                  size="sm"
                  variant="light"
                  onClick={() => handleMenuAction(onAlignRight)}
                  title="Align Right (Ctrl+Shift+→)"
                >
                  <IconAlignRight size="0.8rem" />
                </ActionIcon>
              </Group>

              <Group gap="xs" p="xs">
                <ActionIcon
                  size="sm"
                  variant="light"
                  onClick={() => handleMenuAction(onAlignTop)}
                  title="Align Top (Ctrl+Shift+↑)"
                >
                  <IconBoxAlignTop size="0.8rem" />
                </ActionIcon>

                <ActionIcon
                  size="sm"
                  variant="light"
                  onClick={() => handleMenuAction(onAlignCenterHorizontal)}
                  title="Align Center Horizontally"
                >
                  <IconAlignCenter size="0.8rem" />
                </ActionIcon>

                <ActionIcon
                  size="sm"
                  variant="light"
                  onClick={() => handleMenuAction(onAlignBottom)}
                  title="Align Bottom (Ctrl+Shift+↓)"
                >
                  <IconBoxAlignBottom size="0.8rem" />
                </ActionIcon>
              </Group>
            </>
          )}

          {/* Distribution Operations */}
          {canDistribute && !readonly && (
            <>
              <Menu.Label>Distribute Nodes</Menu.Label>

              <Menu.Item
                leftSection={<IconLayoutDistributeHorizontal size="1rem" />}
                onClick={() => handleMenuAction(onDistributeHorizontal)}
              >
                Distribute Horizontally
              </Menu.Item>

              <Menu.Item
                leftSection={<IconLayoutDistributeVertical size="1rem" />}
                onClick={() => handleMenuAction(onDistributeVertical)}
              >
                Distribute Vertically
              </Menu.Item>
            </>
          )}
        </Menu.Dropdown>
      </Menu>
    </div>
  );
};