import React, { useEffect, useState } from 'react';
import { useQuery } from '@apollo/client/react';
import {
  SequenceArtefactNodeConfig,
  SequenceArtefactRenderTarget,
} from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Checkbox } from '@/components/ui/checkbox';
import { LIST_SEQUENCES, Sequence } from '@/graphql/sequences';

interface SequenceArtefactNodeConfigFormProps {
  config: SequenceArtefactNodeConfig;
  setConfig: (config: SequenceArtefactNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
  storyId?: number; // May be passed from connected StoryNode
}

export const SequenceArtefactNodeConfigForm: React.FC<SequenceArtefactNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
  storyId,
}) => {
  const initialRenderConfig = {
    ...config.renderConfig,
    containNodes: config.renderConfig?.containNodes ?? 'one',
    builtInStyles: config.renderConfig?.builtInStyles ?? 'light',
    showNotes: config.renderConfig?.showNotes ?? true,
    renderAllSequences: config.renderConfig?.renderAllSequences ?? true,
    enabledSequenceIds: config.renderConfig?.enabledSequenceIds ?? [],
  };

  const [localConfig, setLocalConfig] = useState<SequenceArtefactNodeConfig>({
    renderTarget: (config.renderTarget || 'MermaidSequence') as SequenceArtefactRenderTarget,
    outputPath: config.outputPath ?? '',
    renderConfig: initialRenderConfig,
    useStoryLayers: config.useStoryLayers ?? true,
  });
  const [activeTab, setActiveTab] = useState<'general' | 'target' | 'layers'>('general');

  // Fetch sequences if we have a storyId (from connected StoryNode)
  const { data: sequencesData, loading: sequencesLoading } = useQuery(LIST_SEQUENCES, {
    variables: { storyId },
    skip: !storyId,
  });
  const sequences: Sequence[] = (sequencesData as any)?.sequences || [];

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Always valid as long as renderTarget is set
    setIsValid(!!localConfig.renderTarget);
  }, [localConfig, setIsValid]);

  const handleSequenceToggle = (sequenceId: number, enabled: boolean) => {
    setLocalConfig(prev => {
      const currentEnabled = prev.renderConfig?.enabledSequenceIds ?? [];
      const newEnabled = enabled
        ? [...currentEnabled, sequenceId]
        : currentEnabled.filter(id => id !== sequenceId);
      return {
        ...prev,
        renderConfig: {
          ...(prev.renderConfig ?? {}),
          enabledSequenceIds: newEnabled,
        },
      };
    });
  };

  const isSequenceEnabled = (sequenceId: number) => {
    return localConfig.renderConfig?.enabledSequenceIds?.includes(sequenceId) ?? false;
  };

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
              placeholder="e.g., sequence.mmd (auto-generated if not specified)"
              value={localConfig.outputPath}
              onChange={(event) => setLocalConfig(prev => ({ ...prev, outputPath: event.currentTarget.value }))}
            />
            <p className="text-sm text-muted-foreground">
              If not specified, will use project name and file extension
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="contain-nodes">Contain Nodes</Label>
            <Select
              value={localConfig.renderConfig?.containNodes || 'one'}
              onValueChange={(value) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), containNodes: value as 'one' | 'all' }
              }))}
            >
              <SelectTrigger id="contain-nodes">
                <SelectValue placeholder="Select containment" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="one">One container per sequence</SelectItem>
                <SelectItem value="all">All in one container</SelectItem>
              </SelectContent>
            </Select>
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
          </div>

          <div className="flex items-center space-x-2">
            <Switch
              id="show-notes"
              checked={localConfig.renderConfig?.showNotes ?? true}
              onCheckedChange={(checked) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), showNotes: checked }
              }))}
            />
            <div>
              <Label htmlFor="show-notes">Show Notes</Label>
              <p className="text-sm text-muted-foreground">
                Display notes alongside sequence messages
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <Switch
              id="render-all-sequences"
              checked={localConfig.renderConfig?.renderAllSequences ?? true}
              onCheckedChange={(checked) => setLocalConfig(prev => ({
                ...prev,
                renderConfig: { ...(prev.renderConfig ?? {}), renderAllSequences: checked }
              }))}
            />
            <div>
              <Label htmlFor="render-all-sequences">Render All Sequences</Label>
              <p className="text-sm text-muted-foreground">
                When off, select individual sequences to render
              </p>
            </div>
          </div>

          {!(localConfig.renderConfig?.renderAllSequences ?? true) && (
            <div className="space-y-3 rounded-md border p-4">
              <div>
                <Label>Sequences to Render</Label>
                <p className="text-sm text-muted-foreground">
                  Select which sequences to include in the export
                </p>
              </div>

              {sequencesLoading ? (
                <p className="text-sm text-muted-foreground">Loading sequences...</p>
              ) : !storyId ? (
                <p className="text-sm text-muted-foreground">
                  Connect this node to a Story node to see available sequences
                </p>
              ) : sequences.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  No sequences found in the connected story
                </p>
              ) : (
                <div className="space-y-2">
                  {sequences.map(sequence => (
                    <div key={sequence.id} className="flex items-center space-x-2">
                      <Checkbox
                        id={`sequence-${sequence.id}`}
                        checked={isSequenceEnabled(sequence.id)}
                        onCheckedChange={(checked) => handleSequenceToggle(sequence.id, checked === true)}
                      />
                      <Label htmlFor={`sequence-${sequence.id}`} className="font-normal">
                        {sequence.name} ({sequence.edgeCount} edges)
                      </Label>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </TabsContent>

        <TabsContent value="target" className="space-y-4 pt-4">
          <div className="space-y-2">
            <Label htmlFor="render-target">Render Target</Label>
            <Select
              value={localConfig.renderTarget}
              onValueChange={(value) =>
                setLocalConfig(prev => ({ ...prev, renderTarget: value as SequenceArtefactRenderTarget }))
              }
            >
              <SelectTrigger id="render-target">
                <SelectValue placeholder="Select render target" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="MermaidSequence">Mermaid Sequence Diagram</SelectItem>
                <SelectItem value="PlantUmlSequence">PlantUML Sequence Diagram</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </TabsContent>

        <TabsContent value="layers" className="space-y-4 pt-4">
          <div className="flex items-center space-x-2">
            <Switch
              id="use-story-layers"
              checked={localConfig.useStoryLayers ?? true}
              onCheckedChange={(checked) => setLocalConfig(prev => ({
                ...prev,
                useStoryLayers: checked
              }))}
            />
            <div>
              <Label htmlFor="use-story-layers">Use Story Layers</Label>
              <p className="text-sm text-muted-foreground">
                When enabled, layer styling from the connected story will be applied to the sequence diagram
              </p>
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </Stack>
  );
};
