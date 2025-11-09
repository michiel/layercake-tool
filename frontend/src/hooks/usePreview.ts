import { useQuery } from '@apollo/client/react';
import {
  GET_DATASOURCE_PREVIEW,
  GET_GRAPH_PREVIEW,
  GetDataSetPreviewResponse,
  GetDataSetPreviewVariables,
  GetGraphPreviewResponse,
  GetGraphPreviewVariables,
  DataSetPreview,
  GraphPreview,
} from '../graphql/preview';

/**
 * Hook to fetch DataSet preview with table data
 * @param projectId - The project ID
 * @param nodeId - The DAG node ID
 * @param options - Optional query options (limit, offset, skip)
 */
export function useDataSetPreview(
  projectId: number,
  nodeId: string,
  options?: {
    limit?: number;
    offset?: number;
    skip?: boolean;
  }
) {
  const { limit = 100, offset = 0, skip = false } = options || {};

  const { data, loading, error, refetch } = useQuery<
    GetDataSetPreviewResponse,
    GetDataSetPreviewVariables
  >(GET_DATASOURCE_PREVIEW, {
    variables: {
      projectId,
      nodeId,
      limit,
      offset,
    },
    skip,
    fetchPolicy: 'cache-and-network',
  });

  return {
    preview: data?.datasetPreview,
    loading,
    error,
    refetch,
  };
}

/**
 * Hook to fetch Graph preview with nodes and edges
 * @param projectId - The project ID
 * @param nodeId - The DAG node ID
 * @param options - Optional query options (skip)
 */
export function useGraphPreview(
  projectId: number,
  nodeId: string,
  options?: {
    skip?: boolean;
  }
) {
  const { skip = false } = options || {};

  const { data, loading, error, refetch } = useQuery<
    GetGraphPreviewResponse,
    GetGraphPreviewVariables
  >(GET_GRAPH_PREVIEW, {
    variables: {
      projectId,
      nodeId,
    },
    skip,
    fetchPolicy: 'cache-and-network',
  });

  return {
    preview: data?.graphPreview,
    loading,
    error,
    refetch,
  };
}

/**
 * Type guard to check if preview data exists
 */
export function hasPreviewData(
  preview: DataSetPreview | GraphPreview | null | undefined
): preview is DataSetPreview | GraphPreview {
  return preview !== null && preview !== undefined;
}

/**
 * Type guard to check if preview is DataSet
 */
export function isDataSetPreview(
  preview: DataSetPreview | GraphPreview | null | undefined
): preview is DataSetPreview {
  return (
    preview !== null &&
    preview !== undefined &&
    'datasetId' in preview
  );
}

/**
 * Type guard to check if preview is Graph
 */
export function isGraphPreview(
  preview: DataSetPreview | GraphPreview | null | undefined
): preview is GraphPreview {
  return (
    preview !== null &&
    preview !== undefined &&
    'graphId' in preview
  );
}
