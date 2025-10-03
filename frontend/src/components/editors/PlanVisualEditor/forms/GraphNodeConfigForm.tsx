import React, { useEffect, useState } from 'react';
import { Stack, Switch, Alert, Text } from '@mantine/core';
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
    isReference: config.isReference ?? false,
    metadata: config.metadata || {},
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Always valid - graph source comes from incoming edge
    setIsValid(true);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Graph Node Configuration">
        <Text size="sm">
          Configure graph node behavior. Graph source is determined by edge connections in the DAG.
        </Text>
      </Alert>

      <Switch
        label="Is Reference"
        checked={localConfig.isReference}
        onChange={(event) => setLocalConfig(prev => ({ ...prev, isReference: event.currentTarget.checked }))}
      />
    </Stack>
  );
};