import React, { useState, useEffect, useMemo, useRef, useCallback } from 'react'
import { useParams, useNavigate, useLocation } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  IconArrowLeft,
  IconDeviceFloppy,
  IconFile,
  IconCode,
  IconRefresh,
  IconDownload,
  IconAlertCircle,
  IconCheck,
  IconClock,
  IconX,
  IconTable,
  IconPalette
} from '@tabler/icons-react'
import { useForm } from 'react-hook-form'
import { Breadcrumbs } from '../common/Breadcrumbs'
import {
  GET_DATASOURCE,
  UPDATE_DATASOURCE,
  REPROCESS_DATASOURCE,
  UPDATE_DATASOURCE_GRAPH_DATA,
  DataSet,
  UpdateDataSetInput,
  formatFileSize,
  getFileFormatDisplayName
} from '../../graphql/datasets'
import { GET_PROJECT_LAYERS, ProjectLayer } from '../../graphql/layers'
import { GraphSpreadsheetEditor, GraphData } from '../editors/GraphSpreadsheetEditor'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { ScrollArea } from '../ui/scroll-area'
import { Spinner } from '../ui/spinner'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs'
import { Textarea } from '../ui/textarea'
import PageContainer from '../layout/PageContainer'

import { gql } from '@apollo/client'

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

interface DataSetEditorProps {}

export const DataSetEditor: React.FC<DataSetEditorProps> = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const { projectId, dataSetId } = useParams<{ projectId: string; dataSetId: string }>()
  const paletteViewRequested = useMemo(() => {
    const params = new URLSearchParams(location.search)
    if (params.get('view') === 'palette') {
      return true
    }
    const stateView = (location.state as { view?: string } | null)?.view
    return stateView === 'palette'
  }, [location.search, location.state])

  const [activeTab, setActiveTab] = useState<string>('edit')
  const initializedTabRef = useRef(false)
  const [fileUploadMode, setFileUploadMode] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)

  // Query for project info
  const { data: projectsData } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)
  const projects = projectsData?.projects || []
  const selectedProject = projects.find(p => p.id === parseInt(projectId || '0'))

  // Query for DataSet
  const {
    data: dataSourceData,
    loading: dataSourceLoading,
    error: dataSourceError,
    refetch: refetchDataSet
  } = useQuery(GET_DATASOURCE, {
    variables: { id: parseInt(dataSetId || '0') },
    errorPolicy: 'all'
  })

  const {
    data: projectLayersData,
    loading: projectLayersLoading,
    error: projectLayersError
  } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId: parseInt(projectId || '0') },
    skip: !projectId
  })

  // Mutations
  const [updateDataSet, { loading: updateLoading }] = useMutation(UPDATE_DATASOURCE)
  const [reprocessDataSet, { loading: reprocessLoading }] = useMutation(REPROCESS_DATASOURCE)
  const [updateDataSetGraphData] = useMutation(UPDATE_DATASOURCE_GRAPH_DATA)

  const dataSource: DataSet | null = (dataSourceData as any)?.dataSet || null
  const projectPaletteLayers: ProjectLayer[] = useMemo(
    () =>
      ((projectLayersData as any)?.projectLayers as ProjectLayer[] | undefined) ?? [],
    [projectLayersData]
  )

  const originalDatasetLayers = useMemo(() => {
    if (!dataSource?.graphJson) {
      return []
    }
    try {
      const parsed = JSON.parse(dataSource.graphJson)
      return Array.isArray(parsed.layers) ? parsed.layers : []
    } catch (error) {
      console.error('Failed to parse dataset layers', error)
      return []
    }
  }, [dataSource])

  const resolvedGraphData = useMemo((): GraphData | null => {
    if (!dataSource?.graphJson) {
      return null
    }

    const ensureColor = (value?: string | null, fallback = '#f7f7f8') => {
      if (!value) {
        return fallback
      }
      return value.startsWith('#') ? value : `#${value}`
    }

    try {
      const parsed = JSON.parse(dataSource.graphJson)
      const nodesArray = Array.isArray(parsed.nodes) ? parsed.nodes : []
      const edgesArray = Array.isArray(parsed.edges) ? parsed.edges : []

      const nodes = nodesArray.map((node: any) => ({
        id: node.id,
        label: node.label || '',
        layer: node.layer,
        weight: node.weight,
        is_partition: node.is_partition,
        belongs_to: node.belongs_to,
        comment: node.comment,
        ...node
      }))

      const edges = edgesArray.map((edge: any) => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        label: edge.label || '',
        layer: edge.layer,
        weight: edge.weight,
        comment: edge.comment,
        ...edge
      }))

      const referencedLayerIds = new Set<string>()
      nodes.forEach(node => {
        if (node.layer) {
          referencedLayerIds.add(node.layer)
        }
      })
      edges.forEach(edge => {
        if (edge.layer) {
          referencedLayerIds.add(edge.layer)
        }
      })

      const paletteMap = new Map<
        string,
        { label: string; background_color: string; text_color: string; border_color: string }
      >()

      projectPaletteLayers.forEach(layer => {
        const entry = {
          label: layer.name || layer.layerId,
          background_color: ensureColor(layer.backgroundColor, '#f7f7f8'),
          text_color: ensureColor(layer.textColor, '#0f172a'),
          border_color: ensureColor(layer.borderColor, '#dddddd')
        }
        paletteMap.set(layer.layerId, entry)
        layer.aliases?.forEach(alias => {
          paletteMap.set(alias.aliasLayerId, entry)
        })
      })

      const resolvedLayers = Array.from(referencedLayerIds)
        .sort()
        .map(layerId => {
          const resolved = paletteMap.get(layerId)
          return {
            id: layerId,
            label: resolved?.label ?? layerId,
            background_color: resolved?.background_color ?? '#f7f7f8',
            text_color: resolved?.text_color ?? '#0f172a',
            border_color: resolved?.border_color ?? '#dddddd'
          }
        })

      return {
        nodes,
        edges,
        layers: resolvedLayers
      }
    } catch (error) {
      console.error('Failed to parse dataset graph JSON', error)
      return null
    }
  }, [dataSource, projectPaletteLayers])

  const rawGraphData = useMemo((): GraphData | null => {
    if (!dataSource?.graphJson) {
      return null
    }
    try {
      return JSON.parse(dataSource.graphJson)
    } catch (error) {
      console.error('Failed to parse dataset graph JSON', error)
      return null
    }
  }, [dataSource?.graphJson])

  const noopPaletteSave = useCallback(async (_graphData: GraphData) => {}, [])

  // Form for editing DataSet metadata
  const form = useForm<{name: string; description: string}>({
    defaultValues: {
      name: '',
      description: ''
    }
  })

  // Update form when dataSource loads
  useEffect(() => {
    if (dataSource) {
      form.reset({
        name: dataSource.name,
        description: dataSource.description || ''
      })
    }
  }, [dataSource, form])

  useEffect(() => {
    if (initializedTabRef.current) {
      return
    }
    if (paletteViewRequested) {
      setActiveTab('palette')
    }
    initializedTabRef.current = true
  }, [paletteViewRequested])

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleBack = () => {
    navigate(`/projects/${projectId}/datasets`)
  }

  // Convert file to base64
  const fileToBase64 = (file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const result = reader.result as string
        // Remove the data URL prefix (e.g., "data:text/csv;base64,")
        const base64 = result.split(',')[1]
        resolve(base64)
      }
      reader.onerror = reject
      reader.readAsDataURL(file)
    })
  }

  const handleSave = async (values: { name: string; description: string }) => {
    if (!dataSource) return

    try {
      let input: UpdateDataSetInput

      if (selectedFile) {
        // If updating with new file, convert to base64
        const fileContent = await fileToBase64(selectedFile)
        input = {
          name: values.name,
          description: values.description || undefined,
          filename: selectedFile.name,
          fileContent
        }
      } else {
        // If just updating metadata
        input = {
          name: values.name,
          description: values.description || undefined
        }
      }

      await updateDataSet({
        variables: {
          id: dataSource.id,
          input
        }
      })

      await refetchDataSet()
      setSelectedFile(null)
      setFileUploadMode(false)
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to update DataSet:', error)
      // TODO: Show error notification
    }
  }

  const handleReprocess = async () => {
    if (!dataSource) return

    try {
      await reprocessDataSet({
        variables: { id: dataSource.id }
      })
      await refetchDataSet()
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to reprocess DataSet:', error)
      // TODO: Show error notification
    }
  }

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      setSelectedFile(file)
    }
  }

  const handleDownloadRaw = () => {
    if (!dataSource) return
    // TODO: Implement file download via GraphQL endpoint
    console.log('Download raw file for:', dataSource.filename)
  }

  const handleDownloadJson = () => {
    if (!dataSource) return

    const blob = new Blob([dataSource.graphJson], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${dataSource.name}_graph.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const handleSaveGraphData = async (graphData: GraphData) => {
    if (!dataSource) return

    try {
      const graphJson = JSON.stringify({
        nodes: graphData.nodes,
        edges: graphData.edges,
        layers: graphData.layers
      })
      await updateDataSetGraphData({
        variables: {
          id: dataSource.id,
          graphJson
        }
      })
      await refetchDataSet()
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to save graph data:', error)
      // TODO: Show error notification
      throw error
    }
  }

  const getStatusIcon = (status: DataSet['status']) => {
    switch (status) {
      case 'active':
        return <IconCheck size={16} />
      case 'processing':
        return <IconClock size={16} />
      case 'error':
        return <IconX size={16} />
      default:
        return null
    }
  }

  if (dataSourceLoading) {
    return (
      <PageContainer className="py-12">
        <div className="flex items-center justify-center" style={{ height: '400px' }}>
          <Spinner className="h-8 w-8" />
        </div>
      </PageContainer>
    )
  }

  if (dataSourceError || !dataSource) {
    return (
      <PageContainer className="py-12">
        <Alert variant="destructive" className="mb-4">
          <IconAlertCircle className="h-4 w-4" />
          <AlertTitle>Error loading data set</AlertTitle>
          <AlertDescription>
            {dataSourceError?.message || 'Data Set not found'}
          </AlertDescription>
        </Alert>
        <Button onClick={handleBack}>
          <IconArrowLeft className="mr-2 h-4 w-4" />
          Back to data sets
        </Button>
      </PageContainer>
    )
  }

  if (!selectedProject) {
    return (
      <PageContainer className="py-12">
        <h1 className="text-3xl font-bold">Project Not Found</h1>
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to projects
        </Button>
      </PageContainer>
    )
  }

  return (
    <PageContainer className="py-8">
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        sections={[
          { title: 'Data management', href: `/projects/${selectedProject.id}/datasets` },
          { title: 'Data sets', href: `/projects/${selectedProject.id}/datasets` },
        ]}
        currentPage={dataSource.name}
        onNavigate={handleNavigate}
      />

      <Group justify="between" className="mb-4">
        <div>
          <h1 className="text-3xl font-bold">{dataSource.name}</h1>
          <Group gap="xs" className="mt-2">
            <Badge variant="secondary" className="text-xs">
              {getFileFormatDisplayName(dataSource.fileFormat)}
            </Badge>
            <Badge
              variant="secondary"
              className={
                dataSource.status === 'active'
                  ? 'bg-green-100 text-green-900'
                  : dataSource.status === 'processing'
                    ? 'bg-blue-100 text-blue-900'
                    : 'bg-red-100 text-red-900'
              }
            >
              {getStatusIcon(dataSource.status)}
              <span className="ml-1">{dataSource.status}</span>
            </Badge>
          </Group>
        </div>

        <Group gap="sm">
          <Button
            variant="secondary"
            onClick={handleReprocess}
            disabled={dataSource.status === 'processing' || reprocessLoading}
          >
            {reprocessLoading && <Spinner className="mr-2 h-4 w-4" />}
            <IconRefresh className="mr-2 h-4 w-4" />
            Reprocess
          </Button>
          <Button
            variant="secondary"
            onClick={handleDownloadRaw}
          >
            <IconDownload className="mr-2 h-4 w-4" />
            Download Original
          </Button>
          <Button
            variant="secondary"
            onClick={handleDownloadJson}
            disabled={dataSource.status !== 'active'}
          >
            Download JSON
          </Button>
        </Group>
      </Group>

      {dataSource.status === 'error' && dataSource.errorMessage && (
        <Alert variant="destructive" className="mb-4">
          <IconAlertCircle className="h-4 w-4" />
          <AlertTitle>Processing Error</AlertTitle>
          <AlertDescription>{dataSource.errorMessage}</AlertDescription>
        </Alert>
      )}

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="details">
            <IconFile className="mr-2 h-4 w-4" />
            Details
          </TabsTrigger>
          <TabsTrigger
            value="data"
            disabled={dataSource.status !== 'active'}
          >
            <IconCode className="mr-2 h-4 w-4" />
            Graph Data
          </TabsTrigger>
          <TabsTrigger
            value="edit"
            disabled={dataSource.status !== 'active'}
          >
            <IconTable className="mr-2 h-4 w-4" />
            Data Edit
          </TabsTrigger>
          <TabsTrigger value="palette">
            <IconPalette className="mr-2 h-4 w-4" />
            Palette View
          </TabsTrigger>
        </TabsList>

        <TabsContent value="details">
          <Card className="border mt-4">
            <CardContent className="pt-6">
              <form onSubmit={form.handleSubmit(handleSave)}>
                <Stack gap="md">
                  <div className="space-y-2">
                    <Label htmlFor="name">Name *</Label>
                    <Input
                      id="name"
                      placeholder="Enter data source name"
                      {...form.register('name', { required: true })}
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="description">Description</Label>
                    <Textarea
                      id="description"
                      placeholder="Optional description"
                      rows={3}
                      {...form.register('description')}
                    />
                  </div>

                  <div>
                    <p className="text-sm font-medium mb-2">File Information</p>
                    <Group gap="md">
                      <div>
                        <p className="text-xs text-muted-foreground">Filename</p>
                        <p className="text-sm font-mono">{dataSource.filename}</p>
                      </div>
                      <div>
                        <p className="text-xs text-muted-foreground">Size</p>
                        <p className="text-sm">{formatFileSize(dataSource.fileSize)}</p>
                      </div>
                      <div>
                        <p className="text-xs text-muted-foreground">Processed</p>
                        <p className="text-sm">
                          {dataSource.processedAt
                            ? new Date(dataSource.processedAt).toLocaleString()
                            : 'Not processed'
                          }
                        </p>
                      </div>
                    </Group>
                  </div>

                  <div>
                    <Group justify="between" align="center" className="mb-2">
                      <p className="text-sm font-medium">Replace File</p>
                      <Button
                        type="button"
                        variant="secondary"
                        size="sm"
                        onClick={() => setFileUploadMode(!fileUploadMode)}
                      >
                        {fileUploadMode ? 'Cancel' : 'Upload New File'}
                      </Button>
                    </Group>

                    {fileUploadMode && (
                      <Stack gap="sm">
                        <Input
                          type="file"
                          accept=".csv,.json"
                          onChange={handleFileChange}
                        />
                          {selectedFile && (
                            <p className="text-sm text-muted-foreground">
                              Selected: {selectedFile.name} ({formatFileSize(selectedFile.size)})
                            </p>
                          )}
                          <p className="text-xs text-muted-foreground">
                            Supported formats: CSV (nodes, edges, layers) and JSON (graph format)
                          </p>
                        </Stack>
                      )}
                  </div>

                  <Group justify="end">
                    <Button
                      type="submit"
                      disabled={updateLoading}
                    >
                      {updateLoading && <Spinner className="mr-2 h-4 w-4" />}
                      <IconDeviceFloppy className="mr-2 h-4 w-4" />
                      Save Changes
                    </Button>
                  </Group>
                </Stack>
              </form>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="data">
          <Card className="border mt-4">
            <CardContent className="pt-6">
              <Stack gap="md">
                <Group justify="between">
                  <p className="font-medium">Processed Graph Data</p>
                  <p className="text-sm text-muted-foreground">
                    {dataSource.graphJson.length} characters
                  </p>
                </Group>

                <ScrollArea className="h-[400px]">
                  <pre className="text-xs bg-muted p-4 rounded-md overflow-x-auto">
                    <code>{JSON.stringify(JSON.parse(dataSource.graphJson), null, 2)}</code>
                  </pre>
                </ScrollArea>

                <p className="text-xs text-muted-foreground">
                  This is the processed graph data that will be available to Plan DAG nodes.
                  The format includes nodes, edges, and layers arrays as defined by the graph schema.
                </p>
              </Stack>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="edit">
          <Card className="border mt-4">
            <CardContent className="pt-6">
              {dataSource.status === 'active' ? (
                rawGraphData ? (
                  <GraphSpreadsheetEditor
                    graphData={rawGraphData}
                    onSave={handleSaveGraphData}
                  />
                ) : (
                  <Alert variant="destructive">
                    <IconAlertCircle className="h-4 w-4" />
                    <AlertTitle>Invalid Graph Data</AlertTitle>
                    <AlertDescription>
                      Failed to parse graph JSON data. Please check the data format in the "Graph Data" tab.
                    </AlertDescription>
                  </Alert>
                )
              ) : (
                <Alert>
                  <IconAlertCircle className="h-4 w-4" />
                  <AlertTitle>Dataset editing disabled</AlertTitle>
                  <AlertDescription>
                    Graph data can only be edited when the dataset status is active.
                  </AlertDescription>
                </Alert>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="palette">
          <Card className="border mt-4">
            <CardContent className="pt-6">
              {projectLayersLoading && (
                <Stack align="center" className="py-12">
                  <Spinner size="lg" />
                  <p className="text-sm text-muted-foreground">Loading layer palette…</p>
                </Stack>
              )}
              {projectLayersError && (
                <Alert variant="destructive">
                  <IconAlertCircle className="h-4 w-4" />
                  <AlertTitle>Failed to load palette</AlertTitle>
                  <AlertDescription>{projectLayersError.message}</AlertDescription>
                </Alert>
              )}
              {!projectLayersLoading && !projectLayersError && (
                resolvedGraphData ? (
                  <GraphSpreadsheetEditor
                    graphData={resolvedGraphData}
                    onSave={noopPaletteSave}
                    readOnly
                    layersReadOnly
                  />
                ) : (
                  <Alert variant="destructive">
                    <IconAlertCircle className="h-4 w-4" />
                    <AlertTitle>Invalid Graph Data</AlertTitle>
                    <AlertDescription>
                      Failed to parse graph JSON data. Please check the data format in the "Graph Data" tab.
                    </AlertDescription>
                  </Alert>
                )
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </PageContainer>
  )
}
