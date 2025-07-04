import { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { cn } from '@/lib/utils';
import type { Node, Edge, Layer } from '@/types/api';

interface D3Node extends d3.SimulationNodeDatum {
  id: string;
  label: string;
  layerId?: string;
  color: string;
  properties: Record<string, any>;
}

interface D3Edge extends d3.SimulationLinkDatum<D3Node> {
  source: string | D3Node;
  target: string | D3Node;
  properties: Record<string, any>;
}

interface GraphVisualizationProps {
  nodes: Node[];
  edges: Edge[];
  layers: Layer[];
  width?: number;
  height?: number;
  className?: string;
  onNodeClick?: (node: D3Node) => void;
  onEdgeClick?: (edge: D3Edge) => void;
}

export function GraphVisualization({
  nodes = [],
  edges = [],
  layers = [],
  width = 800,
  height = 600,
  className,
  onNodeClick,
  onEdgeClick,
}: GraphVisualizationProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);

  useEffect(() => {
    if (!svgRef.current || nodes.length === 0) return;

    // Clear previous content
    d3.select(svgRef.current).selectAll('*').remove();

    // Create layer color map
    const layerColorMap = new Map<string, string>();
    layers.forEach(layer => {
      layerColorMap.set(layer.layer_id, layer.color || '#6366f1');
    });

    // Transform data for D3
    const d3Nodes: D3Node[] = nodes.map(node => ({
      id: node.node_id,
      label: node.label,
      layerId: node.layer_id,
      color: node.layer_id ? layerColorMap.get(node.layer_id) || '#6366f1' : '#6366f1',
      properties: node.properties || {},
    }));

    const d3Edges: D3Edge[] = edges.map(edge => ({
      source: edge.source_node_id,
      target: edge.target_node_id,
      properties: edge.properties || {},
    }));

    // Set up SVG
    const svg = d3.select(svgRef.current);
    const g = svg.append('g');

    // Set up zoom behavior
    const zoomBehavior = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
        setZoom(event.transform.k);
      });

    svg.call(zoomBehavior);

    // Set up force simulation
    const simulation = d3.forceSimulation<D3Node>(d3Nodes)
      .force('link', d3.forceLink<D3Node, D3Edge>(d3Edges).id(d => d.id).distance(80))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(20));

    // Create edges
    const link = g.append('g')
      .attr('class', 'edges')
      .selectAll('line')
      .data(d3Edges)
      .join('line')
      .attr('stroke', '#999')
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onEdgeClick?.(d);
      });

    // Create node groups
    const node = g.append('g')
      .attr('class', 'nodes')
      .selectAll('g')
      .data(d3Nodes)
      .join('g')
      .style('cursor', 'pointer')
      .call(d3.drag<SVGGElement, D3Node>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        }) as any
      );

    // Add circles to nodes
    node.append('circle')
      .attr('r', 12)
      .attr('fill', d => d.color)
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .on('click', (event, d) => {
        event.stopPropagation();
        setSelectedNode(d.id === selectedNode ? null : d.id);
        onNodeClick?.(d);
      });

    // Add labels to nodes
    node.append('text')
      .attr('dy', 4)
      .attr('text-anchor', 'middle')
      .attr('font-size', '10px')
      .attr('font-family', 'sans-serif')
      .attr('fill', '#333')
      .attr('pointer-events', 'none')
      .text(d => d.label.length > 8 ? d.label.substring(0, 8) + '...' : d.label);

    // Add hover effects
    node.on('mouseenter', function(event, d) {
      d3.select(this).select('circle')
        .transition()
        .duration(200)
        .attr('r', 15)
        .attr('stroke-width', 3);
      
      // Show full label on hover
      const tooltip = d3.select('body').append('div')
        .attr('class', 'graph-tooltip')
        .style('position', 'absolute')
        .style('background', 'rgba(0, 0, 0, 0.8)')
        .style('color', 'white')
        .style('padding', '8px')
        .style('border-radius', '4px')
        .style('font-size', '12px')
        .style('pointer-events', 'none')
        .style('z-index', '1000')
        .text(d.label);

      tooltip.style('left', (event.pageX + 10) + 'px')
        .style('top', (event.pageY - 10) + 'px');
    })
    .on('mouseleave', function() {
      d3.select(this).select('circle')
        .transition()
        .duration(200)
        .attr('r', 12)
        .attr('stroke-width', 2);
      
      d3.selectAll('.graph-tooltip').remove();
    });

    // Update selection styling
    node.select('circle')
      .attr('stroke', d => d.id === selectedNode ? '#ff6b6b' : '#fff')
      .attr('stroke-width', d => d.id === selectedNode ? 3 : 2);

    // Update positions on simulation tick
    simulation.on('tick', () => {
      link
        .attr('x1', d => (d.source as D3Node).x!)
        .attr('y1', d => (d.source as D3Node).y!)
        .attr('x2', d => (d.target as D3Node).x!)
        .attr('y2', d => (d.target as D3Node).y!);

      node.attr('transform', d => `translate(${d.x},${d.y})`);
    });

    // Cleanup function
    return () => {
      simulation.stop();
      d3.selectAll('.graph-tooltip').remove();
    };
  }, [nodes, edges, layers, width, height, selectedNode, onNodeClick, onEdgeClick]);

  // Clear selection when clicking on empty space
  const handleSvgClick = () => {
    setSelectedNode(null);
  };

  if (nodes.length === 0) {
    return (
      <div className={cn(
        'flex items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg',
        className
      )} style={{ width, height }}>
        <div className="text-center text-gray-500 dark:text-gray-400">
          <div className="text-lg font-medium mb-2">No graph data</div>
          <div className="text-sm">Import nodes and edges to visualize the graph</div>
        </div>
      </div>
    );
  }

  return (
    <div className={cn('relative border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden', className)}>
      <svg
        ref={svgRef}
        width={width}
        height={height}
        onClick={handleSvgClick}
        className="bg-white dark:bg-gray-900"
      />
      
      {/* Zoom indicator */}
      <div className="absolute top-2 right-2 bg-black bg-opacity-50 text-white px-2 py-1 rounded text-xs">
        Zoom: {Math.round(zoom * 100)}%
      </div>
      
      {/* Legend */}
      {layers.length > 0 && (
        <div className="absolute bottom-2 left-2 bg-white dark:bg-gray-800 p-2 rounded shadow-lg border border-gray-200 dark:border-gray-700">
          <div className="text-xs font-medium mb-1 text-gray-700 dark:text-gray-300">Layers</div>
          {layers.map(layer => (
            <div key={layer.layer_id} className="flex items-center gap-1 text-xs">
              <div 
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: layer.color || '#6366f1' }}
              />
              <span className="text-gray-600 dark:text-gray-400">{layer.name}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}