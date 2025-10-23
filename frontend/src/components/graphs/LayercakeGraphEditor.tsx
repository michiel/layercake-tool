import React, { useEffect, useCallback, useMemo, useRef, useState } from 'react';
import ReactFlow, {
  Controls,
  Background,
  MiniMap,
  useNodesState,
  useEdgesState,
  useReactFlow,
  BackgroundVariant,
  Node,
  Edge,
  Connection,
  EdgeChange,
  MarkerType,
} from 'reactflow';

import 'reactflow/dist/style.css';
import '../../styles/reactFlow.css'; // Custom styles

import { Graph } from '../../graphql/graphs';
import { getLayoutedElements } from '../../utils/graphUtils';
import { GroupNode } from './GroupNode';
import { FloatingEdge } from './FloatingEdge';
import { EditableNode } from './EditableNode';
import { EditableLabelNode } from './EditableLabelNode';

export type GraphViewMode = 'flow' | 'hierarchy';
export type GraphOrientation = 'vertical' | 'horizontal';
export type HierarchyViewMode = 'graph' | 'containers';

interface LayercakeGraphEditorProps {
  graph: Graph;
  onNodeSelect?: (nodeId: string | null) => void;
  layerVisibility?: Map<string, boolean>;
  onNodesInitialized?: (setNodes: React.Dispatch<React.SetStateAction<Node[]>>, setEdges: React.Dispatch<React.SetStateAction<Edge[]>>) => void;
  mode?: GraphViewMode;
  orientation?: GraphOrientation;
  groupingEnabled?: boolean;
  hierarchyViewMode?: HierarchyViewMode;
  fitViewTrigger?: number;
  wrapperRef?: React.RefObject<HTMLDivElement | null>;
  nodeSpacing?: number;
  rankSpacing?: number;
  minEdgeLength?: number;
  onNodeUpdate?: (nodeId: string, updates: { belongsTo?: string }) => void;
  onNodeAdd?: (node: Node) => void;
  onNodeDelete?: (nodeId: string) => void;
  onEdgeAdd?: (edge: Edge) => void;
  onEdgeDelete?: (edgeId: string) => void;
  onNodeLabelChange?: (nodeId: string, newLabel: string) => void;
  onEdgeLabelChange?: (edgeId: string, newLabel: string) => void;
}

export const LayercakeGraphEditor: React.FC<LayercakeGraphEditorProps> = ({
  graph,
  onNodeSelect,
  layerVisibility,
  onNodesInitialized,
  mode = 'flow',
  orientation = 'vertical',
  groupingEnabled = true,
  hierarchyViewMode = 'graph',
  fitViewTrigger,
  wrapperRef,
  nodeSpacing = 75,
  rankSpacing = 75,
  minEdgeLength = 50,
  onNodeUpdate,
  onNodeAdd,
  onNodeDelete,
  onEdgeAdd,
  onEdgeDelete,
  onNodeLabelChange,
  onEdgeLabelChange,
}) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const { fitView, screenToFlowPosition } = useReactFlow();
  const isInitialLoad = useRef(true);
  const selectedNodeIdsRef = useRef<string[]>([]);
  const prevFitViewTriggerRef = useRef<number | undefined>(undefined);
  const manualEdgesRef = useRef<Edge[]>([]);
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);
  const [selectedEdges, setSelectedEdges] = useState<string[]>([]);

  const nodeTypes = useMemo(() => ({
    group: GroupNode,
    editable: (props: any) => <EditableNode {...props} onLabelChange={onNodeLabelChange} />,
    labelNode: (props: any) => <EditableLabelNode {...props} onLabelChange={onNodeLabelChange} />,
  }), [onNodeLabelChange]);

  const edgeTypes = useMemo(() => ({
    floating: (props: any) => <FloatingEdge {...props} onLabelChange={onEdgeLabelChange} />,
  }), [onEdgeLabelChange]);

  const mergeEdgesWithManual = useCallback((baseEdges: Edge[]): Edge[] => {
    if (mode !== 'flow' || manualEdgesRef.current.length === 0) {
      return baseEdges;
    }

    const connectionKey = (edge: Edge) =>
      `${edge.source || ''}::${edge.sourceHandle || ''}->${edge.target || ''}::${edge.targetHandle || ''}`;

    const baseConnectionKeys = new Set(baseEdges.map(connectionKey));
    const survivingManualEdges = manualEdgesRef.current.filter(edge => {
      const key = connectionKey(edge);
      if (baseConnectionKeys.has(key)) {
        return false;
      }
      return true;
    });

    manualEdgesRef.current = survivingManualEdges;

    if (survivingManualEdges.length === 0) {
      return baseEdges;
    }

    const baseEdgeIds = new Set(baseEdges.map(edge => edge.id));
    const manualToAdd = survivingManualEdges.filter(edge => !baseEdgeIds.has(edge.id));

    return manualToAdd.length > 0 ? [...baseEdges, ...manualToAdd] : baseEdges;
  }, [mode]);

  const renderGraph = useMemo(() => {
    if (!graph) {
      return graph;
    }

    if (mode === 'hierarchy') {
      if (hierarchyViewMode === 'containers') {
        // Show as containers: preserve isPartition (like flow), but remove edges
        return {
          ...graph,
          graphEdges: [], // No edges in containers view
        };
      } else {
        // Show as graph: convert hierarchy to edges, flatten partitions
        const normalizeParentId = (belongsTo?: string | null) =>
          belongsTo ? belongsTo.trim() : undefined;

        const nodeMap = new Map(
          graph.graphNodes.map(node => [node.id, node])
        );

        const hierarchyNodes = graph.graphNodes.map(node => ({
          ...node,
          isPartition: false,
        }));

        const hierarchyEdges = graph.graphNodes
          .map(node => ({
            node,
            parentId: normalizeParentId(node.belongsTo),
          }))
          .filter(({ parentId }) => parentId && nodeMap.has(parentId))
          .map(({ node, parentId }) => ({
            id: `hierarchy-${parentId}-${node.id}`,
            source: parentId as string,
            target: node.id,
            label: '',
            layer: undefined,
            weight: undefined,
            attrs: undefined,
          }));

        return {
          ...graph,
          graphNodes: hierarchyNodes,
          graphEdges: hierarchyEdges,
        };
      }
    }

    if (mode === 'flow' && !groupingEnabled) {
      const filteredNodes = graph.graphNodes.filter(node => !node.isPartition);
      const allowedIds = new Set(filteredNodes.map(node => node.id));

      const filteredEdges = graph.graphEdges.filter(
        edge => allowedIds.has(edge.source) && allowedIds.has(edge.target)
      );

      return {
        ...graph,
        graphNodes: filteredNodes,
        graphEdges: filteredEdges,
      };
    }

    return graph;
  }, [graph, mode, groupingEnabled, hierarchyViewMode]);

  const onLayout = useCallback(
    async (shouldFitView: boolean) => {
      if (!renderGraph || !renderGraph.graphNodes || !renderGraph.graphEdges) return;

      const layouted = await getLayoutedElements(
        renderGraph,
        renderGraph.layers,
        170,
        50,
        {
          disableSubflows: (mode === 'hierarchy' && hierarchyViewMode === 'graph') || (mode === 'flow' && !groupingEnabled),
          orientation,
          nodeSpacing,
          rankSpacing,
          minEdgeLength,
        }
      );

      // Restore selection state from ref (preserved across re-renders)
      const nodesWithSelection = layouted.nodes.map(node => ({
        ...node,
        selected: selectedNodeIdsRef.current.includes(node.id),
      }));

      setNodes(nodesWithSelection);
      setEdges(mergeEdgesWithManual(layouted.edges));

      if (shouldFitView || isInitialLoad.current) {
        window.requestAnimationFrame(() => {
          fitView({ padding: 0.15, minZoom: 0.02 });
          isInitialLoad.current = false;
        });
      }
    },
    [renderGraph, setNodes, setEdges, fitView, orientation, mode, groupingEnabled, hierarchyViewMode, nodeSpacing, rankSpacing, minEdgeLength, mergeEdgesWithManual]
  );

  useEffect(() => {
    onLayout(false);
  }, [renderGraph, onLayout]);

  useEffect(() => {
    if (fitViewTrigger !== undefined && fitViewTrigger !== prevFitViewTriggerRef.current) {
      prevFitViewTriggerRef.current = fitViewTrigger;
      onLayout(true);
    }
  }, [fitViewTrigger, onLayout]);

  // Notify parent that nodes/edges are ready for manipulation
  useEffect(() => {
    if (onNodesInitialized && nodes.length > 0) {
      onNodesInitialized(setNodes, setEdges);
    }
  }, [nodes.length, onNodesInitialized]);

  // Update hidden property when visibility changes (no re-layout!)
  useEffect(() => {
    if (!layerVisibility || layerVisibility.size === 0) return;
    if (!renderGraph) return;

    setNodes(currentNodes => {
      return currentNodes.map(node => {
        // Skip label nodes
        if (node.id.endsWith('-label')) {
          return node;
        }

        // Find the corresponding graph node to get layer info
        const graphNode = renderGraph.graphNodes.find(gn => gn.id === node.id);
        if (!graphNode) return node;

        const shouldHide = graphNode.layer
          ? layerVisibility.get(graphNode.layer) === false
          : false;

        return { ...node, hidden: shouldHide };
      });
    });

    setEdges(currentEdges => {
      return currentEdges.map(edge => {
        const graphEdge = renderGraph.graphEdges.find(ge => ge.id === edge.id);
        if (!graphEdge) return edge;

        // Hide edge if its layer is hidden
        const edgeLayerHidden = graphEdge.layer
          ? layerVisibility.get(graphEdge.layer) === false
          : false;

        // Hide edge if source or target is hidden
        const sourceNode = renderGraph.graphNodes.find(n => n.id === graphEdge.source);
        const targetNode = renderGraph.graphNodes.find(n => n.id === graphEdge.target);

        const sourceHidden = sourceNode?.layer
          ? layerVisibility.get(sourceNode.layer) === false
          : false;
        const targetHidden = targetNode?.layer
          ? layerVisibility.get(targetNode.layer) === false
          : false;

        const shouldHide = edgeLayerHidden || sourceHidden || targetHidden;

        return { ...edge, hidden: shouldHide };
      });
    });
  }, [layerVisibility, renderGraph, setNodes, setEdges]);

  // Handle node selection
  const handleSelectionChange = useCallback(({ nodes, edges }: { nodes: Node[]; edges: Edge[] }) => {
    // Filter out label nodes (they end with '-label')
    const filteredNodes = nodes.filter(node => !node.id.endsWith('-label'));

    // Update ref to preserve selection across re-layouts
    selectedNodeIdsRef.current = filteredNodes.map(node => node.id);

    // Track selected nodes and edges for delete
    setSelectedNodes(filteredNodes.map(n => n.id));
    setSelectedEdges(edges.map(e => e.id));

    if (onNodeSelect) {
      if (filteredNodes.length > 0) {
        onNodeSelect(filteredNodes[0].id);
      } else {
        onNodeSelect(null);
      }
    }
  }, [onNodeSelect]);

  // Handle node/subflow drag stop to detect parent group changes
  const handleNodeDragStop = useCallback((_event: React.MouseEvent, node: Node) => {
    // Only apply drag reparenting in Flow mode or Hierarchy containers mode
    if (mode === 'hierarchy' && hierarchyViewMode !== 'containers') return;
    if (mode !== 'flow' && mode !== 'hierarchy') return;

    if (!onNodeUpdate || !renderGraph) return;

    // Skip only label nodes
    if (node.id.endsWith('-label')) return;

    // Find the graph node to get its original belongsTo (may be undefined for manual nodes)
    const graphNode = renderGraph.graphNodes.find(gn => gn.id === node.id);

    // Helper to get absolute position of a node (accounting for parent offsets)
    const getAbsolutePosition = (n: Node): { x: number; y: number } => {
      let absX = n.position.x;
      let absY = n.position.y;
      let currentNode = n;

      // Traverse up parent chain to get absolute coordinates
      while (currentNode.parentNode) {
        const parent = nodes.find(p => p.id === currentNode.parentNode);
        if (!parent) break;
        absX += parent.position.x;
        absY += parent.position.y;
        currentNode = parent;
      }

      return { x: absX, y: absY };
    };

    // Get absolute position of the dropped node
    const nodeAbsPos = getAbsolutePosition(node);
    const nodeRect = {
      x: nodeAbsPos.x,
      y: nodeAbsPos.y,
      width: (node.width as number) || 170,
      height: (node.height as number) || 50,
    };

    // Check center point of node for containment
    const nodeCenterX = nodeRect.x + nodeRect.width / 2;
    const nodeCenterY = nodeRect.y + nodeRect.height / 2;

    // Find all group nodes and check if node center is inside any of them
    const groupNodes = nodes.filter(n => n.type === 'group');

    // Helper to check if a node is a descendant of another
    const isDescendantOf = (childId: string, ancestorId: string): boolean => {
      let current = nodes.find(n => n.id === childId);
      while (current?.parentNode) {
        if (current.parentNode === ancestorId) return true;
        current = nodes.find(n => n.id === current!.parentNode);
      }
      return false;
    };

    // Find the deepest (most nested) group that contains this node
    let newParent: string | undefined = undefined;
    let maxDepth = -1;

    for (const group of groupNodes) {
      // Skip self and descendants (can't drop a group inside itself or its children)
      if (group.id === node.id || isDescendantOf(group.id, node.id)) continue;

      const groupAbsPos = getAbsolutePosition(group);
      const groupRect = {
        x: groupAbsPos.x,
        y: groupAbsPos.y,
        width: (group.width as number) || (group.style?.width as number) || 0,
        height: (group.height as number) || (group.style?.height as number) || 0,
      };

      // Check if node center is within group bounds
      const isInside =
        nodeCenterX >= groupRect.x &&
        nodeCenterX <= groupRect.x + groupRect.width &&
        nodeCenterY >= groupRect.y &&
        nodeCenterY <= groupRect.y + groupRect.height;

      if (isInside) {
        // Calculate depth (count parents)
        let depth = 0;
        let currentGroup = group;
        while (currentGroup.parentNode) {
          depth++;
          currentGroup = nodes.find(n => n.id === currentGroup.parentNode) || currentGroup;
          if (!currentGroup.parentNode) break;
        }

        // Use deepest group
        if (depth > maxDepth) {
          maxDepth = depth;
          newParent = group.id;
        }
      }
    }

    const originalParent = graphNode?.belongsTo || undefined;

    // Check if parent changed (or if this is a manual node being reparented for the first time)
    if (newParent !== originalParent) {
      const nodeType = node.type === 'group' ? 'Subflow' : 'Node';
      console.log(`${nodeType} ${node.id} parent changed from ${originalParent || 'root'} to ${newParent || 'root'}`);

      // Update local state to set parent immediately
      setNodes(nds => nds.map(n => {
        if (n.id === node.id) {
          return {
            ...n,
            parentNode: newParent,
            extent: newParent ? ('parent' as const) : undefined,
            data: {
              ...n.data,
              belongsTo: newParent,
            },
          };
        }
        return n;
      }));

      // Update backend only if the node exists in the backend graph
      if (graphNode) {
        onNodeUpdate(node.id, {
          belongsTo: newParent || ''  // Empty string means no parent (root level)
        });
      }
    }
  }, [mode, hierarchyViewMode, onNodeUpdate, renderGraph, nodes, setNodes]);

  // Minimap node color customization
  const minimapNodeColor = useCallback((node: Node) => {
    // Hide label nodes completely
    if (node.id.endsWith('-label')) {
      return 'transparent';
    }
    // Make group nodes semi-transparent so child nodes are visible
    if (node.type === 'group') {
      return 'rgba(200, 200, 200, 0.2)';
    }
    // Regular nodes are solid white
    return '#fff';
  }, []);

  // Minimap node stroke color
  const minimapNodeStrokeColor = useCallback((node: Node) => {
    // Hide label nodes completely
    if (node.id.endsWith('-label')) {
      return 'transparent';
    }
    // Semi-transparent border for partition nodes
    if (node.type === 'group') {
      return 'rgba(150, 150, 150, 0.5)';
    }
    // Solid border for regular nodes
    return '#555';
  }, []);

  const handleEdgesChange = useCallback((changes: EdgeChange[]) => {
    onEdgesChange(changes);

    if (!changes || changes.length === 0) {
      return;
    }

    const removedIds = changes
      .filter(change => change.type === 'remove')
      .map(change => change.id)
      .filter((id): id is string => typeof id === 'string');

    if (removedIds.length > 0) {
      // Clean up manual edges bookkeeping
      manualEdgesRef.current = manualEdgesRef.current.filter(edge => !removedIds.includes(edge.id));

      // Persist deletions server-side
      if (onEdgeDelete) {
        removedIds.forEach(edgeId => {
          onEdgeDelete(edgeId);
        });
      }
    }
  }, [onEdgesChange, onEdgeDelete]);

  const handleConnect = useCallback((connection: Connection) => {
    if (mode !== 'flow') return;

    const { source, target } = connection;
    if (!source || !target) return;

    const generateEdgeId = () => {
      if (typeof globalThis !== 'undefined' && globalThis.crypto?.randomUUID) {
        return `manual-edge-${globalThis.crypto.randomUUID()}`;
      }

      return `manual-edge-${Date.now()}-${Math.random().toString(16).slice(2)}`;
    };

    setEdges(currentEdges => {
      const alreadyExists = currentEdges.some(edge =>
        edge.source === source &&
        edge.target === target &&
        (edge.sourceHandle ?? null) === (connection.sourceHandle ?? null) &&
        (edge.targetHandle ?? null) === (connection.targetHandle ?? null)
      );

      if (alreadyExists) {
        return currentEdges;
      }

      const newEdge: Edge = {
        id: generateEdgeId(),
        source,
        target,
        sourceHandle: connection.sourceHandle,
        targetHandle: connection.targetHandle,
        type: 'floating',
        markerEnd: { type: MarkerType.ArrowClosed },
        style: {
          zIndex: 10,
          strokeWidth: 2,
          stroke: '#b1b1b7',
        },
        data: {},
      };

      manualEdgesRef.current = [...manualEdgesRef.current, newEdge];

      // Persist the edge server-side
      if (onEdgeAdd) {
        onEdgeAdd(newEdge);
      }

      return [...currentEdges, newEdge];
    });
  }, [mode, setEdges, onEdgeAdd]);

  const handleDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const type = event.dataTransfer.getData('application/reactflow');
      if (type !== 'node') return;

      const nodeType = event.dataTransfer.getData('nodeType');
      const isPartition = nodeType === 'container';

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const generateNodeId = () => {
        if (typeof globalThis !== 'undefined' && globalThis.crypto?.randomUUID) {
          return `manual-node-${globalThis.crypto.randomUUID()}`;
        }
        return `manual-node-${Date.now()}-${Math.random().toString(16).slice(2)}`;
      };

      // Find parent container if drop position is inside one
      let parentNode: string | undefined;
      let relativePosition = position;

      const containerNodes = nodes.filter(n => n.type === 'group');
      for (const container of containerNodes) {
        const containerWidth = (container.width || container.style?.width || 200) as number;
        const containerHeight = (container.height || container.style?.height || 200) as number;

        // Check if drop position is within container bounds
        if (
          position.x >= container.position.x &&
          position.x <= container.position.x + containerWidth &&
          position.y >= container.position.y &&
          position.y <= container.position.y + containerHeight
        ) {
          parentNode = container.id;
          // Make position relative to parent
          relativePosition = {
            x: position.x - container.position.x,
            y: position.y - container.position.y,
          };
          break; // Use the first matching container
        }
      }

      const newNode: Node = {
        id: generateNodeId(),
        type: isPartition ? 'group' : 'default',
        position: relativePosition,
        data: {
          label: isPartition ? 'New Container' : 'New Node',
          isPartition,
          belongsTo: parentNode,
        },
        style: {
          width: isPartition ? 200 : undefined,
          height: isPartition ? 200 : undefined,
        },
        // Set top-level width/height for containers so drag detection works
        ...(isPartition ? {
          width: 200,
          height: 200,
        } : {}),
        ...(parentNode ? {
          parentNode,
          extent: 'parent' as const,
        } : {}),
      };

      setNodes((nds) => nds.concat(newNode));

      // Persist the node server-side
      if (onNodeAdd) {
        onNodeAdd(newNode);
      }
    },
    [screenToFlowPosition, setNodes, onNodeAdd, nodes]
  );

  // Handle delete key for selected nodes and edges
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Check if Delete or Backspace is pressed
      if (event.key === 'Delete' || event.key === 'Backspace') {
        // Prevent default backspace navigation
        event.preventDefault();

        // Delete selected nodes
        if (selectedNodes.length > 0 && onNodeDelete) {
          selectedNodes.forEach(nodeId => {
            onNodeDelete(nodeId);
          });
          // Remove from local state
          setNodes(nds => nds.filter(n => !selectedNodes.includes(n.id)));
          setSelectedNodes([]);
        }

        // Delete selected edges
        if (selectedEdges.length > 0 && onEdgeDelete) {
          selectedEdges.forEach(edgeId => {
            onEdgeDelete(edgeId);
          });
          // Remove from local state
          setEdges(eds => eds.filter(e => !selectedEdges.includes(e.id)));
          setSelectedEdges([]);
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [selectedNodes, selectedEdges, onNodeDelete, onEdgeDelete, setNodes, setEdges]);

  return (
    <div
      style={{ width: '100%', height: '100%' }}
      ref={wrapperRef}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
    >
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={handleEdgesChange}
        onConnect={handleConnect}
        onSelectionChange={handleSelectionChange}
        onNodeDragStop={handleNodeDragStop}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        attributionPosition="bottom-left"
      >
        <Controls />
        <MiniMap
          nodeColor={minimapNodeColor}
          nodeStrokeColor={minimapNodeStrokeColor}
          nodeStrokeWidth={1}
          maskColor="rgba(240, 240, 240, 0.8)"
        />
        <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
      </ReactFlow>
    </div>
  );
};
