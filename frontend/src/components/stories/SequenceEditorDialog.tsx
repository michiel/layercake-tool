import { useState, useEffect, useMemo } from 'react'
import { useMutation } from '@apollo/client/react'
import {
  IconGripVertical,
  IconTrash,
  IconArrowRight,
} from '@tabler/icons-react'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Spinner } from '@/components/ui/spinner'
import {
  CREATE_SEQUENCE,
  UPDATE_SEQUENCE,
  Sequence,
  SequenceEdgeRef,
} from '@/graphql/sequences'
import { DataSet } from '@/graphql/datasets'

interface GraphEdge {
  id: string
  source: string
  target: string
  comments?: string
  label?: string
  weight?: number
}

interface GraphNode {
  id: string
  label?: string
}

interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

interface SequenceEditorDialogProps {
  open: boolean
  onClose: () => void
  storyId: number
  sequence: Sequence | null
  storyDatasets: DataSet[]
}

export const SequenceEditorDialog = ({
  open,
  onClose,
  storyId,
  sequence,
  storyDatasets,
}: SequenceEditorDialogProps) => {
  const isEditing = !!sequence

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [enabledDatasetIds, setEnabledDatasetIds] = useState<number[]>([])
  const [edgeOrder, setEdgeOrder] = useState<SequenceEdgeRef[]>([])
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)

  // Parse graph data from datasets
  const datasetGraphData = useMemo(() => {
    const result: Record<number, GraphData> = {}
    for (const ds of storyDatasets) {
      try {
        const data = JSON.parse(ds.graphJson)
        result[ds.id] = {
          nodes: data.nodes || [],
          edges: data.edges || [],
        }
      } catch {
        result[ds.id] = { nodes: [], edges: [] }
      }
    }
    return result
  }, [storyDatasets])

  // Get all available edges from enabled datasets
  const availableEdges = useMemo(() => {
    const edges: Array<{ datasetId: number; datasetName: string; edge: GraphEdge }> = []
    for (const datasetId of enabledDatasetIds) {
      const dataset = storyDatasets.find((d) => d.id === datasetId)
      const graphData = datasetGraphData[datasetId]
      if (graphData && dataset) {
        for (const edge of graphData.edges) {
          edges.push({
            datasetId,
            datasetName: dataset.name,
            edge,
          })
        }
      }
    }
    return edges
  }, [enabledDatasetIds, datasetGraphData, storyDatasets])

  // Helper to get node label
  const getNodeLabel = (datasetId: number, nodeId: string): string => {
    const graphData = datasetGraphData[datasetId]
    const node = graphData?.nodes.find((n) => n.id === nodeId)
    return node?.label || nodeId
  }

  // Helper to get edge info
  const getEdgeInfo = (ref: SequenceEdgeRef) => {
    const graphData = datasetGraphData[ref.datasetId]
    const edge = graphData?.edges.find((e) => e.id === ref.edgeId)
    const dataset = storyDatasets.find((d) => d.id === ref.datasetId)
    return {
      edge,
      dataset,
      sourceLabel: edge ? getNodeLabel(ref.datasetId, edge.source) : 'Unknown',
      targetLabel: edge ? getNodeLabel(ref.datasetId, edge.target) : 'Unknown',
    }
  }

  const [createSequence, { loading: createLoading }] = useMutation(CREATE_SEQUENCE, {
    onCompleted: () => {
      onClose()
    },
    onError: (error) => {
      console.error('Failed to create sequence:', error)
      alert(`Failed to create sequence: ${error.message}`)
    },
  })

  const [updateSequence, { loading: updateLoading }] = useMutation(UPDATE_SEQUENCE, {
    onCompleted: () => {
      onClose()
    },
    onError: (error) => {
      console.error('Failed to update sequence:', error)
      alert(`Failed to update sequence: ${error.message}`)
    },
  })

  // Initialize form when dialog opens
  useEffect(() => {
    if (open) {
      if (sequence) {
        setName(sequence.name)
        setDescription(sequence.description || '')
        setEnabledDatasetIds(sequence.enabledDatasetIds)
        setEdgeOrder(sequence.edgeOrder)
      } else {
        setName('')
        setDescription('')
        setEnabledDatasetIds(storyDatasets.map((d) => d.id))
        setEdgeOrder([])
      }
    }
  }, [open, sequence, storyDatasets])

  const handleSave = async () => {
    if (!name.trim()) {
      alert('Please enter a sequence name')
      return
    }

    const input = {
      name: name.trim(),
      description: description.trim() || null,
      enabledDatasetIds,
      edgeOrder,
    }

    if (isEditing) {
      await updateSequence({
        variables: { id: sequence.id, input },
      })
    } else {
      await createSequence({
        variables: {
          input: {
            storyId,
            ...input,
          },
        },
      })
    }
  }

  const toggleDataset = (datasetId: number) => {
    setEnabledDatasetIds((prev) =>
      prev.includes(datasetId)
        ? prev.filter((id) => id !== datasetId)
        : [...prev, datasetId]
    )
    // Remove edges from disabled datasets
    if (enabledDatasetIds.includes(datasetId)) {
      setEdgeOrder((prev) => prev.filter((ref) => ref.datasetId !== datasetId))
    }
  }

  const addEdge = (datasetId: number, edgeId: string) => {
    // Check if already in list
    if (edgeOrder.some((ref) => ref.datasetId === datasetId && ref.edgeId === edgeId)) {
      return
    }
    setEdgeOrder((prev) => [...prev, { datasetId, edgeId }])
  }

  const removeEdge = (index: number) => {
    setEdgeOrder((prev) => prev.filter((_, i) => i !== index))
  }

  // Drag and drop handlers
  const handleDragStart = (index: number) => {
    setDraggedIndex(index)
  }

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault()
    if (draggedIndex === null || draggedIndex === index) return

    const newOrder = [...edgeOrder]
    const [removed] = newOrder.splice(draggedIndex, 1)
    newOrder.splice(index, 0, removed)
    setEdgeOrder(newOrder)
    setDraggedIndex(index)
  }

  const handleDragEnd = () => {
    setDraggedIndex(null)
  }

  const loading = createLoading || updateLoading

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <DialogContent className="sm:max-w-[800px] max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{isEditing ? `Edit Sequence: ${sequence.name}` : 'New Sequence'}</DialogTitle>
        </DialogHeader>

        <div className="flex-1 overflow-hidden py-4">
          <div className="grid grid-cols-[280px_1fr] gap-4 h-full">
            {/* Left panel: Settings and Dataset selection */}
            <div className="flex flex-col gap-4">
              <div className="space-y-2">
                <Label htmlFor="seq-name">Name</Label>
                <Input
                  id="seq-name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Sequence name"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="seq-description">Description</Label>
                <Textarea
                  id="seq-description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Optional description"
                  rows={2}
                />
              </div>

              <div className="space-y-2">
                <Label>Datasets</Label>
                <p className="text-xs text-muted-foreground">
                  Enable datasets to select edges from
                </p>
                <Stack gap="xs">
                  {storyDatasets.map((dataset) => {
                    const graphData = datasetGraphData[dataset.id]
                    const edgeCount = graphData?.edges.length || 0
                    return (
                      <div key={dataset.id} className="flex items-center space-x-2">
                        <Checkbox
                          id={`seq-ds-${dataset.id}`}
                          checked={enabledDatasetIds.includes(dataset.id)}
                          onCheckedChange={() => toggleDataset(dataset.id)}
                        />
                        <label
                          htmlFor={`seq-ds-${dataset.id}`}
                          className="text-sm cursor-pointer flex-1"
                        >
                          {dataset.name}
                          <span className="text-xs text-muted-foreground ml-1">
                            ({edgeCount} edges)
                          </span>
                        </label>
                      </div>
                    )
                  })}
                </Stack>
              </div>

              {/* Add edge selector */}
              <div className="space-y-2">
                <Label>Add Edge</Label>
                <Select
                  value=""
                  onValueChange={(value) => {
                    const [datasetId, edgeId] = value.split(':')
                    addEdge(Number(datasetId), edgeId)
                  }}
                  disabled={availableEdges.length === 0}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select an edge to add" />
                  </SelectTrigger>
                  <SelectContent>
                    {availableEdges.map(({ datasetId, datasetName, edge }) => {
                      const sourceLabel = getNodeLabel(datasetId, edge.source)
                      const targetLabel = getNodeLabel(datasetId, edge.target)
                      const alreadyAdded = edgeOrder.some(
                        (ref) => ref.datasetId === datasetId && ref.edgeId === edge.id
                      )
                      return (
                        <SelectItem
                          key={`${datasetId}:${edge.id}`}
                          value={`${datasetId}:${edge.id}`}
                          disabled={alreadyAdded}
                        >
                          <span className="text-xs text-muted-foreground mr-1">[{datasetName}]</span>
                          {sourceLabel} → {targetLabel}
                        </SelectItem>
                      )
                    })}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Right panel: Edge sequence list */}
            <div className="flex flex-col border rounded-md">
              <div className="p-2 border-b bg-muted/50">
                <Group justify="between" align="center">
                  <span className="text-sm font-medium">Edge Sequence</span>
                  <Badge variant="secondary">{edgeOrder.length} edges</Badge>
                </Group>
              </div>
              <ScrollArea className="flex-1 p-2">
                {edgeOrder.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                    <p className="text-sm">No edges in sequence</p>
                    <p className="text-xs">Add edges using the selector on the left</p>
                  </div>
                ) : (
                  <Stack gap="xs">
                    {edgeOrder.map((ref, index) => {
                      const { edge, dataset, sourceLabel, targetLabel } = getEdgeInfo(ref)
                      return (
                        <div
                          key={`${ref.datasetId}-${ref.edgeId}-${index}`}
                          className={`flex items-center gap-2 p-2 border rounded-md bg-background ${
                            draggedIndex === index ? 'opacity-50' : ''
                          }`}
                          draggable
                          onDragStart={() => handleDragStart(index)}
                          onDragOver={(e) => handleDragOver(e, index)}
                          onDragEnd={handleDragEnd}
                        >
                          <IconGripVertical className="h-4 w-4 text-muted-foreground cursor-grab" />
                          <div className="flex-1 grid grid-cols-[1fr_auto_1fr] gap-2 items-center min-w-0">
                            <div className="text-sm font-medium truncate" title={sourceLabel}>
                              {sourceLabel}
                            </div>
                            <div className="flex items-center gap-1 text-xs text-muted-foreground">
                              <IconArrowRight className="h-3 w-3" />
                              {edge?.comments && (
                                <span className="max-w-[100px] truncate" title={edge.comments}>
                                  {edge.comments}
                                </span>
                              )}
                            </div>
                            <div className="text-sm font-medium truncate text-right" title={targetLabel}>
                              {targetLabel}
                            </div>
                          </div>
                          <Badge variant="outline" className="text-xs shrink-0">
                            {dataset?.name}
                          </Badge>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 w-6 p-0 text-destructive hover:text-destructive/80"
                            onClick={() => removeEdge(index)}
                          >
                            <IconTrash className="h-3 w-3" />
                          </Button>
                        </div>
                      )
                    })}
                  </Stack>
                )}
              </ScrollArea>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={loading}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={loading || !name.trim()}>
            {loading && <Spinner className="mr-2 h-4 w-4" />}
            {isEditing ? 'Save Changes' : 'Create Sequence'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
