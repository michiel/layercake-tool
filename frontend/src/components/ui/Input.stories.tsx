import type { Meta, StoryObj } from '@storybook/react';
import { Input } from './Input';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof Input> = {
  title: 'UI/Input',
  component: Input,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Form input component with label, error states, and various input types. Supports validation and accessibility features.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    label: {
      control: 'text',
      description: 'Label text for the input field',
    },
    placeholder: {
      control: 'text',
      description: 'Placeholder text shown when input is empty',
    },
    error: {
      control: 'text',
      description: 'Error message to display below the input',
    },
    disabled: {
      control: 'boolean',
      description: 'Disables the input field',
    },
    type: {
      control: 'select',
      options: ['text', 'email', 'password', 'number', 'url', 'tel'],
      description: 'HTML input type',
    },
  },
  args: { 
    onChange: fn(),
    onBlur: fn(),
    onFocus: fn(),
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Basic states
export const Default: Story = {
  args: {
    placeholder: 'Enter text...',
  },
};

export const WithLabel: Story = {
  args: {
    label: 'Project Name',
    placeholder: 'Enter project name',
  },
};

export const WithValue: Story = {
  args: {
    label: 'Project Name',
    defaultValue: 'My Awesome Project',
  },
};

export const WithError: Story = {
  args: {
    label: 'Project Name',
    defaultValue: '',
    error: 'Project name is required',
    placeholder: 'Enter project name',
  },
};

export const Disabled: Story = {
  args: {
    label: 'Disabled Input',
    defaultValue: 'Cannot edit this',
    disabled: true,
  },
};

// Input types
export const EmailInput: Story = {
  args: {
    label: 'Email Address',
    type: 'email',
    placeholder: 'user@example.com',
  },
};

export const PasswordInput: Story = {
  args: {
    label: 'Password',
    type: 'password',
    placeholder: 'Enter your password',
  },
};

export const NumberInput: Story = {
  args: {
    label: 'Port Number',
    type: 'number',
    placeholder: '3000',
  },
};

export const UrlInput: Story = {
  args: {
    label: 'Website URL',
    type: 'url',
    placeholder: 'https://example.com',
  },
};

// Form scenarios
export const ValidationStates: Story = {
  render: () => (
    <div className="space-y-4 w-80">
      <Input 
        label="Valid Input" 
        defaultValue="This looks good" 
        placeholder="Enter text..."
      />
      <Input 
        label="Input with Error" 
        defaultValue="" 
        error="This field is required"
        placeholder="Enter text..."
      />
      <Input 
        label="Disabled Input" 
        defaultValue="Cannot edit this" 
        disabled 
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different validation states of the input component.',
      },
    },
  },
};

export const InputTypes: Story = {
  render: () => (
    <div className="space-y-4 w-80">
      <Input 
        label="Text Input" 
        type="text"
        placeholder="Enter text"
      />
      <Input 
        label="Email Input" 
        type="email"
        placeholder="user@example.com"
      />
      <Input 
        label="Password Input" 
        type="password"
        placeholder="Enter password"
      />
      <Input 
        label="Number Input" 
        type="number"
        placeholder="Enter number"
      />
      <Input 
        label="URL Input" 
        type="url"
        placeholder="https://example.com"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different input types supported by the component.',
      },
    },
  },
};

export const FormExample: Story = {
  render: () => (
    <div className="space-y-4 w-96">
      <h3 className="text-lg font-semibold mb-4">Project Settings</h3>
      <Input 
        label="Project Name" 
        defaultValue="Layercake Microservices"
        placeholder="Enter project name"
      />
      <Input 
        label="Description" 
        defaultValue="A sample microservices architecture"
        placeholder="Enter project description"
      />
      <Input 
        label="Repository URL" 
        type="url"
        defaultValue="https://github.com/example/layercake"
        placeholder="https://github.com/username/repo"
      />
      <Input 
        label="Port Number" 
        type="number"
        defaultValue="3000"
        placeholder="3000"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Example form using multiple input components.',
      },
    },
  },
};