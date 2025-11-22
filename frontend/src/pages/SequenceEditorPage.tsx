import { useState, useEffect, useMemo, useRef, useCallback } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import {
  IconGripVertical,
  IconTrash,
  IconArrowRight,
  IconArrowLeft,
  IconChevronLeft,
  IconChevronRight,
  IconChevronDown,
  IconPlus,
  IconFileDescription,
  IconListDetails,
  IconTool,
  IconCheck,
  IconLoader2,
  IconTimeline,
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
import { GET_PROJECT_LAYERS, ProjectLayer } from '@/graphql/layers'
import { MermaidPreviewDialog } from '@/components/visualization/MermaidPreviewDialog'

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
  layer?: string
  attrs?: Record<string, any>
}

interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

const VALID_TABS = ['attributes', 'sequence'] as const
type TabValue = (typeof VALID_TABS)[number]

export const SequenceEditorPage = () => {
  const navigate = useNavigate()
  const { projectId, storyId, sequenceId } = useParams<{
    projectId: string
    storyId: string
    sequenceId: string
  }>()
  const [searchParams, setSearchParams] = useSearchParams()
  const projectIdNum = Number(projectId || 0)
  const storyIdNum = Number(storyId || 0)
  const sequenceIdNum = sequenceId === 'new' ? null : Number(sequenceId || 0)
  const isEditing = sequenceIdNum !== null

  // Get active tab from URL, default to 'attributes'
  const tabParam = searchParams.get('tab')
  const activeTab: TabValue = VALID_TABS.includes(tabParam as TabValue) ? (tabParam as TabValue) : 'attributes'

  const setActiveTab = (tab: string) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev)
      if (tab === 'attributes') {
        next.delete('tab')
      } else {
        next.set('tab', tab)
      }
      return next
    }, { replace: true })
  }
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [enabledDatasetIds, setEnabledDatasetIds] = useState<number[]>([])
  const [edgeOrder, setEdgeOrder] = useState<SequenceEdgeRef[]>([])
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  const [allEdgesCollapsed, setAllEdgesCollapsed] = useState(false)
  const [toolsCollapsed, setToolsCollapsed] = useState(false)
  const [collapsedDatasets, setCollapsedDatasets] = useState<Set<number>>(new Set())
  const [edgeFilter, setEdgeFilter] = useState('')
  const [isInitialized, setIsInitialized] = useState(false)
  const [syncStatus, setSyncStatus] = useState<'idle' | 'syncing' | 'synced' | 'error'>('idle')
  const [diagramDialogOpen, setDiagramDialogOpen] = useState(false)
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

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

  const { data: layersData, loading: layersLoading } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
  })
  const projectLayers: ProjectLayer[] = (layersData as any)?.projectLayers || []

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
        const nodes = data.nodes || []
        const edges = data.edges || data.links || []
        console.log(`Dataset ${ds.id} (${ds.name}):`, {
          nodeCount: nodes.length,
          edgeCount: edges.length,
          sampleNode: nodes[0],
          sampleEdge: edges[0]
        })
        result[ds.id] = { nodes, edges }
      } catch (e) {
        console.error(`Failed to parse graphJson for dataset ${ds.id}:`, e)
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

  const getNodeInfo = useCallback((nodeId: string): { label: string; layer?: string } => {
    for (const dsId of enabledDatasetIds) {
      const graphData = datasetGraphData[dsId]
      if (!graphData) continue
      const node = graphData.nodes.find((n) => n.id === nodeId)
      if (node) {
        const label = node.label || node.name || node.attrs?.label || node.attrs?.name
        return {
          label: label && String(label).trim() ? String(label) : nodeId,
          layer: node.layer,
        }
      }
    }
    return { label: nodeId }
  }, [enabledDatasetIds, datasetGraphData])

  const getLayerColors = (layerId?: string): { bg: string; text: string } | null => {
    if (!layerId) return null
    const layer = projectLayers.find((l) => l.layerId === layerId && l.enabled)
    if (!layer) return null
    return {
      bg: layer.backgroundColor || '#e5e7eb',
      text: layer.textColor || '#000000',
    }
  }

  // Group unselected edges by dataset
  const unselectedEdgesByDataset = useMemo(() => {
    const grouped: Record<
      number,
      {
        datasetName: string
        edges: Array<{
          edge: GraphEdge
          sourceLabel: string
          targetLabel: string
          sourceColors: { bg: string; text: string } | null
          targetColors: { bg: string; text: string } | null
        }>
      }
    > = {}
    const normalizedFilter = edgeFilter.trim().toLowerCase()

    for (const { datasetId, datasetName, edge } of unselectedEdges) {
      const sourceInfo = getNodeInfo(edge.source)
      const targetInfo = getNodeInfo(edge.target)
      const sourceLabel = sourceInfo.label
      const targetLabel = targetInfo.label

      if (
        normalizedFilter &&
        !sourceLabel.toLowerCase().includes(normalizedFilter) &&
        !targetLabel.toLowerCase().includes(normalizedFilter)
      ) {
        continue
      }

      if (!grouped[datasetId]) {
        grouped[datasetId] = { datasetName, edges: [] }
      }

      grouped[datasetId].edges.push({
        edge,
        sourceLabel,
        targetLabel,
        sourceColors: getLayerColors(sourceInfo.layer),
        targetColors: getLayerColors(targetInfo.layer),
      })
    }

    Object.values(grouped).forEach(({ edges }) => {
      edges.sort((a, b) =>
        a.sourceLabel.localeCompare(b.sourceLabel, undefined, { sensitivity: 'base' })
      )
    })

    return grouped
  }, [unselectedEdges, edgeFilter, getNodeInfo])

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

  // Helper to get edge info with node labels and layer info
  const getEdgeInfo = (ref: SequenceEdgeRef) => {
    const graphData = datasetGraphData[ref.datasetId]
    const edge = graphData?.edges.find((e) => e.id === ref.edgeId)
    const dataset = storyDatasets.find((d) => d.id === ref.datasetId)
    const sourceInfo = edge ? getNodeInfo(edge.source) : { label: 'Unknown' }
    const targetInfo = edge ? getNodeInfo(edge.target) : { label: 'Unknown' }
    return {
      edge,
      dataset,
      sourceLabel: sourceInfo.label,
      targetLabel: targetInfo.label,
      sourceColors: getLayerColors(sourceInfo.layer),
      targetColors: getLayerColors(targetInfo.layer),
    }
  }

  // Helper to get layer background color
  const getLayerBgColor = (layerId?: string): string | null => {
    if (!layerId) return null
    const layer = projectLayers.find((l) => l.layerId === layerId && l.enabled)
    if (!layer) return null
    return layer.backgroundColor || null
  }

  // Generate Mermaid sequence diagram
  const mermaidDiagram = useMemo(() => {
    if (!edgeOrder.length) {
      return 'sequenceDiagram\n    Note over A: No edges in sequence'
    }

    const escapeLabel = (label: string): string =>
      label.replace(/"/g, '\\"').replace(/\n/g, ' ')
    const makeParticipantId = (nodeId: string): string =>
      nodeId.replace(/[^a-zA-Z0-9_]/g, '_')

    const lines: string[] = ['sequenceDiagram']
    const participantOrder: string[] = []
    const participantInfo: Map<string, { label: string; color: string | null }> = new Map()

    // First pass: collect participants in order of first appearance
    for (const ref of edgeOrder) {
      const graphData = datasetGraphData[ref.datasetId]
      const edge = graphData?.edges.find((e) => e.id === ref.edgeId)
      if (!edge) continue

      if (!participantInfo.has(edge.source)) {
        participantOrder.push(edge.source)
        const nodeInfo = getNodeInfo(edge.source)
        participantInfo.set(edge.source, {
          label: nodeInfo.label,
          color: getLayerBgColor(nodeInfo.layer),
        })
      }
      if (!participantInfo.has(edge.target)) {
        participantOrder.push(edge.target)
        const nodeInfo = getNodeInfo(edge.target)
        participantInfo.set(edge.target, {
          label: nodeInfo.label,
          color: getLayerBgColor(nodeInfo.layer),
        })
      }
    }

    // Add participant declarations with colored boxes
    for (const nodeId of participantOrder) {
      const info = participantInfo.get(nodeId)
      if (!info) continue
      const participantId = makeParticipantId(nodeId)
      const color = info.color
      if (color) {
        lines.push(`    box ${color}`)
        lines.push(`        participant ${participantId} as "${escapeLabel(info.label)}"`)
        lines.push(`    end`)
      } else {
        lines.push(`    participant ${participantId} as "${escapeLabel(info.label)}"`)
      }
    }

    // Add edges as messages
    for (let i = 0; i < edgeOrder.length; i++) {
      const ref = edgeOrder[i]
      const graphData = datasetGraphData[ref.datasetId]
      const edge = graphData?.edges.find((e) => e.id === ref.edgeId)
      if (!edge) continue

      const sourceId = makeParticipantId(edge.source)
      const targetId = makeParticipantId(edge.target)
      const orderNum = i + 1
      const parts: string[] = [String(orderNum)]
      if (edge.label) parts.push(escapeLabel(edge.label))
      if (edge.comments) parts.push(escapeLabel(edge.comments))
      const message = parts.join(': ')
      lines.push(`    ${sourceId}->>${targetId}: ${message}`)
    }

    return lines.join('\n')
  }, [edgeOrder, datasetGraphData, projectLayers, getNodeInfo])

  // Mutations
  const [createSequence, { loading: createLoading }] = useMutation(CREATE_SEQUENCE, {
    onCompleted: (data) => {
      const newId = (data as { createSequence?: Sequence })?.createSequence?.id
      if (newId) {
        // Redirect to edit URL so auto-save works
        navigate(`/projects/${projectIdNum}/stories/${storyIdNum}/sequences/${newId}`, { replace: true })
      }
    },
    onError: (error) => {
      console.error('Failed to create sequence:', error)
      alert(`Failed to create sequence: ${error.message}`)
    },
  })

  const [updateSequence] = useMutation(UPDATE_SEQUENCE, {
    onCompleted: () => {
      setSyncStatus('synced')
    },
    onError: (error) => {
      console.error('Failed to update sequence:', error)
      setSyncStatus('error')
    },
  })

  // Initialize form when data loads
  useEffect(() => {
    if (isEditing && sequence) {
      setName(sequence.name)
      setDescription(sequence.description || '')
      setEnabledDatasetIds(sequence.enabledDatasetIds)
      setEdgeOrder(sequence.edgeOrder)
      // Mark as initialized after a short delay to avoid immediate save
      setTimeout(() => setIsInitialized(true), 100)
    } else if (!isEditing && story) {
      // New sequence - enable all story datasets by default
      setName('')
      setDescription('')
      setEnabledDatasetIds(story.enabledDatasetIds)
      setEdgeOrder([])
      setIsInitialized(false)
    }
  }, [isEditing, sequence, story])

  // Auto-save for existing sequences
  const performSave = useCallback(async () => {
    if (!isEditing || !sequenceIdNum || !name.trim()) return

    setSyncStatus('syncing')
    const trimmedDescription = description.trim()
    const cleanEdgeOrder = edgeOrder.map(({ datasetId, edgeId }) => ({ datasetId, edgeId }))
    const input = {
      name: name.trim(),
      description: trimmedDescription || undefined,
      enabledDatasetIds,
      edgeOrder: cleanEdgeOrder,
    }

    try {
      await updateSequence({
        variables: { id: sequenceIdNum, input },
      })
    } catch (error) {
      console.error('Auto-save error:', error)
    }
  }, [isEditing, sequenceIdNum, name, description, enabledDatasetIds, edgeOrder, updateSequence])

  // Debounced auto-save effect
  useEffect(() => {
    if (!isInitialized || !isEditing) return

    // Clear any existing timeout
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
    }

    // Set new timeout for debounced save
    saveTimeoutRef.current = setTimeout(() => {
      performSave()
    }, 500)

    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current)
      }
    }
  }, [name, description, enabledDatasetIds, edgeOrder, isInitialized, isEditing, performSave])

  // Create new sequence
  const handleCreate = async () => {
    if (!name.trim()) {
      alert('Please enter a sequence name')
      return
    }

    const trimmedDescription = description.trim()
    const cleanEdgeOrder = edgeOrder.map(({ datasetId, edgeId }) => ({ datasetId, edgeId }))
    const input = {
      storyId: storyIdNum,
      name: name.trim(),
      description: trimmedDescription || undefined,
      enabledDatasetIds,
      edgeOrder: cleanEdgeOrder,
    }

    try {
      await createSequence({ variables: { input } })
    } catch (error) {
      console.error('Create error:', error)
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

  const loading = projectsLoading || storyLoading || sequenceLoading || datasetsLoading || layersLoading

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
        <Group gap="sm" align="center">
          {isEditing && (
            <span className="text-sm text-muted-foreground flex items-center gap-1">
              {syncStatus === 'syncing' && (
                <>
                  <IconLoader2 className="h-4 w-4 animate-spin" />
                  Saving...
                </>
              )}
              {syncStatus === 'synced' && (
                <>
                  <IconCheck className="h-4 w-4 text-green-600" />
                  Saved
                </>
              )}
              {syncStatus === 'error' && (
                <span className="text-destructive">Save failed</span>
              )}
            </span>
          )}
          <Button
            variant="outline"
            onClick={() => setDiagramDialogOpen(true)}
            disabled={edgeOrder.length === 0}
            title="View sequence diagram"
          >
            <IconTimeline className="mr-2 h-4 w-4" />
            Diagram
          </Button>
          <Button variant="outline" onClick={handleBack}>
            <IconArrowLeft className="mr-2 h-4 w-4" />
            Back to Story
          </Button>
          {!isEditing && (
            <Button onClick={handleCreate} disabled={createLoading || !name.trim()}>
              {createLoading && <Spinner className="mr-2 h-4 w-4" />}
              Create Sequence
            </Button>
          )}
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
              <div className="flex items-center gap-2 p-2 border-b bg-muted/30">
                {!allEdgesCollapsed && (
                  <>
                    <span className="font-medium text-sm whitespace-nowrap">All Edges</span>
                    <Input
                      value={edgeFilter}
                      onChange={(e) => setEdgeFilter(e.target.value)}
                      placeholder="Filter nodes"
                      className="h-7 text-xs flex-1"
                    />
                  </>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 w-6 p-0 ml-auto"
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
                                {edges.map(({ edge, sourceLabel, targetLabel, sourceColors, targetColors }) => {
                                  return (
                                    <div
                                      key={edge.id}
                                      className="p-2 rounded text-sm hover:bg-muted/50 cursor-pointer group flex items-center justify-between"
                                      onClick={() => addEdge(datasetId, edge.id)}
                                    >
                                      <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-1">
                                          <span
                                            className="text-xs px-1.5 py-0.5 rounded truncate w-[100px] inline-block text-center"
                                            style={sourceColors ? { backgroundColor: sourceColors.bg, color: sourceColors.text } : undefined}
                                            title={sourceLabel}
                                          >
                                            {sourceLabel}
                                          </span>
                                          <IconArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                                          <span
                                            className="text-xs px-1.5 py-0.5 rounded truncate w-[100px] inline-block text-center"
                                            style={targetColors ? { backgroundColor: targetColors.bg, color: targetColors.text } : undefined}
                                            title={targetLabel}
                                          >
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
                        const { edge, dataset, sourceLabel, targetLabel, sourceColors, targetColors } = getEdgeInfo(ref)
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
                              <span
                                className="text-xs px-1.5 py-0.5 rounded truncate w-[140px] shrink-0 inline-block text-center"
                                style={sourceColors ? { backgroundColor: sourceColors.bg, color: sourceColors.text } : undefined}
                                title={sourceLabel}
                              >
                                {sourceLabel}
                              </span>
                              <IconArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                              <span
                                className="text-xs px-1.5 py-0.5 rounded truncate w-[140px] shrink-0 inline-block text-center"
                                style={targetColors ? { backgroundColor: targetColors.bg, color: targetColors.text } : undefined}
                                title={targetLabel}
                              >
                                {targetLabel}
                              </span>
                              {edge?.comments && (
                                <>
                                  <IconArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                                  <span className="text-xs text-muted-foreground truncate" title={edge.comments}>
                                    {edge.comments}
                                  </span>
                                </>
                              )}
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

      {/* Sequence Diagram Preview */}
      <MermaidPreviewDialog
        open={diagramDialogOpen}
        onClose={() => setDiagramDialogOpen(false)}
        diagram={mermaidDiagram}
        title={`Sequence Diagram: ${name || 'Untitled'}`}
      />
    </PageContainer>
  )
}

export default SequenceEditorPage
