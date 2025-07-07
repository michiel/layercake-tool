import type { Meta, StoryObj } from '@storybook/react';

// Simple introduction component
const Introduction = () => {
  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6">
      <header className="text-center space-y-4">
        <h1 className="text-4xl font-bold text-gray-900">Layercake Design System</h1>
        <p className="text-lg text-gray-600">
          Welcome to the Layercake component library documentation. This Storybook provides comprehensive 
          documentation, examples, and interactive testing for all UI components in the Layercake graph 
          visualization and analysis platform.
        </p>
      </header>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold text-gray-900">What is Layercake?</h2>
        <p className="text-gray-700">
          Layercake is a powerful tool for visualizing and analyzing complex system architectures, data flows, 
          and relationships. It transforms CSV data and YAML/JSON plans into interactive graph visualizations 
          with advanced analysis capabilities.
        </p>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold text-gray-900">Component Library Overview</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">🎨 UI Components</h3>
            <p className="text-gray-600">Basic building blocks for the user interface:</p>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• Button - Primary action buttons with multiple variants</li>
              <li>• Input - Form inputs with validation and error states</li>
              <li>• Modal - Overlay dialogs for forms and content</li>
              <li>• Card - Container components for grouping content</li>
              <li>• Loading - Various loading states and spinners</li>
            </ul>
          </div>

          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">📊 Graph Components</h3>
            <p className="text-gray-600">Specialized components for graph visualization:</p>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• GraphVisualization - D3.js-based interactive graph rendering</li>
              <li>• GraphSettings - Configuration panel for graph display</li>
              <li>• GraphToolbar - Controls for graph manipulation</li>
              <li>• GraphMinimap - Overview navigation for large graphs</li>
              <li>• GraphInspector - Element inspection and editing</li>
            </ul>
          </div>

          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">📝 Form Components</h3>
            <p className="text-gray-600">Domain-specific form components:</p>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• ProjectForm - Project creation and editing</li>
              <li>• PlanForm - Plan configuration with JSON/YAML editing</li>
            </ul>
          </div>

          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">🏗️ Layout Components</h3>
            <p className="text-gray-600">Application structure and navigation:</p>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• AppLayout - Main application shell</li>
              <li>• Header - Navigation and user controls</li>
              <li>• Sidebar - Navigation sidebar</li>
            </ul>
          </div>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold text-gray-900">Key Features</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">Interactive Graph Visualization</h3>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• D3.js force-directed layout</li>
              <li>• Zoom and pan capabilities</li>
              <li>• Node and edge interactions</li>
              <li>• Layer-based styling</li>
              <li>• Performance optimized for 1000+ nodes</li>
            </ul>
          </div>

          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">Comprehensive UI Kit</h3>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• Complete set of form components</li>
              <li>• Modal dialogs and overlays</li>
              <li>• Loading states and error handling</li>
              <li>• Responsive design support</li>
            </ul>
          </div>

          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">Type Safety</h3>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• Full TypeScript support</li>
              <li>• Documented component APIs</li>
              <li>• IntelliSense and autocomplete</li>
              <li>• Runtime type checking</li>
            </ul>
          </div>

          <div className="space-y-3">
            <h3 className="text-lg font-medium text-gray-900">Testing and Quality</h3>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• Visual regression testing</li>
              <li>• Accessibility compliance checking</li>
              <li>• Interactive component testing</li>
              <li>• Performance monitoring</li>
            </ul>
          </div>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold text-gray-900">Getting Started</h2>
        <div className="bg-gray-50 rounded-lg p-4">
          <h3 className="text-lg font-medium text-gray-900 mb-2">Installation</h3>
          <pre className="text-sm text-gray-700 bg-gray-100 p-2 rounded">
{`# Install dependencies
npm install

# Start Storybook
npm run storybook`}
          </pre>
        </div>
      </section>

      <footer className="text-center space-y-2">
        <p className="text-gray-600">
          <strong>Ready to explore?</strong> Start with the Button component or dive into the Graph Visualization 
          to see the power of Layercake components.
        </p>
      </footer>
    </div>
  );
};

const meta: Meta<typeof Introduction> = {
  title: 'Introduction/Overview',
  component: Introduction,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: 'Welcome to the Layercake component library documentation and interactive examples.',
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};