import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { plansApi } from '@/lib/api';
import type { CreatePlanRequest, UpdatePlanRequest } from '@/types/api';

export function usePlans(projectId?: number) {
  return useQuery({
    queryKey: ['plans', projectId],
    queryFn: () => plansApi.getByProject(projectId!),
    enabled: !!projectId,
  });
}

export function usePlan(id: number) {
  return useQuery({
    queryKey: ['plans', id],
    queryFn: () => plansApi.getById(id),
    enabled: !!id,
  });
}

export function useCreatePlan() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ projectId, data }: { projectId: number; data: CreatePlanRequest }) => 
      plansApi.create(projectId, data),
    onSuccess: (_, { projectId }) => {
      queryClient.invalidateQueries({ queryKey: ['plans', projectId] });
    },
  });
}

export function useUpdatePlan() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdatePlanRequest }) => 
      plansApi.update(id, data),
    onSuccess: (updatedPlan) => {
      queryClient.invalidateQueries({ queryKey: ['plans', updatedPlan.project_id] });
      queryClient.setQueryData(['plans', updatedPlan.id], updatedPlan);
    },
  });
}

export function useDeletePlan() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: number) => plansApi.delete(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ['plans'] });
      queryClient.removeQueries({ queryKey: ['plans', id] });
    },
  });
}

export function useExecutePlan() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (planId: number) => plansApi.execute(planId),
    onSuccess: (_, planId) => {
      queryClient.invalidateQueries({ queryKey: ['plans', planId] });
    },
  });
}