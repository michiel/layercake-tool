import React, { useEffect, useMemo, useState } from 'react';
import { useQuery } from '@apollo/client/react';
import {
  GraphArtefactNodeConfig,
  GraphArtefactRenderTarget,
  DEFAULT_GRAPHVIZ_OPTIONS,
  DEFAULT_MERMAID_OPTIONS,
} from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { GET_PROJECT_LAYERS, ProjectLayer } from '@/graphql/layers';
import { GET_DATASOURCES } from '@/graphql/datasets';
import type { LayerSourceStyleMode } from '@/types/plan-dag';

interface GraphArtefactNodeConfigFormProps {
  config: GraphArtefactNodeConfig;
  setConfig: (config: GraphArtefactNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const GraphArtefactNodeConfigForm: React.FC<GraphArtefactNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const legacyUseDefaultStyling = (config.renderConfig as any)?.useDefaultStyling;
  const legacyTheme = (config.renderConfig as any)?.theme;

  const initialRenderConfig = {
    ...config.renderConfig,
    containNodes: config.renderConfig?.containNodes ?? true,
    orientation: config.renderConfig?.orientation ?? 'TB',
    applyLayers: config.renderConfig?.applyLayers ?? legacyUseDefaultStyling ?? true,
    useNodeWeight: config.renderConfig?.useNodeWeight ?? true,
    useEdgeWeight: config.renderConfig?.useEdgeWeight ?? true,
    layerSourceStyles: config.renderConfig?.layerSourceStyles ?? [],
    builtInStyles:
      config.renderConfig?.builtInStyles ||
      (legacyUseDefaultStyling === false
        ? 'none'
        : legacyTheme === 'Dark'
        ? 'dark'
        : 'light'),
    addNodeCommentsAsNotes: config.renderConfig?.addNodeCommentsAsNotes ?? false,
    notePosition: config.renderConfig?.notePosition ?? 'left',
    targetOptions: {
      graphviz: {
        ...DEFAULT_GRAPHVIZ_OPTIONS,
        ...(config.renderConfig?.targetOptions?.graphviz ?? {}),
      },
      mermaid: {
        ...DEFAULT_MERMAID_OPTIONS,
        ...(config.renderConfig?.targetOptions?.mermaid ?? {}),
      },
    },
  };

  const [localConfig, setLocalConfig] = useState<GraphArtefactNodeConfig>({
    renderTarget: (config.renderTarget || 'DOT') as GraphArtefactRenderTarget,
    outputPath: config.outputPath ?? '',
    renderConfig: initialRenderConfig,
    graphConfig: config.graphConfig || {}
  });
  const [activeTab, setActiveTab] = useState<'general' | 'target' | 'layers'>('general');

  const { data: projectLayersData, loading: projectLayersLoading } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId: _projectId },
    skip: !_projectId,
  });
  const { data: layerDatasetsData, loading: layerDatasetsLoading } = useQuery(GET_DATASOURCES, {
    variables: { projectId: _projectId },
    skip: !_projectId,
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Output path is now optional - filename will be auto-generated if not provided
    // Always valid as long as renderTarget is set
    setIsValid(!!localConfig.renderTarget);
  }, [localConfig, setIsValid]);

  const graphvizOptions = {
    ...DEFAULT_GRAPHVIZ_OPTIONS,
    ...(localConfig.renderConfig?.targetOptions?.graphviz ?? {}),
  };
  const mermaidOptions = {
    ...DEFAULT_MERMAID_OPTIONS,
    ...(localConfig.renderConfig?.targetOptions?.mermaid ?? {}),
  };

  const updateGraphvizOptions = (updates: Partial<typeof graphvizOptions>) => {
    setLocalConfig(prev => ({
      ...prev,
      renderConfig: {
        ...(prev.renderConfig ?? {}),
        targetOptions: {
          ...(prev.renderConfig?.targetOptions ?? {}),
          graphviz: {
            ...(prev.renderConfig?.targetOptions?.graphviz ?? DEFAULT_GRAPHVIZ_OPTIONS),
            ...updates,
          },
        },
      },
    }));
  };

  const updateMermaidOptions = (updates: Partial<typeof mermaidOptions>) => {
    setLocalConfig(prev => ({
      ...prev,
      renderConfig: {
        ...(prev.renderConfig ?? {}),
        targetOptions: {
          ...(prev.renderConfig?.targetOptions ?? {}),
          mermaid: {
            ...(prev.renderConfig?.targetOptions?.mermaid ?? DEFAULT_MERMAID_OPTIONS),
            ...updates,
          },
        },
      },
    }));
  };

  const layerSourceOptions = useMemo(() => {
    const layers: ProjectLayer[] = (projectLayersData as any)?.projectLayers ?? [];
    const enabledLayers = layers.filter(layer => layer.enabled);
    if (!enabledLayers.length) {
      return [];
    }

    const datasetNameMap = new Map<number, string>();
    const datasets = (layerDatasetsData as any)?.dataSets ?? [];
    datasets
      .filter((dataset: any) => dataset.hasLayers || (dataset.layerCount ?? 0) > 0)
      .forEach((dataset: any) => {
        datasetNameMap.set(dataset.id, dataset.name ?? `Dataset #${dataset.id}`);
      });

    const grouped = new Map<
      string,
      { datasetId: number | null; count: number; label: string; key: string }
    >();

    enabledLayers.forEach(layer => {
      const datasetId = layer.sourceDatasetId ?? null;
      const key = datasetId === null ? 'manual' : datasetId.toString();
      if (!grouped.has(key)) {
        const label =
          datasetId === null
            ? 'Manual layers'
            : datasetNameMap.get(datasetId) ?? `Dataset #${datasetId}`;
        grouped.set(key, { datasetId, count: 1, label, key });
      } else {
        const entry = grouped.get(key)!;
        entry.count += 1;
      }
    });

    return Array.from(grouped.values()).sort((a, b) => a.label.localeCompare(b.label));
  }, [projectLayersData, layerDatasetsData]);

  const findLayerSourceOverride = (datasetId: number | null) =>
    localConfig.renderConfig?.layerSourceStyles?.find(
      style => (style.sourceDatasetId ?? null) === (datasetId ?? null)
    );

  const getLayerSourceMode = (datasetId: number | null): LayerSourceStyleMode => {
    const existing = findLayerSourceOverride(datasetId);
    return existing?.mode ?? 'default';
  };

  const setLayerSourceOverride = (datasetId: number | null, mode: LayerSourceStyleMode) => {
    setLocalConfig(prev => {
      const existing = prev.renderConfig?.layerSourceStyles ?? [];
      const filtered = existing.filter(
        style => (style.sourceDatasetId ?? null) !== (datasetId ?? null)
      );
      return {
        ...prev,
        renderConfig: {
          ...(prev.renderConfig ?? {}),
          layerSourceStyles: [...filtered, { sourceDatasetId: datasetId, mode }],
        },
      };
    });
  };

  const removeLayerSourceOverride = (datasetId: number | null) => {
    setLocalConfig(prev => ({
      ...prev,
      renderConfig: {
        ...(prev.renderConfig ?? {}),
        layerSourceStyles: (prev.renderConfig?.layerSourceStyles ?? []).filter(
          style => (style.sourceDatasetId ?? null) !== (datasetId ?? null)
        ),
      },
    }));
  };

  const setLayerSourceDisabled = (datasetId: number | null, disabled: boolean) => {
    if (disabled) {
      const mode = getLayerSourceMode(datasetId);
      setLayerSourceOverride(datasetId, mode);
    } else {
      removeLayerSourceOverride(datasetId);
    }
  };

  const layerSourcesLoading = projectLayersLoading || layerDatasetsLoading;

  return (
    <Stack gap="md">
      <Tabs
        value={activeTab}
        onValueChange={(value) => setActiveTab(value as 'general' | 'target' | 'layers')}
        className="w-full"
      >
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="target">Target</TabsTrigger>
          <TabsTrigger value="layers">Layers</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-4 pt-4">
          <div className="space-y-2">
            <Label htmlFor="filename">Filename (optional)</Label>
            <Input
              id="filename"
              placeholder="e.g., myproject.gml (auto-generated if not specified)"
              value={localConfig.outputPath}
              onChange={(event) => setLocalConfig(prev => ({ ...prev, outputPath: event.currentTarget.value }))}
            />
            <p className="text-sm text-muted-foreground">
              If not specified, will use project name and file extension
            </p>
          </div>

          <div className="flex items-center space-x-2">
            <Switch
              id="contain-nodes"
              checked={localConfig.renderConfig?.containNodes ?? true}
              onCheckedChange={(checked) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), containNodes: checked }
              }))}
            />
            <Label htmlFor="contain-nodes">Contain Nodes</Label>
          </div>

          <div className="space-y-2">
            <Label htmlFor="built-in-style">Built-in Theme</Label>
            <Select
              value={localConfig.renderConfig?.builtInStyles || 'light'}
              onValueChange={(value) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), builtInStyles: value as 'none' | 'light' | 'dark' }
              }))}
            >
              <SelectTrigger id="built-in-style">
                <SelectValue placeholder="Select theme" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">None (engine defaults)</SelectItem>
                <SelectItem value="light">Light</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              Built-in styles set global background/font defaults before layer overrides.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="orientation">Orientation</Label>
            <Select
              value={localConfig.renderConfig?.orientation || 'TB'}
              onValueChange={(value) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), orientation: value as any }
              }))}
            >
              <SelectTrigger id="orientation">
                <SelectValue placeholder="Select orientation" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="LR">Left to Right</SelectItem>
                <SelectItem value="TB">Top to Bottom</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center space-x-2">
            <Switch
              id="use-node-weight"
              checked={localConfig.renderConfig?.useNodeWeight ?? true}
              onCheckedChange={(checked) =>
                setLocalConfig(prev => ({
                  ...prev,
                  renderConfig: { ...(prev.renderConfig ?? {}), useNodeWeight: checked },
                }))
              }
            />
            <div>
              <Label htmlFor="use-node-weight">Use Node Weight</Label>
              <p className="text-sm text-muted-foreground">
                Toggles whether exports use stored weights when sizing or shading nodes.
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <Switch
              id="use-edge-weight"
              checked={localConfig.renderConfig?.useEdgeWeight ?? true}
              onCheckedChange={(checked) =>
                setLocalConfig(prev => ({
                  ...prev,
                  renderConfig: { ...(prev.renderConfig ?? {}), useEdgeWeight: checked },
                }))
              }
            />
            <div>
              <Label htmlFor="use-edge-weight">Use Edge Weight</Label>
              <p className="text-sm text-muted-foreground">
                Disable to treat all edges uniformly regardless of their weight field.
              </p>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="target" className="space-y-4 pt-4">
          <div className="space-y-2">
            <Label htmlFor="render-target">Render Target</Label>
            <Select
              value={localConfig.renderTarget}
              onValueChange={(value) =>
                setLocalConfig(prev => ({ ...prev, renderTarget: value as GraphArtefactRenderTarget }))
              }
            >
              <SelectTrigger id="render-target">
                <SelectValue placeholder="Select render target" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="DOT">DOT (Graphviz)</SelectItem>
                <SelectItem value="GML">GML</SelectItem>
                <SelectItem value="JSON">JSON</SelectItem>
                <SelectItem value="PlantUML">PlantUML</SelectItem>
                <SelectItem value="CSVNodes">CSV Nodes</SelectItem>
                <SelectItem value="CSVEdges">CSV Edges</SelectItem>
                <SelectItem value="Mermaid">Mermaid</SelectItem>
                <SelectItem value="Custom">Custom</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {localConfig.renderTarget === 'PlantUML' && (
            <div className="space-y-4 border-t pt-4">
              <div className="flex items-center space-x-2">
                <Switch
                  id="add-node-comments-as-notes"
                  checked={localConfig.renderConfig?.addNodeCommentsAsNotes ?? false}
                  onCheckedChange={(checked) =>
                    setLocalConfig(prev => ({
                      ...prev,
                      renderConfig: { ...(prev.renderConfig ?? {}), addNodeCommentsAsNotes: checked },
                    }))
                  }
                />
                <div>
                  <Label htmlFor="add-node-comments-as-notes">Add node comments as notes</Label>
                  <p className="text-sm text-muted-foreground">
                    When enabled, exports include node comments as PlantUML notes.
                  </p>
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="note-position">Note position</Label>
                <Select
                  value={localConfig.renderConfig?.notePosition || 'left'}
                  onValueChange={(value) =>
                    setLocalConfig(prev => ({
                      ...prev,
                      renderConfig: { ...(prev.renderConfig ?? {}), notePosition: value as any },
                    }))
                  }
                >
                  <SelectTrigger id="note-position">
                    <SelectValue placeholder="Select note position" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="left">Left (default)</SelectItem>
                    <SelectItem value="right">Right</SelectItem>
                    <SelectItem value="top">Top</SelectItem>
                    <SelectItem value="bottom">Bottom</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-sm text-muted-foreground">
                  Position of the note relative to each node when comments are exported.
                </p>
              </div>
            </div>
          )}

          {localConfig.renderTarget === 'DOT' && (
            <div className="space-y-4 border-t pt-4">
              <div className="flex items-center space-x-2">
                <Switch
                  id="graphviz-add-node-comments-as-notes"
                  checked={localConfig.renderConfig?.addNodeCommentsAsNotes ?? false}
                  onCheckedChange={(checked) =>
                    setLocalConfig(prev => ({
                      ...prev,
                      renderConfig: { ...(prev.renderConfig ?? {}), addNodeCommentsAsNotes: checked },
                    }))
                  }
                />
                <div>
                  <Label htmlFor="graphviz-add-node-comments-as-notes">Add node comments as notes</Label>
                  <p className="text-sm text-muted-foreground">
                    When enabled, DOT exports include node comments as an extra label (xlabel) or tooltip attribute.
                  </p>
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="graphviz-comment-style">Comment style</Label>
                <Select
                  value={graphvizOptions.commentStyle ?? DEFAULT_GRAPHVIZ_OPTIONS.commentStyle}
                  onValueChange={(value) =>
                    updateGraphvizOptions({ commentStyle: value as typeof graphvizOptions.commentStyle })
                  }
                  disabled={!(localConfig.renderConfig?.addNodeCommentsAsNotes ?? false)}
                >
                  <SelectTrigger id="graphviz-comment-style">
                    <SelectValue placeholder="Choose how comments are rendered" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="label">Label (uses xlabel)</SelectItem>
                    <SelectItem value="tooltip">Tooltip</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-sm text-muted-foreground">
                  Select whether comments appear as an outer label (xlabel) or as a tooltip on the node.
                </p>
              </div>

              <div>
                <Label>Graphviz Options</Label>
                <p className="text-sm text-muted-foreground">
                  Configure layout and spacing for DOT exports. These settings map directly to Graphviz attributes.
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="graphviz-layout">Layout</Label>
                <Select
                  value={graphvizOptions.layout}
                  onValueChange={(value) => updateGraphvizOptions({ layout: value as typeof graphvizOptions.layout })}
                >
                  <SelectTrigger id="graphviz-layout">
                    <SelectValue placeholder="Select layout" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="dot">Dot</SelectItem>
                    <SelectItem value="neato">Neato</SelectItem>
                    <SelectItem value="fdp">Fdp</SelectItem>
                    <SelectItem value="circo">Circo</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="graphviz-splines"
                  checked={graphvizOptions.splines}
                  onCheckedChange={(checked) => updateGraphvizOptions({ splines: checked })}
                />
                <Label htmlFor="graphviz-splines">Use Splines</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="graphviz-overlap"
                  checked={graphvizOptions.overlap}
                  onCheckedChange={(checked) => updateGraphvizOptions({ overlap: checked })}
                />
                <Label htmlFor="graphviz-overlap">Allow Overlap</Label>
              </div>

              <div className="grid gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="graphviz-nodesep">Node Separation</Label>
                  <Input
                    id="graphviz-nodesep"
                    type="number"
                    step="0.1"
                    value={graphvizOptions.nodesep ?? DEFAULT_GRAPHVIZ_OPTIONS.nodesep}
                    onChange={(event) =>
                      updateGraphvizOptions({
                        nodesep: parseFloat(event.currentTarget.value) || DEFAULT_GRAPHVIZ_OPTIONS.nodesep,
                      })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="graphviz-ranksep">Rank Separation</Label>
                  <Input
                    id="graphviz-ranksep"
                    type="number"
                    step="0.1"
                    value={graphvizOptions.ranksep ?? DEFAULT_GRAPHVIZ_OPTIONS.ranksep}
                    onChange={(event) =>
                      updateGraphvizOptions({
                        ranksep: parseFloat(event.currentTarget.value) || DEFAULT_GRAPHVIZ_OPTIONS.ranksep,
                      })
                    }
                  />
                </div>
              </div>
            </div>
          )}

          {localConfig.renderTarget === 'Mermaid' && (
            <div className="space-y-4 border-t pt-4">
              <div>
                <Label>Mermaid Options</Label>
                <p className="text-sm text-muted-foreground">
                  Control Mermaid&apos;s look (default vs. hand drawn) and layout density.
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="mermaid-look">Look</Label>
                <Select
                  value={mermaidOptions.look}
                  onValueChange={(value) => updateMermaidOptions({ look: value as typeof mermaidOptions.look })}
                >
                  <SelectTrigger id="mermaid-look">
                    <SelectValue placeholder="Select look" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">Default</SelectItem>
                    <SelectItem value="handDrawn">Hand Drawn</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="mermaid-display">Display</Label>
                <Select
                  value={mermaidOptions.display}
                  onValueChange={(value) => updateMermaidOptions({ display: value as typeof mermaidOptions.display })}
                >
                  <SelectTrigger id="mermaid-display">
                    <SelectValue placeholder="Select display mode" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="full">Full</SelectItem>
                    <SelectItem value="compact">Compact</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}
        </TabsContent>

        <TabsContent value="layers" className="space-y-6 pt-4">
          <div className="flex items-center space-x-2">
            <Switch
              id="apply-layer-colors"
              checked={localConfig.renderConfig?.applyLayers ?? true}
              onCheckedChange={(checked) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), applyLayers: checked }
              }))}
            />
            <div>
              <Label htmlFor="apply-layer-colors">Apply Layer Colors</Label>
              <p className="text-sm text-muted-foreground">
                When enabled, nodes and edges inherit the palette defined for each layer.
              </p>
            </div>
          </div>

          <div className="space-y-3 rounded-md border p-4">
            <div>
              <Label>Layer Sources</Label>
              <p className="text-sm text-muted-foreground">
                Disable individual sources to fall back to built-in palettes for their layers.
              </p>
            </div>

            {layerSourcesLoading ? (
              <p className="text-sm text-muted-foreground">Loading layer sources…</p>
            ) : !layerSourceOptions.length ? (
              <p className="text-sm text-muted-foreground">
                No enabled layer sources found for this project.
              </p>
            ) : (
              <div className="space-y-3">
                {layerSourceOptions.map(option => {
                  const datasetKey = option.key;
                  const disabled = !!findLayerSourceOverride(option.datasetId ?? null);
                  const mode = getLayerSourceMode(option.datasetId ?? null);
                  const switchId = `layer-source-disable-${datasetKey}`;
                  const selectId = `layer-source-style-${datasetKey}`;

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
                            Disable source colors
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
                  );
                })}
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </Stack>
  );
};
