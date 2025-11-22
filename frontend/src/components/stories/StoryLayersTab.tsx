import { useMemo } from 'react'
import { useQuery } from '@apollo/client/react'
import { IconStack2 } from '@tabler/icons-react'
import { Group } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { GET_PROJECT_LAYERS, ProjectLayer } from '@/graphql/layers'
import { StoryLayerConfig } from '@/graphql/stories'
import { DataSet } from '@/graphql/datasets'

interface StoryLayersTabProps {
  projectId: number
  layerConfig: StoryLayerConfig[]
  enabledDatasetIds: number[]
  datasets: DataSet[]
  onLayerConfigChange: (config: StoryLayerConfig[]) => void
}

export const StoryLayersTab = ({
  projectId,
  layerConfig,
  enabledDatasetIds,
  datasets,
  onLayerConfigChange,
}: StoryLayersTabProps) => {
  const { data: layersData, loading: layersLoading } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId },
    skip: !projectId,
  })

  const projectLayers: ProjectLayer[] = useMemo(
    () => ((layersData as any)?.projectLayers as ProjectLayer[] | undefined) ?? [],
    [layersData]
  )

  // Filter to only enabled layers from project
  const enabledProjectLayers = useMemo(
    () => projectLayers.filter((l) => l.enabled),
    [projectLayers]
  )

  // Filter datasets to only enabled ones
  const storyDatasets = useMemo(
    () => datasets.filter((d) => enabledDatasetIds.includes(d.id)),
    [datasets, enabledDatasetIds]
  )

  // Build a map of current layer configs for quick lookup
  const configMap = useMemo(() => {
    const map: Record<string, StoryLayerConfig> = {}
    for (const config of layerConfig) {
      map[config.layerId] = config
    }
    return map
  }, [layerConfig])

  // Get or create config for a layer
  const getLayerConfig = (layerId: string, projectLayer: ProjectLayer): StoryLayerConfig => {
    if (configMap[layerId]) {
      return configMap[layerId]
    }
    // Default config from project layer
    return {
      layerId,
      enabled: true,
      color: projectLayer.backgroundColor,
      sourceDatasetId: projectLayer.sourceDatasetId ?? null,
    }
  }

  // Update a specific layer's config
  const updateLayerConfig = (layerId: string, updates: Partial<StoryLayerConfig>) => {
    const existing = configMap[layerId]
    const projectLayer = projectLayers.find((l) => l.layerId === layerId)

    const baseConfig = existing || {
      layerId,
      enabled: true,
      color: projectLayer?.backgroundColor || '#ffffff',
      sourceDatasetId: projectLayer?.sourceDatasetId || null,
    }

    const updated: StoryLayerConfig = {
      ...baseConfig,
      ...updates,
    }

    // Update the config array
    const newConfig = layerConfig.filter((c) => c.layerId !== layerId)
    newConfig.push(updated)
    onLayerConfigChange(newConfig)
  }

  if (layersLoading) {
    return (
      <Card className="border mt-4">
        <CardContent className="py-8">
          <Group gap="sm" align="center" justify="center">
            <Spinner className="h-4 w-4" />
            <span>Loading layers...</span>
          </Group>
        </CardContent>
      </Card>
    )
  }

  if (enabledProjectLayers.length === 0) {
    return (
      <Card className="border mt-4">
        <CardHeader>
          <CardTitle className="text-base">Layer Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            No layers are configured for this project. Configure project layers first in the Workbench → Layers page.
          </p>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="border mt-4">
      <CardHeader>
        <Group justify="between" align="center">
          <CardTitle className="text-base flex items-center gap-2">
            <IconStack2 className="h-4 w-4" />
            Layer Configuration
          </CardTitle>
          <Badge variant="secondary">
            {enabledProjectLayers.length} layer{enabledProjectLayers.length !== 1 ? 's' : ''}
          </Badge>
        </Group>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground mb-4">
          Configure how layers appear in this story. Override colours and select which dataset provides each layer's data.
        </p>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[50px]">Enabled</TableHead>
              <TableHead>Layer</TableHead>
              <TableHead className="w-[100px]">Colour</TableHead>
              <TableHead className="w-[200px]">Source Dataset</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {enabledProjectLayers.map((projectLayer) => {
              const config = getLayerConfig(projectLayer.layerId, projectLayer)
              return (
                <TableRow key={projectLayer.id}>
                  <TableCell>
                    <Checkbox
                      checked={config.enabled}
                      onCheckedChange={(checked) =>
                        updateLayerConfig(projectLayer.layerId, { enabled: !!checked })
                      }
                    />
                  </TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{projectLayer.name}</div>
                      <div className="text-xs text-muted-foreground font-mono">
                        {projectLayer.layerId}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Input
                      type="color"
                      className="w-16 h-8 p-1 cursor-pointer"
                      value={config.color || projectLayer.backgroundColor || '#ffffff'}
                      onChange={(e) =>
                        updateLayerConfig(projectLayer.layerId, { color: e.target.value })
                      }
                      disabled={!config.enabled}
                    />
                  </TableCell>
                  <TableCell>
                    <Select
                      value={config.sourceDatasetId?.toString() || 'none'}
                      onValueChange={(value) =>
                        updateLayerConfig(projectLayer.layerId, {
                          sourceDatasetId: value === 'none' ? null : Number(value),
                        })
                      }
                      disabled={!config.enabled}
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="Select dataset" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">
                          <span className="text-muted-foreground">None (use project default)</span>
                        </SelectItem>
                        {storyDatasets.map((dataset) => (
                          <SelectItem key={dataset.id} value={dataset.id.toString()}>
                            {dataset.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>

        {storyDatasets.length === 0 && (
          <p className="text-sm text-muted-foreground mt-4 italic">
            No datasets enabled for this story. Enable datasets in the Details tab to use them as layer sources.
          </p>
        )}
      </CardContent>
    </Card>
  )
}

export default StoryLayersTab
