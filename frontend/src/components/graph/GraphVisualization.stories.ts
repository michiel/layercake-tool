import type { Meta, StoryObj } from '@storybook/react';
import { GraphVisualization } from './GraphVisualization';
import { mockGraphDatasets } from '../../stories/mockData';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof GraphVisualization> = {
  title: 'Graph/GraphVisualization',
  component: GraphVisualization,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: 'Interactive graph visualization component using D3.js force simulation. Displays nodes, edges, and layers with interactive capabilities including zoom, pan, and selection.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    nodes: {
      control: false,
      description: 'Array of graph nodes to display',
    },
    edges: {
      control: false,
      description: 'Array of edges connecting the nodes',
    },
    layers: {
      control: false,
      description: 'Array of layer definitions for styling',
    },
    width: {
      control: 'number',
      description: 'Width of the visualization canvas',
    },
    height: {
      control: 'number',
      description: 'Height of the visualization canvas',
    },
    onNodeClick: {
      action: 'node-clicked',
      description: 'Callback when a node is clicked',
    },
    onEdgeClick: {
      action: 'edge-clicked',
      description: 'Callback when an edge is clicked',
    },
  },
  args: { 
    onNodeClick: fn(),
    onEdgeClick: fn(),
    width: 800,
    height: 600,
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Basic visualization
export const Default: Story = {
  args: {
    nodes: mockGraphDatasets.default.nodes,
    edges: mockGraphDatasets.default.edges,
    layers: mockGraphDatasets.default.layers,
  },
  parameters: {
    docs: {
      description: {
        story: 'Default graph visualization with sample microservices architecture.',
      },
    },
  },
};

// Small graph for quick testing
export const SmallGraph: Story = {
  args: {
    nodes: mockGraphDatasets.small.nodes,
    edges: mockGraphDatasets.small.edges,
    layers: mockGraphDatasets.small.layers,
    width: 600,
    height: 400,
  },
  parameters: {
    docs: {
      description: {
        story: 'Small graph with just a few nodes, useful for testing basic functionality.',
      },
    },
  },
};

// Medium size for performance testing
export const MediumGraph: Story = {
  args: {
    nodes: mockGraphDatasets.medium.nodes,
    edges: mockGraphDatasets.medium.edges,
    layers: mockGraphDatasets.medium.layers,
    width: 1000,
    height: 700,
  },
  parameters: {
    docs: {
      description: {
        story: 'Medium-sized graph with 30 nodes and 45 edges for performance testing.',
      },
    },
  },
};

// Large graph for stress testing
export const LargeGraph: Story = {
  args: {
    nodes: mockGraphDatasets.large.nodes,
    edges: mockGraphDatasets.large.edges,
    layers: mockGraphDatasets.large.layers,
    width: 1200,
    height: 800,
  },
  parameters: {
    docs: {
      description: {
        story: 'Large graph with 150 nodes and 300 edges for stress testing and performance evaluation.',
      },
    },
  },
};

// Empty state
export const EmptyGraph: Story = {
  args: {
    nodes: mockGraphDatasets.empty.nodes,
    edges: mockGraphDatasets.empty.edges,
    layers: mockGraphDatasets.empty.layers,
  },
  parameters: {
    docs: {
      description: {
        story: 'Empty graph state with no nodes or edges, useful for testing empty state handling.',
      },
    },
  },
};

// Different canvas sizes
export const SmallCanvas: Story = {
  args: {
    nodes: mockGraphDatasets.default.nodes,
    edges: mockGraphDatasets.default.edges,
    layers: mockGraphDatasets.default.layers,
    width: 400,
    height: 300,
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph in a smaller canvas to test responsive behavior.',
      },
    },
  },
};

export const LargeCanvas: Story = {
  args: {
    nodes: mockGraphDatasets.default.nodes,
    edges: mockGraphDatasets.default.edges,
    layers: mockGraphDatasets.default.layers,
    width: 1400,
    height: 900,
  },
  parameters: {
    docs: {
      description: {
        story: 'Graph in a larger canvas for detailed visualization.',
      },
    },
  },
};

// Interactive example with event handling
export const InteractiveDemo: Story = {
  render: (args) => {
    const handleNodeClick = (node: any) => {
      console.log('Node clicked:', node);
      alert(`Clicked node: ${node.label}`);
    };
    
    const handleEdgeClick = (edge: any) => {
      console.log('Edge clicked:', edge);
      alert(`Clicked edge from ${edge.source.id} to ${edge.target.id}`);
    };
    
    return (
      <div className="w-full h-screen p-4">
        <div className="mb-4 p-4 bg-blue-50 rounded-lg">
          <h3 className="font-semibold text-blue-900">Interactive Demo</h3>
          <p className="text-blue-700 text-sm">
            Click on nodes and edges to see interaction events. Use mouse wheel to zoom and drag to pan.
          </p>
        </div>
        <GraphVisualization
          nodes={mockGraphDatasets.default.nodes}
          edges={mockGraphDatasets.default.edges}
          layers={mockGraphDatasets.default.layers}
          onNodeClick={handleNodeClick}
          onEdgeClick={handleEdgeClick}
          width={1000}
          height={600}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive demonstration with click handlers for nodes and edges.',
      },
    },
  },
};

// Performance comparison
export const PerformanceComparison: Story = {
  render: () => (
    <div className="space-y-6 p-4">
      <h2 className="text-xl font-bold">Performance Comparison</h2>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="space-y-2">
          <h3 className="font-semibold">Small Graph (6 nodes, 6 edges)</h3>
          <p className="text-sm text-gray-600">Fast rendering, smooth interactions</p>
          <div className="border rounded-lg">
            <GraphVisualization
              nodes={mockGraphDatasets.small.nodes}
              edges={mockGraphDatasets.small.edges}
              layers={mockGraphDatasets.small.layers}
              width={400}
              height={300}
            />
          </div>
        </div>
        
        <div className="space-y-2">
          <h3 className="font-semibold">Medium Graph (30 nodes, 45 edges)</h3>
          <p className="text-sm text-gray-600">Good performance for typical use cases</p>
          <div className="border rounded-lg">
            <GraphVisualization
              nodes={mockGraphDatasets.medium.nodes}
              edges={mockGraphDatasets.medium.edges}
              layers={mockGraphDatasets.medium.layers}
              width={400}
              height={300}
            />
          </div>
        </div>
      </div>
      
      <div className="space-y-2">
        <h3 className="font-semibold">Large Graph (150 nodes, 300 edges)</h3>
        <p className="text-sm text-gray-600">Stress test for performance optimization</p>
        <div className="border rounded-lg">
          <GraphVisualization
            nodes={mockGraphDatasets.large.nodes}
            edges={mockGraphDatasets.large.edges}
            layers={mockGraphDatasets.large.layers}
            width={800}
            height={400}
          />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Side-by-side comparison of different graph sizes for performance evaluation.',
      },
    },
  },
};

// Layer visualization
export const LayerDemo: Story = {
  render: () => (
    <div className="space-y-4 p-4">
      <div className="space-y-2">
        <h3 className="font-semibold">Layer Color Coding</h3>
        <div className="flex gap-4 flex-wrap">
          {mockGraphDatasets.default.layers.map((layer) => (
            <div key={layer.id} className="flex items-center gap-2">
              <div 
                className="w-4 h-4 rounded border"
                style={{ backgroundColor: layer.color }}
              />
              <span className="text-sm">{layer.name}</span>
            </div>
          ))}
        </div>
      </div>
      
      <GraphVisualization
        nodes={mockGraphDatasets.default.nodes}
        edges={mockGraphDatasets.default.edges}
        layers={mockGraphDatasets.default.layers}
        width={800}
        height={500}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Demonstration of layer-based color coding in the graph visualization.',
      },
    },
  },
};