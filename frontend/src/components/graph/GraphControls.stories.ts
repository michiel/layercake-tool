import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { GraphControls, GraphStats, GraphToolbar } from './GraphControls';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof GraphControls> = {
  title: 'Graph/GraphControls',
  component: GraphControls,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Graph control panel with zoom, simulation, and settings controls. Features intuitive icons and responsive design.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    onZoomIn: {
      action: 'onZoomIn',
      description: 'Callback when zoom in button is clicked',
    },
    onZoomOut: {
      action: 'onZoomOut',
      description: 'Callback when zoom out button is clicked',
    },
    onReset: {
      action: 'onReset',
      description: 'Callback when reset view button is clicked',
    },
    onToggleSimulation: {
      action: 'onToggleSimulation',
      description: 'Callback when simulation toggle button is clicked',
    },
    isSimulationRunning: {
      control: 'boolean',
      description: 'Whether the force simulation is currently running',
    },
    onOpenSettings: {
      action: 'onOpenSettings',
      description: 'Optional callback when settings button is clicked',
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes',
    },
  },
  args: {
    onZoomIn: fn(),
    onZoomOut: fn(),
    onReset: fn(),
    onToggleSimulation: fn(),
    onOpenSettings: fn(),
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Basic controls
export const Default: Story = {
  args: {
    isSimulationRunning: false,
  },
};

export const SimulationRunning: Story = {
  args: {
    isSimulationRunning: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph controls with simulation in running state.',
      },
    },
  },
};

export const WithSettings: Story = {
  args: {
    isSimulationRunning: false,
    onOpenSettings: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph controls with optional settings button.',
      },
    },
  },
};

export const WithoutSettings: Story = {
  args: {
    isSimulationRunning: false,
    onOpenSettings: undefined,
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph controls without settings button.',
      },
    },
  },
};

// Interactive demo
export const InteractiveDemo: Story = {
  render: () => {
    const [isSimulationRunning, setIsSimulationRunning] = useState(false);
    const [zoomLevel, setZoomLevel] = useState(100);

    const handleZoomIn = () => {
      setZoomLevel(prev => Math.min(prev + 25, 400));
      console.log('Zoom in clicked');
    };

    const handleZoomOut = () => {
      setZoomLevel(prev => Math.max(prev - 25, 25));
      console.log('Zoom out clicked');
    };

    const handleReset = () => {
      setZoomLevel(100);
      console.log('Reset view clicked');
    };

    const handleToggleSimulation = () => {
      setIsSimulationRunning(prev => !prev);
      console.log('Simulation toggled');
    };

    const handleOpenSettings = () => {
      alert('Opening graph settings...');
    };

    return (
      <div className="space-y-6">
        <div className="text-center">
          <h3 className="text-lg font-semibold mb-4">Interactive Graph Controls</h3>
          <div className="p-4 bg-blue-50 rounded-lg inline-block">
            <p className="text-blue-800 text-sm mb-2">Current State:</p>
            <div className="flex gap-4 text-sm">
              <span className="text-blue-700">
                Zoom: <strong>{zoomLevel}%</strong>
              </span>
              <span className="text-blue-700">
                Simulation: <strong>{isSimulationRunning ? 'Running' : 'Paused'}</strong>
              </span>
            </div>
          </div>
        </div>
        
        <div className="flex justify-center">
          <GraphControls
            onZoomIn={handleZoomIn}
            onZoomOut={handleZoomOut}
            onReset={handleReset}
            onToggleSimulation={handleToggleSimulation}
            isSimulationRunning={isSimulationRunning}
            onOpenSettings={handleOpenSettings}
          />
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive demonstration of graph controls with state tracking.',
      },
    },
  },
};

// Stats component stories
export const StatsComponent: Story = {
  render: () => (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-center">Graph Statistics</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {/* Small graph */}
        <GraphStats
          nodeCount={6}
          edgeCount={8}
          layerCount={2}
        />
        
        {/* Medium graph */}
        <GraphStats
          nodeCount={42}
          edgeCount={67}
          layerCount={4}
        />
        
        {/* Large graph with selection */}
        <GraphStats
          nodeCount={156}
          edgeCount={298}
          layerCount={5}
          selectedNode="api-gateway"
        />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Graph statistics component showing different data scenarios.',
      },
    },
  },
};

// Toolbar component stories
export const ToolbarComponent: Story = {
  render: () => {
    const [selectedLayout, setSelectedLayout] = useState('force');

    const handleExport = () => {
      alert('Exporting graph...');
    };

    const handleImport = () => {
      alert('Importing graph data...');
    };

    const handleLayoutChange = (layout: string) => {
      setSelectedLayout(layout);
      console.log('Layout changed to:', layout);
    };

    return (
      <div className="space-y-6 w-full max-w-2xl">
        <h3 className="text-lg font-semibold text-center">Graph Toolbar</h3>
        
        <div className="space-y-4">
          {/* Full toolbar */}
          <div>
            <h4 className="font-medium mb-2">Complete Toolbar</h4>
            <GraphToolbar
              onExport={handleExport}
              onImport={handleImport}
              onLayout={handleLayoutChange}
            />
          </div>
          
          {/* Layout only */}
          <div>
            <h4 className="font-medium mb-2">Layout Selection Only</h4>
            <GraphToolbar
              onLayout={handleLayoutChange}
            />
          </div>
          
          {/* Import/Export only */}
          <div>
            <h4 className="font-medium mb-2">Import/Export Only</h4>
            <GraphToolbar
              onExport={handleExport}
              onImport={handleImport}
            />
          </div>
        </div>
        
        <div className="p-4 bg-green-50 rounded-lg">
          <p className="text-green-800 text-sm">
            Current layout: <strong>{selectedLayout}</strong>
          </p>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph toolbar component with layout selection and import/export controls.',
      },
    },
  },
};

// Complete control suite
export const CompleteControlSuite: Story = {
  render: () => {
    const [isSimulationRunning, setIsSimulationRunning] = useState(true);
    const [selectedNode, setSelectedNode] = useState<string | null>(null);
    const [layout, setLayout] = useState('force');

    const nodes = [
      'api-gateway', 'user-service', 'product-service', 'order-service',
      'payment-service', 'notification-service', 'database', 'cache'
    ];

    return (
      <div className="space-y-8 p-6 bg-gray-50 rounded-lg">
        <h3 className="text-lg font-semibold text-center">Complete Graph Control Suite</h3>
        
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Controls Panel */}
          <div className="space-y-4">
            <h4 className="font-medium text-gray-900">Control Panel</h4>
            <GraphControls
              onZoomIn={() => console.log('Zoom in')}
              onZoomOut={() => console.log('Zoom out')}
              onReset={() => console.log('Reset view')}
              onToggleSimulation={() => setIsSimulationRunning(prev => !prev)}
              isSimulationRunning={isSimulationRunning}
              onOpenSettings={() => alert('Opening settings...')}
            />
          </div>

          {/* Statistics */}
          <div className="space-y-4">
            <h4 className="font-medium text-gray-900">Statistics</h4>
            <GraphStats
              nodeCount={147}
              edgeCount={298}
              layerCount={6}
              selectedNode={selectedNode}
            />
            
            {/* Node selection demo */}
            <div className="space-y-2">
              <label className="block text-sm font-medium text-gray-700">
                Select Node:
              </label>
              <select
                value={selectedNode || ''}
                onChange={(e) => setSelectedNode(e.target.value || null)}
                className="w-full text-sm border border-gray-300 rounded px-2 py-1"
              >
                <option value="">None selected</option>
                {nodes.map(node => (
                  <option key={node} value={node}>{node}</option>
                ))}
              </select>
            </div>
          </div>

          {/* Toolbar */}
          <div className="space-y-4">
            <h4 className="font-medium text-gray-900">Toolbar</h4>
            <GraphToolbar
              onExport={() => alert('Exporting graph...')}
              onImport={() => alert('Importing data...')}
              onLayout={(newLayout) => setLayout(newLayout)}
            />
            
            <div className="text-sm text-gray-600">
              Current layout: <strong>{layout}</strong>
            </div>
          </div>
        </div>

        <div className="p-4 bg-blue-50 rounded-lg">
          <h4 className="font-medium text-blue-900 mb-2">Control Suite Features</h4>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• <strong>Zoom Controls:</strong> In, out, and reset view</li>
            <li>• <strong>Simulation:</strong> Start/pause force-directed layout</li>
            <li>• <strong>Statistics:</strong> Real-time graph metrics and selection info</li>
            <li>• <strong>Layout Options:</strong> Force, circular, hierarchical, and grid</li>
            <li>• <strong>Data Operations:</strong> Import and export functionality</li>
            <li>• <strong>Settings:</strong> Access to advanced configuration</li>
          </ul>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Complete graph control suite showing all components working together.',
      },
    },
  },
};

// Mobile responsive
export const MobileControls: Story = {
  render: () => (
    <div className="w-80 space-y-4">
      <h3 className="text-lg font-semibold text-center">Mobile View</h3>
      
      <div className="space-y-4">
        {/* Compact controls */}
        <div>
          <h4 className="font-medium mb-2 text-sm">Compact Controls</h4>
          <GraphControls
            onZoomIn={() => console.log('Mobile zoom in')}
            onZoomOut={() => console.log('Mobile zoom out')}
            onReset={() => console.log('Mobile reset')}
            onToggleSimulation={() => console.log('Mobile simulation toggle')}
            isSimulationRunning={false}
            className="scale-90"
          />
        </div>

        {/* Mobile stats */}
        <div>
          <h4 className="font-medium mb-2 text-sm">Mobile Statistics</h4>
          <GraphStats
            nodeCount={42}
            edgeCount={67}
            layerCount={4}
            selectedNode="mobile-node"
          />
        </div>

        {/* Mobile toolbar */}
        <div>
          <h4 className="font-medium mb-2 text-sm">Mobile Toolbar</h4>
          <GraphToolbar
            onExport={() => alert('Mobile export')}
            onLayout={(layout) => console.log('Mobile layout:', layout)}
          />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Graph controls optimized for mobile viewport.',
      },
    },
  },
};

// Different states
export const ControlStates: Story = {
  render: () => (
    <div className="space-y-8">
      <h3 className="text-lg font-semibold text-center">Control States</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* Default state */}
        <div className="text-center space-y-2">
          <h4 className="font-medium">Default State</h4>
          <GraphControls
            onZoomIn={() => {}}
            onZoomOut={() => {}}
            onReset={() => {}}
            onToggleSimulation={() => {}}
            isSimulationRunning={false}
          />
        </div>

        {/* Simulation running */}
        <div className="text-center space-y-2">
          <h4 className="font-medium">Simulation Running</h4>
          <GraphControls
            onZoomIn={() => {}}
            onZoomOut={() => {}}
            onReset={() => {}}
            onToggleSimulation={() => {}}
            isSimulationRunning={true}
            onOpenSettings={() => {}}
          />
        </div>

        {/* With settings */}
        <div className="text-center space-y-2">
          <h4 className="font-medium">With Settings</h4>
          <GraphControls
            onZoomIn={() => {}}
            onZoomOut={() => {}}
            onReset={() => {}}
            onToggleSimulation={() => {}}
            isSimulationRunning={false}
            onOpenSettings={() => {}}
          />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different states and configurations of graph controls.',
      },
    },
  },
};

// Performance scenarios
export const PerformanceScenarios: Story = {
  render: () => {
    const scenarios = [
      { name: 'Small Graph', nodes: 12, edges: 18, layers: 2 },
      { name: 'Medium Graph', nodes: 87, edges: 156, layers: 4 },
      { name: 'Large Graph', nodes: 543, edges: 1247, layers: 8 },
      { name: 'Huge Graph', nodes: 2156, edges: 5892, layers: 12 },
    ];

    return (
      <div className="space-y-6">
        <h3 className="text-lg font-semibold text-center">Performance Scenarios</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {scenarios.map((scenario, index) => (
            <div key={index} className="border rounded-lg p-4">
              <h4 className="font-medium text-gray-900 mb-3">{scenario.name}</h4>
              
              <div className="space-y-3">
                <GraphStats
                  nodeCount={scenario.nodes}
                  edgeCount={scenario.edges}
                  layerCount={scenario.layers}
                />
                
                <GraphControls
                  onZoomIn={() => console.log(`${scenario.name} zoom in`)}
                  onZoomOut={() => console.log(`${scenario.name} zoom out`)}
                  onReset={() => console.log(`${scenario.name} reset`)}
                  onToggleSimulation={() => console.log(`${scenario.name} toggle`)}
                  isSimulationRunning={scenario.nodes < 100} // Auto-pause for large graphs
                />
              </div>
              
              <div className="mt-3 p-2 bg-gray-50 rounded text-xs text-gray-600">
                Performance: {scenario.nodes < 50 ? 'Excellent' : 
                            scenario.nodes < 200 ? 'Good' : 
                            scenario.nodes < 1000 ? 'Fair' : 'Challenging'}
              </div>
            </div>
          ))}
        </div>
        
        <div className="p-4 bg-yellow-50 rounded-lg">
          <h4 className="font-medium text-yellow-900 mb-2">Performance Guidelines</h4>
          <ul className="text-sm text-yellow-800 space-y-1">
            <li>• Small graphs (&lt;50 nodes): All features enabled, smooth interactions</li>
            <li>• Medium graphs (50-200 nodes): Good performance, minor lag possible</li>
            <li>• Large graphs (200-1000 nodes): Consider disabling simulation</li>
            <li>• Huge graphs (&gt;1000 nodes): Use performance mode, static layouts</li>
          </ul>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph controls in different performance scenarios and data sizes.',
      },
    },
  },
};