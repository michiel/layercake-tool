import React, { useEffect, useState } from 'react';
import { IconInfoCircle } from '@tabler/icons-react';
import {
  TreeArtefactNodeConfig,
  TreeArtefactRenderTarget,
  DEFAULT_MERMAID_OPTIONS,
} from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';

interface TreeArtefactNodeConfigFormProps {
  config: TreeArtefactNodeConfig;
  setConfig: (config: TreeArtefactNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const TreeArtefactNodeConfigForm: React.FC<TreeArtefactNodeConfigFormProps> = ({
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
    builtInStyles:
      config.renderConfig?.builtInStyles ||
      (legacyUseDefaultStyling === false
        ? 'none'
        : legacyTheme === 'Dark'
        ? 'dark'
        : 'light'),
    targetOptions: {
      graphviz: config.renderConfig?.targetOptions?.graphviz,
      mermaid: {
        ...DEFAULT_MERMAID_OPTIONS,
        ...(config.renderConfig?.targetOptions?.mermaid ?? {}),
      },
    },
  };

  const [localConfig, setLocalConfig] = useState<TreeArtefactNodeConfig>({
    renderTarget: (config.renderTarget || 'PlantUmlMindmap') as TreeArtefactRenderTarget,
    outputPath: config.outputPath ?? '',
    renderConfig: initialRenderConfig,
    graphConfig: config.graphConfig || {}
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    setIsValid(!!localConfig.renderTarget);
  }, [localConfig, setIsValid]);

  const mermaidOptions = {
    ...DEFAULT_MERMAID_OPTIONS,
    ...(localConfig.renderConfig?.targetOptions?.mermaid ?? {}),
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

  const isMermaidTarget =
    localConfig.renderTarget === 'MermaidMindmap' ||
    localConfig.renderTarget === 'MermaidTreemap';

  return (
    <Stack gap="md">
      <Alert>
        <IconInfoCircle className="h-4 w-4" />
        <AlertTitle>Tree Artefact Configuration</AlertTitle>
        <AlertDescription>
          Produce hierarchical mindmaps or treemaps using the upstream graph&apos;s partition structure. If no
          filename is specified it will be auto-generated from the project name and file extension.
        </AlertDescription>
      </Alert>

      <div className="space-y-2">
        <Label htmlFor="render-target">Render Target</Label>
        <Select
          value={localConfig.renderTarget}
          onValueChange={(value) =>
            setLocalConfig(prev => ({ ...prev, renderTarget: value as TreeArtefactRenderTarget }))
          }
        >
          <SelectTrigger id="render-target">
            <SelectValue placeholder="Select render target" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="PlantUmlMindmap">PlantUML Mindmap</SelectItem>
            <SelectItem value="PlantUmlWbs">PlantUML WBS</SelectItem>
            <SelectItem value="MermaidMindmap">Mermaid Mindmap</SelectItem>
            <SelectItem value="MermaidTreemap">Mermaid Treemap</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="filename">Filename (optional)</Label>
        <Input
          id="filename"
          placeholder="e.g., hierarchy.puml (auto-generated if not specified)"
          value={localConfig.outputPath}
          onChange={(event) => setLocalConfig(prev => ({ ...prev, outputPath: event.currentTarget.value }))}
        />
        <p className="text-sm text-muted-foreground">
          If not specified, will use project name and file extension.
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

      <div className="space-y-4">
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
              Controls whether mindmap/WBS nodes inherit colors from their assigned layers.
            </p>
          </div>
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
            Sets overall background/font defaults before layer-specific overrides are applied.
          </p>
        </div>
      </div>

      {isMermaidTarget && (
        <div className="space-y-4 border-t pt-4">
          <div>
            <Label>Mermaid Options</Label>
            <p className="text-sm text-muted-foreground">
              Adjust Mermaid&apos;s rendering style for mindmap/treemap exports.
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="tree-mermaid-look">Look</Label>
            <Select
              value={mermaidOptions.look}
              onValueChange={(value) => updateMermaidOptions({ look: value as typeof mermaidOptions.look })}
            >
              <SelectTrigger id="tree-mermaid-look">
                <SelectValue placeholder="Select look" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="default">Default</SelectItem>
                <SelectItem value="handDrawn">Hand Drawn</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="tree-mermaid-display">Display</Label>
            <Select
              value={mermaidOptions.display}
              onValueChange={(value) => updateMermaidOptions({ display: value as typeof mermaidOptions.display })}
            >
              <SelectTrigger id="tree-mermaid-display">
                <SelectValue placeholder="Select display" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="full">Full</SelectItem>
                <SelectItem value="compact">Compact</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      )}
    </Stack>
  );
};
