import { Node, Position, internalsSymbol } from 'reactflow';

interface Point {
  x: number;
  y: number;
}

interface EdgeParams {
  sx: number;  // source x
  sy: number;  // source y
  tx: number;  // target x
  ty: number;  // target y
  sourcePos: Position;
  targetPos: Position;
}

type NodeWithInternals = Node & {
  [internalsSymbol]?: {
    width?: number;
    height?: number;
    positionAbsolute?: { x: number; y: number };
  };
};

function getInternals(node: NodeWithInternals) {
  return node[internalsSymbol] || {};
}

function getNodeDimensions(node: NodeWithInternals) {
  const internals = getInternals(node);

  const width = internals.width ?? (node as any).width ?? 0;
  const height = internals.height ?? (node as any).height ?? 0;
  const positionAbsolute =
    internals.positionAbsolute ??
    {
      x: node.position?.x ?? 0,
      y: node.position?.y ?? 0,
    };

  return { width, height, positionAbsolute };
}

/**
 * Calculate the intersection point between a line from one node center to another
 * and the boundary of the intersection node.
 *
 * This uses vector mathematics to find where a line from the target node center
 * to the intersection node center crosses the intersection node's boundary.
 */
export function getNodeIntersection(
  intersectionNode: NodeWithInternals,
  targetNode: NodeWithInternals
): Point {
  const intersectionDimensions = getNodeDimensions(intersectionNode);
  const targetDimensions = getNodeDimensions(targetNode);

  // Calculate half-widths for easier math
  const w = (intersectionDimensions.width ?? 0) / 2;
  const h = (intersectionDimensions.height ?? 0) / 2;

  // Calculate center points of both nodes
  const x2 = (intersectionDimensions.positionAbsolute?.x ?? 0) + w;
  const y2 = (intersectionDimensions.positionAbsolute?.y ?? 0) + h;
  const x1 = (targetDimensions.positionAbsolute?.x ?? 0) + ((targetDimensions.width ?? 0) / 2);
  const y1 = (targetDimensions.positionAbsolute?.y ?? 0) + ((targetDimensions.height ?? 0) / 2);

  // Calculate the slope of the line between node centers
  // Using diamond-space transformation for axis-aligned intersection
  const xx1 = (x1 - x2) / (2 * w) - (y1 - y2) / (2 * h);
  const yy1 = (x1 - x2) / (2 * w) + (y1 - y2) / (2 * h);
  const a = 1 / (Math.abs(xx1) + Math.abs(yy1));
  const xx3 = a * xx1;
  const yy3 = a * yy1;
  const x = w * (xx3 + yy3) + x2;
  const y = h * (-xx3 + yy3) + y2;

  return { x, y };
}

/**
 * Determine which side (position) of a node an intersection point is closest to.
 * Returns the ReactFlow Position enum value (Top, Right, Bottom, Left).
 */
export function getEdgePosition(node: NodeWithInternals, intersectionPoint: Point): Position {
  const { positionAbsolute, width, height } = getNodeDimensions(node);

  const n = {
    x: positionAbsolute?.x ?? 0,
    y: positionAbsolute?.y ?? 0,
    width: width ?? 0,
    height: height ?? 0,
  };
  const nx = Math.round(n.x);
  const ny = Math.round(n.y);
  const px = Math.round(intersectionPoint.x);
  const py = Math.round(intersectionPoint.y);

  // Determine which edge of the node the intersection is closest to
  // Using 1px threshold to account for rounding
  if (px <= nx + 1) {
    return Position.Left;
  }
  if (px >= nx + n.width - 1) {
    return Position.Right;
  }
  if (py <= ny + 1) {
    return Position.Top;
  }
  if (py >= ny + n.height - 1) {
    return Position.Bottom;
  }

  return Position.Top; // Default fallback
}

/**
 * Calculate all edge parameters needed to render a floating edge between two nodes.
 * Returns source/target coordinates and positions for edge rendering.
 */
export function getEdgeParams(source: NodeWithInternals, target: NodeWithInternals): EdgeParams {
  const sourceIntersectionPoint = getNodeIntersection(source, target);
  const targetIntersectionPoint = getNodeIntersection(target, source);

  const sourcePos = getEdgePosition(source, sourceIntersectionPoint);
  const targetPos = getEdgePosition(target, targetIntersectionPoint);

  return {
    sx: sourceIntersectionPoint.x,
    sy: sourceIntersectionPoint.y,
    tx: targetIntersectionPoint.x,
    ty: targetIntersectionPoint.y,
    sourcePos,
    targetPos,
  };
}
