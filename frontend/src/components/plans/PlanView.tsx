import React, { useState, useEffect } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { Plan, DagPlan } from '../../types/dag';
import { GET_PLAN_DAG, UPDATE_PLAN_NODE } from '../../graphql/dag';
import { DagEditor } from '../dag/DagEditor';
import { DagValidation } from '../dag/DagValidation';
import { PlanForm } from './PlanForm';
import { Button } from '../ui/Button';
import { Card } from '../ui/Card';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';

interface PlanViewProps {
  planId: number;
  projectId: number;
  plan?: Plan;
  onPlanUpdate?: (plan: Plan) => void;
}

type EditMode = 'dag' | 'json' | 'yaml';

export const PlanView: React.FC<PlanViewProps> = ({
  planId,
  projectId,
  plan,
  onPlanUpdate,
}) => {
  const [editMode, setEditMode] = useState<EditMode>('dag');
  const [showPlanForm, setShowPlanForm] = useState(false);

  // Fetch DAG data for the plan
  const { data: dagData, loading: dagLoading, error: dagError, refetch: refetchDag } = useQuery(GET_PLAN_DAG, {
    variables: { planId },
    skip: editMode !== 'dag',
  });

  const [updatePlanNode] = useMutation(UPDATE_PLAN_NODE);

  const dagPlan: DagPlan | undefined = dagData?.plan_dag;

  const handleDagChange = async (updatedDag: DagPlan) => {
    try {
      // Update all modified nodes
      for (const node of updatedDag.nodes) {
        await updatePlanNode({
          variables: {
            id: node.id,
            input: {
              name: node.name,
              description: node.description,
              configuration: node.configuration,
              position_x: node.position_x,
              position_y: node.position_y,
            },
          },
        });
      }
      
      // Refetch to get latest data
      await refetchDag();
    } catch (error) {
      console.error('Failed to update DAG:', error);
    }
  };

  const handlePlanFormSubmit = async (planData: any) => {
    // TODO: Implement plan metadata update mutation
    console.log('Plan form submit:', planData);
    setShowPlanForm(false);
  };

  const renderEditModeSelector = () => (
    <div className="flex items-center space-x-2 mb-4">
      <span className="text-sm font-medium text-gray-700">Edit Mode:</span>
      <div className="flex rounded-md shadow-sm">
        {(['dag', 'json', 'yaml'] as EditMode[]).map((mode) => (
          <button
            key={mode}
            onClick={() => setEditMode(mode)}
            className={`
              px-3 py-2 text-sm font-medium border
              ${mode === 'dag' ? 'rounded-l-md' : mode === 'yaml' ? 'rounded-r-md' : '-ml-px'}
              ${
                editMode === mode
                  ? 'bg-blue-600 text-white border-blue-600 z-10'
                  : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
              }
            `}
          >
            {mode === 'dag' ? 'Visual DAG' : mode.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );

  const renderPlanMetadata = () => (
    <Card className="p-4 mb-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-gray-900">{plan?.name}</h2>
          {plan?.description && (
            <p className="text-sm text-gray-600 mt-1">{plan.description}</p>
          )}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowPlanForm(true)}
        >
          Edit Plan Details
        </Button>
      </div>
    </Card>
  );

  const renderDagEditor = () => {
    if (dagLoading) {
      return (
        <div className="flex items-center justify-center h-96">
          <Loading size="lg" />
        </div>
      );
    }

    if (dagError) {
      return (
        <ErrorMessage
          title="Failed to load DAG"
          message={dagError.message}
          onRetry={() => refetchDag()}
        />
      );
    }

    return (
      <div className="space-y-4">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
          <div className="lg:col-span-3">
            <Card className="h-96">
              <DagEditor
                planId={planId}
                dagPlan={dagPlan}
                onDagChange={handleDagChange}
                readonly={false}
              />
            </Card>
          </div>
          <div className="lg:col-span-1">
            {dagPlan && (
              <DagValidation
                dagPlan={dagPlan}
                autoValidate={true}
              />
            )}
          </div>
        </div>
      </div>
    );
  };

  const renderLegacyEditor = () => {
    if (!plan) {
      return (
        <ErrorMessage
          title="Plan not found"
          message="Plan data is required for legacy editing mode"
        />
      );
    }

    return (
      <Card className="p-4">
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
      case 'json':
      case 'yaml':
        return renderLegacyEditor();
      default:
        return null;
    }
  };

  return (
    <div className="space-y-4">
      {renderPlanMetadata()}
      {renderEditModeSelector()}
      {renderContent()}

      {/* Plan Form Modal */}
      {showPlanForm && plan && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
            <h3 className="text-lg font-semibold mb-4">Edit Plan Details</h3>
            <PlanForm
              plan={plan}
              onSubmit={handlePlanFormSubmit}
              onCancel={() => setShowPlanForm(false)}
            />
          </div>
        </div>
      )}
    </div>
  );
};