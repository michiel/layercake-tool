import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, Settings, Play, Save } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Loading } from '@/components/ui/Loading';
import { ErrorMessage } from '@/components/ui/ErrorMessage';
import { Modal } from '@/components/ui/Modal';
import { PlanForm } from './PlanForm';
import { DagEditor } from '../dag/DagEditor';
import { DagValidation } from '../dag/DagValidation';
import { PlanNodeGraphInspector } from '../graph/PlanNodeGraphInspector';
import { GraphDataGridREST } from '../graph/GraphDataGridREST';
import { usePlan } from '@/hooks/usePlans';
import { usePlanDag } from '@/hooks/usePlanDag';
import type { UpdatePlanRequest } from '@/types/api';
import type { DagPlan } from '@/types/dag';

interface PlanPageParams {
  projectId: string;
  planId: string;
}

type EditMode = 'dag' | 'grid' | 'json' | 'yaml';

export const PlanPageREST: React.FC = () => {
  const { projectId, planId } = useParams<PlanPageParams>();
  const navigate = useNavigate();
  
  const [editMode, setEditMode] = useState<EditMode>('dag');
  const [showPlanForm, setShowPlanForm] = useState(false);
  
  const planIdNum = parseInt(planId || '0', 10);
  const projectIdNum = parseInt(projectId || '0', 10);

  // Fetch plan data using REST API
  const { data: plan, isLoading, error } = usePlan(planIdNum);
  
  // Fetch DAG data for visual editor
  const { 
    dagData, 
    isLoading: isDagLoading, 
    error: dagError,
    createNode,
    updateNode,
    deleteNode,
    addEdge,
    removeEdge,
    updateDag
  } = usePlanDag(planIdNum, projectIdNum);

  const handleGoBack = () => {
    navigate(`/projects/${projectId}/plans`);
  };

  const handlePlanFormSubmit = async (planData: UpdatePlanRequest) => {
    // TODO: Implement plan metadata update
    console.log('Plan form submit:', planData);
    setShowPlanForm(false);
  };

  const renderEditModeSelector = () => (
    <div className="flex items-center space-x-2 mb-4">
      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Edit Mode:</span>
      <div className="flex rounded-md shadow-sm">
        {(['dag', 'grid', 'json', 'yaml'] as EditMode[]).map((mode) => (
          <button
            key={mode}
            onClick={() => setEditMode(mode)}
            className={`
              px-3 py-2 text-sm font-medium border
              ${mode === 'dag' ? 'rounded-l-md' : mode === 'yaml' ? 'rounded-r-md' : '-ml-px'}
              ${
                editMode === mode
                  ? 'bg-blue-600 text-white border-blue-600 z-10'
                  : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600'
              }
            `}
          >
            {mode === 'dag' ? 'Visual DAG' : mode === 'grid' ? 'Data Grid' : mode.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );

  const renderPlanMetadata = () => (
    <Card className="p-4 mb-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleGoBack}
            className="flex items-center gap-2"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Plans
          </Button>
          <div>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{plan?.name}</h2>
            {plan?.description && (
              <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{plan.description}</p>
            )}
            <div className="flex items-center gap-4 mt-2 text-xs text-gray-500 dark:text-gray-400">
              <span>Format: {plan?.plan_format?.toUpperCase()}</span>
              <span>Status: {plan?.status}</span>
              <span>Version: {plan?.plan_schema_version}</span>
            </div>
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowPlanForm(true)}
            className="flex items-center gap-2"
          >
            <Settings className="w-4 h-4" />
            Settings
          </Button>
          <Button
            size="sm"
            disabled={plan?.status === 'running'}
            className="flex items-center gap-2"
          >
            <Play className="w-4 h-4" />
            Execute
          </Button>
        </div>
      </div>
    </Card>
  );

  const renderDagEditor = () => {
    if (isDagLoading) {
      return (
        <div className="flex items-center justify-center h-96">
          <Loading size="lg" />
        </div>
      );
    }

    if (dagError) {
      return (
        <Card className="p-8 text-center">
          <div className="text-center">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">DAG Editor Error</h3>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Failed to load DAG data: {dagError.message}
            </p>
            <div className="mt-4 flex justify-center">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setEditMode('json')}
              >
                Switch to JSON Editor
              </Button>
            </div>
          </div>
        </Card>
      );
    }

    // Convert DAG data to DagPlan format for the editor
    const dagPlan: DagPlan = {
      nodes: dagData?.nodes.map(node => node.data.planNode) || [],
      edges: dagData?.edges || [],
    };

    const handleDagChange = async (updatedDag: DagPlan) => {
      try {
        // Convert DagPlan back to DagData format
        const updatedDagData = {
          nodes: updatedDag.nodes.map(node => ({
            id: node.id,
            type: node.node_type,
            position: { 
              x: node.position_x || 0, 
              y: node.position_y || 0 
            },
            data: {
              label: node.name,
              description: node.description,
              configuration: node.configuration,
              planNode: node,
            },
          })),
          edges: updatedDag.edges.map(edge => ({
            id: `${edge.source}-${edge.target}`,
            source: edge.source,
            target: edge.target,
          })),
        };
        
        await updateDag(updatedDagData);
      } catch (error) {
        console.error('Failed to update DAG:', error);
      }
    };

    return (
      <div className="h-full flex flex-col space-y-4">
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-4 gap-4 min-h-0">
          <div className="lg:col-span-3">
            <Card className="h-full">
              <DagEditor
                planId={planIdNum}
                dagPlan={dagPlan}
                onDagChange={handleDagChange}
                readonly={false}
              />
            </Card>
          </div>
          <div className="lg:col-span-1">
            <DagValidation
              dagPlan={dagPlan}
              autoValidate={true}
            />
          </div>
        </div>
      </div>
    );
  };

  const renderGridEditor = () => {
    return (
      <div className="h-full flex flex-col space-y-4">
        <div className="flex-1 min-h-0">
          <GraphDataGridREST
            projectId={projectIdNum}
            editMode="transformation"
            onDataChange={(changes) => {
              console.log('Graph data changes:', changes);
            }}
          />
        </div>
      </div>
    );
  };

  const renderLegacyEditor = () => {
    if (!plan) {
      return (
        <ErrorMessage
          title="Plan not found"
          message="Plan data is required for editing mode"
        />
      );
    }

    return (
      <Card className="p-4 h-full overflow-auto">
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            {editMode.toUpperCase()} Editor
          </h3>
          <Button size="sm" className="flex items-center gap-2">
            <Save className="w-4 h-4" />
            Save Changes
          </Button>
        </div>
        <PlanForm
          plan={plan}
          onSubmit={handlePlanFormSubmit}
          onCancel={() => setEditMode('dag')}
        />
      </Card>
    );
  };

  const renderContent = () => {
    switch (editMode) {
      case 'dag':
        return renderDagEditor();
      case 'grid':
        return renderGridEditor();
      case 'json':
      case 'yaml':
        return renderLegacyEditor();
      default:
        return null;
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loading size="lg" />
      </div>
    );
  }

  if (error || !plan) {
    return (
      <div className="p-6">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleGoBack}
          className="flex items-center gap-2 mb-4"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Plans
        </Button>
        <ErrorMessage
          title="Plan not found"
          message={error?.message || 'The requested plan could not be found'}
        />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col p-6 space-y-4">
      {renderPlanMetadata()}
      {renderEditModeSelector()}
      <div className="flex-1 min-h-0">
        {renderContent()}
      </div>

      {/* Plan Form Modal */}
      <Modal
        isOpen={showPlanForm}
        onClose={() => setShowPlanForm(false)}
        title="Plan Settings"
        size="xl"
      >
        {plan && (
          <PlanForm
            plan={plan}
            onSubmit={handlePlanFormSubmit}
            onCancel={() => setShowPlanForm(false)}
          />
        )}
      </Modal>
    </div>
  );
};