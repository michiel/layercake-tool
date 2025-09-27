import React, { useEffect, useState } from 'react';
import { Stack, NumberInput, Switch, Alert, Text } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { GraphNodeConfig } from '../../../../types/plan-dag';

interface GraphNodeConfigFormProps {
  config: GraphNodeConfig;
  setConfig: (config: GraphNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const GraphNodeConfigForm: React.FC<GraphNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<GraphNodeConfig>({
    ...config,
    graphId: config.graphId ?? 0,
    isReference: config.isReference ?? false,
    metadata: config.metadata || {},
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    setIsValid(localConfig.graphId > 0);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Graph Node Configuration">
        <Text size="sm">Graph node configuration form will be implemented in a future update.</Text>
      </Alert>

      <NumberInput
        label="Graph ID"
        placeholder="Enter graph ID"
        value={localConfig.graphId}
        onChange={(value) => setLocalConfig(prev => ({ ...prev, graphId: Number(value) || 0 }))}
        required
      />

      <Switch
        label="Is Reference"
        checked={localConfig.isReference}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, isReference: event.currentTarget.checked }))}
      />
    </Stack>
  );
};