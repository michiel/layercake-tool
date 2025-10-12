import { ConnectionLineComponentProps, getBezierPath } from 'reactflow';
import { PlanDagNodeType } from '../../../../types/plan-dag';

export const ConnectionLine = ({
  fromX,
  fromY,
  toX,
  toY,
  fromNode,
}: ConnectionLineComponentProps) => {
  // Determine connection type based on source node type
  const sourceNodeType = fromNode?.data?.nodeType as PlanDagNodeType | undefined;

  let strokeColor = '#868e96'; // Default grey
  let label = '';

  if (sourceNodeType === PlanDagNodeType.GRAPH) {
    strokeColor = '#228be6'; // Blue for Graph Reference
    label = 'Graph Ref';
  } else if (sourceNodeType) {
    strokeColor = '#10b981'; // Green for Data
    label = 'Data';
  }

  const [edgePath] = getBezierPath({
    sourceX: fromX,
    sourceY: fromY,
    targetX: toX,
    targetY: toY,
  });

  return (
    <g>
      <path
        fill="none"
        stroke={strokeColor}
        strokeWidth={2}
        d={edgePath}
        strokeDasharray="5,5"
        style={{
          animation: 'dashdraw 0.5s linear infinite',
        }}
      />
      {label && (
        <text
          x={toX - 50}
          y={toY - 15}
          fill={strokeColor}
          fontSize="12"
          fontWeight="600"
          style={{
            pointerEvents: 'none',
            userSelect: 'none',
          }}
        >
          {label}
        </text>
      )}
      <style>{`
        @keyframes dashdraw {
          from {
            stroke-dashoffset: 10;
          }
          to {
            stroke-dashoffset: 0;
          }
        }
      `}</style>
    </g>
  );
};
