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
    ...config,
    sourceGraphRef: config.sourceGraphRef || '',
    renderTarget: config.renderTarget || 'DOT',
    outputPath: config.outputPath || '',
    renderConfig: config.renderConfig || {},
    graphConfig: config.graphConfig || {},
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    setIsValid(localConfig.sourceGraphRef.trim().length > 0 && localConfig.outputPath.trim().length > 0);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Output Node Configuration">
        <Text size="sm">Output node configuration form will be implemented in a future update.</Text>
      </Alert>

      <TextInput
        label="Source Graph Reference"
        placeholder="Enter source graph reference"
        value={localConfig.sourceGraphRef}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, sourceGraphRef: event.currentTarget.value }))}
        required
      />

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
        label="Output Path"
        placeholder="Enter output file path"
        value={localConfig.outputPath}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, outputPath: event.currentTarget.value }))}
        required
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