import { useCallback, useRef, useEffect } from 'react';

export interface GraphSyncData {
  nodes: Array<{
    id: string;
    label: string;
    layer: string;
    x?: number;
    y?: number;
    weight?: number;
    properties?: Record<string, any>;
  }>;
  edges: Array<{
    id: string;
    source: string;
    target: string;
    label?: string;
    layer?: string;
    weight?: number;
    properties?: Record<string, any>;
  }>;
  layers: Array<{
    id: string;
    name: string;
    color: string;
    description?: string;
    visible?: boolean;
    order?: number;
  }>;
}

export interface GraphVisualizationRef {
  updateNodes: (nodes: GraphSyncData['nodes']) => void;
  updateEdges: (edges: GraphSyncData['edges']) => void;
  updateLayers: (layers: GraphSyncData['layers']) => void;
  highlightElements: (nodeIds: string[], edgeIds: string[]) => void;
  focusElement: (type: 'node' | 'edge', id: string) => void;
  refreshLayout: () => void;
}

export interface UseGraphSyncOptions {
  syncWithVisualization?: boolean;
  debounceMs?: number;
  onSyncError?: (error: Error) => void;
}

export interface UseGraphSyncReturn {
  registerVisualization: (ref: GraphVisualizationRef) => void;
  unregisterVisualization: () => void;
  syncToVisualization: (data: GraphSyncData) => void;
  highlightInVisualization: (nodeIds: string[], edgeIds?: string[]) => void;
  focusInVisualization: (type: 'node' | 'edge', id: string) => void;
  refreshVisualization: () => void;
  isVisualizationConnected: boolean;
}

export const useGraphSync = (options: UseGraphSyncOptions = {}): UseGraphSyncReturn => {
  const {
    syncWithVisualization = true,
    debounceMs = 300,
    onSyncError
  } = options;

  const visualizationRef = useRef<GraphVisualizationRef | null>(null);
  const syncTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const registerVisualization = useCallback((ref: GraphVisualizationRef) => {
    visualizationRef.current = ref;
  }, []);

  const unregisterVisualization = useCallback(() => {
    visualizationRef.current = null;
    if (syncTimeoutRef.current) {
      clearTimeout(syncTimeoutRef.current);
      syncTimeoutRef.current = null;
    }
  }, []);

  const syncToVisualization = useCallback((data: GraphSyncData) => {
    if (!syncWithVisualization || !visualizationRef.current) return;

    // Clear any pending sync
    if (syncTimeoutRef.current) {
      clearTimeout(syncTimeoutRef.current);
    }

    // Debounce the sync operation
    syncTimeoutRef.current = setTimeout(() => {
      try {
        if (visualizationRef.current) {
          visualizationRef.current.updateNodes(data.nodes);
          visualizationRef.current.updateEdges(data.edges);
          visualizationRef.current.updateLayers(data.layers);
          visualizationRef.current.refreshLayout();
        }
      } catch (error) {
        console.error('Graph sync error:', error);
        if (onSyncError) {
          onSyncError(error instanceof Error ? error : new Error(String(error)));
        }
      }
    }, debounceMs);
  }, [syncWithVisualization, debounceMs, onSyncError]);

  const highlightInVisualization = useCallback((nodeIds: string[], edgeIds: string[] = []) => {
    if (!visualizationRef.current) return;

    try {
      visualizationRef.current.highlightElements(nodeIds, edgeIds);
    } catch (error) {
      console.error('Graph highlight error:', error);
      if (onSyncError) {
        onSyncError(error instanceof Error ? error : new Error(String(error)));
      }
    }
  }, [onSyncError]);

  const focusInVisualization = useCallback((type: 'node' | 'edge', id: string) => {
    if (!visualizationRef.current) return;

    try {
      visualizationRef.current.focusElement(type, id);
    } catch (error) {
      console.error('Graph focus error:', error);
      if (onSyncError) {
        onSyncError(error instanceof Error ? error : new Error(String(error)));
      }
    }
  }, [onSyncError]);

  const refreshVisualization = useCallback(() => {
    if (!visualizationRef.current) return;

    try {
      visualizationRef.current.refreshLayout();
    } catch (error) {
      console.error('Graph refresh error:', error);
      if (onSyncError) {
        onSyncError(error instanceof Error ? error : new Error(String(error)));
      }
    }
  }, [onSyncError]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (syncTimeoutRef.current) {
        clearTimeout(syncTimeoutRef.current);
      }
    };
  }, []);

  return {
    registerVisualization,
    unregisterVisualization,
    syncToVisualization,
    highlightInVisualization,
    focusInVisualization,
    refreshVisualization,
    isVisualizationConnected: visualizationRef.current !== null,
  };
};

export default useGraphSync;