import React, { useEffect, useCallback, useMemo, useRef } from 'react';
import ReactFlow, { Controls, Background, MiniMap, useNodesState, useEdgesState, useReactFlow, BackgroundVariant, Node, Edge } from 'reactflow';

import 'reactflow/dist/style.css';
import '../../styles/reactFlow.css'; // Custom styles

import { Graph } from '../../graphql/graphs';
import { getLayoutedElements } from '../../utils/graphUtils';
import { GroupNode } from './GroupNode';
import { FloatingEdge } from './FloatingEdge';

type GraphViewMode = 'flow' | 'hierarchy';
type GraphOrientation = 'vertical' | 'horizontal';

interface LayercakeGraphEditorProps {
  graph: Graph;
  onNodeSelect?: (nodeId: string | null) => void;
  layerVisibility?: Map<string, boolean>;
  onNodesInitialized?: (setNodes: React.Dispatch<React.SetStateAction<Node[]>>, setEdges: React.Dispatch<React.SetStateAction<Edge[]>>) => void;
  mode?: GraphViewMode;
  orientation?: GraphOrientation;
  groupingEnabled?: boolean;
  fitViewTrigger?: number;
  wrapperRef?: React.RefObject<HTMLDivElement | null>;
  nodeSpacing?: number;
  rankSpacing?: number;
  minEdgeLength?: number;
}

export const LayercakeGraphEditor: React.FC<LayercakeGraphEditorProps> = ({
  graph,
  onNodeSelect,
  layerVisibility,
  onNodesInitialized,
  mode = 'flow',
  orientation = 'vertical',
  groupingEnabled = true,
  fitViewTrigger,
  wrapperRef,
  nodeSpacing = 75,
  rankSpacing = 75,
  minEdgeLength = 50,
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
  }, [graph, mode, groupingEnabled]);

  const onLayout = useCallback(
    async (shouldFitView: boolean) => {
      if (!renderGraph || !renderGraph.graphNodes || !renderGraph.graphEdges) return;

      const layouted = await getLayoutedElements(
        renderGraph,
        renderGraph.layers,
        170,
        50,
        {
          disableSubflows: mode === 'hierarchy' || (mode === 'flow' && !groupingEnabled),
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
    [renderGraph, setNodes, setEdges, fitView, orientation, mode, groupingEnabled, nodeSpacing, rankSpacing, minEdgeLength]
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
