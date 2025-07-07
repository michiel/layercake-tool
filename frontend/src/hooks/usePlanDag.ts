import { useState, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import type { DagPlan, PlanNode } from '../types/dag';
import type { Plan } from '../types/api';

interface DagNode {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: {
    label: string;
    description?: string;
    configuration: string;
    planNode: PlanNode;
  };
}

interface DagEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  metadata?: Record<string, any>;
}

interface DagData {
  nodes: DagNode[];
  edges: DagEdge[];
}

interface CreateNodeInput {
  node_type: string;
  name: string;
  description?: string;
  configuration: string;
  position_x?: number;
  position_y?: number;
}

interface UpdateNodeInput {
  name?: string;
  description?: string;
  configuration?: string;
  position_x?: number;
  position_y?: number;
}

interface CreateEdgeInput {
  source: string;
  target: string;
  label?: string;
  metadata?: Record<string, any>;
}

export const usePlanDag = (planId: number, projectId?: number) => {
  const queryClient = useQueryClient();

  // Fetch DAG data
  const { data: dagData, isLoading, error, refetch } = useQuery({
    queryKey: ['plan-dag', planId],
    queryFn: async (): Promise<DagData> => {
      try {
        const response = await axios.get(`/api/v1/plans/${planId}/dag`);
        const dagPlan: DagPlan = response.data;
        
        // Convert DagPlan to ReactFlow format
        const nodes: DagNode[] = dagPlan.nodes?.map((node) => ({
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
        })) || [];

        const edges: DagEdge[] = dagPlan.edges?.map((edge) => ({
          id: edge.source + '-' + edge.target,
          source: edge.source,
          target: edge.target,
          label: edge.label,
          metadata: edge.metadata,
        })) || [];

        return { nodes, edges };
      } catch (error) {
        if (axios.isAxiosError(error) && error.response?.status === 400) {
          // Plan is not a DAG plan, return empty structure
          return { nodes: [], edges: [] };
        }
        throw error;
      }
    },
  });

  // Create node mutation
  const createNodeMutation = useMutation({
    mutationFn: async (input: CreateNodeInput): Promise<PlanNode> => {
      if (!projectId) throw new Error('Project ID is required for creating plan nodes');
      const response = await axios.post(`/api/v1/projects/${projectId}/plans/${planId}/plan-nodes`, input);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plan-dag', planId] });
    },
  });

  // Update node mutation
  const updateNodeMutation = useMutation({
    mutationFn: async ({ nodeId, input }: { nodeId: string; input: UpdateNodeInput }): Promise<PlanNode> => {
      if (!projectId) throw new Error('Project ID is required for updating plan nodes');
      const response = await axios.put(`/api/v1/projects/${projectId}/plans/${planId}/plan-nodes/${nodeId}`, input);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plan-dag', planId] });
    },
  });

  // Delete node mutation
  const deleteNodeMutation = useMutation({
    mutationFn: async (nodeId: string): Promise<void> => {
      if (!projectId) throw new Error('Project ID is required for deleting plan nodes');
      await axios.delete(`/api/v1/projects/${projectId}/plans/${planId}/plan-nodes/${nodeId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plan-dag', planId] });
    },
  });

  // Update DAG structure (for edges and bulk operations)
  const updateDagMutation = useMutation({
    mutationFn: async (updatedDag: DagData): Promise<void> => {
      // First get the current plan
      const planResponse = await axios.get(`/api/v1/plans/${planId}`);
      const plan: Plan = planResponse.data;
      
      // Convert ReactFlow format back to DagPlan
      const dagPlan: Partial<DagPlan> = {
        nodes: updatedDag.nodes.map((node) => ({
          id: node.id,
          plan_id: planId,
          node_type: node.type,
          name: node.data.label,
          description: node.data.description,
          configuration: node.data.configuration,
          graph_id: node.data.planNode.graph_id,
          position_x: node.position.x,
          position_y: node.position.y,
          created_at: node.data.planNode.created_at,
          updated_at: node.data.planNode.updated_at,
        })),
        edges: updatedDag.edges.map((edge) => ({
          source: edge.source,
          target: edge.target,
        })),
      };

      // Update the plan content with new DAG structure
      const updatedPlanContent = {
        ...JSON.parse(plan.plan_content || '{}'),
        nodes: dagPlan.nodes,
        edges: dagPlan.edges,
      };

      await axios.put(`/api/v1/plans/${planId}`, {
        plan_content: JSON.stringify(updatedPlanContent),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plan-dag', planId] });
      queryClient.invalidateQueries({ queryKey: ['plans'] });
    },
  });

  // Helper functions for common operations
  const createNode = useCallback((input: CreateNodeInput) => {
    return createNodeMutation.mutateAsync(input);
  }, [createNodeMutation]);

  const updateNode = useCallback((nodeId: string, input: UpdateNodeInput) => {
    return updateNodeMutation.mutateAsync({ nodeId, input });
  }, [updateNodeMutation]);

  const deleteNode = useCallback((nodeId: string) => {
    return deleteNodeMutation.mutateAsync(nodeId);
  }, [deleteNodeMutation]);

  const updateDag = useCallback((updatedDag: DagData) => {
    return updateDagMutation.mutateAsync(updatedDag);
  }, [updateDagMutation]);

  const addEdge = useCallback((edgeInput: CreateEdgeInput) => {
    if (!dagData) return Promise.reject(new Error('No DAG data available'));
    
    const newEdge: DagEdge = {
      id: `${edgeInput.source}-${edgeInput.target}`,
      source: edgeInput.source,
      target: edgeInput.target,
      label: edgeInput.label,
      metadata: edgeInput.metadata,
    };

    const updatedDag: DagData = {
      nodes: dagData.nodes,
      edges: [...dagData.edges, newEdge],
    };

    return updateDag(updatedDag);
  }, [dagData, updateDag]);

  const removeEdge = useCallback((edgeId: string) => {
    if (!dagData) return Promise.reject(new Error('No DAG data available'));
    
    const updatedDag: DagData = {
      nodes: dagData.nodes,
      edges: dagData.edges.filter(edge => edge.id !== edgeId),
    };

    return updateDag(updatedDag);
  }, [dagData, updateDag]);

  return {
    // Data
    dagData,
    isLoading,
    error,
    
    // Mutations
    createNode,
    updateNode,
    deleteNode,
    updateDag,
    addEdge,
    removeEdge,
    
    // Mutation states
    isCreatingNode: createNodeMutation.isPending,
    isUpdatingNode: updateNodeMutation.isPending,
    isDeletingNode: deleteNodeMutation.isPending,
    isUpdatingDag: updateDagMutation.isPending,
    
    // Utility
    refetch,
  };
};