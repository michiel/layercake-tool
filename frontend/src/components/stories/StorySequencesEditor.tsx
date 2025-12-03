import { useMemo, useState } from 'react'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  IconPlus,
  IconTrash,
  IconChevronDown,
  IconChevronRight,
  IconEye,
} from '@tabler/icons-react'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { Spinner } from '@/components/ui/spinner'
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

type GraphEdge = { id: string; source: string; target: string; label?: string }

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

  const { data: sequencesData, loading: sequencesLoading, refetch: refetchSequences } = useQuery(LIST_SEQUENCES, {
    variables: { storyId },
  })
  const sequences: Sequence[] = (sequencesData as any)?.sequences || []

  const { data: datasetsData, loading: datasetsLoading } = useQuery(GET_DATASOURCES, {
    variables: { projectId },
    skip: !projectId,
  })
  const datasets: DataSet[] = (datasetsData as any)?.dataSets || []

  const edgeCatalog = useMemo(() => {
    const edges: Array<{ datasetId: number; datasetName: string; edge: GraphEdge }> = []
    for (const ds of datasets) {
      if (!enabledDatasetIds.includes(ds.id)) continue
      try {
        const parsed = JSON.parse(ds.graphJson ?? '{}')
        const dsEdges: GraphEdge[] = parsed.edges || []
        dsEdges.forEach((edge) => edges.push({ datasetId: ds.id, datasetName: ds.name, edge }))
      } catch (e) {
        console.error('Failed to parse dataset graphJson', e)
      }
    }
    return edges
  }, [datasets, enabledDatasetIds])

  const [createSequence] = useMutation(CREATE_SEQUENCE)
  const [updateSequence] = useMutation(UPDATE_SEQUENCE)
  const [deleteSequence] = useMutation(DELETE_SEQUENCE)

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
    const newId = result.data?.createSequence?.id
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
    if (!activeSequenceId) return
    const seq = sequences.find((s) => s.id === activeSequenceId)
    if (!seq) return
    const nextEdgeOrder: SequenceEdgeRef[] = [...seq.edgeOrder, { datasetId: edge.datasetId, edgeId: edge.edgeId }]
    await updateSequence({
      variables: { id: activeSequenceId, input: { edgeOrder: nextEdgeOrder } },
    })
    refetchSequences()
  }

  const handleToggleExpand = (id: number) => {
    const next = new Set(expanded)
    if (next.has(id)) {
      next.delete(id)
    } else {
      next.add(id)
    }
    setExpanded(next)
    setActiveSequenceId(id)
  }

  const activeSequence = sequences.find((s) => s.id === activeSequenceId) || sequences[0] || null

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
                          <span className="font-medium">{sequence.name}</span>
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
                          <Input
                            value={sequence.name}
                            onChange={(e) =>
                              updateSequence({
                                variables: { id: sequence.id, input: { name: e.target.value } },
                              })
                            }
                            className="h-8"
                          />
                          {sequence.edgeOrder.length === 0 ? (
                            <p className="text-xs text-muted-foreground">No edges yet. Click an edge to add it.</p>
                          ) : (
                            <Stack gap="xs">
                              {sequence.edgeOrder.map((edge, idx) => (
                                <div
                                  key={`${edge.datasetId}-${edge.edgeId}-${idx}`}
                                  className={cn(
                                    'text-xs px-2 py-1 rounded bg-muted',
                                    isActive && 'border border-primary/40'
                                  )}
                                  onClick={() => setActiveSequenceId(sequence.id)}
                                >
                                  #{idx + 1} · DS {edge.datasetId} · {edge.edgeId}
                                </div>
                              ))}
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
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">Available edges</h3>
              <Badge variant="outline">{edgeCatalog.length}</Badge>
            </div>
            <ScrollArea className="h-[360px] border rounded-md">
              <div className="p-2 space-y-1">
                {edgeCatalog.length === 0 ? (
                  <p className="text-xs text-muted-foreground px-2 py-1">No edges available from enabled datasets.</p>
                ) : (
                  edgeCatalog.map(({ datasetId, datasetName, edge }) => (
                    <button
                      key={`${datasetId}-${edge.id}`}
                      className="w-full text-left text-xs px-2 py-1 rounded hover:bg-muted"
                      onClick={() => handleAppendEdge({ datasetId, edgeId: edge.id })}
                    >
                      <span className="font-medium">{datasetName}</span> · {edge.id}{' '}
                      {edge.label ? `(${edge.label})` : ''}
                    </button>
                  ))
                )}
              </div>
            </ScrollArea>
            <p className="text-[11px] text-muted-foreground">
              Click an edge to append it to the active section. Use the preview button on a section to view its diagram.
            </p>
          </div>
        </CardContent>
      </Card>

      <SequenceDiagramDialog
        open={previewOpen && !!activeSequence}
        onClose={() => setPreviewOpen(false)}
        sequence={activeSequence}
        projectId={projectId}
      />
    </>
  )
}
