import { gql } from '@apollo/client'

// Query to list all stories for a project
export const LIST_STORIES = gql`
  query ListStories($projectId: Int!) {
    stories(projectId: $projectId) {
      id
      projectId
      name
      description
      tags
      enabledDatasetIds
      layerConfig {
        layerId
        enabled
        color
        sourceDatasetId
      }
      sequenceCount
      createdAt
      updatedAt
    }
  }
`

// Query to get a single story by ID
export const GET_STORY = gql`
  query GetStory($id: Int!) {
    story(id: $id) {
      id
      projectId
      name
      description
      tags
      enabledDatasetIds
      layerConfig {
        layerId
        enabled
        color
        sourceDatasetId
      }
      sequenceCount
      createdAt
      updatedAt
    }
  }
`

// Mutation to create a new story
export const CREATE_STORY = gql`
  mutation CreateStory($input: CreateStoryInput!) {
    createStory(input: $input) {
      id
      projectId
      name
      description
      tags
      enabledDatasetIds
      layerConfig {
        layerId
        enabled
        color
        sourceDatasetId
      }
      sequenceCount
      createdAt
      updatedAt
    }
  }
`

// Mutation to update a story
export const UPDATE_STORY = gql`
  mutation UpdateStory($id: Int!, $input: UpdateStoryInput!) {
    updateStory(id: $id, input: $input) {
      id
      projectId
      name
      description
      tags
      enabledDatasetIds
      layerConfig {
        layerId
        enabled
        color
        sourceDatasetId
      }
      sequenceCount
      createdAt
      updatedAt
    }
  }
`

// Mutation to delete a story
export const DELETE_STORY = gql`
  mutation DeleteStory($id: Int!) {
    deleteStory(id: $id)
  }
`

// TypeScript interfaces

export interface StoryLayerConfig {
  layerId: string
  enabled: boolean
  color: string | null
  sourceDatasetId: number | null
}

export interface Story {
  id: number
  projectId: number
  name: string
  description: string | null
  tags: string[]
  enabledDatasetIds: number[]
  layerConfig: StoryLayerConfig[]
  sequenceCount: number
  createdAt: string
  updatedAt: string
}

export interface CreateStoryInput {
  projectId: number
  name: string
  description?: string
  tags?: string[]
  enabledDatasetIds?: number[]
  layerConfig?: StoryLayerConfig[]
}

export interface UpdateStoryInput {
  name?: string
  description?: string
  tags?: string[]
  enabledDatasetIds?: number[]
  layerConfig?: StoryLayerConfig[]
}
