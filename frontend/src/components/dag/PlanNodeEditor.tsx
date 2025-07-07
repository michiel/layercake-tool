import React, { useState, useEffect } from 'react';
import type { PlanNode } from '../../types/dag';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Textarea } from '../ui/Textarea';
import { CodeEditor } from '../ui/CodeEditor';

interface PlanNodeEditorProps {
  planNode: PlanNode;
  isOpen: boolean;
  onSave: (planNode: PlanNode) => void;
  onCancel: () => void;
  onDelete: () => void;
}

export const PlanNodeEditor: React.FC<PlanNodeEditorProps> = ({
  planNode,
  isOpen,
  onSave,
  onCancel,
  onDelete,
}) => {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    configuration: '{}',
  });

  const [configurationError, setConfigurationError] = useState<string | null>(null);

  useEffect(() => {
    if (planNode) {
      setFormData({
        name: planNode.name,
        description: planNode.description || '',
        configuration: planNode.configuration,
      });
    }
  }, [planNode]);

  const handleConfigurationChange = (value: string) => {
    setFormData(prev => ({ ...prev, configuration: value }));
    
    // Validate JSON
    try {
      JSON.parse(value);
      setConfigurationError(null);
    } catch (error) {
      setConfigurationError('Invalid JSON format');
    }
  };

  const handleSave = () => {
    if (configurationError) {
      return;
    }

    try {
      JSON.parse(formData.configuration);
    } catch (error) {
      setConfigurationError('Invalid JSON format');
      return;
    }

    const updatedNode: PlanNode = {
      ...planNode,
      name: formData.name,
      description: formData.description || null,
      configuration: formData.configuration,
      updated_at: new Date().toISOString(),
    };

    onSave(updatedNode);
  };

  const getNodeTypeInfo = (nodeType: string) => {
    const nodeTypes = {
      input: { label: 'Input Node', icon: '📥', color: 'text-green-600' },
      transform: { label: 'Transform Node', icon: '🔄', color: 'text-blue-600' },
      output: { label: 'Output Node', icon: '📤', color: 'text-red-600' },
      merge: { label: 'Merge Node', icon: '🔗', color: 'text-yellow-600' },
      split: { label: 'Split Node', icon: '🔀', color: 'text-purple-600' },
    };
    
    return nodeTypes[nodeType as keyof typeof nodeTypes] || { 
      label: nodeType, 
      icon: '📦', 
      color: 'text-gray-600' 
    };
  };

  const nodeTypeInfo = getNodeTypeInfo(planNode?.node_type || '');

  return (
    <Modal
      isOpen={isOpen}
      onClose={onCancel}
      title={
        <div className="flex items-center space-x-2">
          <span className="text-2xl">{nodeTypeInfo.icon}</span>
          <span>Edit {nodeTypeInfo.label}</span>
        </div>
      }
      size="lg"
    >
      <div className="space-y-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Node Name
          </label>
          <Input
            value={formData.name}
            onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
            placeholder="Enter node name"
            className="w-full"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Description
          </label>
          <Textarea
            value={formData.description}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            placeholder="Enter node description (optional)"
            rows={3}
            className="w-full"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Configuration (JSON)
          </label>
          <CodeEditor
            value={formData.configuration}
            onChange={handleConfigurationChange}
            language="json"
            height="200px"
            className="w-full"
          />
          {configurationError && (
            <div className="mt-2 text-sm text-red-600">
              {configurationError}
            </div>
          )}
        </div>

        <div className="bg-gray-50 p-4 rounded-lg">
          <h4 className="text-sm font-medium text-gray-700 mb-2">Node Information</h4>
          <div className="space-y-1 text-sm text-gray-600">
            <div>ID: {planNode?.id}</div>
            <div>Type: {nodeTypeInfo.label}</div>
            <div>Created: {planNode?.created_at ? new Date(planNode.created_at).toLocaleString() : 'N/A'}</div>
            <div>Updated: {planNode?.updated_at ? new Date(planNode.updated_at).toLocaleString() : 'N/A'}</div>
          </div>
        </div>

        <div className="flex justify-between">
          <Button
            variant="outline"
            onClick={onDelete}
            className="text-red-600 hover:text-red-700 hover:bg-red-50"
          >
            Delete Node
          </Button>
          
          <div className="flex space-x-3">
            <Button variant="outline" onClick={onCancel}>
              Cancel
            </Button>
            <Button
              onClick={handleSave}
              disabled={!!configurationError}
              className="bg-blue-600 hover:bg-blue-700"
            >
              Save Changes
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
};