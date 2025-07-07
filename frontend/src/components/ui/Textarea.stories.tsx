import type { Meta, StoryObj } from '@storybook/react';
import { Textarea } from './Textarea';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof Textarea> = {
  title: 'UI/Textarea',
  component: Textarea,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Textarea component for multi-line text input. Supports labels, error states, helper text, and accessibility features.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    label: {
      control: 'text',
      description: 'Label text for the textarea',
    },
    placeholder: {
      control: 'text',
      description: 'Placeholder text shown when textarea is empty',
    },
    error: {
      control: 'text',
      description: 'Error message to display below the textarea',
    },
    helperText: {
      control: 'text',
      description: 'Helper text to display below the textarea',
    },
    disabled: {
      control: 'boolean',
      description: 'Disables the textarea',
    },
    rows: {
      control: 'number',
      description: 'Number of visible text lines',
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
    placeholder: 'Enter your text...',
    rows: 4,
  },
};

export const WithLabel: Story = {
  args: {
    label: 'Project Description',
    placeholder: 'Describe your project...',
    rows: 4,
  },
};

export const WithValue: Story = {
  args: {
    label: 'Project Description',
    defaultValue: 'This is a comprehensive microservices architecture designed to demonstrate layered system design patterns.',
    rows: 4,
  },
};

export const WithHelperText: Story = {
  args: {
    label: 'Description',
    placeholder: 'Enter a detailed description...',
    helperText: 'This description will be visible to all team members.',
    rows: 4,
  },
  parameters: {
    docs: {
      description: {
        story: 'Textarea with helpful guidance text below.',
      },
    },
  },
};

export const WithError: Story = {
  args: {
    label: 'Project Description',
    defaultValue: '',
    error: 'Description is required and must be at least 10 characters.',
    placeholder: 'Enter project description...',
    rows: 4,
  },
  parameters: {
    docs: {
      description: {
        story: 'Textarea showing error state with validation message.',
      },
    },
  },
};

export const Disabled: Story = {
  args: {
    label: 'Disabled Textarea',
    defaultValue: 'This content cannot be modified.',
    disabled: true,
    rows: 4,
  },
  parameters: {
    docs: {
      description: {
        story: 'Disabled textarea for read-only scenarios.',
      },
    },
  },
};

// Different sizes
export const SmallTextarea: Story = {
  args: {
    label: 'Short Note',
    placeholder: 'Add a quick note...',
    rows: 2,
  },
  parameters: {
    docs: {
      description: {
        story: 'Compact textarea for short text input.',
      },
    },
  },
};

export const LargeTextarea: Story = {
  args: {
    label: 'Detailed Documentation',
    placeholder: 'Enter comprehensive documentation...',
    rows: 8,
  },
  parameters: {
    docs: {
      description: {
        story: 'Large textarea for extensive text content.',
      },
    },
  },
};

// Form scenarios
export const ValidationStates: Story = {
  render: () => (
    <div className="space-y-6 w-96">
      <Textarea 
        label="Valid Input" 
        defaultValue="This description looks good and provides sufficient detail."
        rows={3}
      />
      <Textarea 
        label="Input with Helper Text" 
        placeholder="Enter description..."
        helperText="Minimum 20 characters required"
        rows={3}
      />
      <Textarea 
        label="Input with Error" 
        defaultValue="Too short"
        error="Description must be at least 20 characters long"
        rows={3}
      />
      <Textarea 
        label="Disabled Input" 
        defaultValue="This content is read-only"
        disabled 
        rows={3}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different validation states of the textarea component.',
      },
    },
  },
};

// Use case examples
export const ProjectForm: Story = {
  render: () => (
    <div className="space-y-6 w-[500px]">
      <h3 className="text-lg font-semibold mb-4">Create New Project</h3>
      
      <Textarea
        label="Project Description"
        placeholder="Describe the purpose and scope of your project..."
        helperText="This will help team members understand the project goals."
        rows={4}
      />
      
      <Textarea
        label="Technical Requirements"
        placeholder="List any technical requirements, dependencies, or constraints..."
        rows={5}
      />
      
      <Textarea
        label="Success Criteria"
        placeholder="Define how success will be measured..."
        helperText="Be specific about metrics and deliverables."
        rows={4}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Project creation form using multiple textarea components.',
      },
    },
  },
};

export const ConfigurationForm: Story = {
  render: () => (
    <div className="space-y-6 w-[500px]">
      <h3 className="text-lg font-semibold mb-4">Export Configuration</h3>
      
      <Textarea
        label="Export Template"
        defaultValue={`# {{meta.name}}

Generated on: {{timestamp}}

## Components
{{#each nodes}}
- {{label}}: {{description}}
{{/each}}

## Connections
{{#each edges}}
- {{source.label}} → {{target.label}}
{{/each}}`}
        helperText="Use Handlebars syntax for dynamic content generation."
        rows={12}
      />
      
      <Textarea
        label="Custom CSS Styles"
        placeholder="Enter custom CSS for styling the output..."
        rows={6}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Configuration form for export templates and styling.',
      },
    },
  },
};

export const CommentSystem: Story = {
  render: () => (
    <div className="space-y-4 w-[500px]">
      <h3 className="text-lg font-semibold mb-4">Project Comments</h3>
      
      {/* Existing comments */}
      <div className="space-y-3">
        <div className="p-4 bg-gray-50 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <div className="w-6 h-6 bg-blue-500 rounded-full flex items-center justify-center text-white text-xs">
              JD
            </div>
            <span className="font-medium text-sm">John Doe</span>
            <span className="text-xs text-gray-500">2 hours ago</span>
          </div>
          <p className="text-sm text-gray-700">
            The new architecture looks great! I especially like how you've separated the API layer from the business logic.
          </p>
        </div>
        
        <div className="p-4 bg-gray-50 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center text-white text-xs">
              AS
            </div>
            <span className="font-medium text-sm">Alice Smith</span>
            <span className="text-xs text-gray-500">1 day ago</span>
          </div>
          <p className="text-sm text-gray-700">
            Should we add monitoring components to this diagram? It would help visualize the observability layer.
          </p>
        </div>
      </div>
      
      {/* New comment form */}
      <Textarea
        label="Add Comment"
        placeholder="Share your thoughts about this project..."
        helperText="Comments are visible to all project collaborators."
        rows={4}
      />
      
      <button className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700">
        Post Comment
      </button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Comment system using textarea for user input.',
      },
    },
  },
};

// Character limits and formatting
export const WithCharacterLimit: Story = {
  render: () => {
    const maxLength = 280;
    const sampleText = "This textarea has a character limit to demonstrate how you might implement tweet-like functionality or enforce content constraints.";
    
    return (
      <div className="w-96">
        <Textarea
          label="Status Update"
          placeholder="What's happening with your project?"
          defaultValue={sampleText}
          helperText={`${sampleText.length}/${maxLength} characters`}
          rows={4}
          maxLength={maxLength}
        />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Textarea with character count and limit enforcement.',
      },
    },
  },
};

export const CodeSnippet: Story = {
  render: () => (
    <div className="w-[600px]">
      <Textarea
        label="Code Snippet"
        defaultValue={`function analyzeGraph(nodes, edges) {
  const result = {
    nodeCount: nodes.length,
    edgeCount: edges.length,
    density: (2 * edges.length) / (nodes.length * (nodes.length - 1))
  };
  
  return result;
}`}
        helperText="Share code snippets with your team for review and discussion."
        className="font-mono text-sm"
        rows={8}
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Textarea configured for code snippet input with monospace font.',
      },
    },
  },
};

export const ResizableTextarea: Story = {
  args: {
    label: 'Resizable Content',
    placeholder: 'This textarea can be resized by dragging the bottom-right corner...',
    helperText: 'Drag the corner to resize the textarea as needed.',
    rows: 4,
    className: 'resize',
  },
  parameters: {
    docs: {
      description: {
        story: 'Textarea with resize capability enabled.',
      },
    },
  },
};