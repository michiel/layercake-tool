import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Plus, Search, Edit, Trash2, Play, FileText, ArrowLeft, Eye } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Modal } from '@/components/ui/Modal';
import { PlanForm } from '@/components/plans/PlanForm';
import { usePlans, useCreatePlan, useUpdatePlan, useDeletePlan, useExecutePlan } from '@/hooks/usePlans';
import { useProject } from '@/hooks/useProjects';
import type { Plan, CreatePlanRequest, UpdatePlanRequest } from '@/types/api';

export function Plans() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [editingPlan, setEditingPlan] = useState<Plan | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [deletingPlanId, setDeletingPlanId] = useState<number | null>(null);
  const [executingPlanId, setExecutingPlanId] = useState<number | null>(null);

  const projectIdNum = projectId ? parseInt(projectId, 10) : undefined;
  const { data: project } = useProject(projectIdNum!);
  const { data: plans, isLoading, error } = usePlans(projectIdNum);
  const createPlan = useCreatePlan();
  const updatePlan = useUpdatePlan();
  const deletePlan = useDeletePlan();
  const executePlan = useExecutePlan();

  const filteredPlans = plans?.filter(plan =>
    plan.name.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  const handleCreatePlan = async (data: CreatePlanRequest) => {
    if (!projectIdNum) return;
    
    try {
      await createPlan.mutateAsync({ projectId: projectIdNum, data });
      setIsCreateModalOpen(false);
    } catch (error) {
      console.error('Failed to create plan:', error);
    }
  };

  const handleUpdatePlan = async (data: UpdatePlanRequest) => {
    if (!editingPlan) return;
    
    try {
      await updatePlan.mutateAsync({
        id: editingPlan.id,
        data,
      });
      setEditingPlan(null);
    } catch (error) {
      console.error('Failed to update plan:', error);
    }
  };

  const handleDeletePlan = async (id: number) => {
    try {
      await deletePlan.mutateAsync(id);
      setDeletingPlanId(null);
    } catch (error) {
      console.error('Failed to delete plan:', error);
    }
  };

  const handleExecutePlan = async (id: number) => {
    try {
      await executePlan.mutateAsync(id);
      setExecutingPlanId(null);
    } catch (error) {
      console.error('Failed to execute plan:', error);
    }
  };

  const handleViewPlan = (planId: number) => {
    navigate(`/projects/${projectId}/plans/${planId}`);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
      case 'running':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300';
      case 'failed':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
    }
  };

  if (!projectIdNum) {
    return (
      <div className="text-center py-12">
        <div className="text-red-600 mb-2">Invalid project ID</div>
        <Button onClick={() => window.history.back()}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Go Back
        </Button>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <div className="text-red-600 mb-2">Failed to load plans</div>
        <div className="text-sm text-gray-500">Please try refreshing the page</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => window.history.back()}
            >
              <ArrowLeft className="w-4 h-4" />
            </Button>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
              Plans
            </h1>
          </div>
          <p className="text-gray-600 dark:text-gray-400">
            {project ? `Managing plans for "${project.name}"` : 'Loading project...'}
          </p>
        </div>
        <Button
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center gap-2"
        >
          <Plus className="w-4 h-4" />
          New Plan
        </Button>
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
        <Input
          placeholder="Search plans..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="pl-10"
        />
      </div>

      {/* Plans Grid */}
      {filteredPlans.length === 0 ? (
        <div className="text-center py-12">
          <FileText className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <div className="text-gray-500 mb-2">
            {searchTerm ? 'No plans found matching your search' : 'No plans yet'}
          </div>
          {!searchTerm && (
            <Button
              onClick={() => setIsCreateModalOpen(true)}
              variant="secondary"
            >
              Create your first plan
            </Button>
          )}
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredPlans.map((plan) => (
            <div
              key={plan.id}
              className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 hover:shadow-md transition-shadow cursor-pointer"
              onClick={() => handleViewPlan(plan.id)}
            >
              <div className="flex justify-between items-start mb-3">
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white truncate">
                    {plan.name}
                  </h3>
                  <div className="flex items-center gap-2 mt-1">
                    <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(plan.status)}`}>
                      {plan.status}
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {plan.plan_format.toUpperCase()}
                    </span>
                  </div>
                </div>
                <div className="flex gap-1 ml-2" onClick={(e) => e.stopPropagation()}>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleViewPlan(plan.id)}
                    className="p-1 h-8 w-8 text-blue-600 hover:text-blue-700"
                    title="View plan"
                  >
                    <Eye className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setExecutingPlanId(plan.id)}
                    disabled={plan.status === 'running'}
                    className="p-1 h-8 w-8 text-green-600 hover:text-green-700"
                    title="Execute plan"
                  >
                    <Play className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setEditingPlan(plan)}
                    className="p-1 h-8 w-8"
                    title="Edit plan"
                  >
                    <Edit className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setDeletingPlanId(plan.id)}
                    className="p-1 h-8 w-8 text-red-600 hover:text-red-700"
                    title="Delete plan"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
              
              <div className="text-xs text-gray-500 dark:text-gray-400 space-y-1">
                <div>Created {formatDate(plan.created_at)}</div>
                <div>Updated {formatDate(plan.updated_at)}</div>
                <div>Version {plan.plan_schema_version}</div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create Plan Modal */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="Create New Plan"
        size="xl"
      >
        <PlanForm
          onSubmit={(data) => handleCreatePlan(data as CreatePlanRequest)}
          onCancel={() => setIsCreateModalOpen(false)}
          isLoading={createPlan.isPending}
        />
      </Modal>

      {/* Edit Plan Modal */}
      <Modal
        isOpen={!!editingPlan}
        onClose={() => setEditingPlan(null)}
        title="Edit Plan"
        size="xl"
      >
        {editingPlan && (
          <PlanForm
            plan={editingPlan}
            onSubmit={(data) => handleUpdatePlan(data as UpdatePlanRequest)}
            onCancel={() => setEditingPlan(null)}
            isLoading={updatePlan.isPending}
          />
        )}
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={!!deletingPlanId}
        onClose={() => setDeletingPlanId(null)}
        title="Delete Plan"
        size="sm"
      >
        <div className="space-y-4">
          <p className="text-gray-600 dark:text-gray-400">
            Are you sure you want to delete this plan? This action cannot be undone.
          </p>
          <div className="flex justify-end space-x-3">
            <Button
              variant="secondary"
              onClick={() => setDeletingPlanId(null)}
              disabled={deletePlan.isPending}
            >
              Cancel
            </Button>
            <Button
              variant="danger"
              onClick={() => deletingPlanId && handleDeletePlan(deletingPlanId)}
              loading={deletePlan.isPending}
            >
              Delete Plan
            </Button>
          </div>
        </div>
      </Modal>

      {/* Execute Confirmation Modal */}
      <Modal
        isOpen={!!executingPlanId}
        onClose={() => setExecutingPlanId(null)}
        title="Execute Plan"
        size="sm"
      >
        <div className="space-y-4">
          <p className="text-gray-600 dark:text-gray-400">
            Are you sure you want to execute this plan? This will run the plan's transformation steps.
          </p>
          <div className="flex justify-end space-x-3">
            <Button
              variant="secondary"
              onClick={() => setExecutingPlanId(null)}
              disabled={executePlan.isPending}
            >
              Cancel
            </Button>
            <Button
              onClick={() => executingPlanId && handleExecutePlan(executingPlanId)}
              loading={executePlan.isPending}
            >
              Execute Plan
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}