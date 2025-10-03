import { useCallback, useEffect, useRef } from 'react';
import { Node, Edge } from 'reactflow';
import {
  duplicateNode,
  duplicateNodes,
  copyToClipboard,
  pasteFromClipboard,
  hasClipboardData,
  selectAllNodes,
  deselectAll,
  deleteSelectedNodes,
  alignNodesHorizontally,
  alignNodesVertically,
  distributeNodes,
  getClipboardInfo,
} from '../utils/advancedOperations';

interface UseAdvancedOperationsProps {
  nodes: Node[];
  edges: Edge[];
  setNodes: (nodes: Node[] | ((nodes: Node[]) => Node[])) => void;
  setEdges: (edges: Edge[] | ((edges: Edge[]) => Edge[])) => void;
  readonly?: boolean;
  onDeleteNodes?: (nodeIds: string[]) => void;
  onDeleteEdges?: (edgeIds: string[]) => void;
}

export const useAdvancedOperations = ({
  nodes,
  edges,
  setNodes,
  setEdges,
  readonly = false,
  onDeleteNodes,
  onDeleteEdges
}: UseAdvancedOperationsProps) => {
  const keysPressedRef = useRef(new Set<string>());

  // Get selected nodes and edges
  const selectedNodes = nodes.filter(node => node.selected);
  const selectedEdges = edges.filter(edge => edge.selected);
  const selectedNodeIds = selectedNodes.map(node => node.id);

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (readonly) return;

    keysPressedRef.current.add(event.key.toLowerCase());

    const isCtrlOrCmd = event.ctrlKey || event.metaKey;
    const isShift = event.shiftKey;
    const key = event.key.toLowerCase();

    // Prevent default for our handled shortcuts
    const shouldPreventDefault = [
      'c', 'v', 'x', 'd', 'a', 'delete', 'backspace'
    ].includes(key) && isCtrlOrCmd;

    if (shouldPreventDefault) {
      event.preventDefault();
    }

    // Copy (Ctrl+C)
    if (isCtrlOrCmd && key === 'c' && selectedNodes.length > 0) {
      handleCopy();
    }

    // Paste (Ctrl+V)
    if (isCtrlOrCmd && key === 'v') {
      handlePaste();
    }

    // Cut (Ctrl+X)
    if (isCtrlOrCmd && key === 'x' && selectedNodes.length > 0) {
      handleCut();
    }

    // Duplicate (Ctrl+D)
    if (isCtrlOrCmd && key === 'd' && selectedNodes.length > 0) {
      event.preventDefault();
      handleDuplicate();
    }

    // Select All (Ctrl+A)
    if (isCtrlOrCmd && key === 'a') {
      event.preventDefault();
      handleSelectAll();
    }

    // Delete (Delete or Backspace)
    if ((key === 'delete' || key === 'backspace') && selectedNodes.length > 0) {
      event.preventDefault();
      handleDelete();
    }

    // Deselect (Escape)
    if (key === 'escape') {
      handleDeselectAll();
    }

    // Alignment shortcuts (Ctrl+Shift+Arrow)
    if (isCtrlOrCmd && isShift && selectedNodes.length >= 2) {
      switch (key) {
        case 'arrowup':
          event.preventDefault();
          handleAlignTop();
          break;
        case 'arrowdown':
          event.preventDefault();
          handleAlignBottom();
          break;
        case 'arrowleft':
          event.preventDefault();
          handleAlignLeft();
          break;
        case 'arrowright':
          event.preventDefault();
          handleAlignRight();
          break;
      }
    }
  }, [readonly, selectedNodes, selectedEdges]);

  const handleKeyUp = useCallback((event: KeyboardEvent) => {
    keysPressedRef.current.delete(event.key.toLowerCase());
  }, []);

  // Setup keyboard event listeners
  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keyup', handleKeyUp);
    };
  }, [handleKeyDown, handleKeyUp]);

  // Operation handlers
  const handleDuplicate = useCallback(() => {
    if (readonly || selectedNodes.length === 0) return;

    const duplicatedNodes = selectedNodes.length === 1
      ? [duplicateNode(selectedNodes[0])]
      : duplicateNodes(selectedNodes);

    setNodes(currentNodes => {
      // Deselect all current nodes
      const deselectedNodes = currentNodes.map(node => ({ ...node, selected: false }));
      // Add duplicated nodes
      return [...deselectedNodes, ...duplicatedNodes];
    });

    // Successfully duplicated nodes
  }, [readonly, selectedNodes, setNodes]);

  const handleCopy = useCallback(() => {
    if (readonly || selectedNodes.length === 0) return;

    const success = copyToClipboard(selectedNodes, selectedEdges, edges);
    if (success) {
      // Successfully copied to clipboard
    }
  }, [readonly, selectedNodes, selectedEdges, edges]);

  const handlePaste = useCallback(() => {
    if (readonly || !hasClipboardData()) return;

    const clipboardData = pasteFromClipboard();
    if (!clipboardData) {
      // Paste failed: No valid clipboard data available
      return;
    }

    setNodes(currentNodes => {
      // Deselect all current nodes
      const deselectedNodes = currentNodes.map(node => ({ ...node, selected: false }));
      return [...deselectedNodes, ...clipboardData.nodes];
    });

    setEdges(currentEdges => {
      const deselectedEdges = currentEdges.map(edge => ({ ...edge, selected: false }));
      return [...deselectedEdges, ...clipboardData.edges];
    });

    // Successfully pasted from clipboard
  }, [readonly, setNodes, setEdges]);

  const handleCut = useCallback(() => {
    if (readonly || selectedNodes.length === 0) return;

    handleCopy();
    handleDelete();
  }, [readonly, selectedNodes, handleCopy]);

  const handleDelete = useCallback(() => {
    if (readonly || selectedNodes.length === 0) return;

    const { nodes: newNodes, edges: newEdges } = deleteSelectedNodes(nodes, edges, selectedNodeIds);

    // Get IDs of edges that will be deleted
    const deletedEdgeIds = edges
      .filter(edge => !newEdges.find(e => e.id === edge.id))
      .map(edge => edge.id);

    // Update local state optimistically
    setNodes(newNodes);
    setEdges(newEdges);

    // Persist deletions to backend
    if (onDeleteNodes && selectedNodeIds.length > 0) {
      onDeleteNodes(selectedNodeIds);
    }
    if (onDeleteEdges && deletedEdgeIds.length > 0) {
      onDeleteEdges(deletedEdgeIds);
    }

    console.log(`Deleted ${selectedNodeIds.length} node(s) and ${deletedEdgeIds.length} edge(s)`);
  }, [readonly, selectedNodes, nodes, edges, selectedNodeIds, setNodes, setEdges, onDeleteNodes, onDeleteEdges]);

  const handleSelectAll = useCallback(() => {
    if (readonly) return;

    setNodes(currentNodes => selectAllNodes(currentNodes));

    console.log(`Selected all ${nodes.length} node(s)`);
  }, [readonly, nodes.length, setNodes]);

  const handleDeselectAll = useCallback(() => {
    const { nodes: deselectedNodes, edges: deselectedEdges } = deselectAll(nodes, edges);
    setNodes(deselectedNodes);
    setEdges(deselectedEdges);
  }, [nodes, edges, setNodes, setEdges]);

  // Alignment handlers
  const handleAlignTop = useCallback(() => {
    if (readonly || selectedNodes.length < 2) return;

    const alignedNodes = alignNodesHorizontally(selectedNodes, 'top');
    setNodes(currentNodes =>
      currentNodes.map(node => {
        const aligned = alignedNodes.find(n => n.id === node.id);
        return aligned || node;
      })
    );
  }, [readonly, selectedNodes, setNodes]);

  const handleAlignBottom = useCallback(() => {
    if (readonly || selectedNodes.length < 2) return;

    const alignedNodes = alignNodesHorizontally(selectedNodes, 'bottom');
    setNodes(currentNodes =>
      currentNodes.map(node => {
        const aligned = alignedNodes.find(n => n.id === node.id);
        return aligned || node;
      })
    );
  }, [readonly, selectedNodes, setNodes]);

  const handleAlignLeft = useCallback(() => {
    if (readonly || selectedNodes.length < 2) return;

    const alignedNodes = alignNodesVertically(selectedNodes, 'left');
    setNodes(currentNodes =>
      currentNodes.map(node => {
        const aligned = alignedNodes.find(n => n.id === node.id);
        return aligned || node;
      })
    );
  }, [readonly, selectedNodes, setNodes]);

  const handleAlignRight = useCallback(() => {
    if (readonly || selectedNodes.length < 2) return;

    const alignedNodes = alignNodesVertically(selectedNodes, 'right');
    setNodes(currentNodes =>
      currentNodes.map(node => {
        const aligned = alignedNodes.find(n => n.id === node.id);
        return aligned || node;
      })
    );
  }, [readonly, selectedNodes, setNodes]);

  const handleAlignCenter = useCallback((direction: 'horizontal' | 'vertical') => {
    if (readonly || selectedNodes.length < 2) return;

    const alignedNodes = direction === 'horizontal'
      ? alignNodesHorizontally(selectedNodes, 'center')
      : alignNodesVertically(selectedNodes, 'center');

    setNodes(currentNodes =>
      currentNodes.map(node => {
        const aligned = alignedNodes.find(n => n.id === node.id);
        return aligned || node;
      })
    );
  }, [readonly, selectedNodes, setNodes]);

  const handleDistribute = useCallback((direction: 'horizontal' | 'vertical') => {
    if (readonly || selectedNodes.length < 3) return;

    const distributedNodes = distributeNodes(selectedNodes, direction);
    setNodes(currentNodes =>
      currentNodes.map(node => {
        const distributed = distributedNodes.find(n => n.id === node.id);
        return distributed || node;
      })
    );
  }, [readonly, selectedNodes, setNodes]);

  return {
    // State
    selectedNodes,
    selectedEdges,
    hasClipboardData: hasClipboardData(),
    clipboardInfo: getClipboardInfo(),

    // Operations
    handleDuplicate,
    handleCopy,
    handlePaste,
    handleCut,
    handleDelete,
    handleSelectAll,
    handleDeselectAll,

    // Alignment
    handleAlignTop,
    handleAlignBottom,
    handleAlignLeft,
    handleAlignRight,
    handleAlignCenter,
    handleDistribute,

    // Utility
    canAlign: selectedNodes.length >= 2,
    canDistribute: selectedNodes.length >= 3,
    canDuplicate: selectedNodes.length > 0,
    canDelete: selectedNodes.length > 0,
  };
};