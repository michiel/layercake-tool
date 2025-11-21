import { gql } from '@apollo/client'

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

export const LIST_PLANS = gql`
  query Plans($projectId: Int!) {
    plans(projectId: $projectId) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const GET_PLAN = gql`
  query GetPlan($id: Int!) {
    plan(id: $id) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const CREATE_PLAN = gql`
  mutation CreatePlan($input: CreatePlanInput!) {
    createPlan(input: $input) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const UPDATE_PLAN = gql`
  mutation UpdatePlan($id: Int!, $input: UpdatePlanInput!) {
    updatePlan(id: $id, input: $input) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`

export const DELETE_PLAN = gql`
  mutation DeletePlan($id: Int!) {
    deletePlan(id: $id)
  }
`

export const DUPLICATE_PLAN = gql`
  mutation DuplicatePlan($id: Int!, $name: String!) {
    duplicatePlan(id: $id, name: $name) {
      ...PlanFields
    }
  }
  ${PLAN_FIELDS}
`
