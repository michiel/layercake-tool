import { useState, useEffect, useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import {
  IconGripVertical,
  IconTrash,
  IconArrowRight,
  IconDeviceFloppy,
  IconArrowLeft,
  IconChevronLeft,
  IconChevronRight,
  IconChevronDown,
  IconPlus,
  IconFileDescription,
  IconListDetails,
  IconTool,
} from '@tabler/icons-react'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import { Spinner } from '@/components/ui/spinner'
import { cn } from '@/lib/utils'
import {
  GET_SEQUENCE,
  CREATE_SEQUENCE,
  UPDATE_SEQUENCE,
  Sequence,
  SequenceEdgeRef,
} from '@/graphql/sequences'
import { GET_STORY, Story } from '@/graphql/stories'
import { GET_DATASOURCES, DataSet } from '@/graphql/datasets'

const GET_PROJECTS = gql`
  query GetProjectsForSequenceEditor {
    projects {
      id
      name
      description
    }
  }
`

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
  name?: string
}

interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export const SequenceEditorPage = () => {
  const navigate = useNavigate()
  const { projectId, storyId, sequenceId } = useParams<{
    projectId: string
    storyId: string
    sequenceId: string
  }>()
  const projectIdNum = Number(projectId || 0)
  const storyIdNum = Number(storyId || 0)
  const sequenceIdNum = sequenceId === 'new' ? null : Number(sequenceId || 0)
  const isEditing = sequenceIdNum !== null

  const [activeTab, setActiveTab] = useState<string>('attributes')
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [enabledDatasetIds, setEnabledDatasetIds] = useState<number[]>([])
  const [edgeOrder, setEdgeOrder] = useState<SequenceEdgeRef[]>([])
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  const [allEdgesCollapsed, setAllEdgesCollapsed] = useState(false)
  const [toolsCollapsed, setToolsCollapsed] = useState(false)
  const [collapsedDatasets, setCollapsedDatasets] = useState<Set<number>>(new Set())

  // Queries
  const { data: projectsData, loading: projectsLoading } = useQuery(GET_PROJECTS)
  const projects = (projectsData as any)?.projects || []
  const project = projects.find((p: any) => p.id === projectIdNum)

  const { data: storyData, loading: storyLoading } = useQuery(GET_STORY, {
    variables: { id: storyIdNum },
    skip: !storyIdNum,
  })
  const story: Story | null = (storyData as any)?.story || null

  const { data: sequenceData, loading: sequenceLoading } = useQuery(GET_SEQUENCE, {
    variables: { id: sequenceIdNum },
    skip: !sequenceIdNum,
  })
  const sequence: Sequence | null = (sequenceData as any)?.sequence || null

  const { data: datasetsData, loading: datasetsLoading } = useQuery(GET_DATASOURCES, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
  })
  const allDatasets: DataSet[] = (datasetsData as any)?.dataSets || []

  // Filter datasets to only those enabled in the story
  const storyDatasets = useMemo(() => {
    if (!story) return []
    return allDatasets.filter((d) => story.enabledDatasetIds.includes(d.id))
  }, [allDatasets, story])

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

  // Get edges not yet added to the sequence
  const unselectedEdges = useMemo(() => {
    return availableEdges.filter(
      ({ datasetId, edge }) =>
        !edgeOrder.some((ref) => ref.datasetId === datasetId && ref.edgeId === edge.id)
    )
  }, [availableEdges, edgeOrder])

  // Group unselected edges by dataset
  const unselectedEdgesByDataset = useMemo(() => {
    const grouped: Record<number, { datasetName: string; edges: Array<{ edge: GraphEdge }> }> = {}
    for (const { datasetId, datasetName, edge } of unselectedEdges) {
      if (!grouped[datasetId]) {
        grouped[datasetId] = { datasetName, edges: [] }
      }
      grouped[datasetId].edges.push({ edge })
    }
    return grouped
  }, [unselectedEdges])

  const toggleDatasetCollapse = (datasetId: number) => {
    setCollapsedDatasets((prev) => {
      const next = new Set(prev)
      if (next.has(datasetId)) {
        next.delete(datasetId)
      } else {
        next.add(datasetId)
      }
      return next
    })
  }

  // Helper to get node label
  const getNodeLabel = (datasetId: number, nodeId: string): string => {
    const graphData = datasetGraphData[datasetId]
    const node = graphData?.nodes.find((n) => n.id === nodeId)
    return node?.label || node?.name || nodeId
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

  // Mutations
  const [createSequence, { loading: createLoading }] = useMutation(CREATE_SEQUENCE, {
    onCompleted: () => {
      navigate(`/projects/${projectIdNum}/stories/${storyIdNum}`)
    },
    onError: (error) => {
      console.error('Failed to create sequence:', error)
      alert(`Failed to create sequence: ${error.message}`)
    },
  })

  const [updateSequence, { loading: updateLoading }] = useMutation(UPDATE_SEQUENCE, {
    onCompleted: (data) => {
      console.log('Sequence updated successfully:', data)
    },
    onError: (error) => {
      console.error('Failed to update sequence:', error)
      alert(`Failed to update sequence: ${error.message}`)
    },
  })

  // Initialize form when data loads
  useEffect(() => {
    if (isEditing && sequence) {
      setName(sequence.name)
      setDescription(sequence.description || '')
      setEnabledDatasetIds(sequence.enabledDatasetIds)
      setEdgeOrder(sequence.edgeOrder)
    } else if (!isEditing && story) {
      // New sequence - enable all story datasets by default
      setName('')
      setDescription('')
      setEnabledDatasetIds(story.enabledDatasetIds)
      setEdgeOrder([])
    }
  }, [isEditing, sequence, story])

  const handleSave = async () => {
    if (!name.trim()) {
      alert('Please enter a sequence name')
      return
    }

    const trimmedDescription = description.trim()
    const input = {
      name: name.trim(),
      description: trimmedDescription || undefined,
      enabledDatasetIds,
      edgeOrder,
    }

    try {
      if (isEditing && sequenceIdNum) {
        await updateSequence({
          variables: { id: sequenceIdNum, input },
        })
      } else {
        await createSequence({
          variables: {
            input: {
              storyId: storyIdNum,
              ...input,
            },
          },
        })
      }
    } catch (error) {
      // Error is handled by mutation onError
      console.error('Save error:', error)
    }
  }

  const handleBack = () => {
    navigate(`/projects/${projectIdNum}/stories/${storyIdNum}`)
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

  const loading = projectsLoading || storyLoading || sequenceLoading || datasetsLoading
  const saving = createLoading || updateLoading

  if (loading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading sequence editor...</span>
        </Group>
      </PageContainer>
    )
  }

  if (!project) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Project not found</h1>
        <Button className="mt-4" onClick={() => navigate('/projects')}>
          Back to projects
        </Button>
      </PageContainer>
    )
  }

  if (!story) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Story not found</h1>
        <Button className="mt-4" onClick={() => navigate(`/projects/${projectIdNum}/stories`)}>
          Back to stories
        </Button>
      </PageContainer>
    )
  }

  if (isEditing && !sequence) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Sequence not found</h1>
        <Button className="mt-4" onClick={() => navigate(`/projects/${projectIdNum}/stories/${storyIdNum}`)}>
          Back to story
        </Button>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={project.name}
        projectId={project.id}
        sections={[
          { title: 'Workbench', href: `/projects/${project.id}/workbench` },
          { title: 'Stories', href: `/projects/${project.id}/stories` },
          { title: story.name, href: `/projects/${project.id}/stories/${story.id}` },
        ]}
        currentPage={isEditing ? sequence!.name : 'New Sequence'}
        onNavigate={(route) => navigate(route)}
      />

      <Group justify="between" className="mb-6">
        <div>
          <h1 className="text-3xl font-bold">
            {isEditing ? `Edit: ${sequence!.name}` : 'New Sequence'}
          </h1>
          <p className="text-muted-foreground">
            Build a narrative sequence by selecting and ordering edges from your datasets.
          </p>
        </div>
        <Group gap="sm">
          <Button variant="outline" onClick={handleBack}>
            <IconArrowLeft className="mr-2 h-4 w-4" />
            Back to Story
          </Button>
          <Button onClick={handleSave} disabled={saving || !name.trim()}>
            {saving && <Spinner className="mr-2 h-4 w-4" />}
            <IconDeviceFloppy className="mr-2 h-4 w-4" />
            Save
          </Button>
        </Group>
      </Group>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="h-[calc(100vh-280px)]">
        <TabsList>
          <TabsTrigger value="attributes">
            <IconFileDescription className="mr-2 h-4 w-4" />
            Attributes
          </TabsTrigger>
          <TabsTrigger value="sequence">
            <IconListDetails className="mr-2 h-4 w-4" />
            Sequence
          </TabsTrigger>
        </TabsList>

        {/* Attributes Tab */}
        <TabsContent value="attributes" className="mt-4">
          <div className="grid gap-6 md:grid-cols-2">
            {/* Sequence Details Card */}
            <Card className="border">
              <CardHeader>
                <CardTitle className="text-base">Sequence Details</CardTitle>
              </CardHeader>
              <CardContent>
                <Stack gap="md">
                  <div className="space-y-2">
                    <Label htmlFor="seq-name">Name *</Label>
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
                      rows={3}
                    />
                  </div>
                </Stack>
              </CardContent>
            </Card>

            {/* Datasets Card */}
            <Card className="border">
              <CardHeader>
                <CardTitle className="text-base">Datasets</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground mb-4">
                  Enable datasets to select edges from. Only datasets enabled in the story are shown.
                </p>
                {storyDatasets.length === 0 ? (
                  <p className="text-sm text-muted-foreground italic">
                    No datasets enabled in this story.
                  </p>
                ) : (
                  <Stack gap="sm">
                    {storyDatasets.map((dataset) => {
                      const graphData = datasetGraphData[dataset.id]
                      const edgeCount = graphData?.edges.length || 0
                      return (
                        <div key={dataset.id} className="flex items-center space-x-3">
                          <Checkbox
                            id={`seq-ds-${dataset.id}`}
                            checked={enabledDatasetIds.includes(dataset.id)}
                            onCheckedChange={() => toggleDataset(dataset.id)}
                          />
                          <label
                            htmlFor={`seq-ds-${dataset.id}`}
                            className="text-sm cursor-pointer flex-1"
                          >
                            <div className="font-medium">{dataset.name}</div>
                            <div className="text-xs text-muted-foreground">
                              {edgeCount} edge{edgeCount !== 1 ? 's' : ''}
                            </div>
                          </label>
                        </div>
                      )
                    })}
                  </Stack>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Sequence Tab */}
        <TabsContent value="sequence" className="mt-4 h-[calc(100%-60px)]">
          <div className="flex gap-2 h-full">
            {/* All Edges Column (collapsible) */}
            <div
              className={cn(
                'border rounded-lg flex flex-col transition-all duration-200',
                allEdgesCollapsed ? 'w-10' : 'w-80'
              )}
            >
              <div className="flex items-center justify-between p-2 border-b bg-muted/30">
                {!allEdgesCollapsed && (
                  <span className="font-medium text-sm">All Edges</span>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 w-6 p-0"
                  onClick={() => setAllEdgesCollapsed(!allEdgesCollapsed)}
                >
                  {allEdgesCollapsed ? (
                    <IconChevronRight className="h-4 w-4" />
                  ) : (
                    <IconChevronLeft className="h-4 w-4" />
                  )}
                </Button>
              </div>
              {!allEdgesCollapsed && (
                <ScrollArea className="flex-1">
                  <div className="p-2 space-y-2">
                    {Object.keys(unselectedEdgesByDataset).length === 0 ? (
                      <p className="text-xs text-muted-foreground p-2 text-center">
                        {availableEdges.length === 0
                          ? 'No edges available. Enable datasets in Attributes tab.'
                          : 'All edges have been added to the sequence.'}
                      </p>
                    ) : (
                      Object.entries(unselectedEdgesByDataset).map(([datasetIdStr, { datasetName, edges }]) => {
                        const datasetId = Number(datasetIdStr)
                        const isCollapsed = collapsedDatasets.has(datasetId)
                        return (
                          <div key={datasetId} className="border rounded">
                            <button
                              type="button"
                              className="w-full flex items-center justify-between p-2 hover:bg-muted/50 text-left"
                              onClick={() => toggleDatasetCollapse(datasetId)}
                            >
                              <span className="font-medium text-sm truncate">{datasetName}</span>
                              <div className="flex items-center gap-1">
                                <Badge variant="secondary" className="text-xs">
                                  {edges.length}
                                </Badge>
                                {isCollapsed ? (
                                  <IconChevronRight className="h-4 w-4" />
                                ) : (
                                  <IconChevronDown className="h-4 w-4" />
                                )}
                              </div>
                            </button>
                            {!isCollapsed && (
                              <div className="border-t p-1 space-y-1">
                                {edges.map(({ edge }) => {
                                  const sourceLabel = getNodeLabel(datasetId, edge.source)
                                  const targetLabel = getNodeLabel(datasetId, edge.target)
                                  return (
                                    <div
                                      key={edge.id}
                                      className="p-2 rounded text-sm hover:bg-muted/50 cursor-pointer group flex items-center justify-between"
                                      onClick={() => addEdge(datasetId, edge.id)}
                                    >
                                      <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-1 text-xs">
                                          <span className="truncate max-w-[90px]" title={sourceLabel}>
                                            {sourceLabel}
                                          </span>
                                          <IconArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                                          <span className="truncate max-w-[90px]" title={targetLabel}>
                                            {targetLabel}
                                          </span>
                                        </div>
                                        {edge.comments && (
                                          <p className="text-xs text-muted-foreground mt-0.5 truncate" title={edge.comments}>
                                            {edge.comments}
                                          </p>
                                        )}
                                      </div>
                                      <IconPlus className="h-3 w-3 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" />
                                    </div>
                                  )
                                })}
                              </div>
                            )}
                          </div>
                        )
                      })
                    )}
                  </div>
                </ScrollArea>
              )}
            </div>

            {/* Sequence Column (main, flexible width) */}
            <Card className="border flex-1 flex flex-col min-w-0">
              <CardHeader className="border-b py-2 px-3">
                <Group justify="between" align="center">
                  <CardTitle className="text-sm">Sequence</CardTitle>
                  <Badge variant="secondary">{edgeOrder.length} edge{edgeOrder.length !== 1 ? 's' : ''}</Badge>
                </Group>
              </CardHeader>
              <CardContent className="p-0 flex-1 overflow-hidden">
                <ScrollArea className="h-full">
                  {edgeOrder.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                      <p className="text-sm">No edges in sequence</p>
                      <p className="text-xs mt-1">Click edges from "All Edges" to add them</p>
                    </div>
                  ) : (
                    <Stack gap="xs" className="p-2">
                      {edgeOrder.map((ref, index) => {
                        const { edge, dataset, sourceLabel, targetLabel } = getEdgeInfo(ref)
                        return (
                          <div
                            key={`${ref.datasetId}-${ref.edgeId}-${index}`}
                            className={cn(
                              'flex items-center gap-2 p-2 border rounded-md bg-background hover:bg-muted/50',
                              draggedIndex === index && 'opacity-50'
                            )}
                            draggable
                            onDragStart={() => handleDragStart(index)}
                            onDragOver={(e) => handleDragOver(e, index)}
                            onDragEnd={handleDragEnd}
                          >
                            <IconGripVertical className="h-4 w-4 text-muted-foreground cursor-grab shrink-0" />
                            <Badge variant="outline" className="shrink-0 text-xs">{index + 1}</Badge>
                            <div className="flex-1 flex items-center gap-2 min-w-0">
                              <span className="text-sm truncate" title={sourceLabel}>
                                {sourceLabel}
                              </span>
                              <IconArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                              {edge?.comments && (
                                <span className="text-xs text-muted-foreground truncate max-w-[120px]" title={edge.comments}>
                                  {edge.comments}
                                </span>
                              )}
                              <IconArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                              <span className="text-sm truncate" title={targetLabel}>
                                {targetLabel}
                              </span>
                            </div>
                            <Badge variant="secondary" className="text-xs shrink-0">
                              {dataset?.name}
                            </Badge>
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 w-6 p-0 text-destructive hover:text-destructive/80 shrink-0"
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
              </CardContent>
            </Card>

            {/* Tools Column (collapsible) */}
            <div
              className={cn(
                'border rounded-lg flex flex-col transition-all duration-200',
                toolsCollapsed ? 'w-10' : 'w-48'
              )}
            >
              <div className="flex items-center justify-between p-2 border-b bg-muted/30">
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 w-6 p-0"
                  onClick={() => setToolsCollapsed(!toolsCollapsed)}
                >
                  {toolsCollapsed ? (
                    <IconChevronLeft className="h-4 w-4" />
                  ) : (
                    <IconChevronRight className="h-4 w-4" />
                  )}
                </Button>
                {!toolsCollapsed && (
                  <span className="font-medium text-sm">Tools</span>
                )}
              </div>
              {!toolsCollapsed && (
                <div className="p-2 space-y-2">
                  <Button variant="outline" size="sm" className="w-full" disabled>
                    <IconTool className="mr-2 h-4 w-4" />
                    Placeholder
                  </Button>
                </div>
              )}
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </PageContainer>
  )
}

export default SequenceEditorPage
