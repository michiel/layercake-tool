import { useStore, getBezierPath, EdgeProps, Node, BaseEdge, EdgeLabelRenderer } from 'reactflow';
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
  style,
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
  const [edgePath, labelX, labelY] = getBezierPath({
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
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        markerStart={markerStart}
        style={{
          ...style,
          stroke: edgeColor,
          strokeWidth,
        }}
      />
      {/* Optional: Add edge label rendering */}
      {data?.metadata?.label && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              fontSize: 10,
              fontWeight: 500,
              background: '#fff',
              padding: '2px 4px',
              borderRadius: 3,
              border: '1px solid #ccc',
              pointerEvents: 'all',
            }}
            className="nodrag nopan"
          >
            {data.metadata.label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
