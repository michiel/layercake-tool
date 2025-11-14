import React, { useEffect, useState } from 'react';
import { IconInfoCircle } from '@tabler/icons-react';
import { GraphArtefactNodeConfig, GraphArtefactRenderTarget } from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';

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
  const initialRenderConfig = {
    ...config.renderConfig,
    containNodes: config.renderConfig?.containNodes ?? true,
    orientation: config.renderConfig?.orientation ?? 'TB',
    useDefaultStyling: config.renderConfig?.useDefaultStyling ?? false,
    theme: config.renderConfig?.theme ?? 'Light'
  };

  const [localConfig, setLocalConfig] = useState<GraphArtefactNodeConfig>({
    renderTarget: (config.renderTarget || 'DOT') as GraphArtefactRenderTarget,
    outputPath: config.outputPath ?? '',
    renderConfig: initialRenderConfig,
    graphConfig: config.graphConfig || {}
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Output path is now optional - filename will be auto-generated if not provided
    // Always valid as long as renderTarget is set
    setIsValid(!!localConfig.renderTarget);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert>
        <IconInfoCircle className="h-4 w-4" />
        <AlertTitle>Output Configuration</AlertTitle>
        <AlertDescription>
          Configure export format and optional filename. Source graph comes from incoming edge connection.
          If no filename is specified, it will be auto-generated using the project name and file extension.
        </AlertDescription>
      </Alert>

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

      <div className="flex flex-col space-y-2">
        <div className="flex items-center space-x-2">
          <Switch
            id="use-default-styling"
          checked={localConfig.renderConfig?.useDefaultStyling ?? false}
            onCheckedChange={(checked) => setLocalConfig(prev => ({
              ...prev,
              renderConfig: { ...(prev.renderConfig ?? {}), useDefaultStyling: checked }
            }))}
          />
          <div>
            <Label htmlFor="use-default-styling">Use Default Styling</Label>
            <p className="text-sm text-muted-foreground">
              Apply Layercake&apos;s built-in colors and layout accents in supported exports.
            </p>
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="theme">Default Styling Theme</Label>
          <Select
            value={localConfig.renderConfig?.theme || 'Light'}
            onValueChange={(value) => setLocalConfig(prev => ({
              ...prev,
              renderConfig: { ...(prev.renderConfig ?? {}), theme: value as 'Light' | 'Dark' }
            }))}
            disabled={!(localConfig.renderConfig?.useDefaultStyling ?? false)}
          >
            <SelectTrigger id="theme">
              <SelectValue placeholder="Select theme" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Light">Light</SelectItem>
              <SelectItem value="Dark">Dark</SelectItem>
            </SelectContent>
          </Select>
        </div>
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
    </Stack>
  );
};
