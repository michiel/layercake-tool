import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'

import { useRegisterChatContext } from '../hooks/useRegisterChatContext'
import { Button } from '../components/ui/button'
import { Input } from '../components/ui/input'
import { Textarea } from '../components/ui/textarea'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import PageContainer from '../components/layout/PageContainer'

const GET_PROJECT = gql`
  query GetProjectForEdit($id: Int!) {
    project(id: $id) {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`

const UPDATE_PROJECT = gql`
  mutation UpdateProject($id: Int!, $input: UpdateProjectInput!) {
    updateProject(id: $id, input: $input) {
      id
      name
      description
      updatedAt
    }
  }
`

export const EditProjectPage = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN
  const navigate = useNavigate()

  const { data, loading, error } = useQuery(GET_PROJECT, {
    variables: { id: numericProjectId },
    skip: !Number.isFinite(numericProjectId),
  })

  const [updateProject, { loading: saving }] = useMutation(UPDATE_PROJECT, {
    refetchQueries: ['GetProjectForEdit', 'GetProjects'],
  })

  const project = useMemo(() => (data as any)?.project ?? null, [data])

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [status, setStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [statusMessage, setStatusMessage] = useState<string | null>(null)

  useEffect(() => {
    if (project) {
      setName(project.name ?? '')
      setDescription(project.description ?? '')
    }
  }, [project])

  useRegisterChatContext(
    project
      ? `Editing project details for ${project.name} (#${project.id})`
      : 'Editing project details',
    project?.id,
  )

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    if (!project) return

    try {
      await updateProject({
        variables: {
          id: project.id,
          input: {
            name: name.trim(),
            description: description.trim() || null,
          },
        },
      })
      setStatus('success')
      setStatusMessage('Project details updated successfully.')
    } catch (err: any) {
      setStatus('error')
      setStatusMessage(err.message ?? 'Failed to update project.')
    }
  }

  if (!Number.isFinite(numericProjectId)) {
    return (
      <PageContainer>
        <Alert variant="destructive">
          <AlertTitle>Invalid project</AlertTitle>
          <AlertDescription>Project ID is missing or invalid.</AlertDescription>
        </Alert>
      </PageContainer>
    )
  }

  if (loading) {
    return (
      <PageContainer>
        <p>Loading project...</p>
      </PageContainer>
    )
  }

  if (error || !project) {
    return (
      <PageContainer>
        <Alert variant="destructive">
          <AlertTitle>Project unavailable</AlertTitle>
          <AlertDescription>{error?.message ?? 'Project not found.'}</AlertDescription>
        </Alert>
        <Button className="mt-4" onClick={() => navigate(`/projects/${projectId}`)}>
          Back to Project
        </Button>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={project.name}
        projectId={project.id}
        currentPage="Edit Details"
        onNavigate={(path) => navigate(path)}
      />

      <Card className="mt-6 max-w-3xl">
        <CardHeader>
          <CardTitle>Edit Project Details</CardTitle>
        </CardHeader>
        <CardContent>
          {status !== 'idle' && statusMessage && (
            <Alert variant={status === 'success' ? 'default' : 'destructive'} className="mb-4">
              <AlertTitle>{status === 'success' ? 'Success' : 'Error'}</AlertTitle>
              <AlertDescription>{statusMessage}</AlertDescription>
            </Alert>
          )}

          <form className="space-y-4" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <label className="text-sm font-medium">Name</label>
              <Input
                value={name}
                onChange={(event) => setName(event.target.value)}
                required
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Description</label>
              <Textarea
                rows={6}
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder="Describe the goal of this project, key datasets, or any context collaborators should know."
              />
            </div>

            <div className="flex items-center justify-between pt-4">
              <Button type="button" variant="outline" onClick={() => navigate(-1)}>
                Cancel
              </Button>
              <Button type="submit" disabled={saving}>
                {saving ? 'Saving…' : 'Save Changes'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </PageContainer>
  )
}
