import { useQuery } from '@apollo/client/react';
import {
  GET_DATASOURCE_PREVIEW,
  GET_GRAPH_PREVIEW,
  GetDataSourcePreviewResponse,
  GetDataSourcePreviewVariables,
  GetGraphPreviewResponse,
  GetGraphPreviewVariables,
  DataSourcePreview,
  GraphPreview,
} from '../graphql/preview';

/**
 * Hook to fetch DataSource preview with table data
 * @param projectId - The project ID
 * @param nodeId - The DAG node ID
 * @param options - Optional query options (limit, offset, skip)
 */
export function useDataSourcePreview(
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
    GetDataSourcePreviewResponse,
    GetDataSourcePreviewVariables
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
    preview: data?.datasourcePreview,
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
  preview: DataSourcePreview | GraphPreview | null | undefined
): preview is DataSourcePreview | GraphPreview {
  return preview !== null && preview !== undefined;
}

/**
 * Type guard to check if preview is DataSource
 */
export function isDataSourcePreview(
  preview: DataSourcePreview | GraphPreview | null | undefined
): preview is DataSourcePreview {
  return (
    preview !== null &&
    preview !== undefined &&
    'datasourceId' in preview
  );
}

/**
 * Type guard to check if preview is Graph
 */
export function isGraphPreview(
  preview: DataSourcePreview | GraphPreview | null | undefined
): preview is GraphPreview {
  return (
    preview !== null &&
    preview !== undefined &&
    'graphId' in preview
  );
}
