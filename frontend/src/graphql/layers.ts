import { gql } from '@apollo/client'

export const GET_PROJECT_LAYERS = gql`
  query GetProjectLayers($projectId: Int!) {
    projectLayers(projectId: $projectId) {
      id
      projectId
      layerId
      name
      backgroundColor
      textColor
      borderColor
      sourceDatasetId
      enabled
      createdAt
      updatedAt
    }
    missingLayers(projectId: $projectId)
  }
`

export const UPSERT_PROJECT_LAYER = gql`
  mutation UpsertProjectLayer($projectId: Int!, $input: ProjectLayerInput!) {
    upsertProjectLayer(projectId: $projectId, input: $input) {
      id
      projectId
      layerId
      name
      backgroundColor
      textColor
      borderColor
      sourceDatasetId
      enabled
    }
  }
`

export const DELETE_PROJECT_LAYER = gql`
  mutation DeleteProjectLayer($projectId: Int!, $layerId: String!, $sourceDatasetId: Int) {
    deleteProjectLayer(
      projectId: $projectId
      layerId: $layerId
      sourceDatasetId: $sourceDatasetId
    )
  }
`

export const SET_LAYER_DATASET_ENABLED = gql`
  mutation SetLayerDatasetEnabled($projectId: Int!, $dataSetId: Int!, $enabled: Boolean!) {
    setLayerDatasetEnabled(projectId: $projectId, dataSetId: $dataSetId, enabled: $enabled)
  }
`

export interface ProjectLayerInput {
  layerId: string
  name: string
  backgroundColor?: string
  textColor?: string
  borderColor?: string
  sourceDatasetId?: number | null
  enabled?: boolean
}

export interface ProjectLayer {
  id: number
  projectId: number
  layerId: string
  name: string
  backgroundColor: string
  textColor: string
  borderColor: string
  sourceDatasetId?: number | null
  enabled: boolean
}
