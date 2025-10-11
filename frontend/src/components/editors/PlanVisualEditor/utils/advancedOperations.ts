import { Node, Edge } from 'reactflow';
import { generateNodeId } from './nodeDefaults';

/**
 * Advanced operations for DAG Editor nodes and edges
 */

// Clipboard interface for copy/paste operations
interface ClipboardData {
  type: 'nodes' | 'mixed';
  nodes: Node[];
  edges: Edge[];
  timestamp: number;
}

// Global clipboard (in a real app, this might be in a context or store)
let clipboard: ClipboardData | null = null;

/**
 * Duplicates a node with a new ID and slightly offset position
 */
export const duplicateNode = (node: Node, allNodes: Node[]): Node => {
  const existingNodeIds = allNodes.map(n => n.id);
  const newId = generateNodeId(node.data.nodeType, existingNodeIds);

  return {
    ...node,
    id: newId,
    position: {
      x: node.position.x + 50, // Offset by 50px to the right
      y: node.position.y + 50, // Offset by 50px down
    },
    data: {
      ...node.data,
      metadata: {
        ...node.data.metadata,
        label: `${node.data.metadata.label} (Copy)`,
        description: `Copy of ${node.data.metadata.description || node.data.metadata.label}`
      }
    },
    selected: false // Don't select the duplicate by default
  };
};

/**
 * Duplicates multiple nodes while preserving their relative positions
 */
export const duplicateNodes = (nodes: Node[], allNodes: Node[]): Node[] => {
  if (nodes.length === 0) return [];

  // Calculate the bounding box of selected nodes
  const minX = Math.min(...nodes.map(n => n.position.x));
  const minY = Math.min(...nodes.map(n => n.position.y));

  // Get existing IDs (including nodes being duplicated)
  const existingNodeIds = allNodes.map(n => n.id);

  // Create a mapping from old IDs to new IDs
  const idMapping = new Map<string, string>();
  nodes.forEach((node) => {
    // Pass accumulated IDs to ensure each new ID is unique
    const allIdsIncludingNew = [...existingNodeIds, ...Array.from(idMapping.values())];
    idMapping.set(node.id, generateNodeId(node.data.nodeType, allIdsIncludingNew));
  });

  return nodes.map(node => {
    const newId = idMapping.get(node.id)!;
    return {
      ...node,
      id: newId,
      position: {
        x: node.position.x - minX + minX + 50, // Offset the group by 50px
        y: node.position.y - minY + minY + 50,
      },
      data: {
        ...node.data,
        metadata: {
          ...node.data.metadata,
          label: `${node.data.metadata.label} (Copy)`,
          description: `Copy of ${node.data.metadata.description || node.data.metadata.label}`
        }
      },
      selected: false
    };
  });
};

/**
 * Copies selected nodes and edges to clipboard
 */
export const copyToClipboard = (selectedNodes: Node[], selectedEdges: Edge[], allEdges: Edge[]) => {
  if (selectedNodes.length === 0) return false;

  const selectedNodeIds = new Set(selectedNodes.map(n => n.id));

  // Include only edges that connect selected nodes
  const relevantEdges = allEdges.filter(edge =>
    selectedNodeIds.has(edge.source) && selectedNodeIds.has(edge.target)
  );

  clipboard = {
    type: selectedEdges.length > 0 ? 'mixed' : 'nodes',
    nodes: selectedNodes,
    edges: relevantEdges,
    timestamp: Date.now()
  };

  return true;
};

/**
 * Pastes nodes and edges from clipboard
 */
export const pasteFromClipboard = (allNodes: Node[]): { nodes: Node[], edges: Edge[] } | null => {
  if (!clipboard || Date.now() - clipboard.timestamp > 300000) { // 5 minute timeout
    return null;
  }

  // Get existing IDs
  const existingNodeIds = allNodes.map(n => n.id);

  // Create new IDs for pasted nodes
  const idMapping = new Map<string, string>();
  clipboard.nodes.forEach((node) => {
    // Pass accumulated IDs to ensure each new ID is unique
    const allIdsIncludingNew = [...existingNodeIds, ...Array.from(idMapping.values())];
    idMapping.set(node.id, generateNodeId(node.data.nodeType, allIdsIncludingNew));
  });

  const pastedNodes = clipboard.nodes.map(node => {
    const newId = idMapping.get(node.id)!;
    return {
      ...node,
      id: newId,
      position: {
        x: node.position.x + 100, // Offset pasted nodes
        y: node.position.y + 100,
      },
      data: {
        ...node.data,
        metadata: {
          ...node.data.metadata,
          label: `${node.data.metadata.label} (Pasted)`,
          description: `Pasted copy of ${node.data.metadata.description || node.data.metadata.label}`
        }
      },
      selected: true // Select pasted nodes
    };
  });

  const pastedEdges = clipboard.edges.map(edge => ({
    ...edge,
    id: `${idMapping.get(edge.source)!}-${idMapping.get(edge.target)!}`,
    source: idMapping.get(edge.source)!,
    target: idMapping.get(edge.target)!,
    selected: true
  }));

  return { nodes: pastedNodes, edges: pastedEdges };
};

/**
 * Checks if clipboard has data available
 */
export const hasClipboardData = (): boolean => {
  return clipboard !== null && Date.now() - clipboard.timestamp <= 300000;
};

/**
 * Clears the clipboard
 */
export const clearClipboard = () => {
  clipboard = null;
};

/**
 * Gets information about clipboard content
 */
export const getClipboardInfo = () => {
  if (!hasClipboardData()) return null;

  return {
    nodeCount: clipboard!.nodes.length,
    edgeCount: clipboard!.edges.length,
    type: clipboard!.type,
    age: Date.now() - clipboard!.timestamp
  };
};

/**
 * Selects all nodes in the canvas
 */
export const selectAllNodes = (nodes: Node[]): Node[] => {
  return nodes.map(node => ({ ...node, selected: true }));
};

/**
 * Deselects all nodes and edges
 */
export const deselectAll = (nodes: Node[], edges: Edge[]): { nodes: Node[], edges: Edge[] } => {
  return {
    nodes: nodes.map(node => ({ ...node, selected: false })),
    edges: edges.map(edge => ({ ...edge, selected: false }))
  };
};

/**
 * Deletes selected nodes and their connected edges
 */
export const deleteSelectedNodes = (
  nodes: Node[],
  edges: Edge[],
  selectedNodeIds: string[]
): { nodes: Node[], edges: Edge[] } => {
  const nodeIdSet = new Set(selectedNodeIds);

  return {
    nodes: nodes.filter(node => !nodeIdSet.has(node.id)),
    edges: edges.filter(edge => !nodeIdSet.has(edge.source) && !nodeIdSet.has(edge.target))
  };
};

/**
 * Gets bounding box of selected nodes
 */
export const getSelectionBounds = (selectedNodes: Node[]) => {
  if (selectedNodes.length === 0) return null;

  const minX = Math.min(...selectedNodes.map(n => n.position.x));
  const maxX = Math.max(...selectedNodes.map(n => n.position.x + (n.width || 150)));
  const minY = Math.min(...selectedNodes.map(n => n.position.y));
  const maxY = Math.max(...selectedNodes.map(n => n.position.y + (n.height || 60)));

  return {
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
    centerX: (minX + maxX) / 2,
    centerY: (minY + maxY) / 2
  };
};

/**
 * Aligns selected nodes horizontally
 */
export const alignNodesHorizontally = (selectedNodes: Node[], alignment: 'top' | 'center' | 'bottom'): Node[] => {
  if (selectedNodes.length < 2) return selectedNodes;

  const bounds = getSelectionBounds(selectedNodes);
  if (!bounds) return selectedNodes;

  let targetY: number;
  switch (alignment) {
    case 'top':
      targetY = bounds.y;
      break;
    case 'center':
      targetY = bounds.centerY - 30; // Assuming 60px node height
      break;
    case 'bottom':
      targetY = bounds.y + bounds.height - 60;
      break;
  }

  return selectedNodes.map(node => ({
    ...node,
    position: { ...node.position, y: targetY }
  }));
};

/**
 * Aligns selected nodes vertically
 */
export const alignNodesVertically = (selectedNodes: Node[], alignment: 'left' | 'center' | 'right'): Node[] => {
  if (selectedNodes.length < 2) return selectedNodes;

  const bounds = getSelectionBounds(selectedNodes);
  if (!bounds) return selectedNodes;

  let targetX: number;
  switch (alignment) {
    case 'left':
      targetX = bounds.x;
      break;
    case 'center':
      targetX = bounds.centerX - 75; // Assuming 150px node width
      break;
    case 'right':
      targetX = bounds.x + bounds.width - 150;
      break;
  }

  return selectedNodes.map(node => ({
    ...node,
    position: { ...node.position, x: targetX }
  }));
};

/**
 * Distributes selected nodes evenly
 */
export const distributeNodes = (selectedNodes: Node[], direction: 'horizontal' | 'vertical'): Node[] => {
  if (selectedNodes.length < 3) return selectedNodes;

  const sortedNodes = [...selectedNodes].sort((a, b) =>
    direction === 'horizontal'
      ? a.position.x - b.position.x
      : a.position.y - b.position.y
  );

  const first = sortedNodes[0];
  const last = sortedNodes[sortedNodes.length - 1];
  const totalDistance = direction === 'horizontal'
    ? last.position.x - first.position.x
    : last.position.y - first.position.y;

  const step = totalDistance / (sortedNodes.length - 1);

  return sortedNodes.map((node, index) => {
    if (index === 0 || index === sortedNodes.length - 1) return node;

    const newPosition = direction === 'horizontal'
      ? { ...node.position, x: first.position.x + step * index }
      : { ...node.position, y: first.position.y + step * index };

    return { ...node, position: newPosition };
  });
};