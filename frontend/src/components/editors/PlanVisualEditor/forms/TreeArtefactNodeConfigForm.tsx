import React, { useEffect, useState } from 'react';
import { IconInfoCircle } from '@tabler/icons-react';
import { TreeArtefactNodeConfig, TreeArtefactRenderTarget } from '../../../../types/plan-dag';
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
  const initialRenderConfig = {
    ...config.renderConfig,
    containNodes: config.renderConfig?.containNodes ?? true,
    orientation: config.renderConfig?.orientation ?? 'TB',
    useDefaultStyling: config.renderConfig?.useDefaultStyling ?? true,
    theme: config.renderConfig?.theme ?? 'Light'
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

  return (
    <Stack gap="md">
      <Alert>
        <IconInfoCircle className="h-4 w-4" />
        <AlertTitle>Tree Artefact Configuration</AlertTitle>
        <AlertDescription>
          Produce hierarchical mindmaps using the upstream graph&apos;s partition structure. If no filename is
          specified it will be auto-generated from the project name and file extension.
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
            <SelectItem value="MermaidMindmap">Mermaid Mindmap</SelectItem>
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

      <div className="flex flex-col space-y-2">
        <div className="flex items-center space-x-2">
          <Switch
            id="use-default-styling"
            checked={localConfig.renderConfig?.useDefaultStyling ?? true}
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
            disabled={!(localConfig.renderConfig?.useDefaultStyling ?? true)}
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
    </Stack>
  );
};
