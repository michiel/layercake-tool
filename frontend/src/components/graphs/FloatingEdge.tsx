import React, { useCallback, useState, useRef, useEffect } from 'react';
import { useStore, getStraightPath, EdgeProps, Node, EdgeLabelRenderer } from 'reactflow';

interface GetFloatingEdgeParams {
  sourceNode: Node;
  targetNode: Node;
}

interface FloatingEdgeProps extends EdgeProps {
  onLabelChange?: (edgeId: string, newLabel: string) => void;
}

function getNodeIntersection(intersectionNode: Node, targetNode: Node) {
  const {
    width: intersectionNodeWidth,
    height: intersectionNodeHeight,
    positionAbsolute: intersectionNodePosition,
  } = intersectionNode;
  const targetPosition = targetNode.positionAbsolute;

  // Fallback to center position if dimensions are missing
  if (!intersectionNodePosition || !targetPosition) {
    return { x: 0, y: 0 };
  }

  // Use default dimensions if not provided
  const nodeWidth = intersectionNodeWidth || 170;
  const nodeHeight = intersectionNodeHeight || 50;

  const w = nodeWidth / 2;
  const h = nodeHeight / 2;

  const x2 = intersectionNodePosition.x + w;
  const y2 = intersectionNodePosition.y + h;
  const x1 = targetPosition.x + (targetNode.width || 170) / 2;
  const y1 = targetPosition.y + (targetNode.height || 50) / 2;

  const xx1 = (x1 - x2) / (2 * w) - (y1 - y2) / (2 * h);
  const yy1 = (x1 - x2) / (2 * w) + (y1 - y2) / (2 * h);
  const a = 1 / (Math.abs(xx1) + Math.abs(yy1));
  const xx3 = a * xx1;
  const yy3 = a * yy1;
  const x = w * (xx3 + yy3) + x2;
  const y = h * (-xx3 + yy3) + y2;

  return { x, y };
}

function getFloatingEdgeParams({ sourceNode, targetNode }: GetFloatingEdgeParams) {
  const sourceIntersectionPoint = getNodeIntersection(sourceNode, targetNode);
  const targetIntersectionPoint = getNodeIntersection(targetNode, sourceNode);

  return {
    sx: sourceIntersectionPoint.x,
    sy: sourceIntersectionPoint.y,
    tx: targetIntersectionPoint.x,
    ty: targetIntersectionPoint.y,
  };
}

export const FloatingEdge: React.FC<FloatingEdgeProps> = ({
  id,
  source,
  target,
  markerEnd,
  style,
  label: initialLabel,
  data,
  selected,
  onLabelChange
}) => {
  const sourceNode = useStore(useCallback((store) => store.nodeInternals.get(source), [source]));
  const targetNode = useStore(useCallback((store) => store.nodeInternals.get(target), [target]));

  const [isEditing, setIsEditing] = useState(false);
  const [label, setLabel] = useState<string>(String(initialLabel || data?.label || ''));
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setLabel(String(initialLabel || data?.label || ''));
  }, [initialLabel, data?.label]);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  if (!sourceNode || !targetNode) {
    return null;
  }

  const { sx, sy, tx, ty } = getFloatingEdgeParams({
    sourceNode,
    targetNode,
  });

  const [edgePath] = getStraightPath({
    sourceX: sx,
    sourceY: sy,
    targetX: tx,
    targetY: ty,
  });

  // Calculate label position at the center of the edge
  const labelX = (sx + tx) / 2;
  const labelY = (sy + ty) / 2;

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsEditing(true);
  };

  const handleSave = () => {
    const currentLabel = String(initialLabel || data?.label || '');
    if (onLabelChange && label !== currentLabel) {
      onLabelChange(id, label);
    }
    setIsEditing(false);
  };

  const handleCancel = () => {
    setLabel(String(initialLabel || data?.label || ''));
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    }
  };

  const handleBlur = () => {
    handleCancel();
  };

  const handleInputClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  };

  return (
    <>
      <g className="react-flow__edge">
        <path
          id={id}
          className="react-flow__edge-path"
          d={edgePath}
          markerEnd={markerEnd}
          style={style}
        />
      </g>
      {(label || isEditing) && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              pointerEvents: 'all',
              backgroundColor: isEditing ? '#fff' : 'rgba(255, 255, 255, 0.9)',
              padding: '2px 6px',
              borderRadius: '3px',
              fontSize: '10px',
              border: isEditing ? '2px solid #1a73e8' : selected ? '2px solid #1a73e8' : '1px solid transparent',
              cursor: isEditing ? 'text' : 'pointer',
            }}
            onDoubleClick={handleDoubleClick}
          >
            {isEditing ? (
              <input
                ref={inputRef}
                type="text"
                value={label}
                onChange={(e) => setLabel(e.target.value)}
                onKeyDown={handleKeyDown}
                onBlur={handleBlur}
                onClick={handleInputClick}
                style={{
                  border: 'none',
                  background: 'transparent',
                  outline: 'none',
                  font: 'inherit',
                  color: '#000',
                  padding: 0,
                  width: `${Math.max(50, label.length * 7)}px`,
                }}
              />
            ) : (
              <div>{label}</div>
            )}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
};
