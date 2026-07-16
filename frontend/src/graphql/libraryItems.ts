import { gql, type TypedDocumentNode } from '@apollo/client'
import { FileFormat, DataType, formatFileSize, getFileFormatDisplayName, getDataTypeDisplayName, detectFileFormat, type DataSet } from './datasets'

export const GET_LIBRARY_ITEMS: TypedDocumentNode<
  { libraryItems: LibraryItem[] },
  { filter?: Record<string, unknown> }
> = gql`
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

export const UPLOAD_LIBRARY_ITEM: TypedDocumentNode<
  { uploadLibraryItem: LibraryItem },
  { input: UploadLibraryItemInput }
> = gql`
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

export const DELETE_LIBRARY_ITEM: TypedDocumentNode<
  { deleteLibraryItem: boolean },
  { id: number }
> = gql`
  mutation DeleteLibraryItem($id: Int!) {
    deleteLibraryItem(id: $id)
  }
`

export const UPDATE_LIBRARY_ITEM: TypedDocumentNode<
  { updateLibraryItem: LibraryItem },
  { id: number; input: Record<string, unknown> }
> = gql`
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

export const REDETECT_LIBRARY_DATASET_TYPE: TypedDocumentNode<
  { redetectLibraryDatasetType: LibraryItem },
  { id: number }
> = gql`
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

export const IMPORT_LIBRARY_DATASETS: TypedDocumentNode<
  { importLibraryDatasets: DataSet[] },
  { input: ImportLibraryDatasetsInput }
> = gql`
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

export const SEED_LIBRARY_ITEMS: TypedDocumentNode<
  { seedLibraryItems: SeedLibraryItemsResult },
  Record<string, never>
> = gql`
  mutation SeedLibraryItems {
    seedLibraryItems {
      totalRemoteFiles
      createdCount
      skippedCount
      failedFiles
    }
  }
`

export const EXPORT_PROJECT_AS_TEMPLATE: TypedDocumentNode<
  { exportProjectAsTemplate: LibraryItem },
  { projectId: number }
> = gql`
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

export const EXPORT_PROJECT_ARCHIVE: TypedDocumentNode<
  { exportProjectArchive: ExportProjectArchivePayload },
  { projectId: number }
> = gql`
  mutation ExportProjectArchive($projectId: Int!) {
    exportProjectArchive(projectId: $projectId) {
      filename
      fileContent
    }
  }
`

export const RESET_PROJECT: TypedDocumentNode<
  {
    resetProject: {
      id: number
      name: string
      description?: string
      tags: string[]
      createdAt: string
      updatedAt: string
    }
  },
  { projectId: number }
> = gql`
  mutation ResetProject($projectId: Int!) {
    resetProject(projectId: $projectId) {
      id
      name
      description
      tags
      createdAt
      updatedAt
    }
  }
`

export const CREATE_PROJECT_FROM_LIBRARY: TypedDocumentNode<
  {
    createProjectFromLibrary: {
      id: number
      name: string
      description?: string
      tags: string[]
      createdAt: string
      updatedAt: string
    }
  },
  { libraryItemId: number; name?: string }
> = gql`
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
