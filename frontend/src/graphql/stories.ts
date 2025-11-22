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
        sourceDatasetId
        mode
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
        sourceDatasetId
        mode
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
        sourceDatasetId
        mode
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
        sourceDatasetId
        mode
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

// Layer source configuration - controls how layer sources are rendered
// If a source is in the list, it's disabled and uses the fallback style
// If a source is not in the list, it uses the project layer colours
export interface StoryLayerConfig {
  sourceDatasetId: number | null  // null = manual layers
  mode: 'default' | 'light' | 'dark'  // fallback style when source is disabled
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
