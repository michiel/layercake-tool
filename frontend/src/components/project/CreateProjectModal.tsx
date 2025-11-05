import React from 'react'
import { Modal, Stack, TextInput, Textarea, Button, Group } from '@mantine/core'
import { useForm } from '@mantine/form'
import { useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { UPDATE_PLAN_DAG } from '../../graphql/plan-dag'

const CREATE_PROJECT = gql`
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`

interface CreateProjectModalProps {
  opened: boolean
  onClose: () => void
  onSuccess: (project: { id: number; name: string; description?: string }) => void
}

export const CreateProjectModal: React.FC<CreateProjectModalProps> = ({
  opened,
  onClose,
  onSuccess
}) => {
  const [createProject, { loading }] = useMutation<{
    createProject: {
      id: number
      name: string
      description?: string
      createdAt: string
      updatedAt: string
    }
  }>(CREATE_PROJECT, {
    refetchQueries: [{ query: GET_PROJECTS }],
    awaitRefetchQueries: true
  })

  const [initializePlanDag] = useMutation<{
    updatePlanDag: {
      success: boolean
      errors: string[]
      planDag: any
    }
  }>(UPDATE_PLAN_DAG)

  const form = useForm({
    initialValues: {
      name: '',
      description: ''
    },
    validate: {
      name: (value) => {
        if (!value || value.trim().length === 0) {
          return 'Project name is required'
        }
        if (value.trim().length < 2) {
          return 'Project name must be at least 2 characters'
        }
        if (value.trim().length > 100) {
          return 'Project name must be less than 100 characters'
        }
        return null
      }
    }
  })

  const handleSubmit = async (values: { name: string; description: string }) => {
    try {
      const { data } = await createProject({
        variables: {
          input: {
            name: values.name.trim(),
            description: values.description.trim() || null
          }
        }
      })

      if (data?.createProject) {
        // Initialize empty Plan DAG for the new project
        try {
          await initializePlanDag({
            variables: {
              projectId: data.createProject.id,
              planDag: {
                version: '1.0.0',
                nodes: [],
                edges: [],
                metadata: {
                  version: '1.0.0',
                  name: `Plan DAG for ${data.createProject.name}`,
                  description: 'Auto-generated empty Plan DAG for new project',
                  created: new Date().toISOString(),
                  lastModified: new Date().toISOString(),
                  author: 'user'
                }
              }
            }
          })
          console.log('Empty Plan DAG initialized for new project:', data.createProject.id)
        } catch (planDagError) {
          console.warn('Failed to initialize Plan DAG for new project (non-critical):', planDagError)
          // Don't fail project creation if Plan DAG initialization fails
        }

        onSuccess(data.createProject)
        form.reset()
        onClose()
      }
    } catch (error) {
      console.error('Failed to create project:', error)
      // TODO: Show error notification
    }
  }

  const handleClose = () => {
    form.reset()
    onClose()
  }

  return (
    <Modal
      opened={opened}
      onClose={handleClose}
      title="Create New Project"
      size="md"
    >
      <form onSubmit={form.onSubmit(handleSubmit)}>
        <Stack gap="md">
          <TextInput
            label="Project Name"
            placeholder="Enter project name"
            required
            {...form.getInputProps('name')}
          />

          <Textarea
            label="Description"
            placeholder="Optional project description"
            rows={3}
            {...form.getInputProps('description')}
          />

          <Group justify="flex-end">
            <Button
              variant="subtle"
              onClick={handleClose}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              loading={loading}
            >
              Create Project
            </Button>
          </Group>
        </Stack>
      </form>
    </Modal>
  )
}
