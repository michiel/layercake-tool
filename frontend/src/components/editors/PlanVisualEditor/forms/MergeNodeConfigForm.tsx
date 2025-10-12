import React, { useEffect, useState } from 'react';
import { Stack, Select, Alert, Text } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { MergeNodeConfig } from '../../../../types/plan-dag';

interface MergeNodeConfigFormProps {
  config: MergeNodeConfig;
  setConfig: (config: MergeNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const MergeNodeConfigForm: React.FC<MergeNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<MergeNodeConfig>({
    ...config,
    mergeStrategy: config.mergeStrategy || 'Union',
    conflictResolution: config.conflictResolution || 'PreferFirst',
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
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Merge Configuration">
        <Text size="sm">
          Configure merge behavior. Inputs and output are determined by edge connections in the DAG.
        </Text>
      </Alert>

      <Select
        label="Merge Strategy"
        data={[
          { value: 'Union', label: 'Union' },
          { value: 'Intersection', label: 'Intersection' },
          { value: 'Difference', label: 'Difference' },
        ]}
        value={localConfig.mergeStrategy}
        onChange={(value) => setLocalConfig(prev => ({ ...prev, mergeStrategy: value as any }))}
      />
    </Stack>
  );
};