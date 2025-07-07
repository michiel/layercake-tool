import React from 'react';
import { useQuery } from '@apollo/client';
import { Link } from 'react-router-dom';
import { FileText, Play, Clock, MoreVertical, Eye } from 'lucide-react';
import { GET_PLANS_FOR_PROJECT } from '../../graphql/dag';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';
import { DagPreview } from './DagPreview';

interface PlanSelectorProps {
  projectId: number;
}

export const PlanSelector: React.FC<PlanSelectorProps> = ({ projectId }) => {
  const { data, loading, error } = useQuery(GET_PLANS_FOR_PROJECT, {
    variables: { projectId },
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loading size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        title="Failed to load plans"
        message={error.message}
      />
    );
  }

  const plans = data?.plans || [];

  if (plans.length === 0) {
    return (
      <Card className="p-8 text-center">
        <FileText className="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
          No plans yet
        </h3>
        <p className="text-gray-500 dark:text-gray-400 mb-6">
          Create your first plan to start building graph transformation workflows.
        </p>
        <Button className="mx-auto">
          Create Plan
        </Button>
      </Card>
    );
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6 overflow-auto">
      {plans.map((plan: any) => (
        <PlanCard key={plan.id} plan={plan} projectId={projectId} />
      ))}
    </div>
  );
};

interface PlanCardProps {
  plan: any;
  projectId: number;
}

const PlanCard: React.FC<PlanCardProps> = ({ plan, projectId }) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'text-green-600 bg-green-100 dark:text-green-400 dark:bg-green-900';
      case 'running':
        return 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900';
      case 'failed':
        return 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900';
      default:
        return 'text-gray-600 bg-gray-100 dark:text-gray-400 dark:bg-gray-700';
    }
  };

  return (
    <Card className="h-80 flex flex-col">
      {/* Plan Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white truncate">
              {plan.name}
            </h3>
            {plan.description && (
              <p className="text-sm text-gray-500 dark:text-gray-400 truncate">
                {plan.description}
              </p>
            )}
          </div>
          <button className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
            <MoreVertical className="w-4 h-4" />
          </button>
        </div>

        <div className="flex items-center justify-between mt-3">
          <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(plan.status)}`}>
            {plan.status || 'draft'}
          </span>
          <div className="flex items-center text-xs text-gray-500 dark:text-gray-400">
            <Clock className="w-3 h-3 mr-1" />
            {new Date(plan.updated_at).toLocaleDateString()}
          </div>
        </div>
      </div>

      {/* DAG Preview */}
      <div className="flex-1 min-h-0 p-4">
        <DagPreview planId={plan.id} />
      </div>

      {/* Plan Actions */}
      <div className="p-4 border-t border-gray-200 dark:border-gray-700">
        <div className="flex items-center space-x-3">
          <Link
            to={`/projects/${projectId}/plans/${plan.id}`}
            className="flex-1"
          >
            <Button variant="outline" className="w-full">
              <Eye className="w-4 h-4 mr-2" />
              View Plan
            </Button>
          </Link>
          <Button size="sm" className="px-3">
            <Play className="w-4 h-4" />
          </Button>
        </div>
      </div>
    </Card>
  );
};