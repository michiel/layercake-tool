import { gql, type TypedDocumentNode } from '@apollo/client'

export const GET_PROJECT_LAYERS: TypedDocumentNode<
  { projectLayers: ProjectLayer[]; missingLayers: string[] },
  { projectId: number }
> = gql`
  query GetProjectLayers($projectId: Int!) {
    projectLayers(projectId: $projectId) {
      id
      projectId
      layerId
      name
      backgroundColor
      textColor
      borderColor
      alias
      sourceDatasetId
      enabled
      createdAt
      updatedAt
      aliases {
        id
        aliasLayerId
        targetLayerId
      }
    }
    missingLayers(projectId: $projectId)
  }
`

export const UPSERT_PROJECT_LAYER: TypedDocumentNode<
  { upsertProjectLayer: ProjectLayer },
  { projectId: number; input: ProjectLayerInput }
> = gql`
  mutation UpsertProjectLayer($projectId: Int!, $input: ProjectLayerInput!) {
    upsertProjectLayer(projectId: $projectId, input: $input) {
      id
      projectId
      layerId
      name
      backgroundColor
      textColor
      borderColor
      alias
      sourceDatasetId
      enabled
    }
  }
`

export const DELETE_PROJECT_LAYER: TypedDocumentNode<
  { deleteProjectLayer: boolean },
  { projectId: number; layerId: string; sourceDatasetId?: number | null }
> = gql`
  mutation DeleteProjectLayer($projectId: Int!, $layerId: String!, $sourceDatasetId: Int) {
    deleteProjectLayer(
      projectId: $projectId
      layerId: $layerId
      sourceDatasetId: $sourceDatasetId
    )
  }
`

export const SET_LAYER_DATASET_ENABLED: TypedDocumentNode<
  { setLayerDatasetEnabled: boolean },
  { projectId: number; dataSetId: number; enabled: boolean }
> = gql`
  mutation SetLayerDatasetEnabled($projectId: Int!, $dataSetId: Int!, $enabled: Boolean!) {
    setLayerDatasetEnabled(projectId: $projectId, dataSetId: $dataSetId, enabled: $enabled)
  }
`

export const RESET_PROJECT_LAYERS: TypedDocumentNode<
  { resetProjectLayers: boolean },
  { projectId: number }
> = gql`
  mutation ResetProjectLayers($projectId: Int!) {
    resetProjectLayers(projectId: $projectId)
  }
`

export const LIST_LAYER_ALIASES: TypedDocumentNode<
  { listLayerAliases: LayerAlias[] },
  { projectId: number }
> = gql`
  query ListLayerAliases($projectId: Int!) {
    listLayerAliases(projectId: $projectId) {
      id
      projectId
      aliasLayerId
      targetLayerId
      targetLayer {
        id
        layerId
        name
        backgroundColor
        textColor
        borderColor
      }
      createdAt
    }
  }
`

export const GET_LAYER_ALIASES: TypedDocumentNode<
  { getLayerAliases: LayerAlias[] },
  { projectId: number; targetLayerId: number }
> = gql`
  query GetLayerAliases($projectId: Int!, $targetLayerId: Int!) {
    getLayerAliases(projectId: $projectId, targetLayerId: $targetLayerId) {
      id
      projectId
      aliasLayerId
      targetLayerId
      createdAt
    }
  }
`

export const CREATE_LAYER_ALIAS: TypedDocumentNode<
  { createLayerAlias: LayerAlias },
  { projectId: number; aliasLayerId: string; targetLayerId: number }
> = gql`
  mutation CreateLayerAlias($projectId: Int!, $aliasLayerId: String!, $targetLayerId: Int!) {
    createLayerAlias(
      projectId: $projectId
      aliasLayerId: $aliasLayerId
      targetLayerId: $targetLayerId
    ) {
      id
      aliasLayerId
      targetLayerId
      targetLayer {
        id
        name
        backgroundColor
        textColor
        borderColor
      }
    }
  }
`

export const REMOVE_LAYER_ALIAS: TypedDocumentNode<
  { removeLayerAlias: boolean },
  { projectId: number; aliasLayerId: string }
> = gql`
  mutation RemoveLayerAlias($projectId: Int!, $aliasLayerId: String!) {
    removeLayerAlias(projectId: $projectId, aliasLayerId: $aliasLayerId)
  }
`

export const REMOVE_LAYER_ALIASES: TypedDocumentNode<
  { removeLayerAliases: boolean },
  { projectId: number; targetLayerId: number }
> = gql`
  mutation RemoveLayerAliases($projectId: Int!, $targetLayerId: Int!) {
    removeLayerAliases(projectId: $projectId, targetLayerId: $targetLayerId)
  }
`

export interface ProjectLayerInput {
  layerId: string
  name: string
  backgroundColor?: string
  textColor?: string
  borderColor?: string
  alias?: string | null
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
  alias?: string | null
  sourceDatasetId?: number | null
  enabled: boolean
  aliases?: LayerAlias[]
}

export interface LayerAlias {
  id: number
  projectId: number
  aliasLayerId: string
  targetLayerId: number
  targetLayer?: ProjectLayer
  createdAt: string
}
