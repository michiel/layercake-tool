import { gql } from '@apollo/client';

// DataSource Preview Query
export const GET_DATASOURCE_PREVIEW = gql`
  query GetDataSourcePreview(
    $projectId: Int!
    $nodeId: String!
    $limit: Int
    $offset: Int
  ) {
    datasourcePreview(
      projectId: $projectId
      nodeId: $nodeId
      limit: $limit
      offset: $offset
    ) {
      nodeId
      datasourceId
      name
      filePath
      fileType
      totalRows
      columns {
        name
        dataType
        nullable
      }
      rows {
        rowNumber
        data
      }
      importDate
      executionState
      errorMessage
    }
  }
`;

// Graph Preview Query
export const GET_GRAPH_PREVIEW = gql`
  query GetGraphPreview($projectId: Int!, $nodeId: String!) {
    graphPreview(projectId: $projectId, nodeId: $nodeId) {
      nodeId
      graphId
      name
      nodes {
        id
        label
        layer
        weight
        isPartition
        attrs
      }
      edges {
        id
        source
        target
        label
        layer
        weight
        attrs
      }
      nodeCount
      edgeCount
      executionState
      computedDate
      errorMessage
    }
  }
`;

// TypeScript Interfaces

export interface TableColumn {
  name: string;
  dataType: string;
  nullable: boolean;
}

export interface TableRow {
  rowNumber: number;
  data: Record<string, any>;
}

export interface DataSourcePreview {
  nodeId: string;
  datasourceId: number;
  name: string;
  filePath: string;
  fileType: string;
  totalRows: number;
  columns: TableColumn[];
  rows: TableRow[];
  importDate?: string;
  executionState: string;
  errorMessage?: string;
}

export interface GraphNodePreview {
  id: string;
  label?: string;
  layer?: string;
  weight?: number;
  isPartition: boolean;
  attrs?: Record<string, any>;
}

export interface GraphEdgePreview {
  id: string;
  source: string;
  target: string;
  label?: string;
  layer?: string;
  weight?: number;
  attrs?: Record<string, any>;
}

export interface GraphPreview {
  nodeId: string;
  graphId: number;
  name: string;
  nodes: GraphNodePreview[];
  edges: GraphEdgePreview[];
  nodeCount: number;
  edgeCount: number;
  executionState: string;
  computedDate?: string;
  errorMessage?: string;
}

export interface GetDataSourcePreviewResponse {
  datasourcePreview: DataSourcePreview | null;
}

export interface GetGraphPreviewResponse {
  graphPreview: GraphPreview | null;
}

// Query Variables

export interface GetDataSourcePreviewVariables {
  projectId: number;
  nodeId: string;
  limit?: number;
  offset?: number;
}

export interface GetGraphPreviewVariables {
  projectId: number;
  nodeId: string;
}

// Execution States
export enum ExecutionState {
  NOT_STARTED = 'not_started',
  PENDING = 'pending',
  PROCESSING = 'processing',
  COMPLETED = 'completed',
  ERROR = 'error',
}

// Helper to check if execution is complete
export function isExecutionComplete(state: string): boolean {
  return state === ExecutionState.COMPLETED;
}

// Helper to check if execution failed
export function isExecutionFailed(state: string): boolean {
  return state === ExecutionState.ERROR;
}

// Helper to check if execution is in progress
export function isExecutionInProgress(state: string): boolean {
  return state === ExecutionState.PENDING || state === ExecutionState.PROCESSING;
}

// Helper to get execution state display label
export function getExecutionStateLabel(state: string): string {
  switch (state) {
    case ExecutionState.NOT_STARTED:
      return 'Not Started';
    case ExecutionState.PENDING:
      return 'Pending';
    case ExecutionState.PROCESSING:
      return 'Processing';
    case ExecutionState.COMPLETED:
      return 'Ready';
    case ExecutionState.ERROR:
      return 'Error';
    default:
      return state;
  }
}

// Helper to get execution state color
export function getExecutionStateColor(state: string): string {
  switch (state) {
    case ExecutionState.NOT_STARTED:
      return 'gray';
    case ExecutionState.PENDING:
      return 'yellow';
    case ExecutionState.PROCESSING:
      return 'blue';
    case ExecutionState.COMPLETED:
      return 'green';
    case ExecutionState.ERROR:
      return 'red';
    default:
      return 'gray';
  }
}
