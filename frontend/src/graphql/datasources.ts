import { gql } from '@apollo/client'

// Query to fetch all DataSources for a project
export const GET_DATASOURCES = gql`
  query GetDataSources($projectId: Int!) {
    dataSources(projectId: $projectId) {
      id
      projectId
      name
      description
      sourceType
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
    }
  }
`

// Query to fetch a single DataSource by ID
export const GET_DATASOURCE = gql`
  query GetDataSource($id: Int!) {
    dataSource(id: $id) {
      id
      projectId
      name
      description
      sourceType
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
    }
  }
`

// Mutation to create a new DataSource from uploaded file
export const CREATE_DATASOURCE_FROM_FILE = gql`
  mutation CreateDataSourceFromFile($input: CreateDataSourceInput!) {
    createDataSourceFromFile(input: $input) {
      id
      projectId
      name
      description
      sourceType
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
    }
  }
`

// Mutation to update DataSource metadata
export const UPDATE_DATASOURCE = gql`
  mutation UpdateDataSource($id: Int!, $input: UpdateDataSourceInput!) {
    updateDataSource(id: $id, input: $input) {
      id
      projectId
      name
      description
      sourceType
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
    }
  }
`

// Mutation to delete a DataSource
export const DELETE_DATASOURCE = gql`
  mutation DeleteDataSource($id: Int!) {
    deleteDataSource(id: $id)
  }
`

// Mutation to reprocess an existing DataSource
export const REPROCESS_DATASOURCE = gql`
  mutation ReprocessDataSource($id: Int!) {
    reprocessDataSource(id: $id) {
      id
      projectId
      name
      description
      sourceType
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
    }
  }
`

// Subscription for DataSource updates
export const DATASOURCE_UPDATED = gql`
  subscription DataSourceUpdated($projectId: Int!) {
    dataSourceUpdated(projectId: $projectId) {
      id
      projectId
      name
      description
      sourceType
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
    }
  }
`

// TypeScript interfaces for the GraphQL types
export interface DataSource {
  id: number
  projectId: number
  name: string
  description?: string
  sourceType: 'csv_nodes' | 'csv_edges' | 'csv_layers' | 'json_graph'
  filename: string
  graphJson: string
  status: 'active' | 'processing' | 'error'
  errorMessage?: string
  fileSize: number
  processedAt?: string
  createdAt: string
  updatedAt: string
}

export interface CreateDataSourceInput {
  projectId: number
  name: string
  description?: string
  filename: string
  fileContent: string // Base64 encoded file content
}

export interface UpdateDataSourceInput {
  name?: string
  description?: string
  filename?: string
  fileContent?: string // Base64 encoded file content
}

// Helper function to format file size
export const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) {
    return `${bytes} B`
  } else if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`
  } else if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  } else {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }
}

// Helper function to get data source type display name
export const getDataSourceTypeDisplayName = (sourceType: DataSource['sourceType']): string => {
  switch (sourceType) {
    case 'csv_nodes':
      return 'CSV Nodes'
    case 'csv_edges':
      return 'CSV Edges'
    case 'csv_layers':
      return 'CSV Layers'
    case 'json_graph':
      return 'JSON Graph'
    default:
      return 'Unknown'
  }
}

// Helper function to get status color for badges
export const getStatusColor = (status: DataSource['status']): string => {
  switch (status) {
    case 'active':
      return 'green'
    case 'processing':
      return 'yellow'
    case 'error':
      return 'red'
    default:
      return 'gray'
  }
}