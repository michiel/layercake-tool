import React from 'react';
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
import { Group } from '../../../layout-primitives';
import { Button } from '../../../ui/button';
import { Separator } from '../../../ui/separator';

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
      <div className="min-w-[200px] rounded-md border bg-popover p-1 text-popover-foreground shadow-md">
        {/* Basic Operations */}
        {selectedNodeCount > 0 && !readonly && (
          <>
            <button
              className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
              onClick={() => handleMenuAction(onDuplicate)}
            >
              <IconCopyPlus className="mr-2 h-4 w-4" />
              <span>Duplicate {selectedNodeCount} node{selectedNodeCount > 1 ? 's' : ''}</span>
              <span className="ml-auto text-xs text-muted-foreground">Ctrl+D</span>
            </button>

            <button
              className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
              onClick={() => handleMenuAction(onCopy)}
            >
              <IconCopy className="mr-2 h-4 w-4" />
              <span>Copy</span>
              <span className="ml-auto text-xs text-muted-foreground">Ctrl+C</span>
            </button>

            <button
              className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
              onClick={() => handleMenuAction(onCut)}
            >
              <IconScissors className="mr-2 h-4 w-4" />
              <span>Cut</span>
              <span className="ml-auto text-xs text-muted-foreground">Ctrl+X</span>
            </button>

            <button
              className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm text-red-600 outline-none transition-colors hover:bg-accent hover:text-red-600 focus:bg-accent focus:text-red-600"
              onClick={() => handleMenuAction(onDelete)}
            >
              <IconTrash className="mr-2 h-4 w-4" />
              <span>Delete</span>
              <span className="ml-auto text-xs text-muted-foreground">Del</span>
            </button>

            <Separator className="my-1" />
          </>
        )}

        {/* Paste */}
        {!readonly && (
          <>
            <button
              className={`relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors ${
                hasClipboardData
                  ? 'hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground'
                  : 'cursor-not-allowed opacity-50'
              }`}
              onClick={() => hasClipboardData && handleMenuAction(onPaste)}
              disabled={!hasClipboardData}
            >
              <IconClipboard className="mr-2 h-4 w-4" />
              <span>Paste</span>
              <span className="ml-auto text-xs text-muted-foreground">Ctrl+V</span>
            </button>

            <Separator className="my-1" />
          </>
        )}

        {/* Selection Operations */}
        <button
          className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          onClick={() => handleMenuAction(onSelectAll)}
        >
          <IconSelectAll className="mr-2 h-4 w-4" />
          <span>Select All</span>
          <span className="ml-auto text-xs text-muted-foreground">Ctrl+A</span>
        </button>

        {selectedNodeCount > 0 && (
          <button
            className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
            onClick={() => handleMenuAction(onDeselectAll)}
          >
            <IconClick className="mr-2 h-4 w-4" />
            <span>Deselect All</span>
            <span className="ml-auto text-xs text-muted-foreground">Esc</span>
          </button>
        )}

        {/* Alignment Operations */}
        {canAlign && !readonly && (
          <>
            <Separator className="my-1" />
            <div className="px-2 py-1.5 text-sm font-semibold text-muted-foreground">Align Nodes</div>

            <Group gap="xs" className="p-2">
              <Button
                size="icon"
                variant="secondary"
                className="h-8 w-8"
                onClick={() => handleMenuAction(onAlignLeft)}
                title="Align Left (Ctrl+Shift+←)"
              >
                <IconAlignLeft className="h-3.5 w-3.5" />
              </Button>

              <Button
                size="icon"
                variant="secondary"
                className="h-8 w-8"
                onClick={() => handleMenuAction(onAlignCenterVertical)}
                title="Align Center Vertically"
              >
                <IconAlignCenter className="h-3.5 w-3.5" style={{ transform: 'rotate(90deg)' }} />
              </Button>

              <Button
                size="icon"
                variant="secondary"
                className="h-8 w-8"
                onClick={() => handleMenuAction(onAlignRight)}
                title="Align Right (Ctrl+Shift+→)"
              >
                <IconAlignRight className="h-3.5 w-3.5" />
              </Button>
            </Group>

            <Group gap="xs" className="p-2">
              <Button
                size="icon"
                variant="secondary"
                className="h-8 w-8"
                onClick={() => handleMenuAction(onAlignTop)}
                title="Align Top (Ctrl+Shift+↑)"
              >
                <IconBoxAlignTop className="h-3.5 w-3.5" />
              </Button>

              <Button
                size="icon"
                variant="secondary"
                className="h-8 w-8"
                onClick={() => handleMenuAction(onAlignCenterHorizontal)}
                title="Align Center Horizontally"
              >
                <IconAlignCenter className="h-3.5 w-3.5" />
              </Button>

              <Button
                size="icon"
                variant="secondary"
                className="h-8 w-8"
                onClick={() => handleMenuAction(onAlignBottom)}
                title="Align Bottom (Ctrl+Shift+↓)"
              >
                <IconBoxAlignBottom className="h-3.5 w-3.5" />
              </Button>
            </Group>
          </>
        )}

        {/* Distribution Operations */}
        {canDistribute && !readonly && (
          <>
            <div className="px-2 py-1.5 text-sm font-semibold text-muted-foreground">Distribute Nodes</div>

            <button
              className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
              onClick={() => handleMenuAction(onDistributeHorizontal)}
            >
              <IconLayoutDistributeHorizontal className="mr-2 h-4 w-4" />
              <span>Distribute Horizontally</span>
            </button>

            <button
              className="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
              onClick={() => handleMenuAction(onDistributeVertical)}
            >
              <IconLayoutDistributeVertical className="mr-2 h-4 w-4" />
              <span>Distribute Vertically</span>
            </button>
          </>
        )}
      </div>
    </div>
  );
};