import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { Modal } from './Modal';
import { Button } from './Button';
import { Input } from './Input';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof Modal> = {
  title: 'UI/Modal',
  component: Modal,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: 'Modal overlay component for dialogs, forms, and content display. Supports keyboard navigation and click-outside-to-close.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    isOpen: {
      control: 'boolean',
      description: 'Controls whether the modal is visible',
    },
    title: {
      control: 'text',
      description: 'Modal title displayed in the header',
    },
    onClose: {
      action: 'onClose',
      description: 'Callback function called when modal should close',
    },
    children: {
      control: false,
      description: 'Modal content',
    },
  },
  args: { 
    onClose: fn(),
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Interactive wrapper component for Storybook
const ModalWrapper = ({ 
  children, 
  title = 'Example Modal',
  buttonText = 'Open Modal',
  ...modalProps 
}: any) => {
  const [isOpen, setIsOpen] = useState(false);
  
  return (
    <div className="p-8">
      <Button onClick={() => setIsOpen(true)}>{buttonText}</Button>
      <Modal
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        title={title}
        {...modalProps}
      >
        {children}
      </Modal>
    </div>
  );
};

// Basic modal
export const Default: Story = {
  render: () => (
    <ModalWrapper title="Basic Modal">
      <p className="text-gray-600 mb-4">
        This is a basic modal with some content. You can close it by clicking the X button, 
        pressing Escape, or clicking outside the modal.
      </p>
      <div className="flex gap-2 justify-end">
        <Button variant="outline">Cancel</Button>
        <Button variant="primary">Confirm</Button>
      </div>
    </ModalWrapper>
  ),
};

// Modal with form
export const WithForm: Story = {
  render: () => (
    <ModalWrapper title="Create New Project" buttonText="Create Project">
      <div className="space-y-4">
        <Input 
          label="Project Name" 
          placeholder="Enter project name"
        />
        <Input 
          label="Description" 
          placeholder="Enter project description"
        />
        <Input 
          label="Repository URL" 
          type="url"
          placeholder="https://github.com/username/repo"
        />
        <div className="flex gap-2 justify-end mt-6">
          <Button variant="outline">Cancel</Button>
          <Button variant="primary">Create Project</Button>
        </div>
      </div>
    </ModalWrapper>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Modal containing a form for creating a new project.',
      },
    },
  },
};

// Confirmation modal
export const Confirmation: Story = {
  render: () => (
    <ModalWrapper title="Delete Project" buttonText="Delete Project">
      <div className="space-y-4">
        <p className="text-gray-600">
          Are you sure you want to delete this project? This action cannot be undone.
        </p>
        <div className="bg-red-50 border border-red-200 rounded-lg p-3">
          <p className="text-red-800 text-sm font-medium">
            ⚠️ This will permanently delete:
          </p>
          <ul className="text-red-700 text-sm mt-2 space-y-1">
            <li>• All project data and configurations</li>
            <li>• Graph visualizations and analysis</li>
            <li>• Export templates and outputs</li>
          </ul>
        </div>
        <div className="flex gap-2 justify-end">
          <Button variant="outline">Cancel</Button>
          <Button variant="danger">Delete Project</Button>
        </div>
      </div>
    </ModalWrapper>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Confirmation modal with warning content and destructive action.',
      },
    },
  },
};

// Large content modal
export const LargeContent: Story = {
  render: () => (
    <ModalWrapper title="Project Analysis Report" buttonText="View Report">
      <div className="space-y-4 max-h-96 overflow-y-auto">
        <div className="grid grid-cols-2 gap-4">
          <div className="bg-blue-50 p-4 rounded-lg">
            <h4 className="font-semibold text-blue-900">Nodes</h4>
            <p className="text-2xl font-bold text-blue-600">147</p>
            <p className="text-sm text-blue-700">Total components</p>
          </div>
          <div className="bg-green-50 p-4 rounded-lg">
            <h4 className="font-semibold text-green-900">Connections</h4>
            <p className="text-2xl font-bold text-green-600">298</p>
            <p className="text-sm text-green-700">Active links</p>
          </div>
        </div>
        
        <div className="space-y-3">
          <h4 className="font-semibold">Analysis Results</h4>
          <div className="space-y-2 text-sm">
            <p>• <strong>Connectivity:</strong> High interconnection between services</p>
            <p>• <strong>Bottlenecks:</strong> API Gateway shows high centrality</p>
            <p>• <strong>Redundancy:</strong> Critical services have backup paths</p>
            <p>• <strong>Performance:</strong> Average latency under acceptable limits</p>
          </div>
        </div>
        
        <div className="space-y-3">
          <h4 className="font-semibold">Recommendations</h4>
          <div className="space-y-2 text-sm">
            <p>1. Consider load balancing for API Gateway</p>
            <p>2. Add monitoring for critical data flows</p>
            <p>3. Implement circuit breakers for external dependencies</p>
            <p>4. Optimize database connection pooling</p>
          </div>
        </div>
        
        <div className="flex gap-2 justify-end pt-4 border-t">
          <Button variant="outline">Export Report</Button>
          <Button variant="primary">Close</Button>
        </div>
      </div>
    </ModalWrapper>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Modal with scrollable content and complex layout.',
      },
    },
  },
};

// Without title
export const WithoutTitle: Story = {
  render: () => (
    <ModalWrapper title="" buttonText="Open Simple Modal">
      <div className="text-center space-y-4">
        <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto">
          <span className="text-2xl">✅</span>
        </div>
        <h3 className="text-lg font-semibold">Success!</h3>
        <p className="text-gray-600">
          Your project has been created successfully. You can now start building your graph.
        </p>
        <Button variant="primary" className="w-full">Continue</Button>
      </div>
    </ModalWrapper>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Modal without a title, useful for success messages or simple content.',
      },
    },
  },
};

// Loading state
export const LoadingState: Story = {
  render: () => (
    <ModalWrapper title="Processing..." buttonText="Start Process">
      <div className="text-center space-y-4">
        <div className="w-16 h-16 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin mx-auto"></div>
        <h3 className="text-lg font-semibold">Processing your request</h3>
        <p className="text-gray-600">
          Please wait while we analyze your graph data. This may take a few moments.
        </p>
        <div className="w-full bg-gray-200 rounded-full h-2">
          <div className="bg-blue-600 h-2 rounded-full animate-pulse" style={{ width: '60%' }}></div>
        </div>
        <p className="text-sm text-gray-500">Analyzing 147 nodes and 298 connections...</p>
      </div>
    </ModalWrapper>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Modal showing loading state with progress indicator.',
      },
    },
  },
};