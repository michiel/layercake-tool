import React from 'react';
import { Outlet } from 'react-router-dom';
import { ProjectHeader } from './ProjectHeader';

interface ProjectLayoutProps {
  projectId: string;
}

export const ProjectLayout: React.FC<ProjectLayoutProps> = ({ projectId }) => {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex flex-col">
      <ProjectHeader projectId={projectId} />
      <main className="flex-1 overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
};