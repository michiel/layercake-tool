import React from 'react'
import { useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Stack } from '@/components/layout-primitives'
import { showErrorNotification } from '@/utils/notifications'
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

// Zod schema for form validation
const createProjectSchema = z.object({
  name: z
    .string()
    .min(1, 'Project name is required')
    .min(2, 'Project name must be at least 2 characters')
    .max(100, 'Project name must be less than 100 characters')
    .transform((val) => val.trim()),
  description: z.string().transform((val) => val.trim()).optional(),
})

type CreateProjectFormValues = z.infer<typeof createProjectSchema>

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

  const form = useForm<CreateProjectFormValues>({
    resolver: zodResolver(createProjectSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const handleSubmit = async (values: CreateProjectFormValues) => {
    try {
      const { data } = await createProject({
        variables: {
          input: {
            name: values.name,
            description: values.description || null
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
      showErrorNotification(
        'Failed to create project',
        error instanceof Error ? error.message : 'An unexpected error occurred'
      )
    }
  }

  const handleClose = () => {
    form.reset()
    onClose()
  }

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Create New Project</DialogTitle>
          <DialogDescription>
            Create a new project to organize your work
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)}>
            <Stack gap="lg">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Project Name</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="Enter project name"
                        {...field}
                        disabled={loading}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="description"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Description</FormLabel>
                    <FormControl>
                      <Textarea
                        placeholder="Optional project description"
                        rows={3}
                        {...field}
                        disabled={loading}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleClose}
                  disabled={loading}
                >
                  Cancel
                </Button>
                <Button type="submit" disabled={loading}>
                  {loading ? 'Creating...' : 'Create Project'}
                </Button>
              </DialogFooter>
            </Stack>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
