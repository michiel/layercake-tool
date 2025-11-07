import React, { useEffect, useState } from 'react';
import { IconInfoCircle } from '@tabler/icons-react';
import { GraphNodeConfig, NodeMetadata } from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

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
  const [localConfig] = useState<GraphNodeConfig>({
    ...config,
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
      <div className="space-y-2">
        <Label htmlFor="node-name">
          Node Name <span className="text-red-600">*</span>
        </Label>
        <Input
          id="node-name"
          placeholder="Enter a name for this graph node"
          value={nodeName}
          onChange={handleNameChange}
        />
      </div>

      <Alert>
        <IconInfoCircle className="h-4 w-4" />
        <AlertTitle>Graph Configuration</AlertTitle>
        <AlertDescription>
          Configure graph node behavior. Graph source is determined by edge connections in the DAG.
        </AlertDescription>
      </Alert>

    </Stack>
  );
};
