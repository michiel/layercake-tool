import { gql, type TypedDocumentNode } from '@apollo/client'
import type { Plan } from '../types/plan'

const PLAN_FIELDS = gql`
  fragment PlanFields on Plan {
    id
    projectId
    name
    description
    tags
    status
    version
    yamlContent
    dependencies
    createdAt
    updatedAt
  }
`

export const LIST_PLANS: TypedDocumentNode<
  { plans: Plan[] },
  { projectId: number }
> = gql`
  query Plans($projectId: Int!) {
    plans(projectId: $projectId) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const GET_PLAN: TypedDocumentNode<
  { plan: Plan | null },
  { id: number }
> = gql`
  query GetPlan($id: Int!) {
    plan(id: $id) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const CREATE_PLAN: TypedDocumentNode<
  { createPlan: Plan },
  { input: Record<string, unknown> }
> = gql`
  mutation CreatePlan($input: CreatePlanInput!) {
    createPlan(input: $input) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const UPDATE_PLAN: TypedDocumentNode<
  { updatePlan: Plan },
  { id: number; input: Record<string, unknown> }
> = gql`
  mutation UpdatePlan($id: Int!, $input: UpdatePlanInput!) {
    updatePlan(id: $id, input: $input) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const DELETE_PLAN: TypedDocumentNode<
  { deletePlan: boolean },
  { id: number }
> = gql`
  mutation DeletePlan($id: Int!) {
    deletePlan(id: $id)
  }
`

export const DUPLICATE_PLAN: TypedDocumentNode<
  { duplicatePlan: Plan },
  { id: number; name: string }
> = gql`
  mutation DuplicatePlan($id: Int!, $name: String!) {
    duplicatePlan(id: $id, name: $name) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`
