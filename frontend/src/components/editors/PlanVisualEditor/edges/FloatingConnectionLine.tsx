import { ConnectionLineComponentProps, getBezierPath, Node } from 'reactflow';
import { getEdgeParams } from './edgeUtils';

/**
 * FloatingConnectionLine - Connection line preview shown during edge creation.
 *
 * Uses bezier paths to match the FloatingEdge style and dynamically calculates
 * the connection point on the source node boundary based on cursor position.
 */
export function FloatingConnectionLine({
  toX,
  toY,
  fromNode,
}: ConnectionLineComponentProps) {
  if (!fromNode) {
    return null;
  }

  // Create a virtual target node at the cursor position
  const targetNode = {
    id: 'connection-target',
    [Symbol.for('rf_internals')]: {
      width: 1,
      height: 1,
      positionAbsolute: { x: toX, y: toY },
    },
  } as unknown as Node;

  // Calculate where to start the connection from
  const { sx, sy, tx, ty, sourcePos, targetPos } = getEdgeParams(fromNode, targetNode);

  // Create the bezier path to the cursor (matches ReactFlow floating edges example)
  const [edgePath] = getBezierPath({
    sourceX: sx,
    sourceY: sy,
    sourcePosition: sourcePos,
    targetX: tx || toX,
    targetY: ty || toY,
    targetPosition: targetPos,
  });

  return (
    <g>
      <path
        fill="none"
        stroke="#222"
        strokeWidth={2}
        strokeDasharray="5,5"
        d={edgePath}
      />
      <circle
        cx={tx || toX}
        cy={ty || toY}
        fill="#fff"
        r={4}
        stroke="#222"
        strokeWidth={2}
      />
    </g>
  );
}
