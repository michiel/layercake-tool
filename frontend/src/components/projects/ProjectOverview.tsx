import React, { useState } from 'react';
import { useQuery } from '@apollo/client';
import { Link } from 'react-router-dom';
import { Plus, Play, FileText, Calendar, Clock, Users } from 'lucide-react';
import { GET_PROJECT } from '../../graphql/dag';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';
import { PlanSelector } from './PlanSelector';

interface ProjectOverviewProps {
  projectId: number;
}

export const ProjectOverview: React.FC<ProjectOverviewProps> = ({ projectId }) => {
  const [showCreatePlan, setShowCreatePlan] = useState(false);

  const { data, loading, error } = useQuery(GET_PROJECT, {
    variables: { id: projectId },
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loading size="lg" />
      </div>
    );
  }

  if (error || !data?.project) {
    return (
      <ErrorMessage
        title="Project not found"
        message={error?.message || 'The requested project could not be found'}
      />
    );
  }

  const project = data.project;

  return (
    <div className="h-full flex flex-col p-6 space-y-6">
      {/* Project Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            {project.name}
          </h1>
          {project.description && (
            <p className="text-lg text-gray-600 dark:text-gray-300 mt-2">
              {project.description}
            </p>
          )}
        </div>
        <div className="flex items-center space-x-3">
          <Button
            variant="outline"
            onClick={() => setShowCreatePlan(true)}
            className="flex items-center space-x-2"
          >
            <Plus className="w-4 h-4" />
            <span>New Plan</span>
          </Button>
          <Button variant="outline">
            <Users className="w-4 h-4 mr-2" />
            Share
          </Button>
        </div>
      </div>

      {/* Project Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
              <FileText className="w-5 h-5 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Total Plans</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">0</p>
            </div>
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-green-100 dark:bg-green-900 rounded-lg">
              <Play className="w-5 h-5 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Executions</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">0</p>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-yellow-100 dark:bg-yellow-900 rounded-lg">
              <Clock className="w-5 h-5 text-yellow-600 dark:text-yellow-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Last Activity</p>
              <p className="text-sm font-medium text-gray-900 dark:text-white">
                {new Date(project.updated_at).toLocaleDateString()}
              </p>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-purple-100 dark:bg-purple-900 rounded-lg">
              <Calendar className="w-5 h-5 text-purple-600 dark:text-purple-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Created</p>
              <p className="text-sm font-medium text-gray-900 dark:text-white">
                {new Date(project.created_at).toLocaleDateString()}
              </p>
            </div>
          </div>
        </Card>
      </div>

      {/* Plans Section */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Plans
          </h2>
        </div>
        
        <div className="flex-1 min-h-0">
          <PlanSelector projectId={projectId} />
        </div>
      </div>

      {/* Create Plan Modal - TODO: Implement modal */}
      {showCreatePlan && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold mb-4">Create New Plan</h3>
            <p className="text-gray-600 dark:text-gray-300 mb-4">
              Plan creation interface will be implemented here.
            </p>
            <div className="flex justify-end space-x-3">
              <Button variant="outline" onClick={() => setShowCreatePlan(false)}>
                Cancel
              </Button>
              <Button onClick={() => setShowCreatePlan(false)}>
                Create
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};