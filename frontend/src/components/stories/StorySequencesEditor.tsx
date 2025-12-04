import type { DragEvent } from 'react'
import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  IconPlus,
  IconTrash,
  IconChevronDown,
  IconChevronRight,
  IconEye,
  IconAdjustments,
  IconX,
  IconGripVertical,
  IconArrowNarrowRight,
} from '@tabler/icons-react'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { Spinner } from '@/components/ui/spinner'
import { EdgeEditDialog } from '@/components/stories/EdgeEditDialog'
import {
  LIST_SEQUENCES,
  CREATE_SEQUENCE,
  UPDATE_SEQUENCE,
  DELETE_SEQUENCE,
  Sequence,
  SequenceEdgeRef,
} from '@/graphql/sequences'
import { GET_DATASOURCES, DataSet } from '@/graphql/datasets'
import { SequenceDiagramDialog } from '@/components/stories/SequenceDiagramDialog'
import { GET_PROJECT_LAYERS, ProjectLayer } from '@/graphql/layers'

type GraphEdge = { id: string; source: string; target: string; label?: string; comments?: string }
type GraphNode = { id: string; label?: string; name?: string; layer?: string }
type GraphData = { nodes: GraphNode[]; edges: GraphEdge[] }

interface StorySequencesEditorProps {
  storyId: number
  projectId: number
  enabledDatasetIds: number[]
}

export const StorySequencesEditor = ({
  storyId,
  projectId,
  enabledDatasetIds,
}: StorySequencesEditorProps) => {
  const [activeSequenceId, setActiveSequenceId] = useState<number | null>(null)
  const [expanded, setExpanded] = useState<Set<number>>(new Set())
  const [previewOpen, setPreviewOpen] = useState(false)
  const [edgeFilter, setEdgeFilter] = useState('')
  const [editingTitleId, setEditingTitleId] = useState<number | null>(null)
  const [titleDraft, setTitleDraft] = useState('')
  const [edgeEditorOpen, setEdgeEditorOpen] = useState(false)
  const [expandedDatasets, setExpandedDatasets] = useState<Set<number>>(new Set())
  // Tracks where a dragged edge would land for visual indicators.
  const [dragTarget, setDragTarget] = useState<{ sequenceId: number; index: number | null } | null>(null)
  const [edgeEditorPayload, setEdgeEditorPayload] = useState<{
    edge: GraphEdge | null
    datasetId: number
    note?: string
    notePosition?: SequenceEdgeRef['notePosition']
    graphJson: string
    sequenceId: number
    edgeIndex: number
  } | null>(null)

  const { data: sequencesData, loading: sequencesLoading, refetch: refetchSequences } = useQuery(LIST_SEQUENCES, {
    variables: { storyId },
  })
  const sequencesFromQuery: Sequence[] = (sequencesData as any)?.sequences || []
  const [optimisticSequences, setOptimisticSequences] = useState<Sequence[] | null>(null)
  const sequences = optimisticSequences ?? sequencesFromQuery

  const { data: datasetsData, loading: datasetsLoading } = useQuery(GET_DATASOURCES, {
    variables: { projectId },
    skip: !projectId,
  })
  const datasets: DataSet[] = (datasetsData as any)?.dataSets || []
  const { data: layersData } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId },
    skip: !projectId,
  })
  const projectLayers: ProjectLayer[] = (layersData as any)?.projectLayers || []

  const datasetGraphs = useMemo(() => {
    const map = new Map<number, { graph: GraphData; json: string; name: string }>()
    for (const ds of datasets) {
      try {
        const parsed = JSON.parse(ds.graphJson ?? '{}')
        map.set(ds.id, {
          graph: { nodes: parsed.nodes || [], edges: parsed.edges || [] },
          json: ds.graphJson ?? '{}',
          name: ds.name,
        })
      } catch (e) {
        console.error('Failed to parse dataset graphJson', e)
        map.set(ds.id, { graph: { nodes: [], edges: [] }, json: '{}', name: ds.name })
      }
    }
    return map
  }, [datasets])

  const getLayerColors = (layerId?: string): { bg: string; text: string } | null => {
    if (!layerId) return null
    const layer = projectLayers.find((l) => l.layerId === layerId && l.enabled)
    if (!layer) return null
    return { bg: layer.backgroundColor || '#e5e7eb', text: layer.textColor || '#000' }
  }

  const resolveNode = (datasetId: number, nodeId: string): { label: string; layer?: string } => {
    const ds = datasetGraphs.get(datasetId)
    const node = ds?.graph.nodes.find((n) => n.id === nodeId)
    const label = node?.label || node?.name || nodeId
    return { label, layer: node?.layer }
  }

  const edgeCatalog = useMemo(() => {
    const edges: Array<{
      datasetId: number
      datasetName: string
      edge: GraphEdge
      source: { label: string; layer?: string }
      target: { label: string; layer?: string }
    }> = []
    for (const ds of datasets) {
      if (!enabledDatasetIds.includes(ds.id)) continue
      const dsGraph = datasetGraphs.get(ds.id)
      if (!dsGraph) continue
      dsGraph.graph.edges.forEach((edge) => {
        edges.push({
          datasetId: ds.id,
          datasetName: ds.name,
          edge,
          source: resolveNode(ds.id, edge.source),
          target: resolveNode(ds.id, edge.target),
        })
      })
    }
    const filtered = edges.filter(({ datasetName, edge, source, target }) => {
      if (!edgeFilter.trim()) return true
      const q = edgeFilter.toLowerCase()
      return (
        datasetName.toLowerCase().includes(q) ||
        edge.id.toLowerCase().includes(q) ||
        (edge.label || '').toLowerCase().includes(q) ||
        source.label.toLowerCase().includes(q) ||
        target.label.toLowerCase().includes(q)
      )
    })
    return filtered
  }, [datasets, enabledDatasetIds, datasetGraphs, resolveNode, edgeFilter])

  const groupedEdges = useMemo(() => {
    const groups = new Map<number, { name: string; edges: typeof edgeCatalog }>()
    datasets.forEach((ds) => {
      if (!enabledDatasetIds.includes(ds.id)) return
      groups.set(ds.id, { name: ds.name, edges: [] as any })
    })
    edgeCatalog.forEach((item) => {
      const group = groups.get(item.datasetId)
      if (group) {
        group.edges.push(item)
      }
    })
    return Array.from(groups.entries()).map(([datasetId, group]) => ({
      datasetId,
      name: group.name,
      edges: group.edges,
    }))
  }, [edgeCatalog, datasets, enabledDatasetIds])

  useEffect(() => {
    if (expandedDatasets.size === 0 && groupedEdges.length > 0) {
      setExpandedDatasets(new Set(groupedEdges.map((g) => g.datasetId)))
    }
  }, [groupedEdges, expandedDatasets])

  const toggleDataset = (datasetId: number) => {
    const next = new Set(expandedDatasets)
    if (next.has(datasetId)) {
      next.delete(datasetId)
    } else {
      next.add(datasetId)
    }
    setExpandedDatasets(next)
  }

  const [createSequence] = useMutation(CREATE_SEQUENCE)
  const [updateSequence] = useMutation(UPDATE_SEQUENCE)
  const [deleteSequence] = useMutation(DELETE_SEQUENCE)

  useEffect(() => {
    // Replace optimistic view with server truth when data refreshes.
    setOptimisticSequences(null)
  }, [sequencesData])

  const handleAddSequence = async () => {
    const defaultName = `Sequence ${sequences.length + 1}`
    const result = await createSequence({
      variables: {
        input: {
          storyId,
          name: defaultName,
          enabledDatasetIds,
          edgeOrder: [],
        },
      },
    })
    await refetchSequences()
    const newId = (result.data as any)?.createSequence?.id
    if (newId) {
      setActiveSequenceId(newId)
      setExpanded(new Set([...expanded, newId]))
    }
  }

  const handleDeleteSequence = async (id: number) => {
    await deleteSequence({ variables: { id } })
    await refetchSequences()
    if (activeSequenceId === id) {
      const remaining = sequences.filter((s) => s.id !== id)
      setActiveSequenceId(remaining[0]?.id ?? null)
    }
  }

  const handleAppendEdge = async (edge: { datasetId: number; edgeId: string }) => {
    const targetSequenceId = activeSequenceId ?? sequences[0]?.id
    if (!targetSequenceId) return
    const seq = sequences.find((s) => s.id === targetSequenceId)
    if (!seq) return
    const nextEdgeOrder: SequenceEdgeRef[] = [...seq.edgeOrder, { datasetId: edge.datasetId, edgeId: edge.edgeId }]
    setOptimisticSequences(
      sequences.map((s) => (s.id === targetSequenceId ? { ...s, edgeOrder: nextEdgeOrder, edgeCount: nextEdgeOrder.length } : s))
    )
    await updateSequence({
      variables: { id: targetSequenceId, input: { edgeOrder: nextEdgeOrder } },
    })
    await refetchSequences()
    setActiveSequenceId(targetSequenceId)
  }

  const insertEdgeAt = async (sequenceId: number, index: number | null, edgeRef: SequenceEdgeRef) => {
    const seq = sequences.find((s) => s.id === sequenceId)
    if (!seq) return
    const next = [...seq.edgeOrder]
    const insertAt = index === null ? next.length : Math.max(0, Math.min(index, next.length))
    next.splice(insertAt, 0, edgeRef)
    setOptimisticSequences(
      sequences.map((s) => (s.id === sequenceId ? { ...s, edgeOrder: next, edgeCount: next.length } : s))
    )
    await updateSequence({ variables: { id: sequenceId, input: { edgeOrder: next } } })
    await refetchSequences()
    setActiveSequenceId(sequenceId)
  }

  const handleToggleExpand = (id: number) => {
    setExpanded(new Set([id]))
    setActiveSequenceId(id)
  }

  const activeSequence = sequences.find((s) => s.id === activeSequenceId) || sequences[0] || null
  const startEditTitle = (sequence: Sequence) => {
    setEditingTitleId(sequence.id)
    setTitleDraft(sequence.name)
    setActiveSequenceId(sequence.id)
    setExpanded(new Set([sequence.id]))
  }

  const commitTitle = async (sequenceId: number) => {
    const trimmed = titleDraft.trim()
    if (!trimmed) return
    await updateSequence({ variables: { id: sequenceId, input: { name: trimmed } } })
    setEditingTitleId(null)
    refetchSequences()
  }

  const handleRemoveEdge = async (sequenceId: number, index: number) => {
    const seq = sequences.find((s) => s.id === sequenceId)
    if (!seq) return
    const nextEdgeOrder = seq.edgeOrder.filter((_, idx) => idx !== index)
    await updateSequence({ variables: { id: sequenceId, input: { edgeOrder: nextEdgeOrder } } })
    refetchSequences()
  }

  const openEdgeEditor = (sequenceId: number, edgeRef: SequenceEdgeRef, index: number) => {
    const ds = datasetGraphs.get(edgeRef.datasetId)
    const edge = ds?.graph.edges.find((e) => e.id === edgeRef.edgeId) || null
    setEdgeEditorPayload({
      edge,
      datasetId: edgeRef.datasetId,
      note: edgeRef.note,
      notePosition: edgeRef.notePosition,
      graphJson: ds?.json || '{}',
      sequenceId,
      edgeIndex: index,
    })
    setEdgeEditorOpen(true)
  }

  const handleEdgeEditSave = async (updates: {
    note?: string
    notePosition?: SequenceEdgeRef['notePosition']
  }) => {
    if (!edgeEditorPayload) return
    const seq = sequences.find((s) => s.id === edgeEditorPayload.sequenceId)
    if (!seq) return
    const nextEdgeOrder = seq.edgeOrder.map((ref, idx) =>
      idx === edgeEditorPayload.edgeIndex
        ? { ...ref, note: updates.note ?? ref.note, notePosition: updates.notePosition ?? ref.notePosition }
        : ref
    )
    await updateSequence({ variables: { id: seq.id, input: { edgeOrder: nextEdgeOrder } } })
    refetchSequences()
  }

  useEffect(() => {
    if (!activeSequenceId && sequences.length > 0) {
      const first = sequences[0].id
      setActiveSequenceId(first)
      setExpanded(new Set([first]))
    }
  }, [sequences, activeSequenceId])

  const moveEdge = async (fromSeqId: number, toSeqId: number, fromIndex: number, toIndex: number | null) => {
    const fromSeq = sequences.find((s) => s.id === fromSeqId)
    const toSeq = sequences.find((s) => s.id === toSeqId)
    if (!fromSeq || !toSeq) return
    const fromOrder = [...fromSeq.edgeOrder]
    const [edgeRef] = fromOrder.splice(fromIndex, 1)
    if (!edgeRef) return

    const targetOrder = fromSeqId === toSeqId ? fromOrder : [...toSeq.edgeOrder]
    const insertAt = toIndex === null ? targetOrder.length : Math.max(0, Math.min(toIndex, targetOrder.length))
    targetOrder.splice(insertAt, 0, edgeRef)

    // Optimistic UI update.
    setOptimisticSequences(
      sequences.map((s) => {
        if (s.id === fromSeqId && fromSeqId === toSeqId) {
          return { ...s, edgeOrder: targetOrder, edgeCount: targetOrder.length }
        }
        if (s.id === fromSeqId) {
          return { ...s, edgeOrder: fromOrder, edgeCount: fromOrder.length }
        }
        if (s.id === toSeqId) {
          return { ...s, edgeOrder: targetOrder, edgeCount: targetOrder.length }
        }
        return s
      })
    )

    if (fromSeqId !== toSeqId) {
      await updateSequence({ variables: { id: fromSeqId, input: { edgeOrder: fromOrder } } })
    }
    await updateSequence({ variables: { id: toSeqId, input: { edgeOrder: targetOrder } } })
    await refetchSequences()
  }

  const handleDragStart = (e: DragEvent, sequenceId: number, index: number) => {
    e.dataTransfer.setData('application/json', JSON.stringify({ kind: 'sequence', sequenceId, index }))
    e.dataTransfer.effectAllowed = 'move'
  }

  const handleAvailableDragStart = (
    e: DragEvent,
    edge: { datasetId: number; edgeId: string; source: string; target: string; label?: string }
  ) => {
    e.dataTransfer.setData('application/json', JSON.stringify({ kind: 'available', edge }))
    e.dataTransfer.effectAllowed = 'copy'
  }

  const handleDragEnter = (sequenceId: number, index: number | null) => {
    setDragTarget({ sequenceId, index })
  }

  const handleDrop = async (e: DragEvent, targetSequenceId: number, targetIndex: number | null) => {
    e.preventDefault()
    e.stopPropagation()
    const data = e.dataTransfer.getData('application/json')
    if (!data) return
    try {
      const parsed = JSON.parse(data) as
        | { kind: 'sequence'; sequenceId: number; index: number }
        | { kind: 'available'; edge: { datasetId: number; edgeId: string } }

      if (parsed.kind === 'sequence') {
        await moveEdge(parsed.sequenceId, targetSequenceId, parsed.index, targetIndex)
      } else if (parsed.kind === 'available') {
        await insertEdgeAt(targetSequenceId, targetIndex, {
          datasetId: parsed.edge.datasetId,
          edgeId: parsed.edge.edgeId,
        })
      }
      setActiveSequenceId(targetSequenceId)
      setExpanded(new Set([targetSequenceId]))
      setDragTarget(null)
    } catch (err) {
      console.error('Failed to parse drag data', err)
    }
  }

  return (
    <>
      <Card className="border mt-4">
        <CardHeader>
          <Group justify="between" align="center">
            <CardTitle className="text-base">Sequences</CardTitle>
            <Button size="sm" onClick={handleAddSequence} disabled={sequencesLoading || datasetsLoading}>
              <IconPlus className="mr-2 h-4 w-4" />
              Add Section
            </Button>
          </Group>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-3">
          <div className="space-y-3 h-full lg:max-w-[30vw]">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">Available edges</h3>
              <Badge variant="outline">{edgeCatalog.length}</Badge>
            </div>
            <Input
              placeholder="Filter by dataset or edge id/label"
              value={edgeFilter}
              onChange={(e) => setEdgeFilter(e.target.value)}
              className="h-8"
            />
            <ScrollArea className="h-[520px] border rounded-md">
              <div className="p-2 space-y-3">
                {edgeCatalog.length === 0 ? (
                  <p className="text-xs text-muted-foreground px-2 py-1">No edges available from enabled datasets.</p>
                ) : (
                  groupedEdges.map(({ datasetId, name, edges }) => (
                    <div key={datasetId} className="border rounded-md">
                      <button
                        className="w-full flex items-center justify-between px-2 py-2 text-left"
                        onClick={() => toggleDataset(datasetId)}
                      >
                        <Group gap="sm" align="center">
                          {expandedDatasets.has(datasetId) ? (
                            <IconChevronDown className="h-4 w-4" />
                          ) : (
                            <IconChevronRight className="h-4 w-4" />
                          )}
                          <span className="font-medium text-sm truncate max-w-[160px]">{name}</span>
                        </Group>
                        <Badge variant="secondary">{edges.length}</Badge>
                      </button>
                      {expandedDatasets.has(datasetId) && (
                        <div className="border-t px-2 py-2 space-y-2">
                          {edges.length === 0 ? (
                            <p className="text-[11px] text-muted-foreground px-1">No edges in this dataset.</p>
                          ) : (
                            edges.map(({ edge, source, target }) => {
                              const sourceColors = getLayerColors(source.layer)
                              const targetColors = getLayerColors(target.layer)
                              return (
                                <div
                                  key={`${datasetId}-${edge.id}`}
                                  className="flex items-center gap-2 text-xs px-2 py-2 rounded border hover:bg-muted lg:max-w-[42vw]"
                                >
                                  <button
                                    className="p-1 rounded hover:bg-background"
                                    draggable
                                    onDragStart={(e) =>
                                      handleAvailableDragStart(e, {
                                        datasetId,
                                        edgeId: edge.id,
                                        source: edge.source,
                                        target: edge.target,
                                        label: edge.label,
                                      })
                                    }
                                    onDragEnd={() => setDragTarget(null)}
                                    title="Drag to insert into a sequence"
                                  >
                                    <IconGripVertical className="h-4 w-4 text-muted-foreground" />
                                  </button>
                                  <button
                                    className="flex-1 text-left"
                                    onClick={() => handleAppendEdge({ datasetId, edgeId: edge.id })}
                                    title="Add to active sequence"
                                  >
                                    <div className="grid grid-cols-[1fr_auto_auto_auto_1fr] items-center gap-1 lg:max-w-[38vw]">
                                      <span
                                        className="px-2 py-0.5 rounded text-xs truncate w-[150px]"
                                        style={{
                                          backgroundColor: sourceColors?.bg || '#e5e7eb',
                                          color: sourceColors?.text || '#000',
                                        }}
                                        title={source.label}
                                      >
                                        {source.label}
                                      </span>
                                      <IconArrowNarrowRight className="h-3 w-3 text-muted-foreground" />
                                      <span className="text-[11px] text-muted-foreground text-center px-1 truncate w-[140px]">
                                        {edge.label || 'edge'}
                                      </span>
                                      <IconArrowNarrowRight className="h-3 w-3 text-muted-foreground" />
                                      <span
                                        className="px-2 py-0.5 rounded text-xs truncate w-[150px]"
                                        style={{
                                          backgroundColor: targetColors?.bg || '#e5e7eb',
                                          color: targetColors?.text || '#000',
                                        }}
                                        title={target.label}
                                      >
                                        {target.label}
                                      </span>
                                    </div>
                                  </button>
                                  <Button
                                    variant="ghost"
                                    size="icon"
                                    className="text-muted-foreground"
                                    onClick={() => handleAppendEdge({ datasetId, edgeId: edge.id })}
                                    title="Add to active sequence"
                                  >
                                    <IconPlus className="h-4 w-4" />
                                  </Button>
                                </div>
                              )
                            })
                          )}
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>
            </ScrollArea>
          </div>
          <div className="md:col-span-2 space-y-2">
            {sequencesLoading ? (
              <Group gap="sm" className="text-sm text-muted-foreground">
                <Spinner className="h-4 w-4" />
                Loading sequences...
              </Group>
            ) : sequences.length === 0 ? (
              <div className="flex flex-col items-start gap-3 text-sm text-muted-foreground">
                <p>No sequences yet. Add a section to start.</p>
              </div>
            ) : (
              <Stack gap="sm">
                {sequences.map((sequence) => {
                  const isActive = activeSequenceId
                    ? activeSequenceId === sequence.id
                    : sequences[0]?.id === sequence.id
                  const isExpanded = expanded.has(sequence.id) || isActive
                  return (
                    <div
                      key={sequence.id}
                      className={cn(
                        'border rounded-md',
                        isActive && 'border-primary shadow-sm'
                      )}
                    >
                      <button
                        className="w-full flex items-center justify-between px-3 py-2 text-left"
                        onClick={() => handleToggleExpand(sequence.id)}
                      >
                        <Group gap="sm" align="center">
                          {isExpanded ? <IconChevronDown className="h-4 w-4" /> : <IconChevronRight className="h-4 w-4" />}
                          {editingTitleId === sequence.id ? (
                            <Input
                              value={titleDraft}
                              onChange={(e) => setTitleDraft(e.target.value)}
                              onClick={(e) => e.stopPropagation()}
                              onBlur={() => commitTitle(sequence.id)}
                              onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                  e.preventDefault()
                                  commitTitle(sequence.id)
                                }
                              }}
                              autoFocus
                              className="h-8 w-56"
                            />
                          ) : (
                            <span
                              className="font-medium hover:underline"
                              onClick={(e) => {
                                e.stopPropagation()
                                startEditTitle(sequence)
                              }}
                            >
                              {sequence.name}
                            </span>
                          )}
                          <Badge variant="secondary">
                            {sequence.edgeCount} edge{sequence.edgeCount !== 1 ? 's' : ''}
                          </Badge>
                        </Group>
                        <Group gap="xs">
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={(e) => {
                              e.stopPropagation()
                              setActiveSequenceId(sequence.id)
                              setPreviewOpen(true)
                            }}
                            title="Preview"
                          >
                            <IconEye className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-destructive hover:text-destructive/80"
                            onClick={(e) => {
                              e.stopPropagation()
                              handleDeleteSequence(sequence.id)
                            }}
                            title="Delete sequence"
                          >
                            <IconTrash className="h-4 w-4" />
                          </Button>
                        </Group>
                      </button>
                      {isExpanded && (
                        <div className="px-4 pb-3 space-y-3">
                          {sequence.edgeOrder.length === 0 ? (
                            <p className="text-xs text-muted-foreground">No edges yet. Click an edge to add it.</p>
                          ) : (
                            <Stack
                              gap="xs"
                              onDragOver={(e) => e.preventDefault()}
                              onDrop={(e) => handleDrop(e, sequence.id, dragTarget?.index ?? null)}
                              onDragEnter={() => handleDragEnter(sequence.id, null)}
                            >
                              {sequence.edgeOrder.map((edge, idx) => {
                                const dsGraph = datasetGraphs.get(edge.datasetId)
                                const edgeData = dsGraph?.graph.edges.find((e) => e.id === edge.edgeId)
                                const source = resolveNode(edge.datasetId, edgeData?.source || '')
                                const target = resolveNode(edge.datasetId, edgeData?.target || '')
                                const sourceColors = getLayerColors(source.layer)
                                const targetColors = getLayerColors(target.layer)
                                const isDropHere =
                                  dragTarget?.sequenceId === sequence.id && dragTarget?.index === idx
                                const meta = [edgeData?.comments, edge.note].filter(Boolean).join(' • ')
                                return (
                                  <div key={`${edge.datasetId}-${edge.edgeId}-${idx}`} className="space-y-1">
                                    <div
                                      className={cn(
                                        'h-3 rounded border border-dashed border-transparent transition-colors',
                                        isDropHere && 'border-primary/60 bg-primary/5'
                                      )}
                                      onDragEnter={() => handleDragEnter(sequence.id, idx)}
                                      onDragOver={(e) => e.preventDefault()}
                                      onDrop={(e) => handleDrop(e, sequence.id, idx)}
                                    />
                                    <div
                                      className={cn(
                                        'flex items-center justify-between text-xs px-2 py-1 rounded bg-muted lg:max-w-[50vw] transition-colors',
                                        isActive && 'border border-primary/40'
                                      )}
                                      onClick={() => setActiveSequenceId(sequence.id)}
                                      draggable
                                      onDragStart={(e) => handleDragStart(e, sequence.id, idx)}
                                      onDragOver={(e) => e.preventDefault()}
                                      onDrop={(e) => handleDrop(e, sequence.id, idx)}
                                      onDragEnter={() => handleDragEnter(sequence.id, idx)}
                                      onDragEnd={() => setDragTarget(null)}
                                    >
                                      <Group gap="xs" className="mr-2">
                                        <IconGripVertical className="h-4 w-4 text-muted-foreground" />
                                      </Group>
                                      <div className="flex items-center gap-3 w-full">
                                        <div className="flex-[0_0_60%] min-w-0">
                                          <div className="grid grid-cols-[1fr_auto_auto_auto_1fr_auto] items-center gap-2">
                                            <span
                                              className="px-2 py-0.5 rounded text-xs truncate w-[150px]"
                                              style={{
                                                backgroundColor: sourceColors?.bg || '#e5e7eb',
                                                color: sourceColors?.text || '#000',
                                              }}
                                              title={source.label || edgeData?.source || 'Source'}
                                            >
                                              {source.label || edgeData?.source || 'Source'}
                                            </span>
                                            <IconArrowNarrowRight className="h-3 w-3 text-muted-foreground" />
                                            <span className="text-[11px] text-muted-foreground text-center px-1 truncate w-[140px]">
                                              {edgeData?.label || 'edge'}
                                            </span>
                                            <IconArrowNarrowRight className="h-3 w-3 text-muted-foreground" />
                                            <span
                                              className="px-2 py-0.5 rounded text-xs truncate w-[150px]"
                                              style={{
                                                backgroundColor: targetColors?.bg || '#e5e7eb',
                                                color: targetColors?.text || '#000',
                                              }}
                                              title={target.label || edgeData?.target || 'Target'}
                                            >
                                              {target.label || edgeData?.target || 'Target'}
                                            </span>
                                            <span className="text-[11px] text-muted-foreground truncate max-w-[180px]">
                                              {meta || '\u00a0'}
                                            </span>
                                          </div>
                                        </div>
                                        <Group gap="xs" className="ml-auto w-[40%] justify-end flex-shrink-0">
                                          <Button
                                            variant="ghost"
                                            size="icon"
                                            className="text-muted-foreground"
                                            onClick={(e) => {
                                              e.stopPropagation()
                                              openEdgeEditor(sequence.id, edge, idx)
                                            }}
                                            title="Edit edge"
                                          >
                                            <IconAdjustments className="h-4 w-4" />
                                          </Button>
                                          <Button
                                            variant="ghost"
                                            size="icon"
                                            className="text-destructive hover:text-destructive/80"
                                            onClick={(e) => {
                                              e.stopPropagation()
                                              handleRemoveEdge(sequence.id, idx)
                                            }}
                                            title="Remove edge from sequence"
                                          >
                                            <IconX className="h-4 w-4" />
                                          </Button>
                                        </Group>
                                      </div>
                                    </div>
                                  </div>
                                )
                              })}
                              <div
                                className={cn(
                                  'h-3 rounded border border-dashed border-transparent transition-colors',
                                  dragTarget?.sequenceId === sequence.id &&
                                    dragTarget?.index === null &&
                                    'border-primary/60 bg-primary/5'
                                )}
                                onDragEnter={() => handleDragEnter(sequence.id, null)}
                                onDragOver={(e) => e.preventDefault()}
                                onDrop={(e) => handleDrop(e, sequence.id, null)}
                              />
                            </Stack>
                          )}
                        </div>
                      )}
                    </div>
                  )
                })}
              </Stack>
            )}
          </div>
        </CardContent>
      </Card>

      <SequenceDiagramDialog
        open={previewOpen && !!activeSequence}
        onClose={() => setPreviewOpen(false)}
        sequence={activeSequence}
        projectId={projectId}
      />
      <EdgeEditDialog
        open={edgeEditorOpen}
        onClose={() => setEdgeEditorOpen(false)}
        edge={edgeEditorPayload?.edge || null}
        datasetId={edgeEditorPayload?.datasetId || 0}
        graphJson={edgeEditorPayload?.graphJson || '{}'}
        note={edgeEditorPayload?.note}
        notePosition={edgeEditorPayload?.notePosition}
        onSave={handleEdgeEditSave}
      />
    </>
  )
}
