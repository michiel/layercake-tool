/**
 * Enhanced sample data for graph components based on sample/ directory
 * Provides realistic data for Storybook stories with proper structure
 */

import { 
  sampleProjects, 
  samplePlans, 
  refModelNodes, 
  refModelEdges, 
  refModelLayers,
  distributedMonolithNodes,
  distributedMonolithEdges,
  distributedMonolithLayers,
  generateRandomNodes,
  generateRandomEdges,
  generateLayersFromNodes
} from './sampleDataLoader';

// Enhanced graph settings interfaces for stories
export interface GraphLayoutSettings {
  forceStrength: number;
  linkDistance: number;
  linkStrength: number;
  chargeStrength: number;
  centerStrength: number;
  nodeSize: number;
  levelSeparation: number;
  nodeSeparation: number;
  enableLabels: boolean;
  labelFontSize: number;
  nodeOpacity: number;
  edgeOpacity: number;
  enableAnimations: boolean;
  animationDuration: number;
  enableCollision: boolean;
  collisionRadius: number;
}

export interface GraphDisplaySettings {
  enableZoom: boolean;
  enablePan: boolean;
  zoomExtent: [number, number];
  showNodeLabels: boolean;
  showNodeIcons: boolean;
  nodeScale: number;
  nodeColorScheme: 'layer' | 'weight' | 'custom';
  showEdgeLabels: boolean;
  showEdgeWeights: boolean;
  edgeScale: number;
  edgeColorScheme: 'layer' | 'weight' | 'custom';
  showLayers: boolean;
  layerOpacity: number;
  groupByLayers: boolean;
  enableWebGL: boolean;
  maxVisibleNodes: number;
  levelOfDetail: boolean;
}

export interface GraphFilterSettings {
  minNodeWeight: number;
  maxNodeWeight: number;
  selectedLayers: string[];
  nodeTypes: string[];
  minEdgeWeight: number;
  maxEdgeWeight: number;
  showSelfLoops: boolean;
  showMultiEdges: boolean;
  showIsolatedNodes: boolean;
  minDegree: number;
  maxDegree: number;
}

// Default settings for stories
export const defaultLayoutSettings: GraphLayoutSettings = {
  forceStrength: -30,
  linkDistance: 100,
  linkStrength: 1,
  chargeStrength: -300,
  centerStrength: 0.1,
  nodeSize: 20,
  levelSeparation: 150,
  nodeSeparation: 100,
  enableLabels: true,
  labelFontSize: 12,
  nodeOpacity: 1,
  edgeOpacity: 0.8,
  enableAnimations: true,
  animationDuration: 750,
  enableCollision: false,
  collisionRadius: 30,
};

export const defaultDisplaySettings: GraphDisplaySettings = {
  enableZoom: true,
  enablePan: true,
  zoomExtent: [0.1, 10],
  showNodeLabels: true,
  showNodeIcons: false,
  nodeScale: 1,
  nodeColorScheme: 'layer',
  showEdgeLabels: false,
  showEdgeWeights: false,
  edgeScale: 1,
  edgeColorScheme: 'layer',
  showLayers: true,
  layerOpacity: 0.3,
  groupByLayers: false,
  enableWebGL: false,
  maxVisibleNodes: 1000,
  levelOfDetail: true,
};

export const defaultFilterSettings: GraphFilterSettings = {
  minNodeWeight: 0,
  maxNodeWeight: 100,
  selectedLayers: ['mgmt', 'global', 'drone'],
  nodeTypes: [],
  minEdgeWeight: 0,
  maxEdgeWeight: 100,
  showSelfLoops: true,
  showMultiEdges: true,
  showIsolatedNodes: true,
  minDegree: 0,
  maxDegree: 100,
};

// Sample data for different scenarios
export const enhancedGraphData = {
  refModel: {
    name: 'Reference Model',
    description: 'Hierarchical management and drone network structure',
    nodes: refModelNodes,
    edges: refModelEdges,
    layers: refModelLayers,
    nodeCount: refModelNodes.length,
    edgeCount: refModelEdges.length,
    layerCount: refModelLayers.length,
  },
  
  distributedMonolith: {
    name: 'Distributed Monolith',
    description: 'Microservices architecture with API gateway and data layer',
    nodes: distributedMonolithNodes,
    edges: distributedMonolithEdges,
    layers: distributedMonolithLayers,
    nodeCount: distributedMonolithNodes.length,
    edgeCount: distributedMonolithEdges.length,
    layerCount: distributedMonolithLayers.length,
  },
  
  small: {
    name: 'Small Graph',
    description: 'Small graph for testing basic functionality',
    nodes: refModelNodes.slice(0, 4),
    edges: refModelEdges.slice(0, 3),
    layers: refModelLayers.slice(0, 2),
    nodeCount: 4,
    edgeCount: 3,
    layerCount: 2,
  },
  
  large: (() => {
    const nodes = generateRandomNodes(100, ['layer1', 'layer2', 'layer3', 'layer4', 'layer5']);
    const edges = generateRandomEdges(nodes, 150);
    const layers = generateLayersFromNodes(nodes);
    return {
      name: 'Large Graph',
      description: 'Large graph for performance testing',
      nodes,
      edges,
      layers,
      nodeCount: nodes.length,
      edgeCount: edges.length,
      layerCount: layers.length,
    };
  })(),
  
  empty: {
    name: 'Empty Graph',
    description: 'Empty graph for testing empty states',
    nodes: [],
    edges: [],
    layers: [],
    nodeCount: 0,
    edgeCount: 0,
    layerCount: 0,
  }
};

// Graph toolbar data
export const toolbarData = {
  default: {
    zoomLevel: 1.0,
    isLayoutRunning: false,
    isFullscreen: false,
    showGrid: false,
    showMinimap: true,
    selectedNodeCount: 0,
    selectedEdgeCount: 0,
    searchQuery: '',
    searchResults: 0,
    currentLayout: 'Force-directed',
    availableLayouts: ['Force-directed', 'Hierarchical', 'Circular', 'Grid'],
  },
  
  withSelection: {
    zoomLevel: 1.25,
    isLayoutRunning: false,
    isFullscreen: false,
    showGrid: false,
    showMinimap: true,
    selectedNodeCount: 3,
    selectedEdgeCount: 2,
    searchQuery: 'drone',
    searchResults: 4,
    currentLayout: 'Force-directed',
    availableLayouts: ['Force-directed', 'Hierarchical', 'Circular', 'Grid'],
  },
  
  layoutRunning: {
    zoomLevel: 0.8,
    isLayoutRunning: true,
    isFullscreen: false,
    showGrid: true,
    showMinimap: true,
    selectedNodeCount: 0,
    selectedEdgeCount: 0,
    searchQuery: '',
    searchResults: 0,
    currentLayout: 'Hierarchical',
    availableLayouts: ['Force-directed', 'Hierarchical', 'Circular', 'Grid'],
  }
};

// Graph minimap data
export const minimapData = {
  default: {
    viewport: { x: 50, y: 50, width: 200, height: 150 },
    graphBounds: { minX: 0, maxX: 800, minY: 0, maxY: 600 },
    position: 'bottom-right' as const,
  },
  
  customStyling: {
    viewport: { x: 100, y: 100, width: 250, height: 180 },
    graphBounds: { minX: 0, maxX: 1000, minY: 0, maxY: 800 },
    position: 'top-left' as const,
    width: 300,
    height: 200,
    backgroundColor: '#1f2937',
    nodeColor: '#60a5fa',
    edgeColor: '#4b5563',
    viewportColor: '#f59e0b',
  }
};

// Graph inspector data with realistic examples
export const inspectorData = {
  refModelSelection: {
    selectedNodes: [
      {
        id: 'root',
        label: 'Root',
        layer: 'global',
        isPartition: true,
        weight: 1,
        comment: 'Main root node controlling the entire network',
        x: 400,
        y: 200,
        degree: 12,
        inDegree: 0,
        outDegree: 12
      },
      {
        id: 'mgmt_a',
        label: 'Management A',
        layer: 'mgmt',
        isPartition: true,
        belongsTo: 'root',
        weight: 1,
        x: 200,
        y: 100,
        degree: 8,
        inDegree: 2,
        outDegree: 6
      }
    ],
    selectedEdges: [
      {
        id: 'root_mgmt_a',
        source: 'root',
        target: 'mgmt_a',
        label: 'manages',
        layer: 'mgmt',
        weight: 1,
        comment: 'Primary management connection'
      }
    ],
    selectedLayers: refModelLayers
  },
  
  distributedMonolithSelection: {
    selectedNodes: [
      {
        id: 'api_gateway_instance',
        label: 'API Gateway',
        layer: 'api_gateway',
        isPartition: false,
        weight: 5,
        comment: 'Main entry point for all API requests',
        x: 400,
        y: 200,
        degree: 12,
        inDegree: 0,
        outDegree: 12
      }
    ],
    selectedEdges: [
      {
        id: 'edge_1',
        source: 'api_gateway_instance',
        target: 'lambda_17',
        label: 'Auth Route',
        layer: 'api_gateway',
        weight: 2,
        comment: 'User authentication endpoint'
      }
    ],
    selectedLayers: distributedMonolithLayers
  },
  
  empty: {
    selectedNodes: [],
    selectedEdges: [],
    selectedLayers: refModelLayers
  }
};

// Export for easy access in stories
export { sampleProjects, samplePlans };

// Mock functions for actions
export const mockActions = {
  // Settings actions
  onLayoutSettingsChange: (settings: GraphLayoutSettings) => console.log('Layout settings changed:', settings),
  onDisplayChange: (settings: GraphDisplaySettings) => console.log('Display changed:', settings),
  onFilterChange: (settings: GraphFilterSettings) => console.log('Filter changed:', settings),
  onReset: () => console.log('Reset settings'),
  onExport: () => console.log('Export settings'),
  
  // Toolbar actions
  onZoomIn: () => console.log('Zoom in'),
  onZoomOut: () => console.log('Zoom out'),
  onResetZoom: () => console.log('Reset zoom'),
  onFitToScreen: () => console.log('Fit to screen'),
  onStartLayout: () => console.log('Start layout'),
  onStopLayout: () => console.log('Stop layout'),
  onResetLayout: () => console.log('Reset layout'),
  onToggleFullscreen: () => console.log('Toggle fullscreen'),
  onToggleGrid: () => console.log('Toggle grid'),
  onToggleMinimap: () => console.log('Toggle minimap'),
  onClearSelection: () => console.log('Clear selection'),
  onSearchChange: (query: string) => console.log('Search:', query),
  onOpenSettings: () => console.log('Open settings'),
  onOpenFilters: () => console.log('Open filters'),
  onExportGraph: () => console.log('Export graph'),
  onImportGraph: () => console.log('Import graph'),
  onShowInfo: () => console.log('Show info'),
  onLayoutChange: (layout: string) => console.log('Layout changed:', layout),
  
  // Minimap actions
  onViewportChange: (viewport: any) => console.log('Viewport changed:', viewport),
  
  // Inspector actions
  onNodeUpdate: (nodeId: string, updates: any) => console.log('Update node:', nodeId, updates),
  onEdgeUpdate: (edgeId: string, updates: any) => console.log('Update edge:', edgeId, updates),
  onLayerUpdate: (layerId: string, updates: any) => console.log('Update layer:', layerId, updates),
  onNodeDelete: (nodeId: string) => console.log('Delete node:', nodeId),
  onEdgeDelete: (edgeId: string) => console.log('Delete edge:', edgeId),
  onLayerToggle: (layerId: string) => console.log('Toggle layer:', layerId),
  onFocusElement: (type: 'node' | 'edge', id: string) => console.log('Focus element:', type, id),
  onClose: () => console.log('Close'),
};