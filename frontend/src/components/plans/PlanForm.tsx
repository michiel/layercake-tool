import { useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { CodeEditor } from '@/components/ui/CodeEditor';
import { ModalFooter } from '@/components/ui/Modal';
import type { Plan, CreatePlanRequest, UpdatePlanRequest } from '@/types/api';

interface PlanFormProps {
  plan?: Plan;
  onSubmit: (data: CreatePlanRequest | UpdatePlanRequest) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

// Default plan templates
const PLAN_TEMPLATES = {
  json: {
    name: 'Basic JSON Plan',
    content: JSON.stringify({
      version: '1.0',
      name: 'My Plan',
      description: 'A basic plan template',
      steps: [
        {
          id: 'step1',
          type: 'transformation',
          input: 'nodes.csv',
          output: 'processed_nodes.csv',
          operations: [
            {
              type: 'filter',
              condition: 'status == "active"'
            }
          ]
        }
      ],
      exports: [
        {
          type: 'csv',
          source: 'processed_nodes.csv',
          destination: 'output/filtered_nodes.csv'
        }
      ]
    }, null, 2)
  },
  yaml: {
    name: 'Basic YAML Plan',
    content: `version: '1.0'
name: My Plan
description: A basic plan template
steps:
  - id: step1
    type: transformation
    input: nodes.csv
    output: processed_nodes.csv
    operations:
      - type: filter
        condition: 'status == "active"'
exports:
  - type: csv
    source: processed_nodes.csv
    destination: output/filtered_nodes.csv`
  }
};

export function PlanForm({ 
  plan, 
  onSubmit, 
  onCancel, 
  isLoading = false 
}: PlanFormProps) {
  const [formData, setFormData] = useState({
    name: plan?.name || '',
    plan_content: plan?.plan_content || '',
    plan_format: (plan?.plan_format || 'json') as 'json' | 'yaml',
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Basic validation
    const newErrors: Record<string, string> = {};
    
    if (!formData.name.trim()) {
      newErrors.name = 'Plan name is required';
    }
    
    if (!formData.plan_content.trim()) {
      newErrors.plan_content = 'Plan content is required';
    } else {
      // Validate JSON/YAML format
      try {
        if (formData.plan_format === 'json') {
          JSON.parse(formData.plan_content);
        }
        // TODO: Add YAML validation when yaml parser is available
      } catch (error) {
        newErrors.plan_content = `Invalid ${formData.plan_format.toUpperCase()} format`;
      }
    }
    
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }
    
    setErrors({});
    
    if (plan) {
      // For updates, only send changed fields
      const updates: any = {};
      if (formData.name.trim() !== plan.name) {
        updates.name = formData.name.trim();
      }
      if (formData.plan_content.trim() !== plan.plan_content) {
        updates.plan_content = formData.plan_content.trim();
      }
      onSubmit(updates);
    } else {
      // For creates, send all fields
      onSubmit({
        name: formData.name.trim(),
        plan_content: formData.plan_content.trim(),
      });
    }
  };

  const handleChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }));
    }
  };

  const handleFormatChange = (format: 'json' | 'yaml') => {
    setFormData(prev => ({ ...prev, plan_format: format }));
  };

  const loadTemplate = () => {
    const template = PLAN_TEMPLATES[formData.plan_format];
    setFormData(prev => ({
      ...prev,
      name: prev.name || template.name,
      plan_content: template.content,
    }));
  };

  const isContentEmpty = !formData.plan_content.trim();

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <Input
        label="Plan Name"
        value={formData.name}
        onChange={(e) => handleChange('name', e.target.value)}
        error={errors.name}
        placeholder="Enter plan name"
        required
        disabled={isLoading}
      />
      
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Plan Format
        </label>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => handleFormatChange('json')}
            className={`px-3 py-1 text-xs rounded-md ${
              formData.plan_format === 'json'
                ? 'bg-primary-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300'
            }`}
            disabled={isLoading}
          >
            JSON
          </button>
          <button
            type="button"
            onClick={() => handleFormatChange('yaml')}
            className={`px-3 py-1 text-xs rounded-md ${
              formData.plan_format === 'yaml'
                ? 'bg-primary-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300'
            }`}
            disabled={isLoading}
          >
            YAML
          </button>
          {isContentEmpty && (
            <button
              type="button"
              onClick={loadTemplate}
              className="px-3 py-1 text-xs rounded-md bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900 dark:text-green-300"
              disabled={isLoading}
            >
              Load Template
            </button>
          )}
        </div>
      </div>
      
      <CodeEditor
        value={formData.plan_content}
        onChange={(value) => handleChange('plan_content', value)}
        language={formData.plan_format}
        error={errors.plan_content}
        disabled={isLoading}
        placeholder={`Enter your ${formData.plan_format.toUpperCase()} plan content...`}
      />
      
      <ModalFooter>
        <Button
          type="button"
          variant="secondary"
          onClick={onCancel}
          disabled={isLoading}
        >
          Cancel
        </Button>
        <Button
          type="submit"
          loading={isLoading}
          disabled={!formData.name.trim() || !formData.plan_content.trim()}
        >
          {plan ? 'Update Plan' : 'Create Plan'}
        </Button>
      </ModalFooter>
    </form>
  );
}