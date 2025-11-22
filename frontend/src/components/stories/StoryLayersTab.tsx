import { useMemo } from 'react'
import { useQuery } from '@apollo/client/react'
import { IconStack2 } from '@tabler/icons-react'
import { Group } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { Switch } from '@/components/ui/switch'
import { GET_PROJECT_LAYERS, ProjectLayer } from '@/graphql/layers'
import { GET_DATASOURCES } from '@/graphql/datasets'
import { StoryLayerConfig } from '@/graphql/stories'

type LayerSourceStyleMode = 'default' | 'light' | 'dark'

interface StoryLayersTabProps {
  projectId: number
  layerConfig: StoryLayerConfig[]
  onLayerConfigChange: (config: StoryLayerConfig[]) => void
}

export const StoryLayersTab = ({
  projectId,
  layerConfig,
  onLayerConfigChange,
}: StoryLayersTabProps) => {
  const { data: layersData, loading: layersLoading } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId },
    skip: !projectId,
  })

  const { data: datasetsData, loading: datasetsLoading } = useQuery(GET_DATASOURCES, {
    variables: { projectId },
    skip: !projectId,
  })

  const projectLayers: ProjectLayer[] = useMemo(
    () => ((layersData as any)?.projectLayers as ProjectLayer[] | undefined) ?? [],
    [layersData]
  )

  // Build layer source options (grouped by dataset)
  const layerSourceOptions = useMemo(() => {
    const enabledLayers = projectLayers.filter((layer) => layer.enabled)
    if (!enabledLayers.length) {
      return []
    }

    // Build dataset name map
    const datasetNameMap = new Map<number, string>()
    const datasets = (datasetsData as any)?.dataSets ?? []
    datasets.forEach((dataset: any) => {
      datasetNameMap.set(dataset.id, dataset.name ?? `Dataset #${dataset.id}`)
    })

    // Group layers by source dataset
    const grouped = new Map<
      string,
      { datasetId: number | null; count: number; label: string; key: string }
    >()

    enabledLayers.forEach((layer) => {
      const datasetId = layer.sourceDatasetId ?? null
      const key = datasetId === null ? 'manual' : datasetId.toString()
      if (!grouped.has(key)) {
        const label =
          datasetId === null
            ? 'Manual layers'
            : datasetNameMap.get(datasetId) ?? `Dataset #${datasetId}`
        grouped.set(key, { datasetId, count: 1, label, key })
      } else {
        const entry = grouped.get(key)!
        entry.count += 1
      }
    })

    return Array.from(grouped.values()).sort((a, b) => a.label.localeCompare(b.label))
  }, [projectLayers, datasetsData])

  // Find override for a source
  const findLayerSourceOverride = (datasetId: number | null) =>
    layerConfig.find(
      (config) => (config.sourceDatasetId ?? null) === (datasetId ?? null)
    )

  // Get the mode for a source
  const getLayerSourceMode = (datasetId: number | null): LayerSourceStyleMode => {
    const existing = findLayerSourceOverride(datasetId)
    return (existing?.mode as LayerSourceStyleMode) ?? 'default'
  }

  // Set override for a source (disables source colours, uses fallback)
  const setLayerSourceOverride = (datasetId: number | null, mode: LayerSourceStyleMode) => {
    const filtered = layerConfig.filter(
      (config) => (config.sourceDatasetId ?? null) !== (datasetId ?? null)
    )
    onLayerConfigChange([...filtered, { sourceDatasetId: datasetId, mode }])
  }

  // Remove override for a source (enables source colours)
  const removeLayerSourceOverride = (datasetId: number | null) => {
    onLayerConfigChange(
      layerConfig.filter(
        (config) => (config.sourceDatasetId ?? null) !== (datasetId ?? null)
      )
    )
  }

  // Toggle source disabled state
  const setLayerSourceDisabled = (datasetId: number | null, disabled: boolean) => {
    if (disabled) {
      const mode = getLayerSourceMode(datasetId)
      setLayerSourceOverride(datasetId, mode)
    } else {
      removeLayerSourceOverride(datasetId)
    }
  }

  const loading = layersLoading || datasetsLoading

  if (loading) {
    return (
      <Card className="border mt-4">
        <CardContent className="py-8">
          <Group gap="sm" align="center" justify="center">
            <Spinner className="h-4 w-4" />
            <span>Loading layer sources...</span>
          </Group>
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
            Layer Sources
          </CardTitle>
          <Badge variant="secondary">
            {layerSourceOptions.length} source{layerSourceOptions.length !== 1 ? 's' : ''}
          </Badge>
        </Group>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground mb-4">
          Disable individual layer sources to use a built-in palette instead of project layer colours.
        </p>

        {!layerSourceOptions.length ? (
          <p className="text-sm text-muted-foreground">
            No enabled layer sources found for this project. Configure layers in the Workbench → Layers page.
          </p>
        ) : (
          <div className="space-y-3">
            {layerSourceOptions.map((option) => {
              const datasetKey = option.key
              const disabled = !!findLayerSourceOverride(option.datasetId ?? null)
              const mode = getLayerSourceMode(option.datasetId ?? null)
              const switchId = `layer-source-disable-${datasetKey}`
              const selectId = `layer-source-style-${datasetKey}`

              return (
                <div key={datasetKey} className="rounded-md border p-3">
                  <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                    <div>
                      <p className="font-medium">{option.label}</p>
                      <p className="text-sm text-muted-foreground">
                        {option.count} {option.count === 1 ? 'layer' : 'layers'}{' '}
                        {option.datasetId === null ? 'added manually' : 'from dataset'}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Switch
                        id={switchId}
                        checked={disabled}
                        onCheckedChange={(checked) =>
                          setLayerSourceDisabled(option.datasetId ?? null, checked)
                        }
                      />
                      <Label htmlFor={switchId} className="text-sm font-normal">
                        Disable source colours
                      </Label>
                    </div>
                  </div>
                  {disabled && (
                    <div className="mt-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-3">
                      <Label htmlFor={selectId} className="text-sm font-medium">
                        Fallback palette
                      </Label>
                      <Select
                        value={mode}
                        onValueChange={(value) =>
                          setLayerSourceOverride(option.datasetId ?? null, value as LayerSourceStyleMode)
                        }
                      >
                        <SelectTrigger id={selectId} className="sm:w-48">
                          <SelectValue placeholder="Select palette" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="default">Default</SelectItem>
                          <SelectItem value="light">Light</SelectItem>
                          <SelectItem value="dark">Dark</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default StoryLayersTab
