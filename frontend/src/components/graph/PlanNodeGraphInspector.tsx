import React, { useState } from 'react';
import { Database, Eye, Grid, BarChart3 } from 'lucide-react';
import { GraphDataGrid } from './GraphDataGrid';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';

interface PlanNodeGraphInspectorProps {
  projectId: number;
  planId: number;
  planNodeId: string;
  planNodeName: string;
  planNodeType: string;
  editMode?: 'transformation' | 'in-place' | 'read-only';
  syncWithVisualization?: boolean;
  onDataChange?: (changes: any) => void;
  onNodeSelect?: (nodeIds: string[]) => void;
  onEdgeSelect?: (edgeIds: string[]) => void;
}

type ViewMode = 'grid' | 'visualization' | 'stats';

export const PlanNodeGraphInspector: React.FC<PlanNodeGraphInspectorProps> = ({
  projectId,
  planId,
  planNodeId,
  planNodeName,
  planNodeType,
  editMode = 'transformation',
  syncWithVisualization = true,
  onDataChange,
  onNodeSelect,
  onEdgeSelect,
}) => {
  const [viewMode, setViewMode] = useState<ViewMode>('grid');

  const getNodeTypeIcon = (nodeType: string) => {
    switch (nodeType) {
      case 'input':
        return '📥';
      case 'transform':
        return '🔄';
      case 'output':
        return '📤';
      case 'merge':
        return '🔗';
      case 'split':
        return '🔀';
      default:
        return '📦';
    }
  };

  const getEditModeLabel = (mode: string) => {
    switch (mode) {
      case 'transformation':
        return 'Transformation Node';
      case 'in-place':
        return 'Direct Edit';
      case 'read-only':
        return 'Read Only';
      default:
        return mode;
    }
  };

  const renderViewModeSelector = () => (
    <div className="flex items-center space-x-1">
      {[
        { mode: 'grid' as ViewMode, icon: Grid, label: 'Data Grid' },
        { mode: 'visualization' as ViewMode, icon: Eye, label: 'Visualization' },
        { mode: 'stats' as ViewMode, icon: BarChart3, label: 'Statistics' },
      ].map(({ mode, icon: Icon, label }) => (
        <button
          key={mode}
          onClick={() => setViewMode(mode)}
          className={`
            flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors
            ${viewMode === mode
              ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300'
              : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'
            }
          `}
        >
          <Icon className="w-4 h-4 mr-2" />
          {label}
        </button>
      ))}
    </div>
  );

  const renderContent = () => {
    switch (viewMode) {
      case 'grid':
        return (
          <GraphDataGrid
            projectId={projectId}
            planId={planId}
            planNodeId={planNodeId}
            editMode={editMode}
            syncWithVisualization={syncWithVisualization}
            onDataChange={onDataChange}
            onNodeSelect={onNodeSelect}
            onEdgeSelect={onEdgeSelect}
          />
        );
      case 'visualization':
        return (
          <Card className="h-full flex items-center justify-center">
            <div className="text-center text-gray-500">
              <Eye className="w-12 h-12 mx-auto mb-4 text-gray-400" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                Graph Visualization
              </h3>
              <p className="text-gray-500 dark:text-gray-400">
                Graph visualization will be integrated here
              </p>
            </div>
          </Card>
        );
      case 'stats':
        return (
          <Card className="h-full flex items-center justify-center">
            <div className="text-center text-gray-500">
              <BarChart3 className="w-12 h-12 mx-auto mb-4 text-gray-400" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                Graph Statistics
              </h3>
              <p className="text-gray-500 dark:text-gray-400">
                Statistical analysis will be implemented here
              </p>
            </div>
          </Card>
        );
      default:
        return null;
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="flex items-center space-x-2">
              <span className="text-2xl">{getNodeTypeIcon(planNodeType)}</span>
              <div>
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                  {planNodeName}
                </h2>
                <div className="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
                  <Database className="w-4 h-4" />
                  <span>Graph at {planNodeType} node</span>
                  <span>•</span>
                  <span>{getEditModeLabel(editMode)}</span>
                </div>
              </div>
            </div>
          </div>
          
          {renderViewModeSelector()}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0">
        {renderContent()}
      </div>
    </div>
  );
};