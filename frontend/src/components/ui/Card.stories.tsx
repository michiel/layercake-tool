import type { Meta, StoryObj } from '@storybook/react';
import { Card, CardHeader, CardContent, CardFooter } from './Card';
import { Button } from './Button';

const meta: Meta<typeof Card> = {
  title: 'UI/Card',
  component: Card,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Card component with header, content, and footer sections. Supports dark mode and flexible content layout.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    children: {
      control: false,
      description: 'Card content',
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes',
    },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Basic card
export const Default: Story = {
  render: () => (
    <Card className="w-80">
      <CardContent>
        <p className="text-gray-600 dark:text-gray-400">
          This is a basic card with simple content.
        </p>
      </CardContent>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic card with content only.',
      },
    },
  },
};

// Card with header
export const WithHeader: Story = {
  render: () => (
    <Card className="w-80">
      <CardHeader>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Project Overview
        </h3>
      </CardHeader>
      <CardContent>
        <p className="text-gray-600 dark:text-gray-400">
          A comprehensive view of your project's structure and components.
        </p>
      </CardContent>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Card with a header section containing a title.',
      },
    },
  },
};

// Card with header and footer
export const WithHeaderAndFooter: Story = {
  render: () => (
    <Card className="w-80">
      <CardHeader>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Layercake Project
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
          Created 2 days ago
        </p>
      </CardHeader>
      <CardContent>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          A sample microservices architecture with multiple layers and components.
        </p>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-500">Nodes:</span>
            <span className="font-medium">24</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Edges:</span>
            <span className="font-medium">18</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Layers:</span>
            <span className="font-medium">4</span>
          </div>
        </div>
      </CardContent>
      <CardFooter>
        <div className="flex gap-2 w-full">
          <Button variant="outline" size="sm" className="flex-1">
            Edit
          </Button>
          <Button variant="primary" size="sm" className="flex-1">
            View
          </Button>
        </div>
      </CardFooter>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete card with header, content, and footer with actions.',
      },
    },
  },
};

// Stats card
export const StatsCard: Story = {
  render: () => (
    <Card className="w-64">
      <CardContent>
        <div className="text-center">
          <div className="text-3xl font-bold text-blue-600 dark:text-blue-400">
            147
          </div>
          <div className="text-sm font-medium text-gray-900 dark:text-white mt-1">
            Total Nodes
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            ↑ 12% from last week
          </div>
        </div>
      </CardContent>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Statistics card for displaying metrics and KPIs.',
      },
    },
  },
};

// Feature card
export const FeatureCard: Story = {
  render: () => (
    <Card className="w-80">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-green-100 dark:bg-green-900 rounded-lg flex items-center justify-center">
            <span className="text-green-600 dark:text-green-400 text-lg">📊</span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Graph Analysis
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Advanced analytics
            </p>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          Analyze your graph structure with advanced algorithms including centrality measures, 
          community detection, and connectivity analysis.
        </p>
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm">
            <span className="w-2 h-2 bg-green-500 rounded-full"></span>
            <span className="text-gray-600 dark:text-gray-400">Centrality measures</span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <span className="w-2 h-2 bg-green-500 rounded-full"></span>
            <span className="text-gray-600 dark:text-gray-400">Community detection</span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <span className="w-2 h-2 bg-green-500 rounded-full"></span>
            <span className="text-gray-600 dark:text-gray-400">Path analysis</span>
          </div>
        </div>
      </CardContent>
      <CardFooter>
        <Button variant="primary" className="w-full">
          Start Analysis
        </Button>
      </CardFooter>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Feature showcase card with icon, description, and call-to-action.',
      },
    },
  },
};

// Alert card
export const AlertCard: Story = {
  render: () => (
    <Card className="w-80 border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-900/20">
      <CardContent>
        <div className="flex items-start gap-3">
          <div className="w-6 h-6 text-yellow-600 dark:text-yellow-400 mt-0.5">
            ⚠️
          </div>
          <div className="flex-1">
            <h4 className="font-medium text-yellow-800 dark:text-yellow-200 mb-1">
              Large Graph Detected
            </h4>
            <p className="text-sm text-yellow-700 dark:text-yellow-300 mb-3">
              Your graph contains over 1000 nodes. Consider enabling performance mode 
              for better rendering performance.
            </p>
            <Button variant="outline" size="sm" className="text-yellow-700 border-yellow-300 hover:bg-yellow-100">
              Enable Performance Mode
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Alert card with warning styling and action button.',
      },
    },
  },
};

// Card variants showcase
export const CardVariants: Story = {
  render: () => (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-4">
      {/* Basic Card */}
      <Card className="w-64">
        <CardContent>
          <h4 className="font-medium text-gray-900 dark:text-white mb-2">
            Basic Card
          </h4>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Simple card with content only.
          </p>
        </CardContent>
      </Card>

      {/* Stats Card */}
      <Card className="w-64">
        <CardContent className="text-center">
          <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
            298
          </div>
          <div className="text-sm text-gray-900 dark:text-white">
            Active Connections
          </div>
        </CardContent>
      </Card>

      {/* Action Card */}
      <Card className="w-64">
        <CardHeader>
          <h4 className="font-medium text-gray-900 dark:text-white">
            Quick Actions
          </h4>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
            Common project operations.
          </p>
        </CardContent>
        <CardFooter>
          <Button variant="primary" size="sm" className="w-full">
            Execute Plan
          </Button>
        </CardFooter>
      </Card>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Showcase of different card variants and use cases.',
      },
    },
  },
};

// Complex content card
export const ComplexContent: Story = {
  render: () => (
    <Card className="w-96">
      <CardHeader>
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Execution History
          </h3>
          <span className="text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 px-2 py-1 rounded">
            Last 7 days
          </span>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div className="flex items-center gap-3">
              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
              <div>
                <div className="text-sm font-medium text-gray-900 dark:text-white">
                  Export to DOT format
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  2 hours ago
                </div>
              </div>
            </div>
            <span className="text-xs text-green-600 dark:text-green-400 font-medium">
              Success
            </span>
          </div>
          
          <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div className="flex items-center gap-3">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <div>
                <div className="text-sm font-medium text-gray-900 dark:text-white">
                  Graph analysis
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  1 day ago
                </div>
              </div>
            </div>
            <span className="text-xs text-red-600 dark:text-red-400 font-medium">
              Failed
            </span>
          </div>
          
          <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div className="flex items-center gap-3">
              <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
              <div>
                <div className="text-sm font-medium text-gray-900 dark:text-white">
                  CSV import
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  3 days ago
                </div>
              </div>
            </div>
            <span className="text-xs text-blue-600 dark:text-blue-400 font-medium">
              Running
            </span>
          </div>
        </div>
      </CardContent>
      <CardFooter>
        <Button variant="outline" size="sm" className="w-full">
          View All History
        </Button>
      </CardFooter>
    </Card>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Card with complex content layout including status indicators and lists.',
      },
    },
  },
};