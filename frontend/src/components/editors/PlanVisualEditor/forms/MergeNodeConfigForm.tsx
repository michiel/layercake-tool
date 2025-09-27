import React, { useEffect, useState } from 'react';
import { Stack, Select, Textarea, Alert, Text } from '@mantine/core';
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
    inputRefs: config.inputRefs || [],
    outputGraphRef: config.outputGraphRef || '',
    mergeStrategy: config.mergeStrategy || 'Union',
    conflictResolution: config.conflictResolution || 'PreferFirst',
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    setIsValid(localConfig.inputRefs.length > 0 && localConfig.outputGraphRef.trim().length > 0);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Merge Node Configuration">
        <Text size="sm">Merge node configuration form will be implemented in a future update.</Text>
      </Alert>

      <Textarea
        label="Input References"
        placeholder="Enter input graph references (one per line)"
        value={localConfig.inputRefs.join('\n')}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, inputRefs: event.currentTarget.value.split('\n').filter(r => r.trim()) }))}
        rows={3}
        required
      />

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