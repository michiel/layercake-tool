import { useState, useEffect } from 'react'
import { useMutation } from '@apollo/client/react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { Separator } from '@/components/ui/separator'
import { UPDATE_DATASOURCE_GRAPH_DATA } from '@/graphql/datasets'
import { NotePosition } from '@/graphql/sequences'

interface GraphEdge {
  id: string
  source: string
  target: string
  label?: string
  comments?: string
  weight?: number
}

interface EdgeEditDialogProps {
  open: boolean
  onClose: () => void
  edge: GraphEdge | null
  datasetId: number
  graphJson: string
  note?: string
  notePosition?: NotePosition
  onSave: (updates: {
    edgeLabel?: string
    edgeComment?: string
    note?: string
    notePosition?: NotePosition
  }) => void
  onGraphUpdate?: () => void
}

export const EdgeEditDialog = ({
  open,
  onClose,
  edge,
  datasetId,
  graphJson,
  note: initialNote,
  notePosition: initialNotePosition,
  onSave,
  onGraphUpdate,
}: EdgeEditDialogProps) => {
  const [edgeLabel, setEdgeLabel] = useState('')
  const [edgeComment, setEdgeComment] = useState('')
  const [note, setNote] = useState('')
  const [notePosition, setNotePosition] = useState<NotePosition>('Both')

  const [updateGraphData, { loading: updateLoading }] = useMutation(
    UPDATE_DATASOURCE_GRAPH_DATA,
    {
      onCompleted: () => {
        onGraphUpdate?.()
      },
      onError: (error) => {
        console.error('Failed to update edge in dataset:', error)
        alert(`Failed to update edge: ${error.message}`)
      },
    }
  )

  // Initialize form when dialog opens
  useEffect(() => {
    if (open && edge) {
      setEdgeLabel(edge.label || '')
      setEdgeComment(edge.comments || '')
      setNote(initialNote || '')
      setNotePosition(initialNotePosition || 'Both')
    }
  }, [open, edge, initialNote, initialNotePosition])

  const handleSave = async () => {
    if (!edge) return

    // Check if edge data changed
    const edgeLabelChanged = edgeLabel !== (edge.label || '')
    const edgeCommentChanged = edgeComment !== (edge.comments || '')

    // Update dataset graph if edge data changed
    if (edgeLabelChanged || edgeCommentChanged) {
      try {
        const graphData = JSON.parse(graphJson)
        const edges = graphData.edges || graphData.links || []
        const edgeIndex = edges.findIndex((e: GraphEdge) => e.id === edge.id)

        if (edgeIndex !== -1) {
          edges[edgeIndex] = {
            ...edges[edgeIndex],
            label: edgeLabel || undefined,
            comments: edgeComment || undefined,
          }

          // Update the graph data
          if (graphData.links) {
            graphData.links = edges
          } else {
            graphData.edges = edges
          }

          await updateGraphData({
            variables: {
              id: datasetId,
              graphJson: JSON.stringify(graphData),
            },
          })
        }
      } catch (e) {
        console.error('Failed to parse/update graph JSON:', e)
        alert('Failed to update edge data')
        return
      }
    }

    // Save sequence-level note data
    onSave({
      edgeLabel: edgeLabelChanged ? edgeLabel : undefined,
      edgeComment: edgeCommentChanged ? edgeComment : undefined,
      note: note || undefined,
      notePosition: notePosition || undefined,
    })

    onClose()
  }

  if (!edge) return null

  return (
    <Dialog open={open} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Edit Edge</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Edge Properties (saved to dataset) */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium text-muted-foreground">
              Edge Properties
              <span className="ml-2 text-xs font-normal">(saved to dataset)</span>
            </h4>
            <div className="space-y-2">
              <Label htmlFor="edge-label">Label</Label>
              <Input
                id="edge-label"
                value={edgeLabel}
                onChange={(e) => setEdgeLabel(e.target.value)}
                placeholder="Edge label"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edge-comment">Comment</Label>
              <Input
                id="edge-comment"
                value={edgeComment}
                onChange={(e) => setEdgeComment(e.target.value)}
                placeholder="Edge comment"
              />
            </div>
          </div>

          <Separator />

          {/* Sequence Note (saved to sequence item) */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium text-muted-foreground">
              Sequence Note
              <span className="ml-2 text-xs font-normal">(saved to this sequence item)</span>
            </h4>
            <div className="space-y-2">
              <Label htmlFor="note-text">Note</Label>
              <Input
                id="note-text"
                value={note}
                onChange={(e) => setNote(e.target.value)}
                placeholder="Add a note for this step"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="note-position">Display Position</Label>
              <Select
                value={notePosition}
                onValueChange={(value) => setNotePosition(value as NotePosition)}
              >
                <SelectTrigger id="note-position">
                  <SelectValue placeholder="Select position" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Source">Over Source</SelectItem>
                  <SelectItem value="Target">Over Target</SelectItem>
                  <SelectItem value="Both">Over Both</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={updateLoading}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={updateLoading}>
            {updateLoading && <Spinner className="mr-2 h-4 w-4" />}
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default EdgeEditDialog
