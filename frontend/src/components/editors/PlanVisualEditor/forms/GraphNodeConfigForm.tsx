import React, { useEffect, useState } from 'react';
import { Stack, Switch, Alert, Text, TextInput } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { GraphNodeConfig, NodeMetadata } from '../../../../types/plan-dag';

interface GraphNodeConfigFormProps {
  config: GraphNodeConfig;
  setConfig: (config: GraphNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
  metadata: NodeMetadata;
  setMetadata: React.Dispatch<React.SetStateAction<NodeMetadata>>;
}

export const GraphNodeConfigForm: React.FC<GraphNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
  metadata,
  setMetadata,
}) => {
  const [localConfig, setLocalConfig] = useState<GraphNodeConfig>({
    ...config,
    isReference: config.isReference ?? false,
    metadata: config.metadata || {},
  });
  const [nodeName, setNodeName] = useState<string>(metadata?.label ?? '');

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    setNodeName(metadata?.label ?? '');
  }, [metadata]);

  useEffect(() => {
    setIsValid(nodeName.trim().length > 0);
  }, [nodeName, setIsValid]);

  const handleNameChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const value = event.currentTarget.value;
    setNodeName(value);
    setMetadata(prev => ({
      ...prev,
      label: value,
    }));
  };

  return (
    <Stack gap="md">
      <TextInput
        label="Node Name"
        placeholder="Enter a name for this graph node"
        required
        value={nodeName}
        onChange={handleNameChange}
      />

      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Graph Configuration">
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
