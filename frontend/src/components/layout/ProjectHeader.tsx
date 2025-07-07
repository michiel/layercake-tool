import React from 'react';
import { Link, useParams, useLocation } from 'react-router-dom';
import { useQuery } from '@apollo/client';
import { Home, ChevronRight, LayersIcon, Settings, Bell, User } from 'lucide-react';
import { GET_PROJECT } from '../../graphql/dag';
import { Loading } from '../ui/Loading';

interface ProjectHeaderProps {
  projectId: string;
}

export const ProjectHeader: React.FC<ProjectHeaderProps> = ({ projectId }) => {
  const { planId } = useParams();
  const location = useLocation();
  
  const { data: projectData, loading } = useQuery(GET_PROJECT, {
    variables: { id: parseInt(projectId, 10) },
    skip: !projectId,
  });

  const project = projectData?.project;

  const generateBreadcrumbs = () => {
    const breadcrumbs = [
      {
        label: 'Dashboard',
        href: '/',
        icon: Home,
      },
      {
        label: project?.name || 'Project',
        href: `/projects/${projectId}`,
        icon: null,
      },
    ];

    if (planId) {
      // Try to get plan name from current route data or use ID
      breadcrumbs.push({
        label: `Plan ${planId}`,
        href: `/projects/${projectId}/plans/${planId}`,
        icon: null,
      });
    }

    return breadcrumbs;
  };

  const breadcrumbs = generateBreadcrumbs();

  return (
    <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4 flex-shrink-0">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          {/* Logo/Brand */}
          <Link 
            to="/" 
            className="flex items-center text-primary-600 hover:text-primary-700 transition-colors mr-4"
          >
            <LayersIcon className="w-6 h-6" />
            <span className="ml-2 text-lg font-bold">Layercake</span>
          </Link>

          {/* Breadcrumb Navigation */}
          <nav className="flex items-center space-x-2 text-sm">
            {breadcrumbs.map((breadcrumb, index) => (
              <React.Fragment key={breadcrumb.href}>
                {index > 0 && (
                  <ChevronRight className="w-4 h-4 text-gray-400" />
                )}
                <Link
                  to={breadcrumb.href}
                  className={`
                    flex items-center px-2 py-1 rounded-md transition-colors
                    ${index === breadcrumbs.length - 1
                      ? 'text-gray-900 dark:text-white font-medium bg-gray-100 dark:bg-gray-700'
                      : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-gray-700'
                    }
                  `}
                >
                  {breadcrumb.icon && (
                    <breadcrumb.icon className="w-4 h-4 mr-1" />
                  )}
                  {loading && index === 1 ? (
                    <Loading size="sm" />
                  ) : (
                    breadcrumb.label
                  )}
                </Link>
              </React.Fragment>
            ))}
          </nav>
        </div>

        {/* Project Info and Actions */}
        <div className="flex items-center space-x-4">
          {project && (
            <div className="text-right mr-4">
              <div className="text-sm font-medium text-gray-900 dark:text-white">
                {project.name}
              </div>
              {project.description && (
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  {project.description}
                </div>
              )}
            </div>
          )}
          
          <div className="flex items-center space-x-2">
            <button className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700">
              <Bell className="w-5 h-5" />
            </button>
            <button className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700">
              <Settings className="w-5 h-5" />
            </button>
            <button className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700">
              <User className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>
    </header>
  );
};