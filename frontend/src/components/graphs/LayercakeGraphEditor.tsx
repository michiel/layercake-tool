import React, { useEffect, useCallback, useMemo, useRef } from 'react';
import ReactFlow, { Controls, Background, MiniMap, useNodesState, useEdgesState, useReactFlow, BackgroundVariant, Node, Edge } from 'reactflow';

import 'reactflow/dist/style.css';
import '../../styles/reactFlow.css'; // Custom styles

import { Graph } from '../../graphql/graphs';
import { getLayoutedElements } from '../../utils/graphUtils';
import { GroupNode } from './GroupNode';
import { FloatingEdge } from './FloatingEdge';

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
}) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const { fitView } = useReactFlow();
  const isInitialLoad = useRef(true);
  const selectedNodeIdsRef = useRef<string[]>([]);
  const prevFitViewTriggerRef = useRef<number | undefined>(undefined);

  const nodeTypes = useMemo(() => ({ group: GroupNode }), []);
  const edgeTypes = useMemo(() => ({ floating: FloatingEdge }), []);

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
      setEdges(layouted.edges);

      if (shouldFitView || isInitialLoad.current) {
        window.requestAnimationFrame(() => {
          fitView({ padding: 0.15, minZoom: 0.02 });
          isInitialLoad.current = false;
        });
      }
    },
    [renderGraph, setNodes, setEdges, fitView, orientation, mode, groupingEnabled, hierarchyViewMode, nodeSpacing, rankSpacing, minEdgeLength]
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
  const handleSelectionChange = useCallback(({ nodes }: { nodes: Node[] }) => {
    // Filter out label nodes (they end with '-label')
    const selectedNodes = nodes.filter(node => !node.id.endsWith('-label'));

    // Update ref to preserve selection across re-layouts
    selectedNodeIdsRef.current = selectedNodes.map(node => node.id);

    if (onNodeSelect) {
      if (selectedNodes.length > 0) {
        onNodeSelect(selectedNodes[0].id);
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

    // Find the graph node to get its original belongsTo
    const graphNode = renderGraph.graphNodes.find(gn => gn.id === node.id);
    if (!graphNode) return;

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

    const originalParent = graphNode.belongsTo || undefined;

    // Check if parent changed
    if (newParent !== originalParent) {
      const nodeType = node.type === 'group' ? 'Subflow' : 'Node';
      console.log(`${nodeType} ${node.id} parent changed from ${originalParent || 'root'} to ${newParent || 'root'}`);
      // Update belongsTo relationship
      onNodeUpdate(node.id, {
        belongsTo: newParent || ''  // Empty string means no parent (root level)
      });
    }
  }, [mode, hierarchyViewMode, onNodeUpdate, renderGraph, nodes]);

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

  return (
    <div style={{ width: '100%', height: '100%' }} ref={wrapperRef}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
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
