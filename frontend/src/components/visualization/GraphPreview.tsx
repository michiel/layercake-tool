import { useEffect, useRef } from 'react';
import ForceGraph from 'force-graph';

export interface GraphNode {
  id: string;
  name: string;
  layer: string;
  attrs: Record<string, string>;
}

export interface GraphLink {
  id: string;
  source: string;
  target: string;
  name: string;
  layer: string;
  attrs: Record<string, string>;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

interface GraphPreviewProps {
  data: GraphData;
  width?: number;
  height?: number;
}

export const GraphPreview = ({ data, width, height }: GraphPreviewProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<any>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Clear any existing graph
    if (graphRef.current) {
      graphRef.current._destructor();
    }

    // Initialize force-graph
    const graph = (ForceGraph as any)()(containerRef.current)
      .width(width || containerRef.current.clientWidth)
      .height(height || containerRef.current.clientHeight)
      .graphData(data)
      .nodeId('id')
      .nodeLabel('name')
      .nodeAutoColorBy('layer')
      .nodeCanvasObject((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const label = node.name;
        const fontSize = 12 / globalScale;
        ctx.font = `${fontSize}px Sans-Serif`;
        const textWidth = ctx.measureText(label).width;
        const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2);

        // Draw node circle
        ctx.fillStyle = node.color;
        ctx.beginPath();
        ctx.arc(node.x, node.y, 5, 0, 2 * Math.PI, false);
        ctx.fill();

        // Draw text background
        ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
        ctx.fillRect(
          node.x - bckgDimensions[0] / 2,
          node.y - bckgDimensions[1] / 2,
          bckgDimensions[0],
          bckgDimensions[1]
        );

        // Draw text
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillStyle = '#000';
        ctx.fillText(label, node.x, node.y);

        // Set node area for pointer detection
        (node as any).__bckgDimensions = bckgDimensions;
      })
      .nodePointerAreaPaint((node: any, color: string, ctx: CanvasRenderingContext2D) => {
        ctx.fillStyle = color;
        const bckgDimensions = node.__bckgDimensions;
        bckgDimensions &&
          ctx.fillRect(
            node.x - bckgDimensions[0] / 2,
            node.y - bckgDimensions[1] / 2,
            bckgDimensions[0],
            bckgDimensions[1]
          );
      })
      .linkLabel('name')
      .linkAutoColorBy('layer')
      .linkWidth(2)
      .linkDirectionalArrowLength(6)
      .linkDirectionalArrowRelPos(1)
      .linkCanvasObjectMode(() => 'after')
      .linkCanvasObject((link: any, ctx: CanvasRenderingContext2D) => {
        const MAX_FONT_SIZE = 4;
        const LABEL_NODE_MARGIN = 1.5;

        const start = link.source;
        const end = link.target;

        // Ignore unbound links
        if (typeof start !== 'object' || typeof end !== 'object') return;

        // Calculate label positioning
        const textPos = {
          x: start.x + (end.x - start.x) / 2,
          y: start.y + (end.y - start.y) / 2,
        };

        const relLink = { x: end.x - start.x, y: end.y - start.y };

        const maxTextLength =
          Math.sqrt(Math.pow(relLink.x, 2) + Math.pow(relLink.y, 2)) - LABEL_NODE_MARGIN * 2;

        let textAngle = Math.atan2(relLink.y, relLink.x);
        // Maintain label vertical orientation for legibility
        if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
        if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);

        const label = link.name || '';

        // Estimate text size
        const fontSize = Math.min(MAX_FONT_SIZE, maxTextLength / label.length);
        ctx.font = `${fontSize}px Sans-Serif`;
        const textWidth = ctx.measureText(label).width;

        if (textWidth > 0 && textWidth <= maxTextLength) {
          // Draw text background
          ctx.save();
          ctx.translate(textPos.x, textPos.y);
          ctx.rotate(textAngle);

          ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
          ctx.fillRect(-textWidth / 2, -fontSize / 2, textWidth, fontSize);

          // Draw text
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillStyle = '#000';
          ctx.fillText(label, 0, 0);
          ctx.restore();
        }
      })
      .d3AlphaDecay(0.02)
      .d3VelocityDecay(0.3);

    graphRef.current = graph;

    // Cleanup
    return () => {
      if (graphRef.current) {
        graphRef.current._destructor();
      }
    };
  }, [data, width, height]);

  return (
    <div
      ref={containerRef}
      style={{
        width: '100%',
        height: '100%',
        background: '#fafafa',
      }}
    />
  );
};
