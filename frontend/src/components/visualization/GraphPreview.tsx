import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import ForceGraph from 'force-graph';
import { Pane } from 'tweakpane';

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

export interface GraphLayer {
  layerId: string;
  name: string;
  backgroundColor?: string;
  borderColor?: string;
  textColor?: string;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
  layers?: GraphLayer[];
}

interface GraphPreviewProps {
  data: GraphData;
  width?: number;
  height?: number;
}

export const GraphPreview = ({ data, width, height }: GraphPreviewProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphContainerRef = useRef<HTMLDivElement>(null);
  const paneContainerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<any>(null);
  const paneRef = useRef<Pane | null>(null);

  // Graph rendering parameters (stored in ref for Tweakpane)
  const paramsRef = useRef({
    // Node settings
    nodeRadius: 5,
    nodeLabelSize: 12,

    // Link settings
    linkWidth: 2,
    linkArrowLength: 6,
    linkLabelMaxSize: 4,

    // Force simulation settings
    chargeStrength: -30,
    linkDistance: 100,
    alphaDecay: 0.02,
    velocityDecay: 0.3,
  });

  const [, forceUpdate] = useState({});
  const dataSignatureRef = useRef<string>('');

  // Initialize Tweakpane
  useEffect(() => {
    if (!paneContainerRef.current) return;

    // Create new pane
    const pane = new Pane({
      container: paneContainerRef.current,
      title: 'Graph Settings',
      expanded: false, // Collapsed by default
    });

    const params = paramsRef.current;

    // Node settings folder
    const nodeFolder = pane.addFolder({ title: 'Nodes', expanded: true });
    nodeFolder.addBinding(params, 'nodeRadius', {
      label: 'Radius',
      min: 2,
      max: 20,
      step: 0.5
    });
    nodeFolder.addBinding(params, 'nodeLabelSize', {
      label: 'Label Size',
      min: 6,
      max: 24,
      step: 1
    });

    // Link settings folder
    const linkFolder = pane.addFolder({ title: 'Links', expanded: true });
    linkFolder.addBinding(params, 'linkWidth', {
      label: 'Width',
      min: 0.5,
      max: 10,
      step: 0.5
    });
    linkFolder.addBinding(params, 'linkArrowLength', {
      label: 'Arrow Length',
      min: 0,
      max: 20,
      step: 1
    });
    linkFolder.addBinding(params, 'linkLabelMaxSize', {
      label: 'Label Max Size',
      min: 2,
      max: 12,
      step: 0.5
    });

    // Force settings folder
    const forceFolder = pane.addFolder({ title: 'Forces', expanded: false });
    forceFolder.addBinding(params, 'chargeStrength', {
      label: 'Charge',
      min: -200,
      max: 0,
      step: 5
    });
    forceFolder.addBinding(params, 'linkDistance', {
      label: 'Link Distance',
      min: 10,
      max: 400,
      step: 5
    });
    forceFolder.addBinding(params, 'alphaDecay', {
      label: 'Alpha Decay',
      min: 0,
      max: 0.1,
      step: 0.001
    });
    forceFolder.addBinding(params, 'velocityDecay', {
      label: 'Velocity Decay',
      min: 0,
      max: 1,
      step: 0.05
    });

    // Listen for parameter changes
    pane.on('change', () => {
      if (!graphRef.current) return;

      const p = paramsRef.current;

      // Update graph parameters dynamically
      graphRef.current
        .linkWidth(p.linkWidth)
        .linkDirectionalArrowLength(p.linkArrowLength);

      // Update force simulation parameters
      const chargeForce = graphRef.current.d3Force('charge');
      if (chargeForce) chargeForce.strength(p.chargeStrength);

      const linkForce = graphRef.current.d3Force('link');
      if (linkForce) linkForce.distance(p.linkDistance);

      graphRef.current
        .d3AlphaDecay(p.alphaDecay)
        .d3VelocityDecay(p.velocityDecay);

      // Trigger re-render for visual changes (node/label sizes)
      forceUpdate({});
    });

    paneRef.current = pane;

    return () => {
      if (paneRef.current) {
        paneRef.current.dispose();
        paneRef.current = null;
      }
    };
  }, []);

  const computeSignature = useCallback((graph: GraphData) => {
    const nodeSig = graph.nodes
      .map(node => `${node.id}:${node.layer}:${JSON.stringify(node.attrs)}`)
      .join('|');
    const edgeSig = graph.links
      .map(link => `${link.source}->${link.target}:${link.layer}`)
      .join('|');
    return `${nodeSig}#${edgeSig}`;
  }, []);

  useEffect(() => {
    const signature = computeSignature(data);
    if (signature !== dataSignatureRef.current) {
      dataSignatureRef.current = signature;
      forceUpdate({});
    }
  }, [data, computeSignature]);

  const layerStyles = useMemo(() => {
    const defaults = {
      nodeColor: '#4c6ef5',
      borderColor: '#364fc7',
      textColor: '#f8fafc',
      linkColor: '#64748b'
    };

    return {
      defaults,
      getStyle(_layerId?: string) {
        return defaults;
      }
    };
  }, []);

  // Initialize graph when data changes
  useEffect(() => {
    if (!graphContainerRef.current) return;

    const currentWidth = width || graphContainerRef.current.clientWidth;
    const currentHeight = height || graphContainerRef.current.clientHeight;

    if (currentWidth === 0 || currentHeight === 0) return;

    // Clear any existing graph
    if (graphRef.current) {
      graphRef.current._destructor();
      graphRef.current = null;
    }

    const params = paramsRef.current;

    // Build layer color map
    const layerColorMap = new Map<string, string>();
    if (data.layers) {
      data.layers.forEach(layer => {
        if (layer.backgroundColor) {
          layerColorMap.set(layer.layerId, `#${layer.backgroundColor}`);
        }
      });
    }

    // Generate default colors for layers without explicit colors
    const defaultColors = [
      '#6366f1', '#8b5cf6', '#ec4899', '#ef4444', '#f59e0b',
      '#10b981', '#14b8a6', '#06b6d4', '#3b82f6', '#6366f1'
    ];
    let colorIndex = 0;
    const layerSet = new Set<string>();
    data.nodes.forEach(node => layerSet.add(node.layer));
    layerSet.forEach(layer => {
      if (!layerColorMap.has(layer)) {
        layerColorMap.set(layer, defaultColors[colorIndex % defaultColors.length]);
        colorIndex++;
      }
    });

    // Initialize force-graph
    const getLayerStyle = layerStyles.getStyle;

    const graph = (ForceGraph as any)()(graphContainerRef.current)
      .width(currentWidth)
      .height(currentHeight)
      .graphData(data)
      .nodeId('id')
      .nodeLabel((node: any) => node.name || node.id)
      .nodeCanvasObject((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const p = paramsRef.current;
        const label = node.name;
        const fontSize = p.nodeLabelSize / globalScale;
        ctx.font = `${fontSize}px Sans-Serif`;
        const textWidth = ctx.measureText(label).width;
        const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2);

        const style = getLayerStyle(node.layer);

        ctx.beginPath();
        ctx.fillStyle = style.nodeColor;
        ctx.strokeStyle = style.borderColor;
        ctx.lineWidth = 2 / globalScale;
        ctx.arc(node.x, node.y, p.nodeRadius, 0, 2 * Math.PI, false);
        ctx.fill();
        ctx.stroke();

        ctx.fillStyle = 'rgba(15, 23, 42, 0.85)';
        ctx.fillRect(
          node.x - bckgDimensions[0] / 2,
          node.y - p.nodeRadius - bckgDimensions[1] - fontSize * 0.2,
          bckgDimensions[0],
          bckgDimensions[1]
        );

        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillStyle = '#f8fafc';
        ctx.fillText(label, node.x, node.y - p.nodeRadius - bckgDimensions[1] / 2 - fontSize * 0.2);

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
      .linkColor((link: any) => getLayerStyle(link.layer).linkColor)
      .linkWidth(params.linkWidth)
      .linkDirectionalArrowLength(params.linkArrowLength)
      .linkDirectionalArrowRelPos(1)
      .linkCanvasObjectMode(() => 'after')
      .linkCanvasObject((link: any, ctx: CanvasRenderingContext2D) => {
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

        let textAngle = Math.atan2(relLink.y, relLink.x);
        // Maintain label vertical orientation for legibility
        if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
        if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);

        const label = link.name || '';

        ctx.save();
        ctx.translate(textPos.x, textPos.y);
        ctx.rotate(textAngle);

        const fontSize = params.linkLabelMaxSize;
        ctx.font = `${fontSize}px Sans-Serif`;
        const textWidth = ctx.measureText(label).width;

        if (textWidth > 0) {
          ctx.fillStyle = 'rgba(15, 23, 42, 0.85)';
          ctx.fillRect(-textWidth / 2 - 4, -fontSize / 2 - 2, textWidth + 8, fontSize + 4);
          ctx.fillStyle = '#f8fafc';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(label, 0, 0);
        }

        ctx.restore();
      })
      .d3AlphaDecay(params.alphaDecay)
      .d3VelocityDecay(params.velocityDecay);

    // Configure force simulation parameters
    const chargeForce = graph.d3Force('charge');
    if (chargeForce) chargeForce.strength(params.chargeStrength);

    const linkForce = graph.d3Force('link');
    if (linkForce) linkForce.distance(params.linkDistance);

    graphRef.current = graph;

    // Cleanup
    return () => {
      if (graphRef.current) {
        graphRef.current._destructor();
        graphRef.current = null;
      }
    };
  }, [data, width, height, computeSignature]);

  // Handle window resize events
  useEffect(() => {
    const handleResize = () => {
      if (graphContainerRef.current && graphRef.current) {
        const newWidth = width || graphContainerRef.current.clientWidth;
        const newHeight = height || graphContainerRef.current.clientHeight;

        // Update graph dimensions directly without recreating
        graphRef.current
          .width(newWidth)
          .height(newHeight);
      }
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [width, height]);

  return (
    <div
      ref={containerRef}
      style={{
        width: '100%',
        height: '100%',
        position: 'relative',
        background: '#fafafa',
      }}
    >
      {/* Graph container - full size */}
      <div
        ref={graphContainerRef}
        style={{
          width: '100%',
          height: '100%',
          background: '#fafafa',
        }}
      />

      {/* Tweakpane container - positioned in top-right */}
      <div
        ref={paneContainerRef}
        style={{
          position: 'absolute',
          top: '10px',
          right: '10px',
          zIndex: 1000,
          maxHeight: 'calc(100% - 20px)',
          overflow: 'auto',
        }}
      />
    </div>
  );
};
