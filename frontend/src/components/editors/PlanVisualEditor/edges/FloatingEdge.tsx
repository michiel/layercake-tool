import { useStore, getBezierPath, EdgeProps, Node } from 'reactflow';
import { getEdgeParams } from './edgeUtils';

// Selector for accessing nodes from ReactFlow store
const selector = (s: any) => ({
  nodeInternals: s.nodeInternals,
  edges: s.edges,
});

interface FloatingEdgeData {
  metadata?: {
    label?: string;
    dataType?: string;
  };
  originalEdge?: any;
}

interface FloatingEdgeProps extends EdgeProps {
  data?: FloatingEdgeData;
}

/**
 * FloatingEdge - A ReactFlow edge that dynamically connects to nodes at their boundaries.
 *
 * Uses bezier paths for smooth, natural-looking connections that automatically adjust
 * based on node positions. The edge connects at optimal points on node boundaries
 * rather than fixed handle positions.
 */
export function FloatingEdge({
  id,
  source,
  target,
  markerEnd,
  markerStart,
  data,
  selected,
}: FloatingEdgeProps) {
  // Access ReactFlow store to get current node positions
  const { nodeInternals } = useStore(selector);

  const sourceNode = nodeInternals.get(source) as Node;
  const targetNode = nodeInternals.get(target) as Node;

  // If nodes don't exist yet, don't render the edge
  if (!sourceNode || !targetNode) {
    return null;
  }

  // Calculate dynamic edge parameters
  const { sx, sy, tx, ty, sourcePos, targetPos } = getEdgeParams(
    sourceNode,
    targetNode
  );

  // Generate bezier path (matches ReactFlow floating edges example)
  const [edgePath] = getBezierPath({
    sourceX: sx,
    sourceY: sy,
    sourcePosition: sourcePos,
    targetX: tx,
    targetY: ty,
    targetPosition: targetPos,
  });

  // Apply styling based on edge data type (maintaining current behavior)
  const edgeColor = data?.metadata?.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96';
  const strokeWidth = selected ? 3 : 2;

  return (
    <g>
      {/* Invisible wider path for better interaction */}
      <path
        id={`${id}-interaction`}
        d={edgePath}
        fill="none"
        stroke="transparent"
        strokeWidth={20}
        className="react-flow__edge-interaction"
        style={{ cursor: 'grab' }}
      />
      {/* Visible edge path */}
      <path
        id={id}
        d={edgePath}
        fill="none"
        stroke={edgeColor}
        strokeWidth={strokeWidth}
        markerEnd={markerEnd as string}
        markerStart={markerStart as string}
        className="react-flow__edge-path"
        style={{ pointerEvents: 'none' }}
      />
    </g>
  );
}
