import type { Meta, StoryObj } from '@storybook/react';
import { GraphSettings, GraphLayoutSettings, GraphDisplaySettings, GraphFilterSettings } from './GraphSettings';
import { GraphToolbar } from './GraphToolbar';
import { GraphMinimap } from './GraphMinimap';
import { GraphInspector, NodeData, EdgeData, LayerData } from './GraphInspector';
import {
  defaultLayoutSettings,
  defaultDisplaySettings,
  defaultFilterSettings,
  enhancedGraphData,
  toolbarData,
  minimapData,
  inspectorData,
  mockActions
} from '@/stories/sampleData/enhancedGraphData';

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

// Settings imported from enhanced sample data

type SettingsStory = StoryObj<typeof metaSettings>;

export const DefaultSettings: SettingsStory = {
  args: {
    layoutSettings: defaultLayoutSettings,
    displaySettings: defaultDisplaySettings,
    filterSettings: defaultFilterSettings,
    onLayoutChange: mockActions.onLayoutSettingsChange,
    onDisplayChange: mockActions.onDisplayChange,
    onFilterChange: mockActions.onFilterChange,
    onReset: mockActions.onReset,
    onExport: mockActions.onExport,
    availableLayers: ['mgmt', 'global', 'drone', 'drone2', 'connection'],
    isVisible: true,
    onClose: mockActions.onClose,
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
    ...toolbarData.default,
    ...mockActions,
    nodeCount: enhancedGraphData.refModel.nodeCount,
    edgeCount: enhancedGraphData.refModel.edgeCount,
    layerCount: enhancedGraphData.refModel.layerCount,
  },
};

export const WithSelection: ToolbarStory = {
  args: {
    ...toolbarData.withSelection,
    ...mockActions,
    nodeCount: enhancedGraphData.refModel.nodeCount,
    edgeCount: enhancedGraphData.refModel.edgeCount,
    layerCount: enhancedGraphData.refModel.layerCount,
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
    ...toolbarData.layoutRunning,
    ...mockActions,
    nodeCount: enhancedGraphData.distributedMonolith.nodeCount,
    edgeCount: enhancedGraphData.distributedMonolith.edgeCount,
    layerCount: enhancedGraphData.distributedMonolith.layerCount,
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

// Sample data imported from enhanced sample data

export const DefaultMinimap: MinimapStory = {
  args: {
    nodes: enhancedGraphData.refModel.nodes,
    edges: enhancedGraphData.refModel.edges,
    ...minimapData.default,
    onViewportChange: mockActions.onViewportChange,
    isVisible: true,
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
    nodes: enhancedGraphData.distributedMonolith.nodes,
    edges: enhancedGraphData.distributedMonolith.edges,
    ...minimapData.customStyling,
    onViewportChange: mockActions.onViewportChange,
    isVisible: true,
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

// Inspector data imported from enhanced sample data

export const DefaultInspector: InspectorStory = {
  args: {
    ...inspectorData.refModelSelection,
    ...mockActions,
    isVisible: true,
    position: 'right',
  },
};

export const NoSelection: InspectorStory = {
  args: {
    ...inspectorData.empty,
    ...mockActions,
    isVisible: true,
    position: 'right',
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

export const DistributedMonolithInspector: InspectorStory = {
  args: {
    ...inspectorData.distributedMonolithSelection,
    ...mockActions,
    isVisible: true,
    position: 'right',
  },
  parameters: {
    docs: {
      description: {
        story: 'Inspector showing distributed monolith architecture elements.',
      },
    },
  },
};