import { gql } from '@apollo/client'
import {
  FileFormat,
  DataType,
  formatFileSize,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  getStatusColor,
  detectFileFormat,
} from './datasets'

export const GET_LIBRARY_SOURCES = gql`
  query GetLibrarySources {
    librarySources {
      id
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

export const GET_LIBRARY_SOURCE = gql`
  query GetLibrarySource($id: Int!) {
    librarySource(id: $id) {
      id
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

export const CREATE_LIBRARY_SOURCE = gql`
  mutation CreateLibrarySource($input: CreateLibrarySourceInput!) {
    createLibrarySource(input: $input) {
      id
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

export const UPDATE_LIBRARY_SOURCE = gql`
  mutation UpdateLibrarySource($id: Int!, $input: UpdateLibrarySourceInput!) {
    updateLibrarySource(id: $id, input: $input) {
      id
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

export const DELETE_LIBRARY_SOURCE = gql`
  mutation DeleteLibrarySource($id: Int!) {
    deleteLibrarySource(id: $id)
  }
`

export const REPROCESS_LIBRARY_SOURCE = gql`
  mutation ReprocessLibrarySource($id: Int!) {
    reprocessLibrarySource(id: $id) {
      id
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

export interface LibrarySource {
  id: number
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

export interface CreateLibrarySourceInput {
  name: string
  description?: string
  filename: string
  fileContent: string
  fileFormat: FileFormat
  dataType: DataType
}

export interface UpdateLibrarySourceInput {
  name?: string
  description?: string
  filename?: string
  fileContent?: string
}

export const IMPORT_LIBRARY_SOURCES = gql`
  mutation ImportLibrarySources($input: ImportLibrarySourcesInput!) {
    importLibrarySources(input: $input) {
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

export interface ImportLibrarySourcesInput {
  projectId: number
  librarySourceIds: number[]
}

export const SEED_LIBRARY_SOURCES = gql`
  mutation SeedLibrarySources {
    seedLibrarySources {
      totalRemoteFiles
      createdCount
      skippedCount
      failedFiles
    }
  }
`

export interface SeedLibrarySourcesResult {
  totalRemoteFiles: number
  createdCount: number
  skippedCount: number
  failedFiles: string[]
}

// Re-export helpers for UI reuse
export {
  FileFormat,
  DataType,
  formatFileSize,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  getStatusColor,
  detectFileFormat,
}
