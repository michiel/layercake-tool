import { useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { ModalFooter } from '@/components/ui/Modal';
import type { Project, CreateProjectRequest, UpdateProjectRequest } from '@/types/api';

interface ProjectFormProps {
  project?: Project;
  onSubmit: (data: CreateProjectRequest | UpdateProjectRequest) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

export function ProjectForm({ 
  project, 
  onSubmit, 
  onCancel, 
  isLoading = false 
}: ProjectFormProps) {
  const [formData, setFormData] = useState({
    name: project?.name || '',
    description: project?.description || '',
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Basic validation
    const newErrors: Record<string, string> = {};
    
    if (!formData.name.trim()) {
      newErrors.name = 'Project name is required';
    }
    
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }
    
    setErrors({});
    
    if (project) {
      // For updates, only send changed fields
      const updates: any = {};
      if (formData.name.trim() !== project.name) {
        updates.name = formData.name.trim();
      }
      if (formData.description.trim() !== (project.description || '')) {
        updates.description = formData.description.trim() || undefined;
      }
      onSubmit(updates);
    } else {
      // For creates, send all fields
      onSubmit({
        name: formData.name.trim(),
        description: formData.description.trim() || undefined,
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

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <Input
        label="Project Name"
        value={formData.name}
        onChange={(e) => handleChange('name', e.target.value)}
        error={errors.name}
        placeholder="Enter project name"
        required
        disabled={isLoading}
      />
      
      <Textarea
        label="Description"
        value={formData.description}
        onChange={(e) => handleChange('description', e.target.value)}
        error={errors.description}
        placeholder="Enter project description (optional)"
        rows={4}
        disabled={isLoading}
        helperText="Provide a brief description of what this project is for"
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
          disabled={!formData.name.trim()}
        >
          {project ? 'Update Project' : 'Create Project'}
        </Button>
      </ModalFooter>
    </form>
  );
}