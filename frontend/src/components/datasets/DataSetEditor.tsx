import React, { useState, useEffect, useMemo, useCallback } from 'react'
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
  IconShieldCheck,
  IconHierarchy2,
} from '@tabler/icons-react'
import { useForm } from 'react-hook-form'
import { Breadcrumbs } from '../common/Breadcrumbs'
import {
  GET_DATASOURCE,
  UPDATE_DATASOURCE,
  REPROCESS_DATASOURCE,
  UPDATE_DATASOURCE_GRAPH_DATA,
  VALIDATE_DATASET,
  DataSet,
  DataSetValidationResult,
  UpdateDataSetInput,
  formatFileSize,
  getFileFormatDisplayName
} from '../../graphql/datasets'
import { GraphSpreadsheetEditor, GraphData } from '../editors/GraphSpreadsheetEditor'
import { HierarchyTreeEditor } from './HierarchyTreeEditor'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '../ui/dialog'
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

  const allowedTabs = ['details', 'data', 'edit', 'hierarchy'] as const
  type TabValue = typeof allowedTabs[number]
  const searchParams = useMemo(() => new URLSearchParams(location.search), [location.search])
  const tabParam = searchParams.get('tab') as TabValue | null
  const activeTab: TabValue = tabParam && allowedTabs.includes(tabParam) ? tabParam : 'details'
  const hierarchySplitView = searchParams.get('split') === '1'
  const [fileUploadMode, setFileUploadMode] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [validationDialogOpen, setValidationDialogOpen] = useState(false)
  const [validationResult, setValidationResult] = useState<DataSetValidationResult | null>(null)
  const [validationError, setValidationError] = useState<string | null>(null)

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

  // Mutations
  const [updateDataSet, { loading: updateLoading }] = useMutation(UPDATE_DATASOURCE)
  const [reprocessDataSet, { loading: reprocessLoading }] = useMutation(REPROCESS_DATASOURCE)
  const [updateDataSetGraphData] = useMutation(UPDATE_DATASOURCE_GRAPH_DATA)
  const [validateDataSetMutation, { loading: validateLoading }] = useMutation<{
    validateDataSet: DataSetValidationResult
  }, { id: number }>(VALIDATE_DATASET)

  const dataSource: DataSet | null = (dataSourceData as any)?.dataSet || null
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

  const setQueryParams = useCallback(
    (mutator: (params: URLSearchParams) => void) => {
      const params = new URLSearchParams(location.search)
      mutator(params)
      const next = params.toString()
      navigate(next ? `${location.pathname}?${next}` : location.pathname, { replace: true })
    },
    [location.pathname, location.search, navigate]
  )

  const handleTabChange = useCallback(
    (value: string) => {
      if (!allowedTabs.includes(value as TabValue)) return
      setQueryParams(params => {
        params.set('tab', value)
        if (value !== 'hierarchy') {
          params.delete('split')
        }
      })
    },
    [setQueryParams]
  )

  const handleToggleSplitView = useCallback(() => {
    setQueryParams(params => {
      const nextState = !hierarchySplitView
      if (nextState) {
        params.set('split', '1')
        params.set('tab', 'hierarchy')
      } else {
        params.delete('split')
      }
    })
  }, [hierarchySplitView, setQueryParams])

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

  const handleValidateDataSet = useCallback(async () => {
    if (!dataSetId) return

    try {
      const { data } = await validateDataSetMutation({
        variables: { id: parseInt(dataSetId, 10) }
      })
      setValidationResult(data?.validateDataSet ?? null)
      setValidationError(null)
      setValidationDialogOpen(true)
    } catch (error) {
      console.error('Failed to validate DataSet:', error)
      setValidationResult(null)
      setValidationError(error instanceof Error ? error.message : 'Failed to validate dataset')
      setValidationDialogOpen(true)
    }
  }, [dataSetId, validateDataSetMutation])

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

      <Tabs value={activeTab} onValueChange={handleTabChange}>
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
          <TabsTrigger
            value="hierarchy"
            disabled={dataSource.status !== 'active'}
          >
            <IconHierarchy2 className="mr-2 h-4 w-4" />
            Hierarchy
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

                  <Group justify="end" gap="sm">
                    <Button
                      type="button"
                      variant="outline"
                      disabled={validateLoading || !dataSource}
                      onClick={handleValidateDataSet}
                    >
                      {validateLoading && <Spinner className="mr-2 h-4 w-4" />}
                      <IconShieldCheck className="mr-2 h-4 w-4" />
                      Validate dataset
                    </Button>
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

        <TabsContent value="hierarchy">
          <div className="mt-4 h-[calc(100vh-280px)] min-h-[520px]">
            <HierarchyTreeEditor
              graphData={rawGraphData}
              onSave={handleSaveGraphData}
              splitView={hierarchySplitView}
              onToggleSplitView={handleToggleSplitView}
            />
          </div>
        </TabsContent>

      </Tabs>

      <Dialog open={validationDialogOpen} onOpenChange={setValidationDialogOpen}>
        <DialogContent className="max-w-xl">
          <DialogHeader>
            <DialogTitle>Dataset validation</DialogTitle>
            {validationResult?.checkedAt && (
              <DialogDescription>
                Checked {new Date(validationResult.checkedAt).toLocaleString()}
              </DialogDescription>
            )}
          </DialogHeader>

          {validationError && (
            <Alert variant="destructive" className="mb-4">
              <IconAlertCircle className="h-4 w-4" />
              <AlertTitle>Validation failed</AlertTitle>
              <AlertDescription>{validationError}</AlertDescription>
            </Alert>
          )}

          {validationResult && (
            <Stack gap="md">
              <Group gap="sm" align="center">
                <Badge variant={validationResult.isValid ? 'secondary' : 'destructive'}>
                  {validationResult.isValid ? 'Valid' : 'Invalid'}
                </Badge>
                <span className="text-sm text-muted-foreground">
                  Nodes: {validationResult.nodeCount} · Edges: {validationResult.edgeCount} · Layers: {validationResult.layerCount}
                </span>
              </Group>

              <div>
                <p className="text-sm font-medium mb-1">Issues</p>
                {validationResult.errors.length === 0 ? (
                  <p className="text-sm text-muted-foreground">
                    All structural checks passed. No blocking issues detected.
                  </p>
                ) : (
                  <ul className="text-sm list-disc pl-5 space-y-1">
                    {validationResult.errors.map((error, idx) => (
                      <li key={`${error}-${idx}`}>{error}</li>
                    ))}
                  </ul>
                )}
              </div>

              {validationResult.warnings.length > 0 && (
                <div>
                  <p className="text-sm font-medium mb-1">Warnings</p>
                  <ul className="text-sm list-disc pl-5 space-y-1 text-muted-foreground">
                    {validationResult.warnings.map((warning, idx) => (
                      <li key={`${warning}-${idx}`}>{warning}</li>
                    ))}
                  </ul>
                </div>
              )}
            </Stack>
          )}

          <DialogFooter className="mt-6">
            <Button onClick={() => setValidationDialogOpen(false)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}
