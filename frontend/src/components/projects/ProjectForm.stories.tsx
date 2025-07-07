import type { Meta, StoryObj } from '@storybook/react';
import { ProjectForm } from './ProjectForm';
import type { Project } from '@/types/api';
import { sampleProjects } from '@/stories/sampleData/enhancedGraphData';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof ProjectForm> = {
  title: 'Forms/ProjectForm',
  component: ProjectForm,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Form component for creating and editing projects. Features validation, loading states, and responsive design.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    project: {
      control: false,
      description: 'Existing project data for editing (optional)',
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

// Sample project data from enhanced sample data
const sampleProject: Project = {
  id: '1',
  name: sampleProjects[0].name,
  description: sampleProjects[0].description,
  created_at: sampleProjects[0].created_at,
  updated_at: sampleProjects[0].updated_at,
};

// Create new project
export const CreateProject: Story = {
  args: {
    isLoading: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Form for creating a new project with empty fields.',
      },
    },
  },
};

// Edit existing project
export const EditProject: Story = {
  args: {
    project: sampleProject,
    isLoading: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Form for editing an existing project with pre-filled data.',
      },
    },
  },
};

// Loading state
export const LoadingState: Story = {
  args: {
    project: sampleProject,
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

// Validation scenarios
export const ValidationDemo: Story = {
  render: () => {
    const handleSubmit = (data: any) => {
      console.log('Form submitted:', data);
      alert(`Project "${data.name}" would be created/updated with: ${JSON.stringify(data, null, 2)}`);
    };

    const handleCancel = () => {
      console.log('Form cancelled');
      alert('Form cancelled');
    };

    return (
      <div className="w-[500px]">
        <div className="mb-6 p-4 bg-blue-50 rounded-lg">
          <h3 className="font-semibold text-blue-900 mb-2">Interactive Demo</h3>
          <p className="text-blue-700 text-sm">
            Try submitting the form with empty fields to see validation in action.
          </p>
        </div>
        
        <ProjectForm
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
        story: 'Interactive form demonstrating validation behavior and user feedback.',
      },
    },
  },
};

// Form states showcase
export const FormStates: Story = {
  render: () => (
    <div className="space-y-8">
      <h3 className="text-lg font-semibold text-center">Project Form States</h3>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Empty form */}
        <div className="space-y-4">
          <h4 className="font-medium text-gray-900">Create New Project</h4>
          <div className="border rounded-lg p-4">
            <ProjectForm
              onSubmit={() => console.log('Creating project')}
              onCancel={() => console.log('Cancelled')}
              isLoading={false}
            />
          </div>
        </div>

        {/* Pre-filled form */}
        <div className="space-y-4">
          <h4 className="font-medium text-gray-900">Edit Existing Project</h4>
          <div className="border rounded-lg p-4">
            <ProjectForm
              project={sampleProject}
              onSubmit={() => console.log('Updating project')}
              onCancel={() => console.log('Cancelled')}
              isLoading={false}
            />
          </div>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Side-by-side comparison of create and edit form states.',
      },
    },
  },
};

// Different project examples
export const ProjectExamples: Story = {
  render: () => {
    const projects = [
      {
        name: 'E-commerce Platform',
        description: 'Full-stack e-commerce solution with microservices architecture, featuring user management, product catalog, payment processing, and order fulfillment.',
      },
      {
        name: 'Data Pipeline',
        description: 'Real-time data processing pipeline using Apache Kafka, Apache Spark, and various data storage solutions.',
      },
      {
        name: 'ML Training System',
        description: 'Machine learning model training and deployment system with automated data preprocessing, model versioning, and A/B testing capabilities.',
      },
    ];

    return (
      <div className="space-y-6">
        <h3 className="text-lg font-semibold text-center">Project Examples</h3>
        
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {projects.map((project, index) => (
            <div key={index} className="border rounded-lg p-4">
              <h4 className="font-medium text-gray-900 mb-4">{project.name}</h4>
              <ProjectForm
                project={{
                  id: `${index + 1}`,
                  name: project.name,
                  description: project.description,
                  created_at: '2025-01-15T10:30:00Z',
                  updated_at: '2025-01-15T10:30:00Z',
                }}
                onSubmit={() => console.log(`Updating ${project.name}`)}
                onCancel={() => console.log('Cancelled')}
                isLoading={false}
              />
            </div>
          ))}
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Examples of project forms with different types of content.',
      },
    },
  },
};

// Error handling scenarios
export const ErrorHandling: Story = {
  render: () => {
    const simulateNetworkError = () => {
      return new Promise((_, reject) => {
        setTimeout(() => {
          reject(new Error('Network connection failed'));
        }, 2000);
      });
    };

    const handleSubmitWithError = async (data: any) => {
      console.log('Simulating form submission with error...');
      try {
        await simulateNetworkError();
      } catch (error) {
        alert(`Submission failed: ${error.message}`);
      }
    };

    return (
      <div className="w-[500px]">
        <div className="mb-6 p-4 bg-yellow-50 rounded-lg">
          <h3 className="font-semibold text-yellow-900 mb-2">Error Simulation</h3>
          <p className="text-yellow-700 text-sm">
            This form will simulate a network error when submitted to demonstrate error handling.
          </p>
        </div>
        
        <ProjectForm
          onSubmit={handleSubmitWithError}
          onCancel={() => console.log('Cancelled')}
          isLoading={false}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Form demonstrating error handling and user feedback patterns.',
      },
    },
  },
};

// Accessibility features
export const AccessibilityDemo: Story = {
  render: () => (
    <div className="w-[500px]">
      <div className="mb-6 p-4 bg-green-50 rounded-lg">
        <h3 className="font-semibold text-green-900 mb-2">Accessibility Features</h3>
        <ul className="text-green-700 text-sm space-y-1">
          <li>• Form labels are properly associated with inputs</li>
          <li>• Required fields are indicated and validated</li>
          <li>• Error messages are announced to screen readers</li>
          <li>• Keyboard navigation works throughout the form</li>
          <li>• Loading states provide appropriate feedback</li>
        </ul>
      </div>
      
      <ProjectForm
        onSubmit={(data) => console.log('Accessible form submitted:', data)}
        onCancel={() => console.log('Accessible form cancelled')}
        isLoading={false}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Demonstration of accessibility features and best practices.',
      },
    },
  },
};

// Mobile responsive
export const MobileView: Story = {
  render: () => (
    <div className="w-80">
      <div className="mb-4 p-3 bg-purple-50 rounded-lg">
        <h3 className="font-semibold text-purple-900 mb-1 text-sm">Mobile View</h3>
        <p className="text-purple-700 text-xs">
          Form optimized for mobile devices with touch-friendly inputs.
        </p>
      </div>
      
      <ProjectForm
        project={{
          id: '1',
          name: 'Mobile Project',
          description: 'Testing mobile responsiveness',
          created_at: '2025-01-15T10:30:00Z',
          updated_at: '2025-01-15T10:30:00Z',
        }}
        onSubmit={(data) => console.log('Mobile form submitted:', data)}
        onCancel={() => console.log('Mobile form cancelled')}
        isLoading={false}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Form in mobile viewport demonstrating responsive design.',
      },
    },
  },
};

// Form flow demonstration
export const FormFlow: Story = {
  render: () => {
    const steps = [
      'Fill in project name (required)',
      'Add description (optional)',
      'Click "Create Project" to submit',
      'Form validates and submits data',
    ];

    return (
      <div className="space-y-6 w-[600px]">
        <div className="text-center">
          <h3 className="text-lg font-semibold mb-4">Project Creation Flow</h3>
          
          <div className="flex justify-center mb-6">
            <div className="flex items-center space-x-2">
              {steps.map((step, index) => (
                <div key={index} className="flex items-center">
                  <div className="w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">
                    {index + 1}
                  </div>
                  {index < steps.length - 1 && (
                    <div className="w-8 h-px bg-gray-300 mx-2"></div>
                  )}
                </div>
              ))}
            </div>
          </div>
          
          <div className="grid grid-cols-2 gap-4 text-left mb-6">
            {steps.map((step, index) => (
              <div key={index} className="flex items-start gap-2">
                <div className="w-6 h-6 bg-blue-100 text-blue-600 rounded-full flex items-center justify-center text-xs font-semibold mt-0.5">
                  {index + 1}
                </div>
                <span className="text-sm text-gray-600">{step}</span>
              </div>
            ))}
          </div>
        </div>
        
        <ProjectForm
          onSubmit={(data) => {
            console.log('Form flow completed:', data);
            alert(`Project "${data.name}" created successfully!`);
          }}
          onCancel={() => console.log('Form flow cancelled')}
          isLoading={false}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Complete form flow demonstration with step-by-step guidance.',
      },
    },
  },
};