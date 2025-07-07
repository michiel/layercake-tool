import type { Meta, StoryObj } from '@storybook/react';
import { ErrorMessage } from './ErrorMessage';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof ErrorMessage> = {
  title: 'UI/ErrorMessage',
  component: ErrorMessage,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Error message component for displaying error states with optional retry functionality. Features alert icon and consistent styling.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    message: {
      control: 'text',
      description: 'Error message to display',
    },
    onRetry: {
      action: 'onRetry',
      description: 'Optional callback for retry functionality',
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes',
    },
  },
  args: {
    onRetry: fn(),
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Basic error message
export const Default: Story = {
  args: {
    message: 'Something went wrong while processing your request.',
  },
};

export const WithRetry: Story = {
  args: {
    message: 'Failed to load project data. Please try again.',
    onRetry: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'Error message with retry button for recoverable errors.',
      },
    },
  },
};

export const WithoutRetry: Story = {
  args: {
    message: 'Access denied. You do not have permission to view this resource.',
  },
  parameters: {
    docs: {
      description: {
        story: 'Error message without retry option for non-recoverable errors.',
      },
    },
  },
};

// Different error scenarios
export const NetworkError: Story = {
  args: {
    message: 'Network connection failed. Please check your internet connection and try again.',
    onRetry: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'Network connectivity error with retry option.',
      },
    },
  },
};

export const ValidationError: Story = {
  args: {
    message: 'The uploaded file contains invalid data. Please check the format and try again.',
    onRetry: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'Validation error for user input.',
      },
    },
  },
};

export const ServerError: Story = {
  args: {
    message: 'Server is temporarily unavailable. Our team has been notified and is working to resolve the issue.',
    onRetry: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'Server error with recovery option.',
      },
    },
  },
};

export const AuthenticationError: Story = {
  args: {
    message: 'Your session has expired. Please log in again to continue.',
  },
  parameters: {
    docs: {
      description: {
        story: 'Authentication error without retry (requires login).',
      },
    },
  },
};

export const NotFoundError: Story = {
  args: {
    message: 'The requested project could not be found. It may have been deleted or moved.',
  },
  parameters: {
    docs: {
      description: {
        story: '404 not found error.',
      },
    },
  },
};

// Complex error scenarios
export const APIError: Story = {
  args: {
    message: 'API request failed with status 500. The server encountered an internal error while processing your request.',
    onRetry: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'Detailed API error with technical information.',
      },
    },
  },
};

export const FileUploadError: Story = {
  args: {
    message: 'File upload failed. The file size exceeds the 10MB limit or the format is not supported.',
    onRetry: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: 'File upload error with specific constraints.',
      },
    },
  },
};

// Error message variations
export const ErrorStates: Story = {
  render: () => (
    <div className="space-y-8 w-[500px]">
      <h3 className="text-lg font-semibold text-center">Error Message Types</h3>
      
      <div className="space-y-6">
        {/* Quick error */}
        <div className="border rounded-lg p-4">
          <h4 className="font-medium mb-2 text-gray-900">Quick Error (with retry)</h4>
          <ErrorMessage 
            message="Connection timeout. Please try again."
            onRetry={() => console.log('Retrying...')}
          />
        </div>
        
        {/* Detailed error */}
        <div className="border rounded-lg p-4">
          <h4 className="font-medium mb-2 text-gray-900">Detailed Error</h4>
          <ErrorMessage 
            message="Failed to execute graph analysis. The dataset contains circular dependencies that cannot be resolved automatically."
            onRetry={() => console.log('Retrying analysis...')}
          />
        </div>
        
        {/* Final error (no retry) */}
        <div className="border rounded-lg p-4">
          <h4 className="font-medium mb-2 text-gray-900">Final Error (no retry)</h4>
          <ErrorMessage 
            message="Project deletion failed. Please contact support for assistance."
          />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different types of error messages for various scenarios.',
      },
    },
  },
};

// In-context usage
export const InContext: Story = {
  render: () => (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-center">Error Messages in Context</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Card with error */}
        <div className="border rounded-lg">
          <div className="p-4 border-b">
            <h4 className="font-medium">Project Dashboard</h4>
          </div>
          <div className="p-4">
            <ErrorMessage 
              message="Unable to load project statistics."
              onRetry={() => console.log('Refreshing dashboard...')}
            />
          </div>
        </div>
        
        {/* Full page error */}
        <div className="border rounded-lg h-64 flex items-center justify-center">
          <ErrorMessage 
            message="Page not found. The requested resource could not be located."
          />
        </div>
        
        {/* Form error */}
        <div className="border rounded-lg p-4">
          <h4 className="font-medium mb-4">Create Project</h4>
          <div className="space-y-3">
            <input 
              type="text" 
              placeholder="Project name"
              className="w-full p-2 border rounded"
            />
            <textarea 
              placeholder="Description"
              className="w-full p-2 border rounded h-20 resize-none"
            />
          </div>
          <div className="mt-4">
            <ErrorMessage 
              message="Project name already exists. Please choose a different name."
              onRetry={() => console.log('Checking availability...')}
            />
          </div>
        </div>
        
        {/* List item error */}
        <div className="border rounded-lg p-4">
          <h4 className="font-medium mb-4">Recent Projects</h4>
          <div className="space-y-2">
            <div className="p-2 border rounded">Project Alpha</div>
            <div className="p-2 border rounded">Project Beta</div>
            <div className="p-2 border rounded border-red-200 bg-red-50">
              <ErrorMessage 
                message="Project Gamma failed to load."
                onRetry={() => console.log('Reloading project...')}
                className="p-2"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Error messages used in different UI contexts and layouts.',
      },
    },
  },
};

// Interactive demo
export const InteractiveDemo: Story = {
  render: () => {
    const handleRetry = () => {
      alert('Retry action triggered! In a real app, this would attempt to recover from the error.');
    };
    
    return (
      <div className="space-y-6 w-[500px]">
        <h3 className="text-lg font-semibold text-center">Interactive Error Demo</h3>
        
        <div className="p-4 bg-gray-50 rounded-lg">
          <h4 className="font-medium text-gray-900 mb-2">Simulation</h4>
          <p className="text-sm text-gray-600 mb-4">
            Click the retry button below to see the interaction in action.
          </p>
        </div>
        
        <ErrorMessage 
          message="Graph visualization failed to render. This could be due to a large dataset or browser performance limitations."
          onRetry={handleRetry}
        />
        
        <div className="p-4 bg-blue-50 rounded-lg">
          <h4 className="font-medium text-blue-900 mb-2">Error Handling Best Practices</h4>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• Provide clear, actionable error messages</li>
            <li>• Include retry functionality for recoverable errors</li>
            <li>• Use consistent styling and iconography</li>
            <li>• Consider the user's context and next steps</li>
          </ul>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive demonstration of error message functionality.',
      },
    },
  },
};

// Error message guidelines
export const Guidelines: Story = {
  render: () => (
    <div className="space-y-6 w-[600px]">
      <h3 className="text-lg font-semibold text-center">Error Message Guidelines</h3>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="space-y-4">
          <h4 className="font-medium text-green-600">✅ Good Examples</h4>
          
          <div className="border border-green-200 rounded-lg p-3">
            <ErrorMessage 
              message="File upload failed. Please ensure your file is under 10MB and try again."
              onRetry={() => {}}
            />
          </div>
          
          <div className="border border-green-200 rounded-lg p-3">
            <ErrorMessage 
              message="Connection lost. Reconnecting automatically in 5 seconds..."
            />
          </div>
        </div>
        
        <div className="space-y-4">
          <h4 className="font-medium text-red-600">❌ Poor Examples</h4>
          
          <div className="border border-red-200 rounded-lg p-3 opacity-60">
            <ErrorMessage 
              message="Error 500"
            />
          </div>
          
          <div className="border border-red-200 rounded-lg p-3 opacity-60">
            <ErrorMessage 
              message="An unexpected error occurred. Contact the system administrator."
            />
          </div>
        </div>
      </div>
      
      <div className="p-4 bg-gray-50 rounded-lg">
        <h4 className="font-medium text-gray-900 mb-2">Writing Effective Error Messages</h4>
        <ul className="text-sm text-gray-700 space-y-1">
          <li>• <strong>Be specific:</strong> Explain what went wrong and why</li>
          <li>• <strong>Be helpful:</strong> Suggest what the user can do to fix it</li>
          <li>• <strong>Be human:</strong> Use plain language, avoid technical jargon</li>
          <li>• <strong>Be actionable:</strong> Provide clear next steps or retry options</li>
        </ul>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Guidelines and examples for writing effective error messages.',
      },
    },
  },
};