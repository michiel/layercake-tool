import { gql } from '@apollo/client'

// Query to fetch all DataSources for a project
export const GET_DATASOURCES = gql`
  query GetDataSources($projectId: Int!) {
    dataSources(projectId: $projectId) {
      id
      projectId
      name
      description
      fileFormat
      dataType
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
      fileFormat
      dataType
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
      fileFormat
      dataType
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

// Mutation to bulk upload multiple DataSources with auto-detection
export const BULK_UPLOAD_DATASOURCES = gql`
  mutation BulkUploadDataSources($projectId: Int!, $files: [BulkUploadDataSourceInput!]!) {
    bulkUploadDataSources(projectId: $projectId, files: $files) {
      id
      projectId
      name
      description
      fileFormat
      dataType
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
      fileFormat
      dataType
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
      fileFormat
      dataType
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

// Mutation to update graph JSON data
export const UPDATE_DATASOURCE_GRAPH_DATA = gql`
  mutation UpdateDataSourceGraphData($id: Int!, $graphJson: String!) {
    updateDataSourceGraphData(id: $id, graphJson: $graphJson) {
      id
      projectId
      name
      description
      fileFormat
      dataType
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
      fileFormat
      dataType
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

// Export data sources as spreadsheet
export const EXPORT_DATASOURCES = gql`
  mutation ExportDataSources($input: ExportDataSourcesInput!) {
    exportDataSources(input: $input) {
      fileContent
      filename
      format
    }
  }
`

// Import data sources from spreadsheet
export const IMPORT_DATASOURCES = gql`
  mutation ImportDataSources($input: ImportDataSourcesInput!) {
    importDataSources(input: $input) {
      dataSources {
        id
        projectId
        name
        description
        fileFormat
        dataType
        filename
        graphJson
        status
        errorMessage
        fileSize
        processedAt
        createdAt
        updatedAt
      }
      createdCount
      updatedCount
    }
  }
`

// File format enum (physical representation)
export enum FileFormat {
  CSV = 'CSV',
  TSV = 'TSV',
  JSON = 'JSON',
}

// Data type enum (semantic meaning)
// Note: GraphQL name is DataSourceDataType to avoid conflict with Plan DAG DataType
export enum DataType {
  NODES = 'NODES',
  EDGES = 'EDGES',
  LAYERS = 'LAYERS',
  GRAPH = 'GRAPH',
}

// TypeScript interfaces for the GraphQL types
export interface DataSource {
  id: number
  projectId: number
  name: string
  description?: string
  fileFormat: string
  dataType: string
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
  fileFormat: FileFormat
  dataType: DataType
}

export interface BulkUploadDataSourceInput {
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

// Helper function to get file format display name
export const getFileFormatDisplayName = (format: string): string => {
  switch (format) {
    case 'CSV':
    case 'csv':
      return 'CSV'
    case 'TSV':
    case 'tsv':
      return 'TSV'
    case 'JSON':
    case 'json':
      return 'JSON'
    default:
      return 'Unknown'
  }
}

// Helper function to get data type display name
export const getDataTypeDisplayName = (dataType: string): string => {
  switch (dataType) {
    case 'NODES':
    case 'nodes':
      return 'Nodes'
    case 'EDGES':
    case 'edges':
      return 'Edges'
    case 'LAYERS':
    case 'layers':
      return 'Layers'
    case 'GRAPH':
    case 'graph':
      return 'Graph'
    default:
      return 'Unknown'
  }
}

// Helper function to detect file format from filename
export const detectFileFormat = (filename: string): FileFormat | null => {
  const lower = filename.toLowerCase()
  if (lower.endsWith('.csv')) return FileFormat.CSV
  if (lower.endsWith('.tsv')) return FileFormat.TSV
  if (lower.endsWith('.json')) return FileFormat.JSON
  return null
}

// Helper function to check if format/type combination is valid
export const isValidFormatTypeCombination = (format: FileFormat, type: DataType): boolean => {
  if ((format === FileFormat.CSV || format === FileFormat.TSV) &&
      (type === DataType.NODES || type === DataType.EDGES || type === DataType.LAYERS)) {
    return true
  }
  if (format === FileFormat.JSON && type === DataType.GRAPH) {
    return true
  }
  return false
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
