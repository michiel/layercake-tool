import { gql } from '@apollo/client'
import { FileFormat, DataType, formatFileSize, getFileFormatDisplayName, getDataTypeDisplayName, detectFileFormat } from './datasets'

export const GET_LIBRARY_ITEMS = gql`
  query GetLibraryItems($filter: LibraryItemFilterInput) {
    libraryItems(filter: $filter) {
      id
      type
      name
      description
      tags
      metadata
      contentSize
      createdAt
      updatedAt
    }
  }
`

export const UPLOAD_LIBRARY_ITEM = gql`
  mutation UploadLibraryItem($input: UploadLibraryItemInput!) {
    uploadLibraryItem(input: $input) {
      id
      type
      name
      description
      tags
      metadata
      contentSize
      createdAt
      updatedAt
    }
  }
`

export const DELETE_LIBRARY_ITEM = gql`
  mutation DeleteLibraryItem($id: Int!) {
    deleteLibraryItem(id: $id)
  }
`

export const UPDATE_LIBRARY_ITEM = gql`
  mutation UpdateLibraryItem($id: Int!, $input: UpdateLibraryItemInput!) {
    updateLibraryItem(id: $id, input: $input) {
      id
      type
      name
      description
      tags
      updatedAt
    }
  }
`

export const REDETECT_LIBRARY_DATASET_TYPE = gql`
  mutation RedetectLibraryDatasetType($id: Int!) {
    redetectLibraryDatasetType(id: $id) {
      id
      type
      name
      description
      tags
      metadata
      updatedAt
    }
  }
`

export const IMPORT_LIBRARY_DATASETS = gql`
  mutation ImportLibraryDatasets($input: ImportLibraryDatasetsInput!) {
    importLibraryDatasets(input: $input) {
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
    }
  }
`

export const SEED_LIBRARY_ITEMS = gql`
  mutation SeedLibraryItems {
    seedLibraryItems {
      totalRemoteFiles
      createdCount
      skippedCount
      failedFiles
    }
  }
`

export const EXPORT_PROJECT_AS_TEMPLATE = gql`
  mutation ExportProjectAsTemplate($projectId: Int!) {
    exportProjectAsTemplate(projectId: $projectId) {
      id
      type
      name
      description
      tags
      metadata
      contentSize
      createdAt
      updatedAt
    }
  }
`

export const EXPORT_PROJECT_ARCHIVE = gql`
  mutation ExportProjectArchive($projectId: Int!, $includeKnowledgeBase: Boolean) {
    exportProjectArchive(projectId: $projectId, includeKnowledgeBase: $includeKnowledgeBase) {
      filename
      fileContent
    }
  }
`

export const CREATE_PROJECT_FROM_LIBRARY = gql`
  mutation CreateProjectFromLibrary($libraryItemId: Int!, $name: String) {
    createProjectFromLibrary(libraryItemId: $libraryItemId, name: $name) {
      id
      name
      description
      tags
      createdAt
      updatedAt
    }
  }
`

export enum LibraryItemType {
  DATASET = 'DATASET',
  PROJECT = 'PROJECT',
  PROJECT_TEMPLATE = 'PROJECT_TEMPLATE',
  PROMPT = 'PROMPT',
}

export interface LibraryItem {
  id: number
  type: LibraryItemType
  name: string
  description?: string
  tags: string[]
  metadata: Record<string, any>
  contentSize?: number
  createdAt: string
  updatedAt: string
}

export interface UploadLibraryItemInput {
  type: LibraryItemType
  name: string
  description?: string
  tags?: string[]
  fileName: string
  fileContent: string
  fileFormat?: FileFormat
  tabularDataType?: DataType
  contentType?: string
}

export interface ImportLibraryDatasetsInput {
  projectId: number
  libraryItemIds: number[]
}

export interface SeedLibraryItemsResult {
  totalRemoteFiles: number
  createdCount: number
  skippedCount: number
  failedFiles: string[]
}

export interface ExportProjectArchivePayload {
  filename: string
  fileContent: string
}

export {
  FileFormat,
  DataType,
  formatFileSize,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  detectFileFormat,
}
