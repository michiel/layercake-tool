import { gql } from '@apollo/client'

// Query to list all sequences for a story
export const LIST_SEQUENCES = gql`
  query ListSequences($storyId: Int!) {
    sequences(storyId: $storyId) {
      id
      storyId
      name
      description
      enabledDatasetIds
      edgeOrder {
        datasetId
        edgeId
      }
      edgeCount
      createdAt
      updatedAt
    }
  }
`

// Query to get a single sequence by ID
export const GET_SEQUENCE = gql`
  query GetSequence($id: Int!) {
    sequence(id: $id) {
      id
      storyId
      name
      description
      enabledDatasetIds
      edgeOrder {
        datasetId
        edgeId
      }
      edgeCount
      createdAt
      updatedAt
    }
  }
`

// Mutation to create a new sequence
export const CREATE_SEQUENCE = gql`
  mutation CreateSequence($input: CreateSequenceInput!) {
    createSequence(input: $input) {
      id
      storyId
      name
      description
      enabledDatasetIds
      edgeOrder {
        datasetId
        edgeId
      }
      edgeCount
      createdAt
      updatedAt
    }
  }
`

// Mutation to update a sequence
export const UPDATE_SEQUENCE = gql`
  mutation UpdateSequence($id: Int!, $input: UpdateSequenceInput!) {
    updateSequence(id: $id, input: $input) {
      id
      storyId
      name
      description
      enabledDatasetIds
      edgeOrder {
        datasetId
        edgeId
      }
      edgeCount
      createdAt
      updatedAt
    }
  }
`

// Mutation to delete a sequence
export const DELETE_SEQUENCE = gql`
  mutation DeleteSequence($id: Int!) {
    deleteSequence(id: $id)
  }
`

// TypeScript interfaces

export interface SequenceEdgeRef {
  datasetId: number
  edgeId: string
}

export interface Sequence {
  id: number
  storyId: number
  name: string
  description: string | null
  enabledDatasetIds: number[]
  edgeOrder: SequenceEdgeRef[]
  edgeCount: number
  createdAt: string
  updatedAt: string
}

export interface CreateSequenceInput {
  storyId: number
  name: string
  description?: string
  enabledDatasetIds?: number[]
  edgeOrder?: SequenceEdgeRef[]
}

export interface UpdateSequenceInput {
  name?: string
  description?: string
  enabledDatasetIds?: number[]
  edgeOrder?: SequenceEdgeRef[]
}
