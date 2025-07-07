import React from 'react';
import { useParams, Outlet } from 'react-router-dom';
import { ProjectLayout } from '../layout/ProjectLayout';
import { ProjectOverview } from './ProjectOverview';

export const ProjectPage: React.FC = () => {
  const { projectId, planId } = useParams<{ projectId: string; planId?: string }>();
  
  if (!projectId) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-gray-900">Invalid project</h2>
          <p className="text-gray-600">Project ID is required</p>
        </div>
      </div>
    );
  }

  const projectIdNum = parseInt(projectId, 10);

  return (
    <ProjectLayout projectId={projectId}>
      {planId ? (
        // Show plan-specific content (like plan editor)
        <Outlet />
      ) : (
        // Show project overview when no plan is selected
        <ProjectOverview projectId={projectIdNum} />
      )}
    </ProjectLayout>
  );
};