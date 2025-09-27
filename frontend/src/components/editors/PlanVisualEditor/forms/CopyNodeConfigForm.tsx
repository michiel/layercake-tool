import React, { useEffect, useState } from 'react';
import { Stack, TextInput, Select, Switch, Alert, Text } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { CopyNodeConfig } from '../../../../types/plan-dag';

interface CopyNodeConfigFormProps {
  config: CopyNodeConfig;
  setConfig: (config: CopyNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const CopyNodeConfigForm: React.FC<CopyNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<CopyNodeConfig>({
    ...config,
    sourceGraphRef: config.sourceGraphRef || '',
    outputGraphRef: config.outputGraphRef || '',
    copyType: config.copyType || 'DeepCopy',
    preserveMetadata: config.preserveMetadata ?? true,
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    setIsValid(localConfig.sourceGraphRef.trim().length > 0 && localConfig.outputGraphRef.trim().length > 0);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Copy Node Configuration">
        <Text size="sm">Copy node configuration form will be implemented in a future update.</Text>
      </Alert>

      <TextInput
        label="Source Graph Reference"
        placeholder="Enter source graph reference"
        value={localConfig.sourceGraphRef}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, sourceGraphRef: event.currentTarget.value }))}
        required
      />

      <TextInput
        label="Output Graph Reference"
        placeholder="Enter output graph reference"
        value={localConfig.outputGraphRef}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, outputGraphRef: event.currentTarget.value }))}
        required
      />

      <Select
        label="Copy Type"
        data={[
          { value: 'DeepCopy', label: 'Deep Copy' },
          { value: 'ShallowCopy', label: 'Shallow Copy' },
          { value: 'Reference', label: 'Reference' },
        ]}
        value={localConfig.copyType}
        onChange={(value) => setLocalConfig(prev => ({ ...prev, copyType: value as any }))}
      />

      <Switch
        label="Preserve Metadata"
        checked={localConfig.preserveMetadata}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, preserveMetadata: event.currentTarget.checked }))}
      />
    </Stack>
  );
};