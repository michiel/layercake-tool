import type { Meta, StoryObj } from '@storybook/react';
import { GraphSettings, GraphLayoutSettings, GraphDisplaySettings, GraphFilterSettings } from './GraphSettings';
import { GraphToolbar } from './GraphToolbar';
import { GraphMinimap } from './GraphMinimap';
import { GraphInspector, NodeData, EdgeData, LayerData } from './GraphInspector';

// GraphSettings Stories
const metaSettings: Meta<typeof GraphSettings> = {
  title: 'Graph/Enhanced/GraphSettings',
  component: GraphSettings,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Comprehensive graph settings modal with layout, display, and filter controls.',
      },
    },
  },
  tags: ['autodocs'],
};

export default metaSettings;

const defaultLayoutSettings: GraphLayoutSettings = {
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

const defaultDisplaySettings: GraphDisplaySettings = {
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

const defaultFilterSettings: GraphFilterSettings = {
  minNodeWeight: 0,
  maxNodeWeight: 100,
  selectedLayers: ['layer1', 'layer2'],
  nodeTypes: [],
  minEdgeWeight: 0,
  maxEdgeWeight: 100,
  showSelfLoops: true,
  showMultiEdges: true,
  showIsolatedNodes: true,
  minDegree: 0,
  maxDegree: 100,
};

type SettingsStory = StoryObj<typeof metaSettings>;

export const DefaultSettings: SettingsStory = {
  args: {
    layoutSettings: defaultLayoutSettings,
    displaySettings: defaultDisplaySettings,
    filterSettings: defaultFilterSettings,
    onLayoutChange: (settings) => console.log('Layout changed:', settings),
    onDisplayChange: (settings) => console.log('Display changed:', settings),
    onFilterChange: (settings) => console.log('Filter changed:', settings),
    onReset: () => console.log('Reset settings'),
    onExport: () => console.log('Export settings'),
    availableLayers: ['layer1', 'layer2', 'layer3', 'default'],
    isVisible: true,
    onClose: () => console.log('Close settings'),
  },
};

export const PerformanceOptimized: SettingsStory = {
  args: {
    ...DefaultSettings.args,
    displaySettings: {
      ...defaultDisplaySettings,
      enableWebGL: true,
      maxVisibleNodes: 500,
      levelOfDetail: true,
      showNodeLabels: false,
      showEdgeLabels: false,
    },
  },
  parameters: {
    docs: {
      description: {
        story: 'Settings optimized for large graphs with performance considerations.',
      },
    },
  },
};

export const DetailedVisualization: SettingsStory = {
  args: {
    ...DefaultSettings.args,
    layoutSettings: {
      ...defaultLayoutSettings,
      enableAnimations: true,
      animationDuration: 1500,
      enableCollision: true,
    },
    displaySettings: {
      ...defaultDisplaySettings,
      showNodeLabels: true,
      showEdgeLabels: true,
      showEdgeWeights: true,
      nodeScale: 1.5,
      edgeScale: 1.2,
    },
  },
  parameters: {
    docs: {
      description: {
        story: 'Settings for detailed visualization with all labels and information shown.',
      },
    },
  },
};

// GraphToolbar Stories
const metaToolbar: Meta<typeof GraphToolbar> = {
  title: 'Graph/Enhanced/GraphToolbar',
  component: GraphToolbar,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: 'Interactive toolbar for graph navigation and control.',
      },
    },
  },
  tags: ['autodocs'],
};

type ToolbarStory = StoryObj<typeof metaToolbar>;

export const DefaultToolbar: ToolbarStory = {
  args: {
    zoomLevel: 1.0,
    onZoomIn: () => console.log('Zoom in'),
    onZoomOut: () => console.log('Zoom out'),
    onResetZoom: () => console.log('Reset zoom'),
    onFitToScreen: () => console.log('Fit to screen'),
    isLayoutRunning: false,
    onStartLayout: () => console.log('Start layout'),
    onStopLayout: () => console.log('Stop layout'),
    onResetLayout: () => console.log('Reset layout'),
    isFullscreen: false,
    onToggleFullscreen: () => console.log('Toggle fullscreen'),
    showGrid: false,
    onToggleGrid: () => console.log('Toggle grid'),
    showMinimap: true,
    onToggleMinimap: () => console.log('Toggle minimap'),
    selectedNodeCount: 0,
    selectedEdgeCount: 0,
    onClearSelection: () => console.log('Clear selection'),
    searchQuery: '',
    onSearchChange: (query) => console.log('Search:', query),
    searchResults: 0,
    onOpenSettings: () => console.log('Open settings'),
    onOpenFilters: () => console.log('Open filters'),
    onExportGraph: () => console.log('Export graph'),
    onImportGraph: () => console.log('Import graph'),
    onShowInfo: () => console.log('Show info'),
    nodeCount: 150,
    edgeCount: 200,
    layerCount: 5,
    currentLayout: 'Force-directed',
    availableLayouts: ['Force-directed', 'Hierarchical', 'Circular', 'Grid'],
    onLayoutChange: (layout) => console.log('Layout changed:', layout),
  },
};

export const WithSelection: ToolbarStory = {
  args: {
    ...DefaultToolbar.args,
    selectedNodeCount: 3,
    selectedEdgeCount: 2,
    searchQuery: 'server',
    searchResults: 5,
  },
  parameters: {
    docs: {
      description: {
        story: 'Toolbar with active selection and search query.',
      },
    },
  },
};

export const LayoutRunning: ToolbarStory = {
  args: {
    ...DefaultToolbar.args,
    isLayoutRunning: true,
    zoomLevel: 0.75,
  },
  parameters: {
    docs: {
      description: {
        story: 'Toolbar showing layout algorithm in progress.',
      },
    },
  },
};

// GraphMinimap Stories
const metaMinimap: Meta<typeof GraphMinimap> = {
  title: 'Graph/Enhanced/GraphMinimap',
  component: GraphMinimap,
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: 'Overview minimap for graph navigation.',
      },
    },
  },
  tags: ['autodocs'],
};

type MinimapStory = StoryObj<typeof metaMinimap>;

const sampleNodes = [
  { id: '1', x: 100, y: 100, layer: 'layer1' },
  { id: '2', x: 200, y: 150, layer: 'layer1' },
  { id: '3', x: 300, y: 200, layer: 'layer2' },
  { id: '4', x: 400, y: 100, layer: 'layer2' },
  { id: '5', x: 150, y: 300, layer: 'layer3' },
];

const sampleEdges = [
  { id: 'e1', source: '1', target: '2' },
  { id: 'e2', source: '2', target: '3' },
  { id: 'e3', source: '3', target: '4' },
  { id: 'e4', source: '1', target: '5' },
];

export const DefaultMinimap: MinimapStory = {
  args: {
    nodes: sampleNodes,
    edges: sampleEdges,
    viewport: { x: 50, y: 50, width: 200, height: 150 },
    onViewportChange: (viewport) => console.log('Viewport changed:', viewport),
    graphBounds: { minX: 0, maxX: 500, minY: 0, maxY: 400 },
    isVisible: true,
    position: 'bottom-right',
  },
};

export const TopLeftPosition: MinimapStory = {
  args: {
    ...DefaultMinimap.args,
    position: 'top-left',
  },
};

export const CustomStyling: MinimapStory = {
  args: {
    ...DefaultMinimap.args,
    width: 300,
    height: 200,
    backgroundColor: '#1f2937',
    nodeColor: '#60a5fa',
    edgeColor: '#4b5563',
    viewportColor: '#f59e0b',
  },
  parameters: {
    docs: {
      description: {
        story: 'Minimap with custom dark theme styling.',
      },
    },
  },
};

// GraphInspector Stories
const metaInspector: Meta<typeof GraphInspector> = {
  title: 'Graph/Enhanced/GraphInspector',
  component: GraphInspector,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: 'Side panel for inspecting and editing selected graph elements.',
      },
    },
  },
  tags: ['autodocs'],
};

type InspectorStory = StoryObj<typeof metaInspector>;

const sampleSelectedNodes: NodeData[] = [
  {
    id: 'node1',
    label: 'Web Server',
    layer: 'infrastructure',
    isPartition: false,
    weight: 5,
    comment: 'Main application server',
    x: 100,
    y: 150,
    degree: 8,
    inDegree: 3,
    outDegree: 5,
  },
  {
    id: 'node2',
    label: 'Database Cluster',
    layer: 'data',
    isPartition: true,
    weight: 10,
    x: 300,
    y: 200,
    degree: 12,
    inDegree: 8,
    outDegree: 4,
  },
];

const sampleSelectedEdges: EdgeData[] = [
  {
    id: 'edge1',
    source: 'node1',
    target: 'node2',
    label: 'Database Connection',
    layer: 'network',
    weight: 3,
    comment: 'Primary data connection',
  },
];

const sampleSelectedLayers: LayerData[] = [
  {
    id: 'infrastructure',
    label: 'Infrastructure',
    backgroundColor: '#dbeafe',
    textColor: '#1e40af',
    borderColor: '#3b82f6',
    nodeCount: 15,
    edgeCount: 22,
    visible: true,
  },
  {
    id: 'data',
    label: 'Data Layer',
    backgroundColor: '#dcfce7',
    textColor: '#166534',
    borderColor: '#22c55e',
    nodeCount: 8,
    edgeCount: 12,
    visible: true,
  },
];

export const DefaultInspector: InspectorStory = {
  args: {
    selectedNodes: sampleSelectedNodes,
    selectedEdges: sampleSelectedEdges,
    selectedLayers: sampleSelectedLayers,
    onNodeUpdate: (nodeId, updates) => console.log('Update node:', nodeId, updates),
    onEdgeUpdate: (edgeId, updates) => console.log('Update edge:', edgeId, updates),
    onLayerUpdate: (layerId, updates) => console.log('Update layer:', layerId, updates),
    onNodeDelete: (nodeId) => console.log('Delete node:', nodeId),
    onEdgeDelete: (edgeId) => console.log('Delete edge:', edgeId),
    onLayerToggle: (layerId) => console.log('Toggle layer:', layerId),
    onFocusElement: (type, id) => console.log('Focus element:', type, id),
    onClearSelection: () => console.log('Clear selection'),
    isVisible: true,
    position: 'right',
  },
};

export const NoSelection: InspectorStory = {
  args: {
    ...DefaultInspector.args,
    selectedNodes: [],
    selectedEdges: [],
  },
  parameters: {
    docs: {
      description: {
        story: 'Inspector with no elements selected.',
      },
    },
  },
};

export const LeftPosition: InspectorStory = {
  args: {
    ...DefaultInspector.args,
    position: 'left',
  },
  parameters: {
    docs: {
      description: {
        story: 'Inspector positioned on the left side.',
      },
    },
  },
};

export const LayersOnly: InspectorStory = {
  args: {
    ...DefaultInspector.args,
    selectedNodes: [],
    selectedEdges: [],
  },
  parameters: {
    docs: {
      description: {
        story: 'Inspector showing only layer information.',
      },
    },
  },
};