import { gql } from '@apollo/client'

// Query to fetch all DataSets for a project
export const GET_DATASOURCES = gql`
  query GetDataSets($projectId: Int!) {
    dataSets(projectId: $projectId) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
      annotations {
        title
        date
        body
      }
    }
  }
`

// Query to fetch a single DataSet by ID
export const GET_DATASOURCE = gql`
  query GetDataSet($id: Int!) {
    dataSet(id: $id) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
      annotations {
        title
        date
        body
      }
    }
  }
`

// Mutation to create a new DataSet from uploaded file
export const CREATE_DATASOURCE_FROM_FILE = gql`
  mutation CreateDataSetFromFile($input: CreateDataSetInput!) {
    createDataSetFromFile(input: $input) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
      annotations {
        title
        date
        body
      }
    }
  }
`

// Mutation to create a new empty DataSet (without file upload)
export const CREATE_EMPTY_DATASOURCE = gql`
  mutation CreateEmptyDataSet($input: CreateEmptyDataSetInput!) {
    createEmptyDataSet(input: $input) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
    }
  }
`

// Mutation to bulk upload multiple DataSets with auto-detection
export const BULK_UPLOAD_DATASOURCES = gql`
  mutation BulkUploadDataSets($projectId: Int!, $files: [BulkUploadDataSetInput!]!) {
    bulkUploadDataSets(projectId: $projectId, files: $files) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
    }
  }
`

// Mutation to update DataSet metadata
export const UPDATE_DATASOURCE = gql`
  mutation UpdateDataSet($id: Int!, $input: UpdateDataSetInput!) {
    updateDataSet(id: $id, input: $input) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
    }
  }
`

export const VALIDATE_DATASET = gql`
  mutation ValidateDataSet($id: Int!) {
    validateDataSet(id: $id) {
      dataSetId
      projectId
      isValid
      errors
      warnings
      nodeCount
      edgeCount
      layerCount
      checkedAt
    }
  }
`

// Mutation to delete a DataSet
export const DELETE_DATASOURCE = gql`
  mutation DeleteDataSet($id: Int!) {
    deleteDataSet(id: $id)
  }
`

// Mutation to reprocess an existing DataSet
export const REPROCESS_DATASOURCE = gql`
  mutation ReprocessDataSet($id: Int!) {
    reprocessDataSet(id: $id) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
    }
  }
`

// Mutation to update graph JSON data
export const UPDATE_DATASOURCE_GRAPH_DATA = gql`
  mutation UpdateDataSetGraphData($id: Int!, $graphJson: String!) {
    updateDataSetGraphData(id: $id, graphJson: $graphJson) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
    }
  }
`

// Subscription for DataSet updates
export const DATASOURCE_UPDATED = gql`
  subscription DataSetUpdated($projectId: Int!) {
    dataSetUpdated(projectId: $projectId) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
    }
  }
`

// Export data sources as spreadsheet
export const EXPORT_DATASOURCES = gql`
  mutation ExportDataSets($input: ExportDataSetsInput!) {
    exportDataSets(input: $input) {
      fileContent
      filename
      format
    }
  }
`

// Import data sources from spreadsheet
export const IMPORT_DATASOURCES = gql`
  mutation ImportDataSets($input: ImportDataSetsInput!) {
    importDataSets(input: $input) {
      dataSets {
        id
        projectId
        name
        description
        fileFormat
        filename
        graphJson
        status
        errorMessage
        fileSize
        processedAt
        createdAt
        updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
      }
      createdCount
      updatedCount
    }
  }
`

// Merge multiple data sets into a single new data set
export const MERGE_DATASOURCES = gql`
  mutation MergeDataSets($input: MergeDataSetsInput!) {
    mergeDataSets(input: $input) {
      id
      projectId
      name
      description
      fileFormat
      origin
      filename
      graphJson
      status
      errorMessage
      fileSize
      processedAt
      createdAt
      updatedAt
      nodeCount
      edgeCount
      layerCount
      hasLayers
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
// Note: GraphQL name is DataSetDataType to avoid conflict with Plan DAG DataType
export enum DataType {
  NODES = 'NODES',
  EDGES = 'EDGES',
  LAYERS = 'LAYERS',
  GRAPH = 'GRAPH',
}

// TypeScript interfaces for the GraphQL types
export interface DataSet {
  id: number
  projectId: number
  name: string
  description?: string
  annotations?: Array<{ title: string; date: string; body: string }>
  fileFormat: string
  origin: string
  filename: string
  graphJson: string
  status: 'active' | 'processing' | 'error'
  errorMessage?: string
  fileSize: number
  processedAt?: string
  createdAt: string
  updatedAt: string
  nodeCount?: number
  edgeCount?: number
  layerCount?: number
  hasLayers?: boolean
}

export const GET_GRAPH_SUMMARY = gql`
  query GraphSummary($datasetId: Int!) {
    graphSummary(datasetId: $datasetId) {
      nodeCount
      edgeCount
      layerCount
      layers
    }
  }
`

export const GET_GRAPH_PAGE = gql`
  query GraphPage($datasetId: Int!, $limit: Int!, $offset: Int!, $layers: [String!]) {
    graphPage(datasetId: $datasetId, limit: $limit, offset: $offset, layers: $layers) {
      hasMore
      nodes {
        id
        label
        layer
        belongsTo
        weight
        isPartition
        comment
        dataset
        attributes
      }
      edges {
        id
        source
        target
        label
        layer
        weight
        comment
        dataset
        attributes
      }
      layers {
        id
        label
        backgroundColor
        textColor
        borderColor
      }
    }
  }
`

export interface GraphSummary {
  nodeCount: number
  edgeCount: number
  layerCount: number
  layers: string[]
}

export interface GraphPageSlice {
  hasMore: boolean
  nodes: Array<{
    id: string
    label: string
    layer: string
    belongsTo?: string | null
    weight: number
    isPartition: boolean
    comment?: string | null
    dataset?: string | null
    attributes?: any
  }>
  edges: Array<{
    id: string
    source: string
    target: string
    label: string
    layer: string
    weight: number
    comment?: string | null
    dataset?: string | null
    attributes?: any
  }>
  layers: Array<{
    id: string
    label: string
    backgroundColor?: string | null
    textColor?: string | null
    borderColor?: string | null
  }>
}

export interface CreateDataSetInput {
  projectId: number
  name: string
  description?: string
  filename: string
  fileContent: string // Base64 encoded file content
  fileFormat: FileFormat
  tabularDataType?: DataType
}

export interface CreateEmptyDataSetInput {
  projectId: number
  name: string
  description?: string
}

export interface BulkUploadDataSetInput {
  name: string
  description?: string
  filename: string
  fileContent: string // Base64 encoded file content
}

export interface UpdateDataSetInput {
  name?: string
  description?: string
  filename?: string
  fileContent?: string // Base64 encoded file content
}

export interface DataSetValidationResult {
  dataSetId: number
  projectId: number
  isValid: boolean
  errors: string[]
  warnings: string[]
  nodeCount: number
  edgeCount: number
  layerCount: number
  checkedAt: string
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

// Helper function to get origin display name
export const getOriginDisplayName = (origin: string): string => {
  switch (origin) {
    case 'file_upload':
      return 'File upload'
    case 'manual_edit':
      return 'Manual edit'
    case 'rag_agent':
      return 'RAG Agent'
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
export const getStatusColor = (status: DataSet['status']): string => {
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
