import { useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  IconPlus,
  IconTrash,
  IconEdit,
  IconListDetails,
  IconTimeline,
} from '@tabler/icons-react'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Spinner } from '@/components/ui/spinner'
import { SequenceDiagramDialog } from '@/components/stories/SequenceDiagramDialog'
import {
  LIST_SEQUENCES,
  DELETE_SEQUENCE,
  Sequence,
} from '@/graphql/sequences'

interface StorySequencesTabProps {
  storyId: number
  projectId: number
}

export const StorySequencesTab = ({
  storyId,
  projectId,
}: StorySequencesTabProps) => {
  const navigate = useNavigate()
  const { projectId: routeProjectId } = useParams<{ projectId: string }>()
  const effectiveProjectId = projectId || Number(routeProjectId || 0)
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [sequenceToDelete, setSequenceToDelete] = useState<Sequence | null>(null)
  const [diagramDialogOpen, setDiagramDialogOpen] = useState(false)
  const [diagramSequence, setDiagramSequence] = useState<Sequence | null>(null)

  const { data, loading, refetch } = useQuery(LIST_SEQUENCES, {
    variables: { storyId },
  })
  const sequences: Sequence[] = (data as any)?.sequences || []

  const [deleteSequence, { loading: deleteLoading }] = useMutation(DELETE_SEQUENCE, {
    onCompleted: () => {
      setDeleteModalOpen(false)
      setSequenceToDelete(null)
      refetch()
    },
    onError: (error) => {
      console.error('Failed to delete sequence:', error)
      alert(`Failed to delete sequence: ${error.message}`)
    },
  })

  const handleNewSequence = () => {
    navigate(`/projects/${effectiveProjectId}/stories/${storyId}/sequences/new`)
  }

  const handleEditSequence = (sequence: Sequence) => {
    navigate(`/projects/${effectiveProjectId}/stories/${storyId}/sequences/${sequence.id}`)
  }

  const handleOpenDiagram = (sequence: Sequence, e: React.MouseEvent) => {
    e.stopPropagation()
    setDiagramSequence(sequence)
    setDiagramDialogOpen(true)
  }

  const handleOpenDelete = (sequence: Sequence, e: React.MouseEvent) => {
    e.stopPropagation()
    setSequenceToDelete(sequence)
    setDeleteModalOpen(true)
  }

  const handleDeleteSequence = async () => {
    if (!sequenceToDelete) return

    await deleteSequence({
      variables: { id: sequenceToDelete.id },
    })
  }

  if (loading) {
    return (
      <Card className="border mt-4">
        <CardContent className="py-6">
          <Group gap="sm" align="center" justify="center">
            <Spinner className="h-4 w-4" />
            <span>Loading sequences...</span>
          </Group>
        </CardContent>
      </Card>
    )
  }

  return (
    <>
      <Card className="border mt-4">
        <CardHeader>
          <Group justify="between" align="center">
            <CardTitle className="text-base">Sequences</CardTitle>
            <Button size="sm" onClick={handleNewSequence}>
              <IconPlus className="mr-2 h-4 w-4" />
              New Sequence
            </Button>
          </Group>
        </CardHeader>
        <CardContent>
          {sequences.length === 0 ? (
            <div className="flex flex-col items-center gap-4 py-6">
              <IconListDetails size={48} className="text-muted-foreground" />
              <p className="text-center text-muted-foreground">
                No sequences yet. Create a sequence to start building a narrative from your graph edges.
              </p>
              <Button onClick={handleNewSequence}>
                <IconPlus className="mr-2 h-4 w-4" />
                Create First Sequence
              </Button>
            </div>
          ) : (
            <Stack gap="sm">
              {sequences.map((sequence) => (
                <div
                  key={sequence.id}
                  className="flex items-center justify-between p-3 border rounded-md hover:bg-muted/50 cursor-pointer"
                  onClick={() => handleEditSequence(sequence)}
                >
                  <div>
                    <Group gap="sm" className="mb-1">
                      <span className="font-medium">{sequence.name}</span>
                      <Badge variant="secondary">
                        {sequence.edgeCount} edge{sequence.edgeCount !== 1 ? 's' : ''}
                      </Badge>
                    </Group>
                    {sequence.description && (
                      <p className="text-sm text-muted-foreground">{sequence.description}</p>
                    )}
                    <p className="text-xs text-muted-foreground mt-1">
                      Updated: {new Date(sequence.updatedAt).toLocaleDateString()}
                    </p>
                  </div>
                  <Group gap="xs">
                    <Button
                      variant="ghost"
                      size="sm"
                      title="View sequence diagram"
                      onClick={(e) => handleOpenDiagram(sequence, e)}
                    >
                      <IconTimeline className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      title="Edit sequence"
                      onClick={(e) => {
                        e.stopPropagation()
                        handleEditSequence(sequence)
                      }}
                    >
                      <IconEdit className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-destructive hover:text-destructive/80"
                      title="Delete sequence"
                      onClick={(e) => handleOpenDelete(sequence, e)}
                    >
                      <IconTrash className="h-4 w-4" />
                    </Button>
                  </Group>
                </div>
              ))}
            </Stack>
          )}
        </CardContent>
      </Card>

      {/* Delete Confirmation Modal */}
      <Dialog open={deleteModalOpen} onOpenChange={setDeleteModalOpen}>
        <DialogContent className="sm:max-w-[400px]">
          <DialogHeader>
            <DialogTitle>Delete Sequence</DialogTitle>
          </DialogHeader>
          <p className="py-4">
            Are you sure you want to delete "{sequenceToDelete?.name}"?
          </p>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setDeleteModalOpen(false)} disabled={deleteLoading}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDeleteSequence} disabled={deleteLoading}>
              {deleteLoading && <Spinner className="mr-2 h-4 w-4" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Sequence Diagram Preview */}
      <SequenceDiagramDialog
        open={diagramDialogOpen}
        onClose={() => setDiagramDialogOpen(false)}
        sequence={diagramSequence}
        projectId={effectiveProjectId}
      />
    </>
  )
}
