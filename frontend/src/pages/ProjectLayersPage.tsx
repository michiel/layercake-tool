import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useMutation, useQuery } from '@apollo/client/react'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { GET_DATASOURCES } from '@/graphql/datasets'
import {
  DELETE_PROJECT_LAYER,
  GET_PROJECT_LAYERS,
  ProjectLayer,
  ProjectLayerInput,
  SET_LAYER_DATASET_ENABLED,
  UPSERT_PROJECT_LAYER,
} from '@/graphql/layers'
import { IconLayersIntersect } from '@tabler/icons-react'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'

const DEFAULT_LAYER_COLORS = {
  backgroundColor: '#ffffff',
  textColor: '#000000',
  borderColor: '#000000',
}

export const ProjectLayersPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectIdNum = Number(projectId || 0)

  const {
    data: layersData,
    loading: layersLoading,
    refetch: refetchLayers,
  } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'network-only',
    nextFetchPolicy: 'cache-and-network',
  })

  const { data: datasetsData, loading: datasetsLoading, refetch: refetchDatasets } = useQuery(
    GET_DATASOURCES,
    {
      variables: { projectId: projectIdNum },
      skip: !projectIdNum,
    }
  )

  const [upsertLayer, { loading: upserting }] = useMutation(UPSERT_PROJECT_LAYER)
  const [deleteLayer] = useMutation(DELETE_PROJECT_LAYER)
  const [setDatasetEnabled, { loading: togglingDataset }] = useMutation(SET_LAYER_DATASET_ENABLED)

  const projectLayers: ProjectLayer[] = useMemo(
    () => ((layersData as any)?.projectLayers as ProjectLayer[] | undefined) ?? [],
    [layersData]
  )
  const missingLayers: string[] = useMemo(
    () => ((layersData as any)?.missingLayers as string[] | undefined) ?? [],
    [layersData]
  )
  const layerDatasets = useMemo(
    () =>
      ((datasetsData as any)?.dataSets as any[] | undefined)?.filter(
        (ds: any) => ds.dataType?.toLowerCase() === 'layers'
      ) ?? [],
    [datasetsData]
  )

  const [editableLayers, setEditableLayers] = useState<ProjectLayer[]>([])
  const [newLayer, setNewLayer] = useState<ProjectLayerInput>({
    layerId: '',
    name: '',
    ...DEFAULT_LAYER_COLORS,
  })
  const [datasetToggleState, setDatasetToggleState] = useState<Record<number, boolean>>({})

  useEffect(() => {
    setEditableLayers(projectLayers)
    const nextState: Record<number, boolean> = {}
    projectLayers
      .filter((l) => l.sourceDatasetId)
      .forEach((l) => {
        nextState[l.sourceDatasetId as number] = nextState[l.sourceDatasetId as number] || l.enabled
      })
    setDatasetToggleState((prev) => ({ ...prev, ...nextState }))
  }, [projectLayers])

  const handleSaveLayer = async (layer: ProjectLayerInput) => {
    try {
      await upsertLayer({
        variables: {
          projectId: projectIdNum,
          input: {
            layerId: layer.layerId,
            name: layer.name,
            backgroundColor: layer.backgroundColor ?? DEFAULT_LAYER_COLORS.backgroundColor,
            textColor: layer.textColor ?? DEFAULT_LAYER_COLORS.textColor,
            borderColor: layer.borderColor ?? DEFAULT_LAYER_COLORS.borderColor,
            sourceDatasetId: layer.sourceDatasetId ?? null,
            enabled: layer.enabled ?? true,
          },
        },
      })
      const refreshed = await refetchLayers()
      const refreshedLayers: ProjectLayer[] = (refreshed.data as any)?.projectLayers ?? []
      if (refreshedLayers.length) {
        setEditableLayers(refreshedLayers)
      }
      showSuccessNotification('Layer saved', `Layer ${layer.layerId} updated`)
    } catch (error: any) {
      showErrorNotification('Failed to save layer', error?.message || 'Unknown error')
    }
  }

  const handleDeleteLayer = async (layer: ProjectLayer) => {
    try {
      await deleteLayer({
        variables: {
          projectId: projectIdNum,
          layerId: layer.layerId,
          sourceDatasetId: layer.sourceDatasetId,
        },
      })
      await refetchLayers()
      showSuccessNotification('Layer deleted', `Layer ${layer.layerId} removed`)
    } catch (error: any) {
      showErrorNotification('Failed to delete layer', error?.message || 'Unknown error')
    }
  }

  const handleToggleDataset = async (datasetId: number, enabled: boolean) => {
    setDatasetToggleState((prev) => ({ ...prev, [datasetId]: enabled }))
    try {
      await setDatasetEnabled({
        variables: { projectId: projectIdNum, dataSetId: datasetId, enabled },
      })
      const [refetchedLayers] = await Promise.all([refetchLayers(), refetchDatasets()])
      const layers: ProjectLayer[] = (refetchedLayers.data as any)?.projectLayers ?? []
      if (layers.length) {
        setEditableLayers(layers)
      }
      const nextState: Record<number, boolean> = {}
      layers
        .filter((l) => l.sourceDatasetId)
        .forEach((l) => {
          nextState[l.sourceDatasetId as number] = nextState[l.sourceDatasetId as number] || l.enabled
        })
      setDatasetToggleState((prev) => ({ ...prev, ...nextState }))
      showSuccessNotification(
        enabled ? 'Dataset layers enabled' : 'Dataset layers disabled',
        enabled ? 'Imported layer definitions' : 'Disabled dataset-provided layers'
      )
    } catch (error: any) {
      setDatasetToggleState((prev) => ({ ...prev, [datasetId]: !enabled }))
      showErrorNotification('Failed to toggle dataset layers', error?.message || 'Unknown error')
    }
  }

  const handleAddMissing = async (layerId: string) => {
    const name = layerId
    await handleSaveLayer({
      layerId,
      name,
      ...DEFAULT_LAYER_COLORS,
      enabled: true,
    })
  }

  const handleBulkMissing = async () => {
    for (const id of missingLayers) {
      // Best-effort; stop on error
      await handleAddMissing(id)
    }
  }

  if (!projectIdNum) {
    return null
  }

  const loading = layersLoading || datasetsLoading || upserting || togglingDataset

  return (
    <PageContainer>
      <div className="relative">
        {loading && (
          <div className="pointer-events-none absolute inset-x-0 top-0 z-10">
            <div className="h-1 w-full overflow-hidden rounded bg-muted">
              <div className="h-full w-1/2 animate-pulse bg-primary" />
            </div>
          </div>
        )}

      <Breadcrumbs
        projectId={projectIdNum}
        projectName={`Project ${projectIdNum}`}
        currentPage="Layers"
        onNavigate={(route) => navigate(route)}
        sections={[
          { title: 'Workbench', href: `/projects/${projectIdNum}/workbench` },
          { title: 'Layers', href: `/projects/${projectIdNum}/workbench/layers` },
        ]}
      />

      <Group justify="between" className="mb-4">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <IconLayersIntersect className="h-6 w-6 text-primary" />
            Layers
          </h1>
          <p className="text-muted-foreground">
            Manage project-wide layers, import from datasets, and address missing layer references.
          </p>
        </div>
        <Group gap="sm">
          <Button variant="secondary" onClick={() => navigate(-1)}>
            Back
          </Button>
        </Group>
      </Group>

      <Tabs defaultValue="sources" className="space-y-4">
        <TabsList>
          <TabsTrigger value="sources">Sources</TabsTrigger>
          <TabsTrigger value="palette">Palette</TabsTrigger>
          <TabsTrigger value="missing">
            Missing {missingLayers.length > 0 && <Badge className="ml-1">{missingLayers.length}</Badge>}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="sources">
          <Card>
            <CardHeader>
              <CardTitle>Layer datasets</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {layerDatasets.length === 0 && (
                <p className="text-sm text-muted-foreground">No layer datasets available.</p>
              )}
                <Stack gap="md">
                  {layerDatasets.map((dataset: any) => {
                  const enabled =
                    datasetToggleState[dataset.id] ??
                    projectLayers.some(
                      (l) => l.sourceDatasetId === dataset.id && l.enabled
                    )
                  return (
                    <div
                      key={dataset.id}
                      className="flex items-center justify-between rounded border p-3"
                    >
                      <div>
                        <p className="font-medium">{dataset.name}</p>
                        <p className="text-sm text-muted-foreground">{dataset.filename}</p>
                      </div>
                      <Group gap="sm" align="center">
                        <Switch
                          checked={enabled}
                          disabled={togglingDataset}
                          onCheckedChange={(checked) => handleToggleDataset(dataset.id, checked)}
                        />
                        <span className="text-sm text-muted-foreground">
                          {enabled ? 'Enabled' : 'Disabled'}
                        </span>
                      </Group>
                    </div>
                  )
                })}
              </Stack>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="palette">
          <Card>
            <CardHeader>
              <CardTitle>Project palette</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-3">
                <Label>Add layer</Label>
                <div className="grid gap-3 md:grid-cols-5">
                  <Input
                    placeholder="Layer ID"
                    value={newLayer.layerId}
                    onChange={(e) => setNewLayer((prev) => ({ ...prev, layerId: e.target.value }))}
                  />
                  <Input
                    placeholder="Name"
                    value={newLayer.name}
                    onChange={(e) => setNewLayer((prev) => ({ ...prev, name: e.target.value }))}
                  />
                  <Input
                    type="color"
                    value={newLayer.backgroundColor || DEFAULT_LAYER_COLORS.backgroundColor}
                    onChange={(e) =>
                      setNewLayer((prev) => ({ ...prev, backgroundColor: e.target.value }))
                    }
                    title="Background"
                  />
                  <Input
                    type="color"
                    value={newLayer.textColor || DEFAULT_LAYER_COLORS.textColor}
                    onChange={(e) =>
                      setNewLayer((prev) => ({ ...prev, textColor: e.target.value }))
                    }
                    title="Text"
                  />
                  <Button
                    onClick={() => {
                      if (!newLayer.layerId || !newLayer.name) {
                        showErrorNotification('Layer ID and name are required', '')
                        return
                      }
                      handleSaveLayer(newLayer)
                      setNewLayer({ layerId: '', name: '', ...DEFAULT_LAYER_COLORS })
                    }}
                  >
                    Add
                  </Button>
                </div>
              </div>

              <Separator />

              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Layer ID</TableHead>
                    <TableHead>Name</TableHead>
                    <TableHead>Background</TableHead>
                    <TableHead>Text</TableHead>
                    <TableHead>Border</TableHead>
                    <TableHead>Source</TableHead>
                    <TableHead>Enabled</TableHead>
                    <TableHead />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {editableLayers.map((layer) => (
                    <TableRow key={`${layer.layerId}-${layer.sourceDatasetId ?? 'manual'}`}>
                      <TableCell className="font-mono">{layer.layerId}</TableCell>
                      <TableCell>
                        <Input
                          value={layer.name}
                          onChange={(e) =>
                            setEditableLayers((prev) =>
                              prev.map((l) =>
                                l.id === layer.id ? { ...l, name: e.target.value } : l
                              )
                            )
                          }
                        />
                      </TableCell>
                      <TableCell>
                        <Input
                          type="color"
                          value={layer.backgroundColor}
                          onChange={(e) =>
                            setEditableLayers((prev) =>
                              prev.map((l) =>
                                l.id === layer.id
                                  ? { ...l, backgroundColor: e.target.value }
                                  : l
                              )
                            )
                          }
                        />
                      </TableCell>
                      <TableCell>
                        <Input
                          type="color"
                          value={layer.textColor}
                          onChange={(e) =>
                            setEditableLayers((prev) =>
                              prev.map((l) =>
                                l.id === layer.id ? { ...l, textColor: e.target.value } : l
                              )
                            )
                          }
                        />
                      </TableCell>
                      <TableCell>
                        <Input
                          type="color"
                          value={layer.borderColor}
                          onChange={(e) =>
                            setEditableLayers((prev) =>
                              prev.map((l) =>
                                l.id === layer.id ? { ...l, borderColor: e.target.value } : l
                              )
                            )
                          }
                        />
                      </TableCell>
                      <TableCell>
                        {layer.sourceDatasetId ? (
                          <Badge variant="secondary">Dataset #{layer.sourceDatasetId}</Badge>
                        ) : (
                          <Badge variant="secondary">Manual</Badge>
                        )}
                      </TableCell>
                      <TableCell>
                        <Switch
                          checked={layer.enabled}
                          onCheckedChange={(checked) =>
                            setEditableLayers((prev) =>
                              prev.map((l) =>
                                l.id === layer.id ? { ...l, enabled: checked } : l
                              )
                            )
                          }
                        />
                      </TableCell>
                      <TableCell className="space-x-2">
                        <Button
                          size="sm"
                          variant="secondary"
                          onClick={() => {
                            const latest = editableLayers.find((l) => l.id === layer.id) || layer
                            handleSaveLayer({
                              layerId: latest.layerId,
                              name: latest.name,
                              backgroundColor: latest.backgroundColor,
                              textColor: latest.textColor,
                              borderColor: latest.borderColor,
                              sourceDatasetId: latest.sourceDatasetId,
                              enabled: latest.enabled,
                            })
                          }}
                        >
                          Save
                        </Button>
                        {!layer.sourceDatasetId && (
                          <Button size="sm" variant="destructive" onClick={() => handleDeleteLayer(layer)}>
                            Delete
                          </Button>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="missing">
          <Card>
            <CardHeader>
              <CardTitle>Missing layers</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {missingLayers.length === 0 ? (
                <p className="text-sm text-muted-foreground">No missing layers detected.</p>
              ) : (
                <>
                  <Group justify="between">
                    <p className="text-sm text-muted-foreground">
                      {missingLayers.length} layer(s) referenced by nodes/edges are not in the palette.
                    </p>
                    <Button onClick={handleBulkMissing}>Add all</Button>
                  </Group>
                  <Stack gap="sm">
                    {missingLayers.map((layerId) => (
                      <div key={layerId} className="flex items-center justify-between rounded border p-2">
                        <span className="font-mono">{layerId}</span>
                        <Button size="sm" variant="secondary" onClick={() => handleAddMissing(layerId)}>
                          Add
                        </Button>
                      </div>
                    ))}
                  </Stack>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
      </div>
    </PageContainer>
  )
}

export default ProjectLayersPage
