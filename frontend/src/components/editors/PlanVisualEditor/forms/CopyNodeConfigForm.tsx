import React, { useEffect, useState } from 'react';
import { Stack, Select, Switch, Alert, Text } from '@mantine/core';
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
    copyType: config.copyType || 'DeepCopy',
    preserveMetadata: config.preserveMetadata ?? true,
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Always valid - connections handled by edges
    setIsValid(true);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Copy Node Configuration">
        <Text size="sm">
          Configure copy behavior. Source and output are determined by edge connections in the DAG.
        </Text>
      </Alert>

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