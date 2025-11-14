import React, { useEffect, useState } from 'react'
import { useMutation, useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Textarea } from '../ui/textarea'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '../ui/dialog'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Stack } from '../layout-primitives'

const GET_PROJECT = gql`
  query GetProjectForEdit($id: Int!) {
    project(id: $id) {
      id
      name
      description
      tags
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
      tags
      updatedAt
    }
  }
`

interface EditProjectDialogProps {
  projectId: number | null
  open: boolean
  onClose: () => void
}

export const EditProjectDialog: React.FC<EditProjectDialogProps> = ({
  projectId,
  open,
  onClose,
}) => {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [tags, setTags] = useState('')
  const [status, setStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [statusMessage, setStatusMessage] = useState<string | null>(null)

  const { data, loading } = useQuery(GET_PROJECT, {
    variables: { id: projectId },
    skip: !projectId || !open,
  })

  const [updateProject, { loading: saving }] = useMutation(UPDATE_PROJECT, {
    refetchQueries: ['GetProjects'],
    onCompleted: () => {
      setStatus('success')
      setStatusMessage('Project details updated successfully.')
      setTimeout(() => {
        onClose()
      }, 1000)
    },
    onError: (err: any) => {
      setStatus('error')
      setStatusMessage(err.message ?? 'Failed to update project.')
    },
  })

  const project = (data as any)?.project ?? null

  useEffect(() => {
    if (project) {
      setName(project.name ?? '')
      setDescription(project.description ?? '')
      setTags((project.tags ?? []).join(', '))
    }
  }, [project])

  useEffect(() => {
    if (!open) {
      // Reset status when dialog closes
      setStatus('idle')
      setStatusMessage(null)
    }
  }, [open])

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    if (!project) return

    try {
      const tagsArray = tags
        .split(',')
        .map((t) => t.trim())
        .filter((t) => t.length > 0)

      await updateProject({
        variables: {
          id: project.id,
          input: {
            name: name.trim(),
            description: description.trim() || null,
            tags: tagsArray.length > 0 ? tagsArray : [],
          },
        },
      })
    } catch (err: any) {
      // Error handled by mutation's onError
    }
  }

  if (!projectId) return null

  return (
    <Dialog open={open} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Edit Project Details</DialogTitle>
        </DialogHeader>

        {loading ? (
          <div className="py-8 text-center">
            <p>Loading project...</p>
          </div>
        ) : !project ? (
          <div className="py-8">
            <Alert variant="destructive">
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>Project not found.</AlertDescription>
            </Alert>
          </div>
        ) : (
          <form onSubmit={handleSubmit}>
            <Stack gap="md" className="py-4">
              {status !== 'idle' && statusMessage && (
                <Alert variant={status === 'success' ? 'default' : 'destructive'}>
                  <AlertTitle>{status === 'success' ? 'Success' : 'Error'}</AlertTitle>
                  <AlertDescription>{statusMessage}</AlertDescription>
                </Alert>
              )}

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

              <div className="space-y-2">
                <label className="text-sm font-medium">Tags</label>
                <Input
                  value={tags}
                  onChange={(event) => setTags(event.target.value)}
                  placeholder="e.g., client-work, analysis, prototype (comma-separated)"
                />
                <p className="text-xs text-muted-foreground">
                  Separate multiple tags with commas. Tags help filter and organize projects.
                </p>
              </div>
            </Stack>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={onClose} disabled={saving}>
                Cancel
              </Button>
              <Button type="submit" disabled={saving}>
                {saving ? 'Saving…' : 'Save Changes'}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  )
}
