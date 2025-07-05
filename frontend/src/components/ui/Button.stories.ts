import type { Meta, StoryObj } from '@storybook/react';
import { Button } from './Button';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof Button> = {
  title: 'UI/Button',
  component: Button,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Primary button component with multiple variants, sizes, and states. Supports loading states and all standard button interactions.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'danger', 'ghost', 'outline'],
      description: 'Visual style variant of the button',
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Size of the button',
    },
    loading: {
      control: 'boolean',
      description: 'Shows loading spinner and disables the button',
    },
    disabled: {
      control: 'boolean',
      description: 'Disables the button interaction',
    },
    children: {
      control: 'text',
      description: 'Button content',
    },
  },
  args: { 
    onClick: fn(),
    children: 'Button'
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Basic variants
export const Primary: Story = {
  args: {
    variant: 'primary',
    children: 'Primary Button',
  },
};

export const Secondary: Story = {
  args: {
    variant: 'secondary',
    children: 'Secondary Button',
  },
};

export const Danger: Story = {
  args: {
    variant: 'danger',
    children: 'Danger Button',
  },
};

export const Ghost: Story = {
  args: {
    variant: 'ghost',
    children: 'Ghost Button',
  },
};

export const Outline: Story = {
  args: {
    variant: 'outline',
    children: 'Outline Button',
  },
};

// Size variants
export const Small: Story = {
  args: {
    size: 'sm',
    children: 'Small Button',
  },
};

export const Medium: Story = {
  args: {
    size: 'md',
    children: 'Medium Button',
  },
};

export const Large: Story = {
  args: {
    size: 'lg',
    children: 'Large Button',
  },
};

// State variants
export const Loading: Story = {
  args: {
    loading: true,
    children: 'Loading...',
  },
};

export const Disabled: Story = {
  args: {
    disabled: true,
    children: 'Disabled Button',
  },
};

export const LoadingSecondary: Story = {
  args: {
    variant: 'secondary',
    loading: true,
    children: 'Loading...',
  },
};

// Combined examples
export const AllVariants: Story = {
  render: () => (
    <div className="flex gap-4 flex-wrap items-center">
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="danger">Danger</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="outline">Outline</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All button variants displayed together for comparison.',
      },
    },
  },
};

export const AllSizes: Story = {
  render: () => (
    <div className="flex gap-4 items-center">
      <Button size="sm">Small</Button>
      <Button size="md">Medium</Button>
      <Button size="lg">Large</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All button sizes displayed together for comparison.',
      },
    },
  },
};

export const LoadingStates: Story = {
  render: () => (
    <div className="flex gap-4 flex-wrap items-center">
      <Button loading variant="primary">Loading Primary</Button>
      <Button loading variant="secondary">Loading Secondary</Button>
      <Button loading variant="danger">Loading Danger</Button>
      <Button loading variant="ghost">Loading Ghost</Button>
      <Button loading variant="outline">Loading Outline</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All button variants in loading state.',
      },
    },
  },
};

export const DisabledStates: Story = {
  render: () => (
    <div className="flex gap-4 flex-wrap items-center">
      <Button disabled variant="primary">Disabled Primary</Button>
      <Button disabled variant="secondary">Disabled Secondary</Button>
      <Button disabled variant="danger">Disabled Danger</Button>
      <Button disabled variant="ghost">Disabled Ghost</Button>
      <Button disabled variant="outline">Disabled Outline</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All button variants in disabled state.',
      },
    },
  },
};

// Interactive examples
export const WithClickHandler: Story = {
  args: {
    variant: 'primary',
    children: 'Click me!',
    onClick: () => alert('Button clicked!'),
  },
  parameters: {
    docs: {
      description: {
        story: 'Button with click handler that shows an alert.',
      },
    },
  },
};