import { useQuery } from '@tanstack/react-query';
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
  const nodesQuery = useNodes(projectId);
  const edgesQuery = useEdges(projectId);
  const layersQuery = useLayers(projectId);

  return {
    nodes: nodesQuery.data || [],
    edges: edgesQuery.data || [],
    layers: layersQuery.data || [],
    isLoading: nodesQuery.isLoading || edgesQuery.isLoading || layersQuery.isLoading,
    error: nodesQuery.error || edgesQuery.error || layersQuery.error,
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