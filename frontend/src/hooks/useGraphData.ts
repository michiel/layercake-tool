import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { graphDataApi } from '@/lib/api';
import type { Node, Edge, Layer } from '@/types/api';

export function useNodes(projectId: number) {
  return useQuery({
    queryKey: ['nodes', projectId],
    queryFn: () => graphDataApi.getNodes(projectId),
    enabled: !!projectId,
  });
}

export function useEdges(projectId: number) {
  return useQuery({
    queryKey: ['edges', projectId],
    queryFn: () => graphDataApi.getEdges(projectId),
    enabled: !!projectId,
  });
}

export function useLayers(projectId: number) {
  return useQuery({
    queryKey: ['layers', projectId],
    queryFn: () => graphDataApi.getLayers(projectId),
    enabled: !!projectId,
  });
}

export function useGraphData(projectId: number) {
  const queryClient = useQueryClient();
  const nodesQuery = useNodes(projectId);
  const edgesQuery = useEdges(projectId);
  const layersQuery = useLayers(projectId);

  // Node mutations
  const createNodeMutation = useMutation({
    mutationFn: (data: Partial<Node>) => graphDataApi.createNode(projectId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes', projectId] });
    },
  });

  const updateNodeMutation = useMutation({
    mutationFn: ({ nodeId, data }: { nodeId: string; data: Partial<Node> }) =>
      graphDataApi.updateNode(projectId, nodeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes', projectId] });
    },
  });

  const deleteNodeMutation = useMutation({
    mutationFn: (nodeId: string) => graphDataApi.deleteNode(projectId, nodeId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes', projectId] });
      queryClient.invalidateQueries({ queryKey: ['edges', projectId] });
    },
  });

  // Edge mutations
  const createEdgeMutation = useMutation({
    mutationFn: (data: Partial<Edge>) => graphDataApi.createEdge(projectId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['edges', projectId] });
    },
  });

  const updateEdgeMutation = useMutation({
    mutationFn: ({ edgeId, data }: { edgeId: string; data: Partial<Edge> }) =>
      graphDataApi.updateEdge(projectId, edgeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['edges', projectId] });
    },
  });

  const deleteEdgeMutation = useMutation({
    mutationFn: (edgeId: string) => graphDataApi.deleteEdge(projectId, edgeId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['edges', projectId] });
    },
  });

  // Layer mutations
  const createLayerMutation = useMutation({
    mutationFn: (data: Partial<Layer>) => graphDataApi.createLayer(projectId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['layers', projectId] });
    },
  });

  const updateLayerMutation = useMutation({
    mutationFn: ({ layerId, data }: { layerId: string; data: Partial<Layer> }) =>
      graphDataApi.updateLayer(projectId, layerId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['layers', projectId] });
    },
  });

  const deleteLayerMutation = useMutation({
    mutationFn: (layerId: string) => graphDataApi.deleteLayer(projectId, layerId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['layers', projectId] });
      queryClient.invalidateQueries({ queryKey: ['nodes', projectId] });
    },
  });

  return {
    // Data
    nodes: nodesQuery.data || [],
    edges: edgesQuery.data || [],
    layers: layersQuery.data || [],
    isLoading: nodesQuery.isLoading || edgesQuery.isLoading || layersQuery.isLoading,
    error: nodesQuery.error || edgesQuery.error || layersQuery.error,
    
    // Node operations
    createNode: createNodeMutation.mutateAsync,
    updateNode: (nodeId: string, data: Partial<Node>) => updateNodeMutation.mutateAsync({ nodeId, data }),
    deleteNode: deleteNodeMutation.mutateAsync,
    
    // Edge operations
    createEdge: createEdgeMutation.mutateAsync,
    updateEdge: (edgeId: string, data: Partial<Edge>) => updateEdgeMutation.mutateAsync({ edgeId, data }),
    deleteEdge: deleteEdgeMutation.mutateAsync,
    
    // Layer operations
    createLayer: createLayerMutation.mutateAsync,
    updateLayer: (layerId: string, data: Partial<Layer>) => updateLayerMutation.mutateAsync({ layerId, data }),
    deleteLayer: deleteLayerMutation.mutateAsync,

    // Mutation states
    isCreatingNode: createNodeMutation.isPending,
    isUpdatingNode: updateNodeMutation.isPending,
    isDeletingNode: deleteNodeMutation.isPending,
    isCreatingEdge: createEdgeMutation.isPending,
    isUpdatingEdge: updateEdgeMutation.isPending,
    isDeletingEdge: deleteEdgeMutation.isPending,
    isCreatingLayer: createLayerMutation.isPending,
    isUpdatingLayer: updateLayerMutation.isPending,
    isDeletingLayer: deleteLayerMutation.isPending,
    
    refetch: () => {
      nodesQuery.refetch();
      edgesQuery.refetch();
      layersQuery.refetch();
    },
  };
}

// Transform data for D3.js format
export function transformGraphDataForD3(nodes: Node[], edges: Edge[], layers: Layer[]) {
  // Create layer color map
  const layerColorMap = new Map<string, string>();
  layers.forEach(layer => {
    layerColorMap.set(layer.layer_id, layer.color || '#6366f1');
  });

  // Transform nodes for D3
  const d3Nodes = nodes.map(node => ({
    id: node.node_id,
    label: node.label,
    layerId: node.layer_id,
    color: node.layer_id ? layerColorMap.get(node.layer_id) : '#6366f1',
    properties: node.properties || {},
    // D3 will add x, y, vx, vy during simulation
  }));

  // Transform edges for D3
  const d3Edges = edges.map(edge => ({
    source: edge.source_node_id,
    target: edge.target_node_id,
    properties: edge.properties || {},
  }));

  return {
    nodes: d3Nodes,
    edges: d3Edges,
    layers: layers.map(layer => ({
      id: layer.layer_id,
      name: layer.name,
      color: layer.color || '#6366f1',
      properties: layer.properties || {},
    })),
  };
}