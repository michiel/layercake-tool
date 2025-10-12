import React, { useEffect, useState } from 'react';
import { Stack, TextInput, Select, Switch, Alert, Text } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { OutputNodeConfig } from '../../../../types/plan-dag';

interface OutputNodeConfigFormProps {
  config: OutputNodeConfig;
  setConfig: (config: OutputNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const OutputNodeConfigForm: React.FC<OutputNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<OutputNodeConfig>({
    renderTarget: config.renderTarget || 'DOT',
    outputPath: config.outputPath ?? '',
    renderConfig: config.renderConfig || {},
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
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Output Configuration">
        <Text size="sm">
          Configure export format and optional filename. Source graph comes from incoming edge connection.
          If no filename is specified, it will be auto-generated using the project name and file extension.
        </Text>
      </Alert>

      <Select
        label="Render Target"
        data={[
          { value: 'DOT', label: 'DOT (Graphviz)' },
          { value: 'GML', label: 'GML' },
          { value: 'JSON', label: 'JSON' },
          { value: 'PlantUML', label: 'PlantUML' },
          { value: 'CSVNodes', label: 'CSV Nodes' },
          { value: 'CSVEdges', label: 'CSV Edges' },
          { value: 'Mermaid', label: 'Mermaid' },
          { value: 'Custom', label: 'Custom' },
        ]}
        value={localConfig.renderTarget}
        onChange={(value) => setLocalConfig(prev => ({ ...prev, renderTarget: value as any }))}
      />

      <TextInput
        label="Filename (optional)"
        placeholder="e.g., myproject.gml (auto-generated if not specified)"
        description="If not specified, will use project name and file extension"
        value={localConfig.outputPath}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, outputPath: event.currentTarget.value }))}
      />

      <Switch
        label="Contain Nodes"
        checked={localConfig.renderConfig?.containNodes || false}
        onChange={(event) => setLocalConfig(prev => ({
          ...prev,
          renderConfig: { ...prev.renderConfig, containNodes: event.currentTarget.checked }
        }))}
      />

      <Select
        label="Orientation"
        data={[
          { value: 'LR', label: 'Left to Right' },
          { value: 'TB', label: 'Top to Bottom' },
        ]}
        value={localConfig.renderConfig?.orientation || 'TB'}
        onChange={(value) => setLocalConfig(prev => ({
          ...prev,
          renderConfig: { ...prev.renderConfig, orientation: value as any }
        }))}
      />
    </Stack>
  );
};