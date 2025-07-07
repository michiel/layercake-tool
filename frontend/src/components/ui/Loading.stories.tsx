import type { Meta, StoryObj } from '@storybook/react';
import { Loading } from './Loading';

const meta: Meta<typeof Loading> = {
  title: 'UI/Loading',
  component: Loading,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Loading spinner component with customizable size and message. Uses Lucide React icons with smooth animations.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Size of the loading spinner',
    },
    message: {
      control: 'text',
      description: 'Loading message to display below the spinner',
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes',
    },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Default loading
export const Default: Story = {
  args: {},
};

// Different sizes
export const Small: Story = {
  args: {
    size: 'sm',
    message: 'Loading...',
  },
  parameters: {
    docs: {
      description: {
        story: 'Small loading spinner for inline use or compact spaces.',
      },
    },
  },
};

export const Medium: Story = {
  args: {
    size: 'md',
    message: 'Loading...',
  },
  parameters: {
    docs: {
      description: {
        story: 'Medium loading spinner for general use cases.',
      },
    },
  },
};

export const Large: Story = {
  args: {
    size: 'lg',
    message: 'Loading...',
  },
  parameters: {
    docs: {
      description: {
        story: 'Large loading spinner for prominent loading states.',
      },
    },
  },
};

// Custom messages
export const LoadingProjects: Story = {
  args: {
    size: 'md',
    message: 'Loading projects...',
  },
  parameters: {
    docs: {
      description: {
        story: 'Loading state for project data.',
      },
    },
  },
};

export const ProcessingGraph: Story = {
  args: {
    size: 'lg',
    message: 'Processing graph data...',
  },
  parameters: {
    docs: {
      description: {
        story: 'Loading state for graph processing operations.',
      },
    },
  },
};

export const AnalyzingData: Story = {
  args: {
    size: 'md',
    message: 'Analyzing graph structure and connections...',
  },
  parameters: {
    docs: {
      description: {
        story: 'Loading state for data analysis operations.',
      },
    },
  },
};

export const NoMessage: Story = {
  args: {
    size: 'md',
    message: '',
  },
  parameters: {
    docs: {
      description: {
        story: 'Loading spinner without message text.',
      },
    },
  },
};

// Contextual loading states
export const AllSizes: Story = {
  render: () => (
    <div className="space-y-8">
      <div className="text-center">
        <h3 className="text-lg font-semibold mb-4">Loading Spinner Sizes</h3>
        <div className="flex items-center justify-center gap-8">
          <div className="text-center">
            <Loading size="sm" message="Small" />
          </div>
          <div className="text-center">
            <Loading size="md" message="Medium" />
          </div>
          <div className="text-center">
            <Loading size="lg" message="Large" />
          </div>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Comparison of all available loading spinner sizes.',
      },
    },
  },
};

// In-context examples
export const InlineLoading: Story = {
  render: () => (
    <div className="space-y-6 w-96">
      {/* Inline with content */}
      <div className="flex items-center gap-3 p-4 border rounded-lg">
        <Loading size="sm" message="" className="p-0" />
        <span className="text-gray-700">Saving changes...</span>
      </div>

      {/* Button-like loading */}
      <div className="flex items-center justify-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg">
        <Loading size="sm" message="" className="p-0" />
        <span>Processing</span>
      </div>

      {/* Card loading state */}
      <div className="border rounded-lg p-6">
        <Loading size="md" message="Loading graph visualization..." />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Loading component used in different contexts and layouts.',
      },
    },
  },
};

// Different use cases
export const UseCases: Story = {
  render: () => (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 p-4">
      {/* Page loading */}
      <div className="border rounded-lg h-48 flex items-center justify-center bg-gray-50">
        <Loading size="lg" message="Loading application..." />
      </div>

      {/* Data loading */}
      <div className="border rounded-lg h-48 flex items-center justify-center">
        <Loading size="md" message="Fetching project data..." />
      </div>

      {/* Quick action */}
      <div className="border rounded-lg h-48 flex items-center justify-center bg-blue-50">
        <Loading size="sm" message="Executing plan..." />
      </div>

      {/* Background process */}
      <div className="border rounded-lg h-48 flex items-center justify-center bg-green-50">
        <Loading size="md" message="Analyzing graph topology..." />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different use cases showing loading states in various contexts.',
      },
    },
  },
};

// Performance scenarios
export const PerformanceScenarios: Story = {
  render: () => (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-center">Performance Loading States</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Fast operation */}
        <div className="text-center p-4 border rounded-lg">
          <Loading size="sm" message="Saving..." />
          <p className="text-xs text-gray-500 mt-2">Quick operations (~1s)</p>
        </div>

        {/* Medium operation */}
        <div className="text-center p-4 border rounded-lg">
          <Loading size="md" message="Processing..." />
          <p className="text-xs text-gray-500 mt-2">Medium operations (~10s)</p>
        </div>

        {/* Long operation */}
        <div className="text-center p-4 border rounded-lg">
          <Loading size="lg" message="Analyzing large dataset..." />
          <p className="text-xs text-gray-500 mt-2">Long operations (~60s+)</p>
        </div>
      </div>

      <div className="p-4 bg-blue-50 rounded-lg">
        <h4 className="font-medium text-blue-900 mb-2">Loading Guidelines</h4>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• Use <strong>small</strong> spinners for quick actions and inline loading</li>
          <li>• Use <strong>medium</strong> spinners for standard data loading</li>
          <li>• Use <strong>large</strong> spinners for page-level loading or long operations</li>
          <li>• Always provide meaningful messages for operations over 2 seconds</li>
        </ul>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Guidelines and examples for different performance scenarios.',
      },
    },
  },
};