import React, { useRef, useEffect, useState } from 'react';

export interface MinimapViewport {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface GraphMinimapProps {
  // Graph data for minimap rendering
  nodes: Array<{
    id: string;
    x: number;
    y: number;
    layer: string;
  }>;
  edges: Array<{
    id: string;
    source: string;
    target: string;
  }>;
  
  // Main viewport information
  viewport: MinimapViewport;
  onViewportChange: (viewport: MinimapViewport) => void;
  
  // Graph bounds
  graphBounds: {
    minX: number;
    maxX: number;
    minY: number;
    maxY: number;
  };
  
  // Styling
  width?: number;
  height?: number;
  backgroundColor?: string;
  nodeColor?: string;
  edgeColor?: string;
  viewportColor?: string;
  
  // Visibility
  isVisible?: boolean;
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
}

export const GraphMinimap: React.FC<GraphMinimapProps> = ({
  nodes,
  edges,
  viewport,
  onViewportChange,
  graphBounds,
  width = 200,
  height = 150,
  backgroundColor = '#f8f9fa',
  nodeColor = '#3b82f6',
  edgeColor = '#e5e7eb',
  viewportColor = '#ef4444',
  isVisible = true,
  position = 'bottom-right',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Calculate scale factors to fit graph in minimap
  const scaleX = width / (graphBounds.maxX - graphBounds.minX || 1);
  const scaleY = height / (graphBounds.maxY - graphBounds.minY || 1);
  const scale = Math.min(scaleX, scaleY) * 0.9; // Add some padding

  // Transform graph coordinates to minimap coordinates
  const transformX = (x: number) => {
    return ((x - graphBounds.minX) * scale) + (width - (graphBounds.maxX - graphBounds.minX) * scale) / 2;
  };

  const transformY = (y: number) => {
    return ((y - graphBounds.minY) * scale) + (height - (graphBounds.maxY - graphBounds.minY) * scale) / 2;
  };

  // Transform minimap coordinates back to graph coordinates
  const inverseTransformX = (x: number) => {
    return ((x - (width - (graphBounds.maxX - graphBounds.minX) * scale) / 2) / scale) + graphBounds.minX;
  };

  const inverseTransformY = (y: number) => {
    return ((y - (height - (graphBounds.maxY - graphBounds.minY) * scale) / 2) / scale) + graphBounds.minY;
  };

  // Create node position lookup for edges
  const nodePositions = new Map(
    nodes.map(node => [node.id, { x: transformX(node.x), y: transformY(node.y) }])
  );

  // Render the minimap
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = width;
    canvas.height = height;

    // Clear canvas
    ctx.fillStyle = backgroundColor;
    ctx.fillRect(0, 0, width, height);

    // Draw edges
    ctx.strokeStyle = edgeColor;
    ctx.lineWidth = 1;
    ctx.beginPath();
    
    edges.forEach(edge => {
      const sourcePos = nodePositions.get(edge.source);
      const targetPos = nodePositions.get(edge.target);
      
      if (sourcePos && targetPos) {
        ctx.moveTo(sourcePos.x, sourcePos.y);
        ctx.lineTo(targetPos.x, targetPos.y);
      }
    });
    
    ctx.stroke();

    // Draw nodes
    ctx.fillStyle = nodeColor;
    nodes.forEach(node => {
      const x = transformX(node.x);
      const y = transformY(node.y);
      
      ctx.beginPath();
      ctx.arc(x, y, 2, 0, 2 * Math.PI);
      ctx.fill();
    });

    // Draw viewport rectangle
    const viewportX = transformX(viewport.x);
    const viewportY = transformY(viewport.y);
    const viewportW = viewport.width * scale;
    const viewportH = viewport.height * scale;

    ctx.strokeStyle = viewportColor;
    ctx.lineWidth = 2;
    ctx.setLineDash([4, 4]);
    ctx.strokeRect(viewportX, viewportY, viewportW, viewportH);
    ctx.setLineDash([]);

    // Add semi-transparent overlay outside viewport
    ctx.fillStyle = 'rgba(0, 0, 0, 0.1)';
    
    // Top
    ctx.fillRect(0, 0, width, viewportY);
    // Bottom
    ctx.fillRect(0, viewportY + viewportH, width, height - viewportY - viewportH);
    // Left
    ctx.fillRect(0, viewportY, viewportX, viewportH);
    // Right
    ctx.fillRect(viewportX + viewportW, viewportY, width - viewportX - viewportW, viewportH);

  }, [nodes, edges, viewport, graphBounds, width, height, backgroundColor, nodeColor, edgeColor, viewportColor]);

  // Handle mouse events for viewport dragging
  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    setIsDragging(true);
    setDragStart({ x, y });
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDragging) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const dx = x - dragStart.x;
    const dy = y - dragStart.y;

    // Convert minimap delta to graph coordinates
    const graphDx = dx / scale;
    const graphDy = dy / scale;

    // Update viewport position
    const newViewport = {
      ...viewport,
      x: viewport.x + graphDx,
      y: viewport.y + graphDy,
    };

    onViewportChange(newViewport);
    setDragStart({ x, y });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (isDragging) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Convert click position to graph coordinates
    const graphX = inverseTransformX(x);
    const graphY = inverseTransformY(y);

    // Center viewport on clicked position
    const newViewport = {
      ...viewport,
      x: graphX - viewport.width / 2,
      y: graphY - viewport.height / 2,
    };

    onViewportChange(newViewport);
  };

  if (!isVisible) return null;

  const positionClasses = {
    'top-left': 'top-4 left-4',
    'top-right': 'top-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'bottom-right': 'bottom-4 right-4',
  };

  return (
    <div 
      className={`absolute ${positionClasses[position]} z-10 bg-white border border-gray-300 rounded-lg shadow-lg overflow-hidden`}
      style={{ width: width + 2, height: height + 2 }}
    >
      {/* Header */}
      <div className="bg-gray-50 px-2 py-1 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-gray-700">Overview</span>
          <div className="flex items-center space-x-1">
            <div className="w-2 h-2 rounded-full bg-blue-500" title="Nodes"></div>
            <div className="w-3 h-0.5 bg-gray-400" title="Edges"></div>
            <div className="w-3 h-0.5 border border-red-500" title="Viewport"></div>
          </div>
        </div>
      </div>

      {/* Canvas */}
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        className="cursor-crosshair"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onClick={handleClick}
        style={{ display: 'block' }}
      />

      {/* Info overlay */}
      <div className="absolute bottom-1 left-1 text-xs text-gray-500 bg-white bg-opacity-75 px-1 rounded">
        {nodes.length}N {edges.length}E
      </div>
    </div>
  );
};

export default GraphMinimap;