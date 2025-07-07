import React from 'react';
import { useParams } from 'react-router-dom';
import { useQuery } from '@apollo/client';
import { GET_PLAN } from '../../graphql/dag';
import { PlanView } from './PlanView';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';

interface PlanPageParams {
  projectId: string;
  planId: string;
}

export const PlanPage: React.FC = () => {
  const { projectId, planId } = useParams<PlanPageParams>();
  
  const planIdNum = parseInt(planId || '0', 10);
  const projectIdNum = parseInt(projectId || '0', 10);

  // Fetch plan metadata
  const { data: planData, loading: planLoading, error: planError } = useQuery(GET_PLAN, {
    variables: { id: planIdNum },
    skip: !planIdNum,
  });

  if (planLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loading size="lg" />
      </div>
    );
  }

  if (planError || !planData?.plan) {
    return (
      <ErrorMessage
        title="Plan not found"
        message={planError?.message || 'The requested plan could not be found'}
      />
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <PlanView
        planId={planIdNum}
        projectId={projectIdNum}
        plan={planData.plan}
      />
    </div>
  );
};