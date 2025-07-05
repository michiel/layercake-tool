import type { Meta, StoryObj } from '@storybook/react';
import { PlanForm } from './PlanForm';
import type { Plan } from '@/types/api';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof PlanForm> = {
  title: 'Forms/PlanForm',
  component: PlanForm,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Form component for creating and editing execution plans. Features JSON/YAML format switching, template loading, and code validation.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    plan: {
      control: false,
      description: 'Existing plan data for editing (optional)',
    },
    onSubmit: {
      action: 'onSubmit',
      description: 'Callback when form is submitted with valid data',
    },
    onCancel: {
      action: 'onCancel',
      description: 'Callback when form is cancelled',
    },
    isLoading: {
      control: 'boolean',
      description: 'Loading state for submit button and form',
    },
  },
  args: {
    onSubmit: fn(),
    onCancel: fn(),
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Sample plan data
const sampleJsonPlan: Plan = {
  id: '1',
  name: 'Microservices Analysis Plan',
  plan_content: JSON.stringify({
    version: '1.0',
    name: 'Microservices Analysis',
    description: 'Analyze microservices architecture and generate reports',
    steps: [
      {
        id: 'load_data',
        type: 'import',
        source: 'services.csv',
        target: 'nodes'
      },
      {
        id: 'analyze_connectivity',
        type: 'analysis',
        input: 'nodes',
        algorithms: ['centrality', 'clustering']
      }
    ],
    exports: [
      {
        type: 'report',
        format: 'html',
        destination: 'analysis_report.html'
      }
    ]
  }, null, 2),
  plan_format: 'json',
  created_at: '2025-01-15T10:30:00Z',
  updated_at: '2025-01-15T10:30:00Z',
};

const sampleYamlPlan: Plan = {
  id: '2',
  name: 'Data Pipeline Plan',
  plan_content: `version: '1.0'
name: Data Processing Pipeline
description: ETL pipeline for processing graph data
steps:
  - id: extract
    type: import
    source: raw_data.csv
    target: staging
  - id: transform
    type: transformation
    input: staging
    operations:
      - type: filter
        condition: 'status == "active"'
      - type: aggregate
        groupBy: ['category']
  - id: load
    type: export
    input: transformed
    destination: processed_data.csv
exports:
  - type: visualization
    format: svg
    destination: graph_view.svg`,
  plan_format: 'yaml',
  created_at: '2025-01-15T10:30:00Z',
  updated_at: '2025-01-15T10:30:00Z',
};

// Create new plan
export const CreatePlan: Story = {
  args: {
    isLoading: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Form for creating a new plan with empty fields.',
      },
    },
  },
};

// Edit JSON plan
export const EditJsonPlan: Story = {
  args: {
    plan: sampleJsonPlan,
    isLoading: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Form for editing an existing JSON plan with syntax highlighting.',
      },
    },
  },
};

// Edit YAML plan
export const EditYamlPlan: Story = {
  args: {
    plan: sampleYamlPlan,
    isLoading: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Form for editing an existing YAML plan with proper formatting.',
      },
    },
  },
};

// Loading state
export const LoadingState: Story = {
  args: {
    plan: sampleJsonPlan,
    isLoading: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Form in loading state with disabled inputs and loading button.',
      },
    },
  },
};

// Template loading demo
export const TemplateDemo: Story = {
  render: () => {
    const handleSubmit = (data: any) => {
      console.log('Plan submitted:', data);
      alert(`Plan "${data.name}" would be created/updated`);
    };

    const handleCancel = () => {
      console.log('Plan form cancelled');
      alert('Form cancelled');
    };

    return (
      <div className="w-[600px]">
        <div className="mb-6 p-4 bg-green-50 rounded-lg">
          <h3 className="font-semibold text-green-900 mb-2">Template Loading</h3>
          <p className="text-green-700 text-sm">
            Start with an empty form and click "Load Template" to see pre-built plan examples.
            You can switch between JSON and YAML formats.
          </p>
        </div>
        
        <PlanForm
          onSubmit={handleSubmit}
          onCancel={handleCancel}
          isLoading={false}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive demo showing template loading functionality.',
      },
    },
  },
};

// Format switching
export const FormatSwitching: Story = {
  render: () => (
    <div className="space-y-8">
      <h3 className="text-lg font-semibold text-center">JSON vs YAML Format</h3>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* JSON Format */}
        <div className="space-y-4">
          <h4 className="font-medium text-gray-900">JSON Format</h4>
          <div className="border rounded-lg p-4">
            <PlanForm
              plan={sampleJsonPlan}
              onSubmit={() => console.log('JSON plan submitted')}
              onCancel={() => console.log('Cancelled')}
              isLoading={false}
            />
          </div>
        </div>

        {/* YAML Format */}
        <div className="space-y-4">
          <h4 className="font-medium text-gray-900">YAML Format</h4>
          <div className="border rounded-lg p-4">
            <PlanForm
              plan={sampleYamlPlan}
              onSubmit={() => console.log('YAML plan submitted')}
              onCancel={() => console.log('Cancelled')}
              isLoading={false}
            />
          </div>
        </div>
      </div>
      
      <div className="p-4 bg-blue-50 rounded-lg">
        <h4 className="font-medium text-blue-900 mb-2">Format Guidelines</h4>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• <strong>JSON</strong>: Strict syntax, better for APIs and structured data</li>
          <li>• <strong>YAML</strong>: Human-readable, better for configuration files</li>
          <li>• Both formats support the same plan structure and features</li>
          <li>• Use the format buttons to switch between modes</li>
        </ul>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Side-by-side comparison of JSON and YAML plan editing.',
      },
    },
  },
};

// Validation scenarios
export const ValidationDemo: Story = {
  render: () => {
    const invalidJsonPlan: Plan = {
      id: '3',
      name: 'Invalid Plan',
      plan_content: '{ "name": "test", "invalid": }', // Invalid JSON
      plan_format: 'json',
      created_at: '2025-01-15T10:30:00Z',
      updated_at: '2025-01-15T10:30:00Z',
    };

    return (
      <div className="space-y-6 w-[600px]">
        <div className="p-4 bg-red-50 rounded-lg">
          <h3 className="font-semibold text-red-900 mb-2">Validation Demo</h3>
          <p className="text-red-700 text-sm">
            This plan contains invalid JSON syntax. Try to submit it to see validation errors.
          </p>
        </div>
        
        <PlanForm
          plan={invalidJsonPlan}
          onSubmit={(data) => console.log('Plan with validation submitted:', data)}
          onCancel={() => console.log('Validation demo cancelled')}
          isLoading={false}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Form demonstrating validation behavior with invalid content.',
      },
    },
  },
};

// Complex plan examples
export const ComplexPlans: Story = {
  render: () => {
    const complexPlan: Plan = {
      id: '4',
      name: 'Advanced Data Processing',
      plan_content: JSON.stringify({
        version: '2.0',
        name: 'Advanced Data Processing Pipeline',
        description: 'Comprehensive data processing with multiple stages and conditional logic',
        metadata: {
          author: 'Data Engineering Team',
          version: '2.0.0',
          tags: ['etl', 'analytics', 'automation']
        },
        variables: {
          input_path: '/data/raw',
          output_path: '/data/processed',
          batch_size: 1000
        },
        steps: [
          {
            id: 'data_validation',
            type: 'validation',
            input: '${input_path}/raw_data.csv',
            rules: [
              { field: 'id', type: 'required' },
              { field: 'timestamp', type: 'datetime' },
              { field: 'value', type: 'numeric', min: 0 }
            ]
          },
          {
            id: 'data_cleaning',
            type: 'transformation',
            input: 'validated_data',
            operations: [
              { type: 'remove_duplicates', key: 'id' },
              { type: 'fill_missing', strategy: 'forward_fill' },
              { type: 'normalize', fields: ['value'] }
            ]
          },
          {
            id: 'feature_engineering',
            type: 'transformation',
            input: 'cleaned_data',
            operations: [
              { type: 'time_features', source: 'timestamp' },
              { type: 'aggregations', window: '1h', functions: ['mean', 'std'] }
            ]
          },
          {
            id: 'quality_check',
            type: 'validation',
            input: 'features',
            conditions: [
              { metric: 'completeness', threshold: 0.95 },
              { metric: 'consistency', threshold: 0.90 }
            ]
          }
        ],
        exports: [
          {
            type: 'parquet',
            source: 'features',
            destination: '${output_path}/features.parquet',
            partitionBy: ['date']
          },
          {
            type: 'report',
            format: 'html',
            destination: '${output_path}/quality_report.html',
            template: 'data_quality_template.html'
          }
        ],
        monitoring: {
          alerts: [
            {
              condition: 'quality_score < 0.8',
              action: 'email',
              recipients: ['data-team@company.com']
            }
          ]
        }
      }, null, 2),
      plan_format: 'json',
      created_at: '2025-01-15T10:30:00Z',
      updated_at: '2025-01-15T10:30:00Z',
    };

    return (
      <div className="w-[700px]">
        <div className="mb-6 p-4 bg-purple-50 rounded-lg">
          <h3 className="font-semibold text-purple-900 mb-2">Complex Plan Example</h3>
          <p className="text-purple-700 text-sm">
            This example shows a comprehensive data processing plan with multiple stages,
            variables, conditional logic, and monitoring configuration.
          </p>
        </div>
        
        <PlanForm
          plan={complexPlan}
          onSubmit={(data) => console.log('Complex plan submitted:', data)}
          onCancel={() => console.log('Complex plan cancelled')}
          isLoading={false}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Example of a complex, real-world plan with advanced features.',
      },
    },
  },
};

// Plan templates showcase
export const PlanTemplates: Story = {
  render: () => {
    const templates = [
      {
        name: 'Basic ETL',
        description: 'Simple extract, transform, load pipeline',
        format: 'json' as const,
      },
      {
        name: 'Graph Analysis',
        description: 'Network analysis and visualization',
        format: 'yaml' as const,
      },
      {
        name: 'Data Validation',
        description: 'Data quality checking and reporting',
        format: 'json' as const,
      },
    ];

    return (
      <div className="space-y-6">
        <h3 className="text-lg font-semibold text-center">Plan Templates</h3>
        
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {templates.map((template, index) => (
            <div key={index} className="border rounded-lg p-4">
              <div className="mb-4">
                <h4 className="font-medium text-gray-900">{template.name}</h4>
                <p className="text-sm text-gray-600 mt-1">{template.description}</p>
                <span className="inline-block mt-2 px-2 py-1 text-xs bg-gray-100 text-gray-700 rounded">
                  {template.format.toUpperCase()}
                </span>
              </div>
              
              <PlanForm
                onSubmit={() => console.log(`${template.name} template submitted`)}
                onCancel={() => console.log('Template cancelled')}
                isLoading={false}
              />
            </div>
          ))}
        </div>
        
        <div className="p-4 bg-gray-50 rounded-lg">
          <h4 className="font-medium text-gray-900 mb-2">Getting Started with Templates</h4>
          <ul className="text-sm text-gray-700 space-y-1">
            <li>• Click "Load Template" in any empty form to start with a pre-built example</li>
            <li>• Modify the template content to match your specific requirements</li>
            <li>• Switch between JSON and YAML formats as needed</li>
            <li>• Use the format button to auto-format JSON content</li>
          </ul>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Showcase of different plan templates and their use cases.',
      },
    },
  },
};

// Error handling
export const ErrorHandling: Story = {
  render: () => {
    const simulateValidationError = (data: any) => {
      console.log('Simulating validation error...');
      alert('Validation failed: Plan content contains syntax errors');
    };

    const simulateNetworkError = (data: any) => {
      console.log('Simulating network error...');
      setTimeout(() => {
        alert('Network error: Failed to save plan. Please try again.');
      }, 1000);
    };

    return (
      <div className="space-y-8 w-[600px]">
        <h3 className="text-lg font-semibold text-center">Error Handling Scenarios</h3>
        
        <div className="space-y-6">
          {/* Validation Error */}
          <div className="border rounded-lg p-4">
            <h4 className="font-medium text-gray-900 mb-4">Validation Error Simulation</h4>
            <PlanForm
              plan={{
                id: '1',
                name: 'Test Plan',
                plan_content: '{ invalid json }',
                plan_format: 'json',
                created_at: '2025-01-15T10:30:00Z',
                updated_at: '2025-01-15T10:30:00Z',
              }}
              onSubmit={simulateValidationError}
              onCancel={() => console.log('Cancelled')}
              isLoading={false}
            />
          </div>

          {/* Network Error */}
          <div className="border rounded-lg p-4">
            <h4 className="font-medium text-gray-900 mb-4">Network Error Simulation</h4>
            <PlanForm
              plan={sampleJsonPlan}
              onSubmit={simulateNetworkError}
              onCancel={() => console.log('Cancelled')}
              isLoading={false}
            />
          </div>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Demonstration of error handling patterns and user feedback.',
      },
    },
  },
};