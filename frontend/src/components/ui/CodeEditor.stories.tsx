import type { Meta, StoryObj } from '@storybook/react';
import { useState } from 'react';
import { CodeEditor } from './CodeEditor';

// Mock function for actions
const fn = () => () => {};

const meta: Meta<typeof CodeEditor> = {
  title: 'UI/CodeEditor',
  component: CodeEditor,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Code editor component with syntax highlighting, line numbers, and formatting. Supports JSON and YAML languages with tab indentation.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    value: {
      control: 'text',
      description: 'Current value of the editor',
    },
    onChange: {
      action: 'onChange',
      description: 'Callback when value changes',
    },
    language: {
      control: 'select',
      options: ['json', 'yaml'],
      description: 'Programming language for syntax highlighting',
    },
    placeholder: {
      control: 'text',
      description: 'Placeholder text when editor is empty',
    },
    disabled: {
      control: 'boolean',
      description: 'Disables the editor',
    },
    error: {
      control: 'text',
      description: 'Error message to display',
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes',
    },
  },
  args: {
    onChange: fn(),
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

// Sample JSON content
const sampleJson = `{
  "meta": {
    "name": "Sample Project",
    "description": "A sample layercake project"
  },
  "import": {
    "profiles": [
      {
        "filename": "nodes.csv",
        "filetype": "Nodes"
      },
      {
        "filename": "edges.csv",
        "filetype": "Edges"
      }
    ]
  },
  "export": {
    "profiles": [
      {
        "filename": "output.dot",
        "exporter": "DOT"
      }
    ]
  }
}`;

// Sample YAML content
const sampleYaml = `meta:
  name: Sample Project
  description: A sample layercake project

import:
  profiles:
    - filename: nodes.csv
      filetype: Nodes
    - filename: edges.csv
      filetype: Edges

export:
  profiles:
    - filename: output.dot
      exporter: DOT`;

// Interactive wrapper for Storybook
const EditorWrapper = ({ initialValue = '', ...props }: any) => {
  const [value, setValue] = useState(initialValue);
  
  return (
    <div className="w-[600px]">
      <CodeEditor
        value={value}
        onChange={setValue}
        {...props}
      />
    </div>
  );
};

// Basic usage
export const Default: Story = {
  render: () => (
    <EditorWrapper
      initialValue=""
      language="json"
      placeholder="Enter your JSON content..."
    />
  ),
};

export const JSONEditor: Story = {
  render: () => (
    <EditorWrapper
      initialValue={sampleJson}
      language="json"
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'JSON editor with sample layercake plan configuration.',
      },
    },
  },
};

export const YAMLEditor: Story = {
  render: () => (
    <EditorWrapper
      initialValue={sampleYaml}
      language="yaml"
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'YAML editor with sample layercake plan configuration.',
      },
    },
  },
};

export const EmptyEditor: Story = {
  render: () => (
    <EditorWrapper
      initialValue=""
      language="json"
      placeholder="Start typing your plan configuration..."
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Empty editor ready for new content input.',
      },
    },
  },
};

export const WithError: Story = {
  render: () => (
    <EditorWrapper
      initialValue='{"invalid": json syntax}'
      language="json"
      error="Invalid JSON syntax: Unexpected token 'j' at position 12"
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Editor showing validation error state.',
      },
    },
  },
};

export const DisabledEditor: Story = {
  render: () => (
    <EditorWrapper
      initialValue={sampleJson}
      language="json"
      disabled={true}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Disabled editor for read-only scenarios.',
      },
    },
  },
};

// Different content types
export const MinimalJSON: Story = {
  render: () => (
    <EditorWrapper
      initialValue='{"name": "Simple Project"}'
      language="json"
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Editor with minimal JSON content.',
      },
    },
  },
};

export const ComplexJSON: Story = {
  render: () => (
    <EditorWrapper
      initialValue={`{
  "meta": {
    "name": "Complex Microservices Architecture",
    "description": "A comprehensive example of a distributed system",
    "version": "2.0.0",
    "created": "2025-01-15T10:30:00Z",
    "tags": ["microservices", "docker", "kubernetes"]
  },
  "import": {
    "profiles": [
      {
        "filename": "services.csv",
        "filetype": "Nodes",
        "delimiter": ",",
        "headers": true
      },
      {
        "filename": "dependencies.csv",
        "filetype": "Edges",
        "delimiter": ",",
        "headers": true
      },
      {
        "filename": "layers.csv",
        "filetype": "Layers",
        "delimiter": ",",
        "headers": true
      }
    ]
  },
  "export": {
    "profiles": [
      {
        "filename": "architecture.dot",
        "exporter": "DOT",
        "graph_config": {
          "layout": "hierarchical",
          "node_shape": "box",
          "edge_style": "solid"
        }
      },
      {
        "filename": "deployment.yaml",
        "exporter": "Kubernetes",
        "render_config": {
          "namespace": "production",
          "replicas": 3
        }
      }
    ]
  },
  "analysis": {
    "enabled": true,
    "algorithms": ["centrality", "community_detection", "shortest_path"],
    "output_format": "json"
  }
}`}
      language="json"
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'Editor with complex JSON configuration showcasing advanced features.',
      },
    },
  },
};

// Feature demonstrations
export const LanguageComparison: Story = {
  render: () => (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-center">JSON vs YAML</h3>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div>
          <h4 className="font-medium mb-2">JSON Format</h4>
          <EditorWrapper
            initialValue={sampleJson}
            language="json"
          />
        </div>
        
        <div>
          <h4 className="font-medium mb-2">YAML Format</h4>
          <EditorWrapper
            initialValue={sampleYaml}
            language="yaml"
          />
        </div>
      </div>
      
      <div className="p-4 bg-blue-50 rounded-lg">
        <h4 className="font-medium text-blue-900 mb-2">Format Guidelines</h4>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• <strong>JSON</strong>: Strict syntax, better for APIs and data exchange</li>
          <li>• <strong>YAML</strong>: Human-readable, better for configuration files</li>
          <li>• Both formats support the same layercake plan structure</li>
          <li>• Use the format button to auto-format JSON content</li>
        </ul>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Side-by-side comparison of JSON and YAML editors.',
      },
    },
  },
};

export const EditorFeatures: Story = {
  render: () => (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-center">Editor Features</h3>
      
      <EditorWrapper
        initialValue={`{
  "example": "content",
  "features": {
    "line_numbers": true,
    "tab_indentation": true,
    "format_button": true,
    "syntax_highlighting": true
  }
}`}
        language="json"
      />
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="p-4 border rounded-lg">
          <h4 className="font-medium mb-2">🎯 Key Features</h4>
          <ul className="text-sm space-y-1">
            <li>• Line numbers for easy navigation</li>
            <li>• Tab key inserts 2 spaces</li>
            <li>• Format button for JSON prettification</li>
            <li>• Monospace font for code readability</li>
          </ul>
        </div>
        
        <div className="p-4 border rounded-lg">
          <h4 className="font-medium mb-2">⌨️ Keyboard Shortcuts</h4>
          <ul className="text-sm space-y-1">
            <li>• <kbd>Tab</kbd> - Insert indentation</li>
            <li>• <kbd>Ctrl/Cmd + A</kbd> - Select all</li>
            <li>• <kbd>Ctrl/Cmd + Z</kbd> - Undo</li>
            <li>• <kbd>Ctrl/Cmd + Y</kbd> - Redo</li>
          </ul>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Demonstration of editor features and keyboard shortcuts.',
      },
    },
  },
};

// Error scenarios
export const ErrorStates: Story = {
  render: () => (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-center">Error Handling</h3>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div>
          <h4 className="font-medium mb-2 text-red-600">JSON Syntax Error</h4>
          <EditorWrapper
            initialValue='{"name": "test", "invalid": }'
            language="json"
            error="Unexpected token '}' at position 25"
          />
        </div>
        
        <div>
          <h4 className="font-medium mb-2 text-red-600">Validation Error</h4>
          <EditorWrapper
            initialValue='{"meta": {}}'
            language="json"
            error="Required field 'name' is missing from meta object"
          />
        </div>
      </div>
      
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <h4 className="font-medium text-red-800 mb-2">Error Handling</h4>
        <p className="text-sm text-red-700">
          The editor provides real-time validation feedback and clear error messages 
          to help users identify and fix issues in their configuration files.
        </p>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Examples of error states and validation feedback.',
      },
    },
  },
};